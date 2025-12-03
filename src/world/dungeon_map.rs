/// Dungeon Map Data Structures
/// 
/// This module implements the map data structures equivalent to C++:
/// - `dungeon[DMAXX][DMAXY]` (40x40) - MegaTile indices
/// - `dPiece[MAXDUNX][MAXDUNY]` (112x112) - levelPieceId for each MicroTile position
/// 
/// # Reference
/// - Original: `Source/levels/gendung.cpp` Line 40, 60
/// - Original: `Source/levels/gendung_defs.hpp` Line 5-9

/// MegaTile map dimensions (40x40)
pub const DMAXX: usize = 40;
pub const DMAXY: usize = 40;

/// MicroTile map dimensions (112x112)
/// Formula: 16 + DMAX * 2 + 16
pub const MAXDUNX: usize = 16 + DMAXX * 2 + 16; // 112
pub const MAXDUNY: usize = 16 + DMAXY * 2 + 16; // 112

/// Dungeon map structure
/// 
/// Holds the map data for rendering:
/// - `dungeon`: MegaTile indices (1-based, 0 = empty)
/// - `d_piece`: Expanded levelPieceId for each MicroTile position
/// 
/// # Reference
/// Original: `Source/levels/gendung.cpp` Line 40, 60
pub struct DungeonMap {
    /// MegaTile indices (40x40), 1-based (0 = empty)
    /// Corresponds to C++ `dungeon[DMAXX][DMAXY]`
    pub dungeon: [[u8; DMAXY]; DMAXX],
    
    /// Expanded levelPieceId for each MicroTile position (112x112)
    /// Corresponds to C++ `dPiece[MAXDUNX][MAXDUNY]`
    pub d_piece: [[u16; MAXDUNY]; MAXDUNX],
}

impl DungeonMap {
    /// Create a new empty dungeon map
    pub fn new() -> Self {
        Self {
            dungeon: [[0; DMAXY]; DMAXX],
            d_piece: [[0; MAXDUNY]; MAXDUNX],
        }
    }
    
    /// Expand MegaTile map to MicroTile map
    /// 
    /// This is equivalent to C++ `DRLG_LPass3()` in `Source/levels/gendung.cpp` Line 768-801
    /// 
    /// # Arguments
    /// * `til_data` - TIL data containing MegaTile definitions
    /// * `default_tile` - Default tile index to fill borders
    /// 
    /// # Process
    /// 1. Fill entire dPiece with default tile's micro indices
    /// 2. For each dungeon[i][j], look up MegaTile and expand to 2x2 dPiece entries
    pub fn expand_to_d_piece(&mut self, til_data: &crate::tiles::TilData, default_tile: usize) {
        // Step 1: Fill with default tile (like C++ lines 771-785)
        if let Some(default_mega) = til_data.get(default_tile) {
            let v1 = default_mega.micro1;
            let v2 = default_mega.micro2;
            let v3 = default_mega.micro3;
            let v4 = default_mega.micro4;
            
            for j in (0..MAXDUNY).step_by(2) {
                for i in (0..MAXDUNX).step_by(2) {
                    self.d_piece[i][j] = v1;
                    self.d_piece[i + 1][j] = v2;
                    self.d_piece[i][j + 1] = v3;
                    self.d_piece[i + 1][j + 1] = v4;
                }
            }
        }
        
        // Step 2: Expand dungeon to dPiece (like C++ lines 787-800)
        // dPiece coordinates start at offset 16 from dungeon coordinates
        let mut yy = 16usize;
        for j in 0..DMAXY {
            let mut xx = 16usize;
            for i in 0..DMAXX {
                let tile_id = self.dungeon[i][j];
                if tile_id > 0 {
                    // dungeon is 1-based, convert to 0-based for TIL lookup
                    let tile_idx = (tile_id - 1) as usize;
                    if let Some(mega) = til_data.get(tile_idx) {
                        self.d_piece[xx][yy] = mega.micro1;
                        self.d_piece[xx + 1][yy] = mega.micro2;
                        self.d_piece[xx][yy + 1] = mega.micro3;
                        self.d_piece[xx + 1][yy + 1] = mega.micro4;
                    }
                }
                xx += 2;
            }
            yy += 2;
        }
    }
    
    /// Get levelPieceId at dPiece coordinates
    /// 
    /// # Arguments
    /// * `x` - X coordinate (0-111)
    /// * `y` - Y coordinate (0-111)
    /// 
    /// # Returns
    /// levelPieceId at the given position, or 0 if out of bounds
    pub fn get_piece(&self, x: i32, y: i32) -> u16 {
        if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
            self.d_piece[x as usize][y as usize]
        } else {
            0
        }
    }
    
    /// Check if coordinates are within dungeon bounds
    /// 
    /// # Reference
    /// Original: `Source/levels/gendung.h` Line 200
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32
    }
    
    /// Set a MegaTile in the dungeon map
    /// 
    /// # Arguments
    /// * `x` - X coordinate (0-39)
    /// * `y` - Y coordinate (0-39)
    /// * `tile_id` - MegaTile index (1-based)
    pub fn set_dungeon_tile(&mut self, x: usize, y: usize, tile_id: u8) {
        if x < DMAXX && y < DMAXY {
            self.dungeon[x][y] = tile_id;
        }
    }
}

