/// TIL file format loader
/// 
/// TIL files define MegaTiles, where each MegaTile is composed of 4 MicroTiles (2x2 grid).
/// 
/// # File Structure
/// ```
/// TIL file = [MegaTile; n]
/// 
/// struct MegaTile {
///     micro1: u16,  // Top-left MicroTile index (points to MIN data)
///     micro2: u16,  // Top-right
///     micro3: u16,  // Bottom-left
///     micro4: u16,  // Bottom-right
/// }
/// ```
/// 
/// Each MegaTile is 8 bytes (4 u16 values, little-endian).
/// 
/// # Coordinate Relationship
/// ```
/// MegaTile (1x1) → MicroTile (2x2)
/// +-------+-------+
/// | micro1| micro2|  ← 2 MicroTile width
/// +-------+-------+
/// | micro3| micro4|
/// +-------+-------+
/// ```
/// 
/// # References
/// - Original code: `Source/levels/gendung.h::MegaTile` Line 94-98
/// - Original code: `Source/levels/gendung.cpp::DRLG_LPass3()` Line 768-801

use anyhow::Result;

/// MegaTile structure
/// 
/// A MegaTile consists of 4 MicroTile indices arranged in a 2x2 grid.
/// These indices point to entries in the MIN data.
/// 
/// # Layout
/// ```
/// +-------+-------+
/// | micro1| micro2|
/// +-------+-------+
/// | micro3| micro4|
/// +-------+-------+
/// ```
/// 
/// # Reference
/// Original code: `Source/levels/gendung.h::MegaTile`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MegaTile {
    /// Top-left MicroTile index
    pub micro1: u16,
    /// Top-right MicroTile index
    pub micro2: u16,
    /// Bottom-left MicroTile index
    pub micro3: u16,
    /// Bottom-right MicroTile index
    pub micro4: u16,
}

impl MegaTile {
    /// Create a new MegaTile
    pub fn new(micro1: u16, micro2: u16, micro3: u16, micro4: u16) -> Self {
        Self {
            micro1,
            micro2,
            micro3,
            micro4,
        }
    }

    /// Create an empty MegaTile (all zeros)
    pub fn empty() -> Self {
        Self {
            micro1: 0,
            micro2: 0,
            micro3: 0,
            micro4: 0,
        }
    }

    /// Get MicroTile index by position
    /// 
    /// # Arguments
    /// * `x` - X position (0 or 1)
    /// * `y` - Y position (0 or 1)
    /// 
    /// # Returns
    /// The MicroTile index at the given position, or None if out of bounds
    pub fn get_micro(&self, x: usize, y: usize) -> Option<u16> {
        match (x, y) {
            (0, 0) => Some(self.micro1),
            (1, 0) => Some(self.micro2),
            (0, 1) => Some(self.micro3),
            (1, 1) => Some(self.micro4),
            _ => None,
        }
    }

    /// Set MicroTile index by position
    /// 
    /// # Arguments
    /// * `x` - X position (0 or 1)
    /// * `y` - Y position (0 or 1)
    /// * `value` - MicroTile index to set
    /// 
    /// # Returns
    /// true if the position was valid and the value was set, false otherwise
    pub fn set_micro(&mut self, x: usize, y: usize, value: u16) -> bool {
        match (x, y) {
            (0, 0) => { self.micro1 = value; true }
            (1, 0) => { self.micro2 = value; true }
            (0, 1) => { self.micro3 = value; true }
            (1, 1) => { self.micro4 = value; true }
            _ => false,
        }
    }
}

impl Default for MegaTile {
    fn default() -> Self {
        Self::empty()
    }
}

/// TIL file data container
/// 
/// Stores the MegaTile data loaded from a TIL file.
pub struct TilData {
    /// MegaTile entries
    pub mega_tiles: Vec<MegaTile>,
}

impl TilData {
    /// Create an empty TilData
    pub fn new() -> Self {
        Self {
            mega_tiles: Vec::new(),
        }
    }

    /// Create TilData with a given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            mega_tiles: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of MegaTiles
    pub fn len(&self) -> usize {
        self.mega_tiles.len()
    }

    /// Check if the data is empty
    pub fn is_empty(&self) -> bool {
        self.mega_tiles.is_empty()
    }

