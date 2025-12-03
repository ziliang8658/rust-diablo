/// Tile Texture Manager
/// 
/// Manages dungeon tile textures with automatic decoding and caching.
/// 
/// # Architecture
/// 
/// The texture manager integrates three key components:
/// 1. **CEL Data**: Raw encoded tile frames
/// 2. **Palette**: Color index to RGB mapping
/// 3. **MIN Data**: TileType and frame index mappings
/// 
/// When a tile is requested, the manager:
/// 1. Looks up the MicroTile definition from MIN data (gets TileType + frame index)
/// 2. Retrieves raw CEL frame data
/// 3. Decodes using the appropriate TileType decoder
/// 4. Applies the palette to convert indexed colors to RGBA
/// 5. Caches the result for future requests
/// 
/// # Reference
/// 
/// Original code: `Source/levels/gendung.cpp`
/// - `LoadLvlGFX()` Line 1235-1264
/// - `SetDungeonMicros()` Line 155-188
/// - `GetDunFrame()` (inline) in `Source/engine/render/dun_render.hpp` Line 27-31

use anyhow::{Result, Context, bail, anyhow};
use std::collections::HashMap;

use crate::resources::{MpqManager, Palette};
use crate::resources::dungeon_cel::DungeonCelSprite;
use crate::tiles::{MinData, DungeonType, TileType};
use crate::tiles::decoder::decode_tile;

/// Get the path to the special CEL file for a dungeon type
/// 
/// Special CEL files (l1s.cel, l2s.cel, etc.) contain special tiles like doors,
/// decorations, and special room structures. These files have a .cel extension
/// but are actually in CLX format.
/// 
/// # Reference
/// Original: Source/levels/gendung.cpp::LoadLvlGFX() Line 1235-1264
fn get_special_cel_paths(dungeon_type: DungeonType) -> Vec<&'static str> {
    // C++ LoadCelWithStatus tries multiple extensions (.cel, .clx, or no extension)
    // Reference: Source/diablo.cpp:1372 uses "levels\\towndata\\towns" (no extension)
    match dungeon_type {
        DungeonType::Town => vec![
            "levels/towndata/towns.clx",  // Try .clx first (CLX format)
            "levels/towndata/towns.cel",  // Then .cel
            "levels/towndata/towns",      // Then no extension
        ],
        DungeonType::Cathedral => vec!["levels/l1data/l1s.cel"],
        DungeonType::Catacombs => vec!["levels/l2data/l2s.cel"],
        DungeonType::Caves => vec!["levels/l3data/l3s.cel"],
        DungeonType::Hell => vec!["levels/l4data/l4s.cel"],
    }
}

/// Tile texture manager with caching
/// 
/// # Example
/// ```no_run
/// use rust_diablo::tiles::texture_manager::TileTextureManager;
/// use rust_diablo::tiles::DungeonType;
/// use rust_diablo::resources::MpqManager;
/// 
/// let mut mpq = MpqManager::new();
/// mpq.load_mpq("DIABDAT.MPQ", 1000)?;
/// 
/// let mut tex_mgr = TileTextureManager::load_for_dungeon(
///     DungeonType::Cathedral,
///     &mut mpq
/// )?;
/// 
/// // Get decoded tile (automatically cached)
/// let tile_data = tex_mgr.get_decoded_tile(1)?;
/// println!("Tile size: {} bytes", tile_data.len());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct TileTextureManager {
    /// Main CEL sprite data (e.g., l1.cel)
    cel_sprite: DungeonCelSprite,
    
    /// Special CEL sprite data (e.g., l1s.cel) - Optional
    /// Used for doors, decorations, and special room tiles
    special_cel_sprite: Option<DungeonCelSprite>,
    
    /// Color palette
    palette: Palette,
    
    /// MIN data (TileType and frame mappings)
    min_data: MinData,
    
    /// Cache of decoded tiles (micro_index -> decoded RGBA pixels)
    decoded_cache: HashMap<usize, Vec<u8>>,
    
    /// Cache of indexed tiles (frame_index -> decoded indexed pixels, 0 = transparent)
    indexed_cache: HashMap<usize, Vec<u8>>,
}

