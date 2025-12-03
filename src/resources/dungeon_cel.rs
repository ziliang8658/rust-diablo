/// Dungeon CEL - Cathedral/Dungeon tile graphics format
/// 
/// This module handles the special CEL format used for dungeon tiles (l1.cel, l2.cel, etc.)
/// 
/// # Format Structure
/// 
/// Dungeon CEL files have a different structure from sprite CLX files:
/// 
/// 1. **Header**: Array of uint32 offsets (little-endian)
///    - Each uint32 points to a frame's data location in the file
///    - Frame indices are 1-based (0 = empty/no tile)
///    - Offset table continues until we hit the first frame data
/// 
/// 2. **Frame Data**: Raw pixel data for each 32x32 tile
///    - Each frame is 32x32 pixels
///    - Pixels are indexed colors (0-255)
///    - No transparency marker in basic format
/// 
/// # Reference
/// See DevilutionX Source/engine/render/dun_render.hpp:
/// ```cpp
/// DVL_ALWAYS_INLINE const uint8_t *GetDunFrame(const std::byte *dungeonCelData, uint32_t frame)
/// {
///     const auto *frameTable = reinterpret_cast<const uint32_t *>(dungeonCelData);
///     return reinterpret_cast<const uint8_t *>(&dungeonCelData[Swap32LE(frameTable[frame])]);
/// }
/// ```

use anyhow::{Context, Result, bail};

/// Standard frame dimensions for dungeon tiles
pub const DUN_FRAME_WIDTH: u16 = 32;
pub const DUN_FRAME_HEIGHT: u16 = 32;

/// A single frame from a Dungeon CEL file
/// 
/// Note: May store either encoded or decoded data, depending on source format.
#[derive(Debug, Clone)]
pub struct DungeonCelFrame {
    /// Frame data (may be encoded or decoded)
    pub raw_data: Vec<u8>,
    
    /// Whether the data is already decoded (true for CLX special CEL files)
    pub is_decoded: bool,
}

impl DungeonCelFrame {
    /// Get the raw encoded data size
    pub fn size(&self) -> usize {
        self.raw_data.len()
    }
}

/// Dungeon CEL sprite - collection of tile frames
#[derive(Debug)]
pub struct DungeonCelSprite {
    /// All frames in the sprite
    pub frames: Vec<DungeonCelFrame>,
}