    /// Get a MegaTile by index
    pub fn get(&self, index: usize) -> Option<&MegaTile> {
        self.mega_tiles.get(index)
    }

    /// Load TIL file from raw bytes
    /// 
    /// # Arguments
    /// * `data` - Raw file data (should be multiple of 8 bytes)
    /// 
    /// # Returns
    /// TilData structure with parsed MegaTiles
    /// 
    /// # Errors
    /// - If data length is not a multiple of 8 bytes
    /// 
    /// # Examples
    /// ```no_run
    /// use rust_diablo::tiles::til::TilData;
    /// 
    /// // Assuming you have MPQ file data
    /// let file_data: Vec<u8> = vec![/* ... */];
    /// let til_data = TilData::from_bytes(&file_data)?;
    /// println!("Loaded {} MegaTiles", til_data.len());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    /// 
    /// # Reference
    /// Original code: `Source/levels/gendung.cpp::DRLG_LPass3()`
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // Verify data length is multiple of 8 (size of MegaTile)
        if data.len() % 8 != 0 {
            return Err(anyhow::anyhow!(
                "Invalid TIL file size: {} bytes (must be multiple of 8)",
                data.len()
            ));
        }

        let count = data.len() / 8;
        let mut mega_tiles = Vec::with_capacity(count);

        // Parse MegaTile structures (8 bytes each, little-endian)
        for chunk in data.chunks_exact(8) {
            let micro1 = u16::from_le_bytes([chunk[0], chunk[1]]);
            let micro2 = u16::from_le_bytes([chunk[2], chunk[3]]);
            let micro3 = u16::from_le_bytes([chunk[4], chunk[5]]);
            let micro4 = u16::from_le_bytes([chunk[6], chunk[7]]);
            
            mega_tiles.push(MegaTile::new(micro1, micro2, micro3, micro4));
        }

        Ok(Self { mega_tiles })
    }

    /// Load TIL file from MPQ archive using MpqManager
    /// 
    /// # Arguments
    /// * `mpq_manager` - MpqManager reference
    /// * `path` - Path to TIL file within MPQ
    /// 
    /// # Returns
    /// TilData structure with parsed MegaTiles
    /// 
    /// # Examples
    /// ```no_run
    /// use rust_diablo::tiles::til::TilData;
    /// use rust_diablo::resources::MpqManager;
    /// 
    /// let mpq_manager = MpqManager::new();
    /// // Load MPQ first...
    /// let til_data = TilData::from_mpq(&mpq_manager, "levels/l1data/l1.til")?;
    /// println!("Loaded {} Cathedral MegaTiles", til_data.len());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_mpq(mpq_manager: &mut crate::resources::MpqManager, path: &str) -> Result<Self> {
        // Try both Unix-style (/) and Windows-style (\) path separators
        let unix_path = path.replace('\\', "/");
        let windows_path = path.replace('/', "\\");
        
        let data = mpq_manager.find_file(&unix_path)
            .or_else(|| mpq_manager.find_file(&windows_path))
            .ok_or_else(|| anyhow::anyhow!("TIL file not found (tried both '{}' and '{}')", 
                unix_path, windows_path))?;
        Self::from_bytes(&data)
    }

    /// Load TIL file for a specific dungeon type
    /// 
    /// This is a convenience function that automatically selects the correct TIL file
    /// based on the dungeon type.
    /// 
    /// # Arguments
    /// * `mpq_manager` - MpqManager reference
    /// * `dungeon_type` - Type of dungeon (Town, Cathedral, Catacombs, etc.)
    /// 
    /// # Returns
    /// TilData structure with parsed MegaTiles
    /// 
    /// # Examples
    /// ```no_run
    /// use rust_diablo::tiles::til::TilData;
    /// use rust_diablo::tiles::min::DungeonType;
    /// use rust_diablo::resources::MpqManager;
    /// 
    /// let mpq_manager = MpqManager::new();
    /// // Load MPQ first...
    /// let til_data = TilData::load_for_dungeon(&mpq_manager, DungeonType::Cathedral)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load_for_dungeon(mpq_manager: &mut crate::resources::MpqManager, dungeon_type: crate::tiles::min::DungeonType) -> Result<Self> {
        let path = match dungeon_type {
            crate::tiles::min::DungeonType::Town => "levels/towndata/town.til",
            crate::tiles::min::DungeonType::Cathedral => "levels/l1data/l1.til",
            crate::tiles::min::DungeonType::Catacombs => "levels/l2data/l2.til",
            crate::tiles::min::DungeonType::Caves => "levels/l3data/l3.til",
            crate::tiles::min::DungeonType::Hell => "levels/l4data/l4.til",
        };

        Self::from_mpq(mpq_manager, path)
    }
}