impl Default for DungeonMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Load dungeon from a .dun file
/// 
/// .dun file format:
/// - First 2 bytes: width (uint16_t)
/// - Next 2 bytes: height (uint16_t)
/// - Remaining: tile data (width * height uint16_t values)
/// 
/// # Reference
/// Original: `Source/levels/gendung.cpp` PlaceDunTiles() Line 690-707
/// 
/// # Arguments
/// * `mpq_manager` - MPQ manager for file access
/// * `path` - Path to .dun file within MPQ
/// * `floor_id` - Default floor tile ID for empty tiles (0 = don't fill)
/// * `dirt_id` - Default dirt tile ID for initial fill
/// 
/// # Returns
/// A DungeonMap filled with data from the .dun file
pub fn load_dun_file(
    mpq_manager: &mut crate::resources::MpqManager,
    path: &str,
    floor_id: u8,
    dirt_id: u8,
) -> Result<DungeonMap, String> {
    // Load file data
    let data = mpq_manager.find_file(path)
        .ok_or_else(|| format!("Failed to load .dun file: {}", path))?;
    
    if data.len() < 4 {
        return Err(format!(".dun file too small: {} bytes", data.len()));
    }
    
    // Parse header: width and height (little-endian uint16)
    let width = u16::from_le_bytes([data[0], data[1]]) as usize;
    let height = u16::from_le_bytes([data[2], data[3]]) as usize;
    
    println!("Loading .dun file: {} ({}x{})", path, width, height);
    
    let expected_size = 4 + width * height * 2;
    if data.len() < expected_size {
        return Err(format!(
            ".dun file size mismatch: expected {} bytes, got {}",
            expected_size, data.len()
        ));
    }
    
    // Create dungeon map
    let mut dungeon_map = DungeonMap::new();
    
    // Fill with dirt_id first (like C++ memset(dungeon, dirtId, sizeof(dungeon)))
    for j in 0..DMAXY {
        for i in 0..DMAXX {
            dungeon_map.dungeon[i][j] = dirt_id;
        }
    }
    
    // Parse tile data
    // C++ PlaceDunTiles: tileLayer = &dunData[2], then tileLayer[j * size.width + i]
    let tile_layer = &data[4..];
    
    for j in 0..height {
        for i in 0..width {
            if i >= DMAXX || j >= DMAXY {
                continue;
            }
            
            let idx = (j * width + i) * 2;
            let tile_id = u16::from_le_bytes([tile_layer[idx], tile_layer[idx + 1]]) as u8;
            
            if tile_id != 0 {
                dungeon_map.dungeon[i][j] = tile_id;
            } else if floor_id != 0 {
                dungeon_map.dungeon[i][j] = floor_id;
            }
        }
    }
    
    // Debug: Print some dungeon data
    println!("=== Dungeon from .dun file ===");
    for y in 0..height.min(15) {
        let row: Vec<String> = (0..width.min(20))
            .map(|x| format!("{:3}", dungeon_map.dungeon[x][y]))
            .collect();
        println!("dungeon[0..{}][{}]: [{}]", width.min(20), y, row.join(", "));
    }
    
    Ok(dungeon_map)
}