impl TileTextureManager {
    /// Load tileset for a specific dungeon type
    /// 
    /// # Arguments
    /// * `dungeon_type` - Type of dungeon (Cathedral, Catacombs, etc.)
    /// * `mpq_manager` - MPQ manager to load files from
    /// 
    /// # Returns
    /// Initialized texture manager
    /// 
    /// # Reference
    /// Original code: `Source/levels/gendung.cpp::LoadLvlGFX()` Line 1235-1264
    pub fn load_for_dungeon(
        dungeon_type: DungeonType,
        mpq_manager: &mut MpqManager,
    ) -> Result<Self> {
        // Determine file paths based on dungeon type
        // Reference: Source/diablo.cpp:1346-1408 (LoadLvlGFX)
        // 
        // NOTE: Special CEL files (l1s, l2s, etc.) are CLX format and require
        // separate implementation. They will be added in a future step.
        let (cel_path, _special_cel_path, pal_path, min_path): (&str, Option<&str>, &str, &str) = match dungeon_type {
            DungeonType::Town => (
                "levels/towndata/town.cel",
                None,  // TODO: towns.cel (CLX format)
                "levels/towndata/town.pal",
                "levels/towndata/town.min",
            ),
            DungeonType::Cathedral => (
                "levels/l1data/l1.cel",
                None,  // TODO: l1s.cel (CLX format, doors/decorations)
                "levels/l1data/l1_2.pal",  // l1_2.pal - gray/blue tone (C++ uses random l1_1-4.pal)
                "levels/l1data/l1.min",
            ),
            DungeonType::Catacombs => (
                "levels/l2data/l2.cel",
                None,  // TODO: l2s.cel (CLX format)
                "levels/l2data/l2.pal",
                "levels/l2data/l2.min",
            ),
            DungeonType::Caves => (
                "levels/l3data/l3.cel",
                None,  // TODO: l1s.cel (CLX format)
                "levels/l3data/l3.pal",
                "levels/l3data/l3.min",
            ),
            DungeonType::Hell => (
                "levels/l4data/l4.cel",
                None,  // TODO: l2s.cel (CLX format)
                "levels/l4data/l4.pal",
                "levels/l4data/l4.min",
            ),
        };
        
        println!("\n=== Loading Tileset for {:?} ===", dungeon_type);
        println!("  CEL: {}", cel_path);
        println!("  PAL: {}", pal_path);
        println!("  MIN: {}", min_path);
        
        // Load CEL data
        // Reference: Source/diablo.cpp:1352
        // Try both Unix-style and Windows-style path separators
        let cel_unix = cel_path.replace('\\', "/");
        let cel_windows = cel_path.replace('/', "\\");
        let cel_data = mpq_manager
            .find_file(&cel_unix)
            .or_else(|| mpq_manager.find_file(&cel_windows))
            .ok_or_else(|| anyhow::anyhow!("CEL file not found (tried {} and {})", cel_unix, cel_windows))?;
        let cel_sprite = DungeonCelSprite::from_bytes(&cel_data)
            .with_context(|| format!("Failed to parse CEL file: {}", cel_path))?;
        println!("  ✓ Loaded CEL: {} frames", cel_sprite.frames.len());
        
        // Load special CEL files (l1s, l2s, etc.) for doors/decorations
        // These files have .cel extension but are actually CLX format
        // Reference: Source/levels/gendung.cpp::LoadLvlGFX() Line 1235-1264
        let special_cel_paths = get_special_cel_paths(dungeon_type);
        println!("  Loading special CEL, trying {} paths...", special_cel_paths.len());
        
        let special_cel_sprite = {
            let mut loaded_sprite = None;
            
            for special_cel_path in &special_cel_paths {
                println!("    Trying: {}", special_cel_path);
                
                // Try both Unix-style and Windows-style path separators
                let special_unix = special_cel_path.replace('\\', "/");
                let special_windows = special_cel_path.replace('/', "\\");
                
                if let Some(special_data) = mpq_manager.find_file(&special_unix)
                    .or_else(|| mpq_manager.find_file(&special_windows))
                {
                    println!("    Found file: {} ({} bytes)", special_cel_path, special_data.len());
                    
                    // Try CLX format first (for l1s.cel, l2s.cel, etc.)
                    match DungeonCelSprite::from_clx_bytes(&special_data) {
                        Ok(sprite) => {
                            println!("  ✓ Loaded special CEL from {} as CLX: {} frames", special_cel_path, sprite.frames.len());
                            loaded_sprite = Some(sprite);
                            break;  // Success, stop trying
                        }
                        Err(clx_err) => {
                            // CLX failed, try CEL RLE format with width 64 (for towns.cel)
                            // Reference: Source/diablo.cpp:1372 LoadCelWithStatus("levels\\towndata\\towns", 64)
                            const SPECIAL_CEL_WIDTH: usize = 64;
                            
                            match DungeonCelSprite::from_cel_bytes_with_width(&special_data, SPECIAL_CEL_WIDTH) {
                                Ok(sprite) => {
                                    println!("  ✓ Loaded special CEL from {} as CEL RLE (width={}): {} frames", 
                                        special_cel_path, SPECIAL_CEL_WIDTH, sprite.frames.len());
                                    loaded_sprite = Some(sprite);
                                    break;  // Success, stop trying
                                }
                                Err(cel_err) => {
                                    eprintln!("    ⚠ Failed to parse {} as CLX: {}", special_cel_path, clx_err);
                                    eprintln!("    ⚠ Failed to parse {} as CEL RLE: {}", special_cel_path, cel_err);
                                    // Continue trying other paths
                                }
                            }
                        }
                    }
                } else {
                    println!("    File not found: {}", special_cel_path);
                }
            }
            
            if loaded_sprite.is_none() {
                println!("  ℹ No special CEL loaded (tried {} paths)", special_cel_paths.len());
            }
            
            loaded_sprite
        };
        
        // Load palette
        // Reference: Source/engine/palette.cpp:175-186
        let palette = Palette::from_mpq(mpq_manager, pal_path)
            .with_context(|| format!("Failed to load palette: {}", pal_path))?;
        println!("  ✓ Loaded palette: {} colors", palette.len());
        
        // Load MIN data
        // Reference: Source/levels/gendung.cpp:155-188
        let min_data = MinData::from_mpq(mpq_manager, min_path, dungeon_type)
            .with_context(|| format!("Failed to load MIN file: {}", min_path))?;
        println!("  ✓ Loaded MIN: {} pieces ({} blocks/piece)", min_data.len(), min_data.blocks_per_piece);
        
        // DEBUG: Check MIN frame index range
        let mut max_frame_idx = 0;
        let mut frame_counts = std::collections::HashMap::new();
        for piece_idx in 0..min_data.len() {
            if let Some(piece) = min_data.get(piece_idx) {
                for block in &piece.mt {
                    if block.has_value() {
                        let frame_idx = block.frame() as usize;
                        max_frame_idx = max_frame_idx.max(frame_idx);
                        *frame_counts.entry(frame_idx).or_insert(0) += 1;
                    }
                }
            }
        }
        println!("\n=== MIN Frame Index Analysis ===");
        println!("  Main CEL frames: {}", cel_sprite.frames.len());
        println!("  Special CEL frames: {}", special_cel_sprite.as_ref().map(|s| s.frames.len()).unwrap_or(0));
        println!("  Total available frames: {}", cel_sprite.frames.len() + special_cel_sprite.as_ref().map(|s| s.frames.len()).unwrap_or(0));
        println!("  MAX frame index in MIN: {}", max_frame_idx);
        println!("  Unique frame indices used: {}", frame_counts.len());
        
        let total_frames = cel_sprite.frames.len() + special_cel_sprite.as_ref().map(|s| s.frames.len()).unwrap_or(0);
        if max_frame_idx >= total_frames {
            println!("  ⚠️ WARNING: MIN references frame {} but only {} frames available!", max_frame_idx, total_frames);
            println!("  ⚠️ Out-of-range frames will render as transparent");
        }
        
        Ok(Self {
            cel_sprite,
            special_cel_sprite,
            palette,
            min_data,
            decoded_cache: HashMap::new(),
            indexed_cache: HashMap::new(),
        })
    }
    