impl Default for TilData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mega_tile_new() {
        let tile = MegaTile::new(1, 2, 3, 4);
        assert_eq!(tile.micro1, 1);
        assert_eq!(tile.micro2, 2);
        assert_eq!(tile.micro3, 3);
        assert_eq!(tile.micro4, 4);
    }

    #[test]
    fn test_mega_tile_get_micro() {
        let tile = MegaTile::new(10, 20, 30, 40);
        
        assert_eq!(tile.get_micro(0, 0), Some(10));
        assert_eq!(tile.get_micro(1, 0), Some(20));
        assert_eq!(tile.get_micro(0, 1), Some(30));
        assert_eq!(tile.get_micro(1, 1), Some(40));
        assert_eq!(tile.get_micro(2, 0), None);
        assert_eq!(tile.get_micro(0, 2), None);
    }

    #[test]
    fn test_mega_tile_set_micro() {
        let mut tile = MegaTile::empty();
        
        assert!(tile.set_micro(0, 0, 100));
        assert!(tile.set_micro(1, 0, 200));
        assert!(tile.set_micro(0, 1, 300));
        assert!(tile.set_micro(1, 1, 400));
        
        assert_eq!(tile.micro1, 100);
        assert_eq!(tile.micro2, 200);
        assert_eq!(tile.micro3, 300);
        assert_eq!(tile.micro4, 400);
        
        assert!(!tile.set_micro(2, 0, 500));
    }

    #[test]
    fn test_til_data_from_bytes_valid() {
        // Test data: 2 MegaTiles
        let data: Vec<u8> = vec![
            // MegaTile 1
            0x01, 0x00,  // micro1 = 1
            0x02, 0x00,  // micro2 = 2
            0x03, 0x00,  // micro3 = 3
            0x04, 0x00,  // micro4 = 4
            // MegaTile 2
            0x0A, 0x00,  // micro1 = 10
            0x14, 0x00,  // micro2 = 20
            0x1E, 0x00,  // micro3 = 30
            0x28, 0x00,  // micro4 = 40
        ];

        let til_data = TilData::from_bytes(&data).unwrap();
        
        assert_eq!(til_data.len(), 2);
        
        let tile1 = til_data.get(0).unwrap();
        assert_eq!(tile1.micro1, 1);
        assert_eq!(tile1.micro2, 2);
        assert_eq!(tile1.micro3, 3);
        assert_eq!(tile1.micro4, 4);
        
        let tile2 = til_data.get(1).unwrap();
        assert_eq!(tile2.micro1, 10);
        assert_eq!(tile2.micro2, 20);
        assert_eq!(tile2.micro3, 30);
        assert_eq!(tile2.micro4, 40);
    }

    #[test]
    fn test_til_data_from_bytes_invalid_length() {
        // Test data with invalid length (not multiple of 8)
        let data: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        
        let result = TilData::from_bytes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_til_data_empty() {
        let data: Vec<u8> = vec![];
        
        let til_data = TilData::from_bytes(&data).unwrap();
        assert_eq!(til_data.len(), 0);
        assert!(til_data.is_empty());
    }

    #[test]
    fn test_til_data_single_tile() {
        // Test data: 1 MegaTile
        let data: Vec<u8> = vec![
            0x12, 0x34,  // micro1 = 0x3412
            0x56, 0x78,  // micro2 = 0x7856
            0x9A, 0xBC,  // micro3 = 0xBC9A
            0xDE, 0xF0,  // micro4 = 0xF0DE
        ];

        let til_data = TilData::from_bytes(&data).unwrap();
        
        assert_eq!(til_data.len(), 1);
        
        let tile = til_data.get(0).unwrap();
        assert_eq!(tile.micro1, 0x3412);
        assert_eq!(tile.micro2, 0x7856);
        assert_eq!(tile.micro3, 0xBC9A);
        assert_eq!(tile.micro4, 0xF0DE);
    }
}