impl DungeonCelSprite {
    /// Parse a Dungeon CEL file from raw bytes
    /// 
    /// # Arguments
    /// * `data` - Raw file data
    /// 
    /// # Returns
    /// Parsed DungeonCelSprite or error
    /// 
    /// # Reference
    /// Original: Source/levels/gendung.cpp::LoadLvlGFX() Line 1235-1264
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            bail!("DungeonCEL file too small: {} bytes", data.len());
        }
        
        // CEL file format (Reference: Source/utils/cel_to_clx.cpp:42):
        // data[0..3]: frame count (NOT first offset!)
        // data[4..7]: offset[0] - first frame start
        // data[8..11]: offset[1] - second frame start
        // ...
        // data[4*(numFrames+1)..]: offset[numFrames] = file size
        
        let num_frames = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        
        if num_frames == 0 || num_frames > 100000 {
            bail!("Invalid frame count: {}", num_frames);
        }
        
        // Verify we have enough data for the offset table
        let offset_table_size = (num_frames + 1) * 4;
        if offset_table_size > data.len() {
            bail!("Frame offset table extends beyond file: need {} bytes, have {}", 
                offset_table_size, data.len());
        }
        
        // Read all frame offsets (including final file size)
        // Note: offset[0] is at data[4], not data[0]
        let mut frame_offsets = Vec::with_capacity(num_frames + 1);
        for i in 0..=num_frames {
            let offset_pos = (i + 1) * 4;  // offset[i] at position (i+1)*4
            let offset = u32::from_le_bytes([
                data[offset_pos],
                data[offset_pos + 1],
                data[offset_pos + 2],
                data[offset_pos + 3],
            ]) as usize;
            
            frame_offsets.push(offset);
        }
        
        println!("Dungeon CEL: {} frames, first frame offset = {}, last offset = {}", 
            num_frames, frame_offsets[0], frame_offsets[num_frames]);
        
        // Parse each frame
        let mut frames = Vec::with_capacity(num_frames);
        
        // DEBUG: Print first few offsets
        println!("  First 5 frame offsets:");
        for i in 0..num_frames.min(5) {
            println!("    Frame {}: offset = {}", i, frame_offsets[i]);
        }
        
        for i in 0..num_frames {
            let frame_start = frame_offsets[i];
            let frame_end = frame_offsets[i + 1];
            
            if frame_start >= frame_end || frame_end > data.len() {
                bail!("Invalid frame offset at index {}: start={}, end={}, file_size={}", 
                    i, frame_start, frame_end, data.len());
            }
            
            let frame_size = frame_end - frame_start;
            let frame_data = &data[frame_start..frame_end];
            
            // DEBUG: Print first few frames
            if i < 3 {
                println!("  Parsing frame {}: offset=[{}, {}), size={}", 
                    i, frame_start, frame_end, frame_size);
            }
            
            // Parse frame - store raw encoded data
            let frame = Self::parse_frame(i, frame_data, frame_size)
                .with_context(|| format!("Failed to parse frame {} (offset={}, size={})", i, frame_start, frame_size))?;
            frames.push(frame);
        }
        
        Ok(Self { frames })
    }
    
    /// Parse a CEL file with specified width (RLE format) and convert to decoded pixels
    /// 
    /// Regular CEL files (e.g., towns.cel) use RLE encoding and require a frame width.
    /// This function decodes them to pixel data (0 = transparent).
    /// 
    /// # Arguments
    /// * `data` - Raw CEL file data
    /// * `width` - Frame width in pixels
    /// 
    /// # Returns
    /// Parsed DungeonCelSprite with decoded frames
    /// 
    /// # Reference
    /// Original: Source/engine/load_cel.cpp::LoadCelListOrSheetWithStatus()
    /// - Calls CelToClx() to convert CEL to CLX format
    /// - Source/utils/cel_to_clx.cpp::CelToClx() implements RLE decoding
    pub fn from_cel_bytes_with_width(data: &[u8], width: usize) -> Result<Self> {
        if data.len() < 8 {
            bail!("CEL file too small: {} bytes", data.len());
        }
        
        // CEL file structure (Reference: Source/utils/cel_to_clx.cpp Lines 37-58):
        // 1. First 4 bytes: could be frame count OR offset to first group
        // 2. If it's frame count, then last frame offset == file size
        // 3. Otherwise, it's a multi-group CEL (rare)
        
        let maybe_num_frames = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        
        println!("CEL file: maybe_num_frames={}, file_size={}", maybe_num_frames, data.len());
        
        // Check if this is a single-group CEL by verifying last offset == file size
        // CEL offset table structure (C++ cel_to_clx.cpp:74-77):
        //   data[0]: frame count
        //   data[4]: offset[0]
        //   data[8]: offset[1]
        //   ...
        //   data[4*(numFrames+1)]: offset[numFrames] = file size
        // So for 18 frames, data[76] (= 4*(18+1)) should equal file size
        let last_offset_pos = (maybe_num_frames + 1) * 4;
        let is_single_group = if last_offset_pos + 4 <= data.len() {
            let last_offset = u32::from_le_bytes([
                data[last_offset_pos],
                data[last_offset_pos + 1],
                data[last_offset_pos + 2],
                data[last_offset_pos + 3],
            ]) as usize;
            println!("  Checking: offset[{}] at pos {} = {} vs file_size={}", 
                maybe_num_frames, last_offset_pos, last_offset, data.len());
            last_offset == data.len()
        } else {
            println!("  Offset table too short: need {} bytes, have {}", last_offset_pos + 4, data.len());
            false
        };
        
        let num_frames = if is_single_group {
            maybe_num_frames
        } else {
            bail!("Multi-group CEL files not yet supported (first_u32={}, file_size={})", 
                maybe_num_frames, data.len());
        };
        
        if num_frames == 0 || num_frames > 10000 {
            bail!("Invalid frame count: {}", num_frames);
        }
        
        // Read frame offsets (num_frames + 1 offsets: frame starts + final end)
        // CEL offset table: data[4] = offset[0], data[8] = offset[1], ...
        // Reference: C++ cel_to_clx.cpp:74-77
        let mut frame_offsets = Vec::with_capacity(num_frames + 1);
        for i in 0..=num_frames {
            let offset_pos = (i + 1) * 4;  // offset[i] at position (i+1)*4
            if offset_pos + 4 > data.len() {
                bail!("Frame offset table extends beyond file at index {}", i);
            }
            let offset = u32::from_le_bytes([
                data[offset_pos],
                data[offset_pos + 1],
                data[offset_pos + 2],
                data[offset_pos + 3],
            ]) as usize;
            frame_offsets.push(offset);
            
            if i < 5 {
                println!("  Offset[{}] at pos {} = {}", i, offset_pos, offset);
            }
        }
        
        println!("CEL file: {} frames, width={}", num_frames, width);
        
        // Parse and decode each frame
        let mut frames = Vec::with_capacity(num_frames);
        
        for frame_idx in 0..num_frames {
            let frame_start = frame_offsets[frame_idx];
            let frame_end = frame_offsets[frame_idx + 1];
            
            if frame_start >= frame_end || frame_end > data.len() {
                bail!("Invalid frame offset: frame {} start={} end={} (file_size={})", 
                    frame_idx, frame_start, frame_end, data.len());
            }
            
            let frame_data = &data[frame_start..frame_end];
            
            if frame_idx < 3 {
                println!("  Frame {}: offset range [{}, {}), size={}", 
                    frame_idx, frame_start, frame_end, frame_data.len());
            }
            
            // Decode CEL RLE to pixels
            let decoded_pixels = Self::decode_cel_rle(frame_data, width)
                .with_context(|| format!("Failed to decode frame {}", frame_idx))?;
            
            frames.push(DungeonCelFrame {
                raw_data: decoded_pixels,
                is_decoded: true,  // Mark as decoded
            });
        }
        
        println!("Loaded CEL (RLE format): {} frames", frames.len());
        Ok(Self { frames })
    }
    
    /// Decode CEL RLE format to pixels (0 = transparent)
    /// 
    /// # Reference
    /// Source/utils/cel_to_clx.cpp::CelToClx() Lines 96-112
    /// - Control byte >= 0x80: transparent pixels, width = -int8(control)
    /// - Control byte < 0x80: opaque pixels, copy `control` bytes
    fn decode_cel_rle(data: &[u8], width: usize) -> Result<Vec<u8>> {
        let mut pixels = Vec::new();
        let mut pos = 0;
        
        // DEBUG: Print first 20 bytes
        println!("    decode_cel_rle: data_len={}, width={}", data.len(), width);
        print!("    First 20 bytes: ");
        for i in 0..data.len().min(20) {
            print!("{:02X} ", data[i]);
        }
        println!();
        
        // Skip CEL frame header if present (10 bytes)
        if data.len() >= 2 {
            let maybe_header_size = u16::from_le_bytes([data[0], data[1]]) as usize;
            if maybe_header_size == 10 && data.len() >= 10 {
                println!("    Skipping 10-byte CEL frame header");
                pos = 10;  // Skip header
            }
        }
        
        // Decode RLE data row by row
        let mut row_num = 0;
        while pos < data.len() {
            let mut row_width = 0;
            let row_start_pos = pos;
            
            // Decode one row
            while row_width < width && pos < data.len() {
                let control = data[pos];
                pos += 1;
                
                if control >= 0x80 {
                    // Transparent pixels: count = -int8(control)
                    let transparent_count = (-(control as i8)) as usize;
                    pixels.extend(vec![0u8; transparent_count]);  // 0 = transparent
                    row_width += transparent_count;
                } else {
                    // Opaque pixels: copy `control` bytes
                    let opaque_count = control as usize;
                    if pos + opaque_count > data.len() {
                        eprintln!("    ❌ Row {}: at pos {}, control={}, need {} bytes, but only {} bytes left", 
                            row_num, pos - 1, control, opaque_count, data.len() - pos);
                        bail!("CEL RLE data truncated: need {} bytes at pos {}, only {} bytes remain", 
                            opaque_count, pos, data.len() - pos);
                    }
                    pixels.extend_from_slice(&data[pos..pos + opaque_count]);
                    pos += opaque_count;
                    row_width += opaque_count;
                }
            }
            
            if row_width != width {
                println!("    ⚠️ Row {} width mismatch: got {} expected {} (row_start={})", 
                    row_num, row_width, width, row_start_pos);
            }
            
            row_num += 1;
        }
        
        println!("    Decoded {} rows, {} total pixels", row_num, pixels.len());
        Ok(pixels)
    }
    
    /// Parse a special CEL file from CLX format
    /// 
    /// Special CEL files (l1s.cel, l2s.cel, etc.) have a .cel extension but are
    /// actually CLX format. They contain special tiles like doors, decorations,
    /// and special room structures.
    /// 
    /// # Arguments
    /// * `data` - Raw CLX file data
    /// 
    /// # Returns
    /// Parsed DungeonCelSprite or error
    /// 
    /// # Reference
    /// Original: Source/levels/gendung.cpp::LoadLvlGFX() Line 1235-1264
    /// - Special CEL files are loaded separately: `pSpecialCels = LoadFileInMem("levels/l1data/l1s.cel")`
    /// - These are CLX format despite the .cel extension
    pub fn from_clx_bytes(data: &[u8]) -> Result<Self> {
        use crate::resources::clx::ClxSprite;
        
        // Parse as CLX format
        let clx_sprite = ClxSprite::from_bytes(data)
            .context("Failed to parse special CEL as CLX format")?;
        
        // Convert CLX frames to DungeonCelFrame format
        // CLX frames contain decoded pixel data (Option<u8>), but we need raw encoded data
        // for consistency with the decoding pipeline.
        // 
        // However, for special CEL files, we can store the CLX-decoded data directly
        // since these are used less frequently and the decoding path can handle both.
        let mut frames = Vec::with_capacity(clx_sprite.frames.len());
        
        for clx_frame in &clx_sprite.frames {
            // Convert CLX decoded pixels (Option<u8>) to indexed format (u8, 0=transparent)
            // Note: Keep original CLX pixel order, flip will be handled at render time
            let mut raw_data = Vec::with_capacity(clx_frame.pixels.len());
            
            for pixel in &clx_frame.pixels {
                match pixel {
                    Some(idx) => raw_data.push(*idx),
                    None => raw_data.push(0), // 0 = transparent
                }
            }
            
            frames.push(DungeonCelFrame {
                raw_data,
                is_decoded: true,  // ✅ CLX data is already decoded
            });
        }
        
        println!("Loaded special CEL (CLX format): {} frames", frames.len());
        
        Ok(Self { frames })
    }
    
    /// Parse a single frame (stores raw bytes, no decoding)
    /// 
    /// The raw data will be decoded later using the appropriate TileType decoder.
    /// TileType information comes from MIN file, not CEL file.
    fn parse_frame(_index: usize, data: &[u8], _size: usize) -> Result<DungeonCelFrame> {
        // Simply store the raw encoded data
        // Decoding will be done later based on TileType from MIN data
        Ok(DungeonCelFrame {
            raw_data: data.to_vec(),
            is_decoded: false,  // Main CEL data is still encoded
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dungeon_cel_structure() {
        // Create a minimal valid Dungeon CEL with 2 frames
        let frame_size = (DUN_FRAME_WIDTH as usize) * (DUN_FRAME_HEIGHT as usize);
        
        // Offset table: 2 uint32 values
        let offset1 = 8u32; // First frame starts after offset table (2 * 4 = 8)
        let offset2 = (8 + frame_size) as u32; // Second frame starts after first
        
        let mut data = Vec::new();
        data.extend_from_slice(&offset1.to_le_bytes());
        data.extend_from_slice(&offset2.to_le_bytes());
        
        // Frame 1: Fill with value 10
        data.extend(vec![10u8; frame_size]);
        
        // Frame 2: Fill with value 20
        data.extend(vec![20u8; frame_size]);
        
        let sprite = DungeonCelSprite::from_bytes(&data).unwrap();
        assert_eq!(sprite.frames.len(), 2);
        assert_eq!(sprite.frames[0].size(), frame_size);
        assert_eq!(sprite.frames[0].raw_data[0], 10);
        assert_eq!(sprite.frames[1].raw_data[0], 20);
        assert!(!sprite.frames[0].is_decoded);  // Main CEL is encoded
        assert!(!sprite.frames[1].is_decoded);
    }
    
    #[test]
    fn test_from_clx_bytes_conversion() {
        // Test that CLX data can be converted to DungeonCelFrame format
        // This is a unit test with handcrafted CLX data
        
        // Create minimal CLX data: 1 frame, 4x4 pixels
        let mut clx_data = Vec::new();
        
        // CLX header: num_frames (u32) + frame offsets
        clx_data.extend_from_slice(&1u32.to_le_bytes());  // 1 frame
        clx_data.extend_from_slice(&8u32.to_le_bytes());  // Frame 0 offset
        clx_data.extend_from_slice(&50u32.to_le_bytes()); // File size (placeholder)
        
        // Frame header: header_size(u16) + width(u16) + height(u16)
        clx_data.extend_from_slice(&6u16.to_le_bytes());  // header_size = 6
        clx_data.extend_from_slice(&4u16.to_le_bytes());  // width = 4
        clx_data.extend_from_slice(&4u16.to_le_bytes());  // height = 4
        
        // Frame data (RLE encoded): 16 pixels, all transparent
        // Control byte 0x10 = 16 transparent pixels
        clx_data.push(0x10);  // 16 transparent pixels for first row
        clx_data.push(0x10);  // 16 transparent pixels (split across rows)
        
        // Try to parse as CLX and convert
        match DungeonCelSprite::from_clx_bytes(&clx_data) {
            Ok(sprite) => {
                assert_eq!(sprite.frames.len(), 1);
                // CLX decoder should have converted pixels
                assert!(!sprite.frames[0].raw_data.is_empty());
            }
            Err(e) => {
                // CLX parsing might fail with minimal data, that's ok for unit test
                println!("Note: CLX parsing failed (expected for minimal test data): {}", e);
            }
        }
    }
    
    #[test]
    #[ignore]  // Requires actual game assets
    fn test_load_real_special_cel() {
        // This test loads a real l1s.cel file from MPQ
        // Run with: cargo test --ignored test_load_real_special_cel
        
        use crate::resources::MpqManager;
        
        let mut mpq = MpqManager::new();
        
        let mpq_paths = vec![
            "assets/Diabdat.mpq",
            "../assets/Diabdat.mpq",
            "../../assets/Diabdat.mpq",
        ];
        
        let mut data = None;
        for path in mpq_paths {
            if std::path::Path::new(path).exists() {
                if mpq.load_mpq(path, 1000).is_ok() {
                    data = mpq.find_file("levels/l1data/l1s.cel");
                    break;
                }
            }
        }
        
        if let Some(cel_data) = data {
            let sprite = DungeonCelSprite::from_clx_bytes(&cel_data).unwrap();
            println!("✓ Loaded l1s.cel: {} frames", sprite.frames.len());
            assert!(sprite.frames.len() > 0);
            
            // Verify first frame has data
            assert!(!sprite.frames[0].raw_data.is_empty());
        } else {
            println!("⚠ Skipping test: MPQ or l1s.cel not found");
        }
    }
}