/// Load dungeon directly to dPiece (like C++ FillSector for Town)
/// 
/// This function reads .dun file and writes directly to dPiece array,
/// using TIL data to expand MegaTile IDs to piece IDs.
/// 
/// # Reference
/// Original: `Source/levels/town.cpp` FillSector() Line 26-48
pub fn load_dun_to_dpiece(
    mpq_manager: &mut crate::resources::MpqManager,
    path: &str,
    til_data: &crate::tiles::TilData,
    offset_x: usize,
    offset_y: usize,
    default_piece: u16,
) -> Result<DungeonMap, String> {
    let data = mpq_manager.find_file(path)
        .ok_or_else(|| format!("Failed to load .dun file: {}", path))?;
    
    if data.len() < 4 {
        return Err(format!(".dun file too small: {} bytes", data.len()));
    }
    
    let width = u16::from_le_bytes([data[0], data[1]]) as usize;
    let height = u16::from_le_bytes([data[2], data[3]]) as usize;
    
    println!("Loading .dun to dPiece: {} ({}x{}) at ({}, {})", 
        path, width, height, offset_x, offset_y);
    
    let expected_size = 4 + width * height * 2;
    if data.len() < expected_size {
        return Err(format!(".dun file size mismatch: expected {} bytes, got {}", 
            expected_size, data.len()));
    }
    
    let mut dungeon_map = DungeonMap::new();
    
    // Fill dPiece with default value
    for y in 0..MAXDUNY {
        for x in 0..MAXDUNX {
            dungeon_map.d_piece[x][y] = default_piece;
        }
    }
    
    let tile_layer = &data[4..];
    
    for j in 0..height {
        let yy = offset_y + j * 2;
        if yy + 1 >= MAXDUNY { continue; }
        
        for i in 0..width {
            let xx = offset_x + i * 2;
            if xx + 1 >= MAXDUNX { continue; }
            
            let idx = (j * width + i) * 2;
            let tile_id = u16::from_le_bytes([tile_layer[idx], tile_layer[idx + 1]]);
            
            let (v1, v2, v3, v4) = if tile_id > 0 {
                let tile_idx = (tile_id - 1) as usize;
                if let Some(mega) = til_data.mega_tiles.get(tile_idx) {
                    (mega.micro1, mega.micro2, mega.micro3, mega.micro4)
                } else {
                    (default_piece, default_piece, default_piece, default_piece)
                }
            } else {
                (default_piece, default_piece, default_piece, default_piece)
            };
            
            dungeon_map.d_piece[xx][yy] = v1;
            dungeon_map.d_piece[xx + 1][yy] = v2;
            dungeon_map.d_piece[xx][yy + 1] = v3;
            dungeon_map.d_piece[xx + 1][yy + 1] = v4;
        }
    }
    
    println!("✓ Loaded {} tiles to dPiece", width * height);
    Ok(dungeon_map)
}

/// Load a sector into an existing dPiece map (for loading multiple sectors)
/// 
/// This function loads a .dun file and writes it to an existing DungeonMap
/// at the specified offset, without overwriting the entire map.
/// 
/// # Reference
/// Original: `Source/levels/town.cpp` FillSector() Line 26-48
pub fn load_sector_to_dpiece(
    dungeon_map: &mut DungeonMap,
    mpq_manager: &mut crate::resources::MpqManager,
    path: &str,
    til_data: &crate::tiles::TilData,
    offset_x: usize,
    offset_y: usize,
    default_piece: u16,
) -> Result<(), String> {
    let data = mpq_manager.find_file(path)
        .ok_or_else(|| format!("Failed to load .dun file: {}", path))?;
    
    if data.len() < 4 {
        return Err(format!(".dun file too small: {} bytes", data.len()));
    }
    
    let width = u16::from_le_bytes([data[0], data[1]]) as usize;
    let height = u16::from_le_bytes([data[2], data[3]]) as usize;
    
    println!("Loading .dun to dPiece: {} ({}x{}) at ({}, {})", 
        path, width, height, offset_x, offset_y);
    
    let expected_size = 4 + width * height * 2;
    if data.len() < expected_size {
        return Err(format!(".dun file size mismatch: expected {} bytes, got {}", 
            expected_size, data.len()));
    }
    
    let tile_layer = &data[4..];
    
    // Match C++ FillSector logic exactly:
    // - Reset xx at start of each row (like C++: int xx = xi;)
    // - Increment xx by 2 after each column (like C++: xx += 2;)
    for j in 0..height {
        let yy = offset_y + j * 2;
        if yy + 1 >= MAXDUNY { continue; }
        
        let mut xx = offset_x;  // Reset xx at start of each row (like C++: int xx = xi;)
        for i in 0..width {
            if xx + 1 >= MAXDUNX { break; }  // Break if out of bounds
            
            // Calculate tile index in .dun file (same as C++: tileLayer[j * size.width + i])
            let idx = (j * width + i) * 2;
            let tile_id = u16::from_le_bytes([tile_layer[idx], tile_layer[idx + 1]]);
            
            let (v1, v2, v3, v4) = if tile_id > 0 {
                let tile_idx = (tile_id - 1) as usize;
                if let Some(mega) = til_data.mega_tiles.get(tile_idx) {
                    (mega.micro1, mega.micro2, mega.micro3, mega.micro4)
                } else {
                    (default_piece, default_piece, default_piece, default_piece)
                }
            } else {
                (default_piece, default_piece, default_piece, default_piece)
            };
            
            dungeon_map.d_piece[xx][yy] = v1;
            dungeon_map.d_piece[xx + 1][yy] = v2;
            dungeon_map.d_piece[xx][yy + 1] = v3;
            dungeon_map.d_piece[xx + 1][yy + 1] = v4;
            
            xx += 2;  // Increment xx by 2 (like C++: xx += 2;)
        }
    }
    
    println!("✓ Loaded {} tiles to dPiece", width * height);
    Ok(())
}