    /// Get decoded tile as RGBA pixels (with caching)
    /// 
    /// # Arguments
    /// * `piece_index` - Piece index in MIN data (levelPieceId)
    /// * `block_index` - Block index within the piece (0..blocks_per_piece)
    /// 
    /// # Returns
    /// RGBA pixel data (4 bytes per pixel)
    /// Save RGBA data as PNG file for debugging
    pub fn save_debug_png(rgba_data: &[u8], width: u32, height: u32, filename: &str) -> Result<()> {
        use image::{ImageBuffer, Rgba};
        
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgba_data.to_vec())
            .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;
        
        img.save(filename)?;
        println!("Saved debug PNG: {} ({}x{})", filename, width, height);
        Ok(())
    }
    
    pub fn get_decoded_tile(&mut self, piece_index: usize, block_index: usize) -> Result<&[u8]> {
        // Create a unique cache key from piece and block indices
        let cache_key = piece_index * 16 + block_index;

        // Check cache first
        if self.decoded_cache.contains_key(&cache_key) {
            return Ok(&self.decoded_cache[&cache_key]);
        }
        
        // Get PieceMicros from MIN data
        let piece = self.min_data
            .get(piece_index)
            .ok_or_else(|| anyhow::anyhow!("Piece index out of range: {}", piece_index))?;
        
        // Get the specific block
        let block = piece.mt.get(block_index)
            .ok_or_else(|| anyhow::anyhow!("Block index out of range: {} (piece has {} blocks)", 
                block_index, piece.mt.len()))?;
        
        if !block.has_value() {
            // Empty tile - return transparent
            let size = 32 * 32 * 4;  // RGBA
            let transparent = vec![0u8; size];
            self.decoded_cache.insert(cache_key, transparent);
            return Ok(&self.decoded_cache[&cache_key]);
        }
        
        // Get TileType and frame index
        let tile_type = block.tile_type();
        let frame_idx = block.frame() as usize;
        
        // Get decoded indexed pixels
        let indexed_pixels = self.get_indexed_tile(frame_idx, tile_type)?;
        
        // Clone indexed pixels so we can release the mutable borrow on self
        let indexed_pixels = indexed_pixels.to_vec();
        
        // Apply palette to convert to RGBA
        let rgba_pixels = self.palette.indices_to_rgba(&indexed_pixels, true);
        
        // Cache and return (no flip - decoder already outputs in correct order)
        self.decoded_cache.insert(cache_key, rgba_pixels);
        Ok(&self.decoded_cache[&cache_key])
    }

    /// Get decoded tile with forced TileType (for floor rendering)
    /// 
    /// This is used when we need to force a specific TileType for decoding,
    /// regardless of what's stored in the block (e.g., C++ DrawFloorTile forces Triangle types)
    /// 
    /// # Arguments
    /// * `piece_index` - Piece index in MIN data
    /// * `block_index` - Block index within the piece
    /// * `forced_tile_type` - TileType to use for decoding (ignores block's stored type)
    /// 
    /// # Returns
    /// RGBA pixel data
    pub fn get_decoded_tile_with_type(
        &mut self,
        piece_index: usize,
        block_index: usize,
        forced_tile_type: crate::tiles::types::TileType,
    ) -> Result<&[u8]> {
        // Create a unique cache key that includes the forced type
        let cache_key = (piece_index << 24) | (block_index << 16) | ((forced_tile_type as usize) << 8);

        // Check cache first
        if self.decoded_cache.contains_key(&cache_key) {
            return Ok(&self.decoded_cache[&cache_key]);
        }
        
        // Get PieceMicros from MIN data
        let piece = self.min_data
            .get(piece_index)
            .ok_or_else(|| anyhow::anyhow!("Piece index out of range: {}", piece_index))?;
        
        // Get the specific block
        let block = piece.mt.get(block_index)
            .ok_or_else(|| anyhow::anyhow!("Block index out of range: {} (piece has {} blocks)", 
                block_index, piece.mt.len()))?;
        
        if !block.has_value() {
            // Empty tile - return transparent
            let size = 32 * 32 * 4;  // RGBA
            let transparent = vec![0u8; size];
            self.decoded_cache.insert(cache_key, transparent);
            return Ok(&self.decoded_cache[&cache_key]);
        }
        
        // Get frame index (use block's frame)
        let frame_idx = block.frame() as usize;
        
        // Get decoded indexed pixels using FORCED tile type
        let indexed_pixels = self.get_indexed_tile(frame_idx, forced_tile_type)?;
        
        // Clone indexed pixels so we can release the mutable borrow on self
        let indexed_pixels = indexed_pixels.to_vec();
        
        // Apply palette to convert to RGBA
        let rgba_pixels = self.palette.indices_to_rgba(&indexed_pixels, true);
        
        // Cache and return
        self.decoded_cache.insert(cache_key, rgba_pixels);
        Ok(&self.decoded_cache[&cache_key])
    }

    /// Get decoded foliage (grass) for floor tiles with TransparentSquare type
    /// 
    /// # Reference
    /// Original: `Source/engine/render/dun_render.hpp::GetDunFrameFoliage()` Line 132-135
    /// - Foliage data is at: GetDunFrame(frame) + ReencodedTriangleFrameSize
    /// - ReencodedTriangleFrameSize = 544 - 32 = 512 bytes
    /// - Foliage is 16×32 TransparentSquare (512 bytes decoded)
    /// 
    /// # Returns
    /// RGBA pixel data (16×32×4 = 2048 bytes)
    pub fn get_decoded_foliage(&mut self, piece_index: usize, block_index: usize) -> Result<&[u8]> {
        use crate::tiles::types::TileType;
        use crate::tiles::decoder::decode_tile;
        
        // Cache key for foliage (different from regular tile)
        let cache_key = (piece_index << 16) | (block_index << 8) | 0xFF;  // 0xFF marks as foliage
        
        // Check cache
        if self.decoded_cache.contains_key(&cache_key) {
            return Ok(&self.decoded_cache[&cache_key]);
        }
        
        // Get PieceMicros from MIN data
        let piece = self.min_data
            .get(piece_index)
            .ok_or_else(|| anyhow::anyhow!("Piece index out of range: {}", piece_index))?;
        
        // Get the specific block
        let block = piece.mt.get(block_index)
            .ok_or_else(|| anyhow::anyhow!("Block index out of range: {} (piece has {} blocks)", 
                block_index, piece.mt.len()))?;
        
        if !block.has_value() {
            // Empty block - return transparent
            let size = 16 * 32 * 4;  // RGBA for 16×32 foliage
            let transparent = vec![0u8; size];
            self.decoded_cache.insert(cache_key, transparent);
            return Ok(&self.decoded_cache[&cache_key]);
        }
        
        // Get frame index
        let frame_idx = block.frame() as usize;
        if frame_idx == 0 {
            bail!("Invalid frame index: 0 (frame indices are 1-based in MIN data)");
        }
        let array_idx = frame_idx - 1;  // Convert 1-based to 0-based
        
        // Determine which CEL to use
        let main_cel_len = self.cel_sprite.frames.len();
        let cel_frame = if array_idx < main_cel_len {
            &self.cel_sprite.frames[array_idx]
        } else if let Some(ref special_cel) = self.special_cel_sprite {
            let adjusted_idx = array_idx - main_cel_len;
            if adjusted_idx >= special_cel.frames.len() {
                bail!("Frame index out of range for foliage: {}", frame_idx);
            }
            &special_cel.frames[adjusted_idx]
        } else {
            bail!("Frame index out of range for foliage: {}", frame_idx);
        };
        
        // Get raw encoded data from CEL frame
        let raw_data = &cel_frame.raw_data;
        
        // Foliage data is at offset ReencodedTriangleFrameSize = 512 bytes
        // Reference: Source/levels/dun_tile.hpp:131
        const REENCODED_TRIANGLE_FRAME_SIZE: usize = 544 - 32;  // 512 bytes
        
        if raw_data.len() < REENCODED_TRIANGLE_FRAME_SIZE + 512 {
            // Not enough data for foliage (foliage is 512 bytes encoded)
            let size = 16 * 32 * 4;  // RGBA for 16×32 foliage
            let transparent = vec![0u8; size];
            self.decoded_cache.insert(cache_key, transparent);
            return Ok(&self.decoded_cache[&cache_key]);
        }
        
        // Extract foliage data (512 bytes after main tile data)
        let foliage_data = &raw_data[REENCODED_TRIANGLE_FRAME_SIZE..REENCODED_TRIANGLE_FRAME_SIZE + 512];
        
        // Decode foliage as TransparentSquare (16×32)
        let decoded = decode_tile(TileType::TransparentSquare, foliage_data)
            .with_context(|| format!(
                "Failed to decode foliage for frame {} (piece={}, block={})",
                frame_idx, piece_index, block_index
            ))?;
        
        // Foliage should be 16×32 = 512 bytes (indexed)
        if decoded.len() != 512 {
            bail!("Foliage decoded size mismatch: expected 512 bytes, got {}", decoded.len());
        }
        
        // Apply palette to convert to RGBA
        let rgba_pixels = self.palette.indices_to_rgba(&decoded, true);
        
        // Cache and return
        self.decoded_cache.insert(cache_key, rgba_pixels);
        Ok(&self.decoded_cache[&cache_key])
    }

    
    /// Get decoded tile as indexed colors (with caching)
    /// 
    /// # Arguments
    /// * `frame_idx` - Frame index in CEL file (1-based, from LevelCelBlock.frame())
    ///   - Frame 0 is invalid (represents empty/no tile)
    ///   - Frame 1 to main_cel_len: Use main CEL
    ///   - Frame main_cel_len+1+: Use special CEL (if available)
    /// * `tile_type` - Type of tile to decode
    /// 
    /// # Returns
    /// Indexed pixel data (1 byte per pixel, 0 = transparent)
    /// 
    /// # Note
    /// This is an internal method used by `get_decoded_tile()`.
    /// The indexed cache is separate from the RGBA cache.
    /// 
    /// # Reference
    /// C++: `Source/levels/dun_tile.hpp::LevelCelBlock::frame()` returns 1-based index
    /// C++: `Source/engine/render/dun_render.hpp::GetDunFrame()` uses frame directly as array index
    /// 
    /// # Frame Index Logic
    /// - frame_idx is 1-based (matches C++ LevelCelBlock.frame())
    /// - We convert to 0-based for Rust array access: array_idx = frame_idx - 1
    /// - Frames 1 to main_cel_len: Use main CEL (array_idx 0 to main_cel_len-1)
    /// - Frames main_cel_len+1+: Use special CEL (array_idx main_cel_len+)
    fn get_indexed_tile(&mut self, frame_idx: usize, tile_type: TileType) -> Result<&[u8]> {
        // Cache key includes both frame index and tile type
        // This is important because the same frame can be decoded differently for different tile types
        let cache_key = frame_idx * 8 + (tile_type as usize);
        
        // Check indexed cache
        if self.indexed_cache.contains_key(&cache_key) {
            return Ok(&self.indexed_cache[&cache_key]);
        }
        
        // frame_idx is 1-based (from LevelCelBlock.frame())
        // We need to convert it to 0-based for array access
        if frame_idx == 0 {
            bail!("Invalid frame index: 0 (frame indices are 1-based in MIN data)");
        }
        let array_idx = frame_idx - 1;  // Convert 1-based to 0-based
        
        // Determine which CEL to use and adjust index
        let main_cel_len = self.cel_sprite.frames.len();
        let special_cel_len = self.special_cel_sprite.as_ref().map(|s| s.frames.len()).unwrap_or(0);
        let total_frames = main_cel_len + special_cel_len;
        
        // ⚠️ COMPATIBILITY: Handle out-of-range frames gracefully
        // Some MIN data (especially Town) references frames that don't exist in the CEL files.
        // Original C++ likely had implicit handling or these pieces were never rendered.
        // Return transparent tile for out-of-range frames instead of erroring.
        if array_idx >= total_frames {
            static mut FRAME_OOR_COUNT: usize = 0;
            unsafe {
                FRAME_OOR_COUNT += 1;
                if FRAME_OOR_COUNT <= 3 {
                    eprintln!("⚠️ Frame out of range #{}: frame={} (0-based={}), total_frames={} (main={}, special={})", 
                        FRAME_OOR_COUNT, frame_idx, array_idx, total_frames, main_cel_len, special_cel_len);
                    eprintln!("   Returning transparent tile as fallback");
                }
            }
            
            // Return transparent tile (32x32, all zeros)
            let transparent_tile = vec![0u8; 32 * 32];
            self.indexed_cache.insert(cache_key, transparent_tile);
            return Ok(&self.indexed_cache[&cache_key]);
        }
        
        let (cel_frame, source_name) = if array_idx < main_cel_len {
            // Use main CEL
            (&self.cel_sprite.frames[array_idx], "main")
        } else {
            // Use special CEL
            let adjusted_idx = array_idx - main_cel_len;
            (&self.special_cel_sprite.as_ref().unwrap().frames[adjusted_idx], "special")
        };
        
        // Get raw data from CEL frame
        let raw_data = &cel_frame.raw_data;
        
        // ✅ KEY FIX: Check if data is already decoded (CLX special CEL files)
        let decoded = if cel_frame.is_decoded {
            // Special CEL from CLX format - already decoded, just clone
            raw_data.clone()
        } else {
            // Main CEL - needs decoding with appropriate TileType decoder
            // Reference: Source/levels/gendung.cpp::ReencodeDungeonCels()
            decode_tile(tile_type, raw_data)
                .with_context(|| format!(
                    "Failed to decode frame {} from {} CEL (type={:?})",
                    frame_idx, source_name, tile_type
                ))?
        };
        
        // Cache and return
        self.indexed_cache.insert(cache_key, decoded);
        Ok(&self.indexed_cache[&cache_key])
    }
    
    /// Preload all tiles into cache
    /// 
    /// This can be called once at startup to avoid runtime decoding overhead.
    /// 
    /// # Returns
    /// Number of tiles successfully preloaded
    pub fn preload_all(&mut self) -> Result<usize> {
        println!("\n=== Preloading Tiles ===");
        let num_pieces = self.min_data.len();
        let blocks_per_piece = self.min_data.blocks_per_piece;
        let mut loaded = 0;
        let mut errors = 0;
        
        for piece_idx in 0..num_pieces {
            for block_idx in 0..blocks_per_piece {
                match self.get_decoded_tile(piece_idx, block_idx) {
                    Ok(_) => loaded += 1,
                    Err(e) => {
                        if errors < 5 {
                            eprintln!("Failed to preload tile ({}, {}): {}", piece_idx, block_idx, e);
                        }
                        errors += 1;
                    }
                }
            }
        }
        
        println!("  Preloaded: {} tiles", loaded);
        println!("  Errors: {}", errors);
        println!("  Cache size: {} decoded, {} indexed",
            self.decoded_cache.len(), self.indexed_cache.len());
        
        Ok(loaded)
    }
    
    /// Get a CEL frame by index (supports both main and special CEL)
    /// 
    /// # Arguments
    /// * `frame_idx` - Frame index (1-based, as stored in MIN file)
    ///   - Frame 1 to main_cel_count: Use main CEL
    ///   - Frame main_cel_count+1+: Use special CEL
    /// 
    /// # Returns
    /// Reference to the frame data
    /// 
    /// # Reference
    /// Original: Source/levels/gendung.cpp::GetDunFrame() Line 1165-1180
    pub fn get_frame(&self, frame_idx: usize) -> Result<&crate::resources::dungeon_cel::DungeonCelFrame> {
        if frame_idx == 0 {
            bail!("Invalid frame index: 0 (frame indices are 1-based)");
        }
        
        let array_idx = frame_idx - 1;  // Convert 1-based to 0-based
        let main_count = self.cel_sprite.frames.len();
        
        if array_idx < main_count {
            // Main CEL
            Ok(&self.cel_sprite.frames[array_idx])
        } else {
            // Special CEL
            let special = self.special_cel_sprite.as_ref()
                .ok_or_else(|| anyhow!("No special CEL loaded"))?;
            
            let special_idx = array_idx - main_count;
            if special_idx >= special.frames.len() {
                bail!(
                    "Frame index {} out of range (main has {} frames, special has {} frames)",
                    frame_idx, main_count, special.frames.len()
                );
            }
            
            Ok(&special.frames[special_idx])
        }
    }
    
    /// Get number of pieces (levelPieceIds)
    pub fn len(&self) -> usize {
        self.min_data.len()
    }
    
    /// Get a piece by index (for checking block.has_value())
    pub fn get_piece(&self, index: usize) -> Option<&crate::tiles::min::PieceMicros> {
        self.min_data.get(index)
    }
    
    /// Get number of blocks per piece
    pub fn blocks_per_piece(&self) -> usize {
        self.min_data.blocks_per_piece
    }
    
    /// Get palette reference (for line-by-line rendering)
    pub fn palette(&self) -> &Palette {
        &self.palette
    }
    
    /// Get raw encoded tile data (for line-by-line rendering)
    /// 
    /// # Arguments
    /// * `piece_index` - Piece index (levelPieceId)
    /// * `block_index` - Block index within the piece
    /// * `forced_tile_type` - Tile type to use (for floor tiles, this is forced)
    /// 
    /// # Returns
    /// Raw encoded data bytes (not decoded)
    /// 
    /// # Reference
    /// Used by C++ RenderTileFrame to get src pointer for line-by-line rendering
    pub fn get_raw_tile_data(
        &self,
        piece_index: usize,
        block_index: usize,
        forced_tile_type: Option<TileType>,
    ) -> Result<&[u8]> {
        // Get PieceMicros from MIN data
        let piece = self.min_data
            .get(piece_index)
            .ok_or_else(|| anyhow::anyhow!("Piece index out of range: {}", piece_index))?;
        
        // Get the specific block
        let block = piece.mt.get(block_index)
            .ok_or_else(|| anyhow::anyhow!("Block index out of range: {} (piece has {} blocks)", 
                block_index, piece.mt.len()))?;
        
        if !block.has_value() {
            bail!("Block has no value");
        }
        
        // Get frame index
        let frame_idx = block.frame() as usize;
        
        // Use forced tile type if provided, otherwise use block's tile type
        let _tile_type = forced_tile_type.unwrap_or_else(|| block.tile_type());
        
        // Convert 1-based frame index to 0-based array index
        if frame_idx == 0 {
            bail!("Invalid frame index: 0 (frame indices are 1-based in MIN data)");
        }
        let array_idx = frame_idx - 1;
        
        // Determine which CEL to use
        let main_cel_len = self.cel_sprite.frames.len();
        let cel_frame = if array_idx < main_cel_len {
            &self.cel_sprite.frames[array_idx]
        } else if let Some(ref special_cel) = self.special_cel_sprite {
            let adjusted_idx = array_idx - main_cel_len;
            if adjusted_idx >= special_cel.frames.len() {
                bail!("Frame index out of range: {}", frame_idx);
            }
            &special_cel.frames[adjusted_idx]
        } else {
            bail!("Frame index out of range: {}", frame_idx);
        };
        
        // Return raw encoded data
        Ok(&cel_frame.raw_data)
    }
    
    /// Get total number of CEL frames (main + special)
    pub fn total_frames(&self) -> usize {
        self.cel_sprite.frames.len() + 
            self.special_cel_sprite.as_ref().map_or(0, |s| s.frames.len())
    }
    
    /// Check if tileset is empty
    pub fn is_empty(&self) -> bool {
        self.min_data.is_empty()
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            decoded_count: self.decoded_cache.len(),
            indexed_count: self.indexed_cache.len(),
            total_tiles: self.min_data.len(),
        }
    }
    
    /// Clear all caches (to free memory)
    pub fn clear_cache(&mut self) {
        self.decoded_cache.clear();
        self.indexed_cache.clear();
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub decoded_count: usize,
    pub indexed_count: usize,
    pub total_tiles: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f32 {
        if self.total_tiles == 0 {
            0.0
        } else {
            self.decoded_count as f32 / self.total_tiles as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_texture_manager_basic() {
        // This is a placeholder test - real testing requires MPQ files
        // See tests/tile_texture_tests.rs for integration tests
    }
    
    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            decoded_count: 50,
            indexed_count: 50,
            total_tiles: 100,
        };
        
        assert_eq!(stats.hit_rate(), 0.5);
    }
    
    #[test]
    fn test_cache_stats_empty() {
        let stats = CacheStats {
            decoded_count: 0,
            indexed_count: 0,
            total_tiles: 0,
        };
        
        assert_eq!(stats.hit_rate(), 0.0);
    }
    
    #[test]
    fn test_get_special_cel_paths() {
        assert!(get_special_cel_paths(DungeonType::Town).contains(&"levels/towndata/towns.clx"));
        assert!(get_special_cel_paths(DungeonType::Cathedral).contains(&"levels/l1data/l1s.cel"));
        assert!(get_special_cel_paths(DungeonType::Catacombs).contains(&"levels/l2data/l2s.cel"));
        assert!(get_special_cel_paths(DungeonType::Caves).contains(&"levels/l3data/l3s.cel"));
        assert!(get_special_cel_paths(DungeonType::Hell).contains(&"levels/l4data/l4s.cel"));
    }
    
    #[test]
    #[ignore]  // Requires MPQ files
    fn test_load_cathedral_special_cel() {
        // This test requires actual game assets
        // Run with: cargo test --ignored test_load_cathedral_special_cel
        
        let mut mpq = MpqManager::new();
        
        // Try to load the MPQ
        let mpq_paths = vec![
            "assets/Diabdat.mpq",
            "../assets/Diabdat.mpq",
            "../../assets/Diabdat.mpq",
        ];
        
        let mut mpq_loaded = false;
        for path in mpq_paths {
            if std::path::Path::new(path).exists() {
                if mpq.load_mpq(path, 1000).is_ok() {
                    mpq_loaded = true;
                    break;
                }
            }
        }
        
        if !mpq_loaded {
            println!("⚠ Skipping test: MPQ file not found");
            return;
        }
        
        // Load Cathedral tileset
        let tex_mgr = TileTextureManager::load_for_dungeon(
            DungeonType::Cathedral,
            &mut mpq
        ).unwrap();
        
        // Verify main CEL loaded
        assert!(tex_mgr.cel_sprite.frames.len() > 0);
        
        // Verify special CEL loaded (should be Some for Cathedral)
        if let Some(ref special) = tex_mgr.special_cel_sprite {
            assert!(special.frames.len() > 0);
            println!("✓ Special CEL loaded: {} frames", special.frames.len());
            
            // Try to get a frame from special CEL
            let main_count = tex_mgr.cel_sprite.frames.len();
            let special_frame_idx = main_count + 1;  // First frame in special CEL
            
            let frame_result = tex_mgr.get_frame(special_frame_idx);
            assert!(frame_result.is_ok(), "Should be able to access special CEL frames");
            
            if let Ok(frame) = frame_result {
                assert!(!frame.raw_data.is_empty(), "Special CEL frame should have data");
            }
        } else {
            println!("⚠ Special CEL not loaded (may not exist in MPQ)");
        }
    }
    
    #[test]
    fn test_get_frame_invalid_index() {
        // Create a minimal texture manager for testing
        // This would need proper setup in a real scenario
        // For now, just test the path resolution
        
        // Test that frame index 0 is invalid
        // (actual test would require a real TileTextureManager instance)
    }
}

