use crate::tiles::types::TileProperties;
/// SOL file format loader
///
/// SOL files define tile properties (walkability, light blocking, etc.).
/// Each entry is a single byte containing TileProperties flags.
///
/// # File Structure
/// ```
/// SOL file = [u8; n]  // n bytes, each byte is TileProperties flags
/// ```
///
/// # Properties
/// - SOLID: Cannot be walked through
/// - BLOCK_LIGHT: Blocks line of sight
/// - BLOCK_MISSILE: Blocks projectiles
/// - TRANSPARENT: Has transparency
/// - TRAP: Is a trap tile
///
/// # References
/// - Original code: `Source/levels/gendung.cpp::LoadLevelSOLData()` Line 442-507
use anyhow::Result;

/// Maximum number of tiles
pub const MAXTILES: usize = 2048;

/// SOL file data container
///
/// Stores the tile properties loaded from a SOL file.
pub struct SolData {
    /// Tile properties (one per tile)
    pub properties: Vec<TileProperties>,
}

impl SolData {
    /// Create an empty SolData
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }

    /// Create SolData with a given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            properties: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of tile properties
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Check if the data is empty
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    /// Get tile properties by index
    pub fn get(&self, index: usize) -> Option<TileProperties> {
        self.properties.get(index).copied()
    }

    /// Set tile properties by index
    pub fn set(&mut self, index: usize, props: TileProperties) -> bool {
        if index < self.properties.len() {
            self.properties[index] = props;
            true
        } else {
            false
        }
    }

    /// Load SOL file from raw bytes
    ///
    /// # Arguments
    /// * `data` - Raw file data (array of property bytes)
    ///
    /// # Returns
    /// SolData structure with parsed tile properties
    ///
    /// # Examples
    /// ```no_run
    /// use rust_diablo::tiles::sol::SolData;
    ///
    /// // Assuming you have MPQ file data
    /// let file_data: Vec<u8> = vec![/* ... */];
    /// let sol_data = SolData::from_bytes(&file_data)?;
    /// println!("Loaded {} tile properties", sol_data.len());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Reference
    /// Original code: `Source/levels/gendung.cpp::LoadLevelSOLData()`
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let properties: Vec<TileProperties> = data
            .iter()
            .map(|&byte| TileProperties::from_byte(byte))
            .collect();

        Ok(Self { properties })
    }

    /// Load SOL file from MPQ archive using MpqManager
    ///
    /// # Arguments
    /// * `mpq_manager` - MpqManager reference
    /// * `path` - Path to SOL file within MPQ
    ///
    /// # Returns
    /// SolData structure with parsed tile properties
    ///
    /// # Examples
    /// ```no_run
    /// use rust_diablo::tiles::sol::SolData;
    /// use rust_diablo::resources::MpqManager;
    ///
    /// let mpq_manager = MpqManager::new();
    /// // Load MPQ first...
    /// let sol_data = SolData::from_mpq(&mpq_manager, "levels/l1data/l1.sol")?;
    /// println!("Loaded {} Cathedral tile properties", sol_data.len());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_mpq(mpq_manager: &mut crate::resources::MpqManager, path: &str) -> Result<Self> {
        // Try both Unix-style (/) and Windows-style (\) path separators
        let unix_path = path.replace('\\', "/");
        let windows_path = path.replace('/', "\\");

        let data = mpq_manager
            .find_file(&unix_path)
            .or_else(|| mpq_manager.find_file(&windows_path))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "SOL file not found (tried both '{}' and '{}')",
                    unix_path,
                    windows_path
                )
            })?;
        Self::from_bytes(&data)
    }

    /// Load SOL file for a specific dungeon type
    ///
    /// This is a convenience function that automatically selects the correct SOL file
    /// based on the dungeon type and applies necessary fixes for known issues in the
    /// original data.
    ///
    /// # Arguments
    /// * `mpq_manager` - MpqManager reference
    /// * `dungeon_type` - Type of dungeon (Town, Cathedral, Catacombs, etc.)
    ///
    /// # Returns
    /// SolData structure with parsed and corrected tile properties
    ///
    /// # Examples
    /// ```no_run
    /// use rust_diablo::tiles::sol::SolData;
    /// use rust_diablo::tiles::min::DungeonType;
    /// use rust_diablo::resources::MpqManager;
    ///
    /// let mpq_manager = MpqManager::new();
    /// // Load MPQ first...
    /// let sol_data = SolData::load_for_dungeon(&mpq_manager, DungeonType::Cathedral)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Reference
    /// Original code: `Source/levels/gendung.cpp::LoadLevelSOLData()` Line 444-506
    pub fn load_for_dungeon(
        mpq_manager: &mut crate::resources::MpqManager,
        dungeon_type: crate::tiles::min::DungeonType,
    ) -> Result<Self> {
        use crate::tiles::min::DungeonType;

        let candidates: &[&str] = match dungeon_type {
            DungeonType::Town => &[
                // Match DevilutionX C++ preference order (Source/levels/gendung.cpp loads nlevels first)
                "nlevels/towndata/town.sol",
                "levels/towndata/town.sol",
            ],
            DungeonType::Cathedral => &["levels/l1data/l1.sol"],
            DungeonType::Catacombs => &["levels/l2data/l2.sol"],
            DungeonType::Caves => &["levels/l3data/l3.sol"],
            DungeonType::Hell => &["levels/l4data/l4.sol"],
        };

        let mut last_err: Option<anyhow::Error> = None;
        let mut sol_data = None;
        for path in candidates {
            match Self::from_mpq(mpq_manager, path) {
                Ok(result) => {
                    println!("Loaded SOL: {}", path);
                    sol_data = Some(result);
                    break;
                }
                Err(e) => last_err = Some(e),
            }
        }

        let mut sol_data = sol_data.ok_or_else(|| {
            last_err.unwrap_or_else(|| anyhow::anyhow!("Failed to load SOL file"))
        })?;

        // Apply fixes for known issues in original data
        match dungeon_type {
            DungeonType::Cathedral => {
                // Fix incorrectly marked arched tiles
                let block_light_missile =
                    TileProperties::BLOCK_LIGHT | TileProperties::BLOCK_MISSILE;
                sol_data.apply_fix(9, block_light_missile);
                sol_data.apply_fix(15, block_light_missile);
                sol_data.apply_fix(16, block_light_missile);
                sol_data.apply_fix(20, block_light_missile);
                sol_data.apply_fix(21, block_light_missile);
                sol_data.apply_fix(27, TileProperties::BLOCK_MISSILE);
                sol_data.apply_fix(28, TileProperties::BLOCK_MISSILE);
                sol_data.apply_fix(51, block_light_missile);
                sol_data.apply_fix(56, block_light_missile);
                sol_data.apply_fix(58, block_light_missile);
                sol_data.apply_fix(61, block_light_missile);
                sol_data.apply_fix(63, block_light_missile);
                sol_data.apply_fix(65, block_light_missile);
                sol_data.apply_fix(72, block_light_missile);
                sol_data.apply_fix(208, block_light_missile);
                sol_data.apply_fix(247, block_light_missile);
                sol_data.apply_fix(253, block_light_missile);
                sol_data.apply_fix(257, block_light_missile);
                sol_data.apply_fix(323, block_light_missile);
                sol_data.apply_fix(403, TileProperties::BLOCK_LIGHT);

                // Fix incorrectly marked pillar tile
                sol_data.apply_fix(24, TileProperties::BLOCK_LIGHT);

                // Fix incorrectly marked wall tile
                sol_data.apply_fix(450, block_light_missile);
            }
            DungeonType::Caves => {
                // Fix tile 48 sub-tile 171 frame 461
                sol_data.apply_fix(170, TileProperties::BLOCK_MISSILE);

                // Fix fence sub-tiles 481 and 487
                sol_data.apply_fix(481, TileProperties::SOLID);
                sol_data.apply_fix(487, TileProperties::SOLID);
            }
            DungeonType::Hell => {
                // Tile is incorrectly marked as being solid
                sol_data.set(210, TileProperties::NONE);
            }
            _ => {}
        }

        Ok(sol_data)
    }

    /// Apply a fix by ORing the given properties with the existing properties at the index
    ///
    /// This is used to add missing flags to tile properties.
    fn apply_fix(&mut self, index: usize, props: TileProperties) {
        if let Some(existing) = self.properties.get_mut(index) {
            *existing |= props;
        }
    }
}

impl Default for SolData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sol_data_from_bytes() {
        // Test data: 4 tiles with different properties
        let data: Vec<u8> = vec![
            0b0000_0001, // SOLID
            0b0000_0010, // BLOCK_LIGHT
            0b0000_0111, // SOLID | BLOCK_LIGHT | BLOCK_MISSILE
            0b1000_0000, // TRAP
        ];

        let sol_data = SolData::from_bytes(&data).unwrap();

        assert_eq!(sol_data.len(), 4);

        let props0 = sol_data.get(0).unwrap();
        assert!(props0.contains(TileProperties::SOLID));
        assert!(!props0.contains(TileProperties::BLOCK_LIGHT));

        let props1 = sol_data.get(1).unwrap();
        assert!(!props1.contains(TileProperties::SOLID));
        assert!(props1.contains(TileProperties::BLOCK_LIGHT));

        let props2 = sol_data.get(2).unwrap();
        assert!(props2.contains(TileProperties::SOLID));
        assert!(props2.contains(TileProperties::BLOCK_LIGHT));
        assert!(props2.contains(TileProperties::BLOCK_MISSILE));

        let props3 = sol_data.get(3).unwrap();
        assert!(props3.contains(TileProperties::TRAP));
    }

    #[test]
    fn test_sol_data_empty() {
        let data: Vec<u8> = vec![];

        let sol_data = SolData::from_bytes(&data).unwrap();
        assert_eq!(sol_data.len(), 0);
        assert!(sol_data.is_empty());
    }

    #[test]
    fn test_sol_data_set() {
        let data: Vec<u8> = vec![0, 0, 0];
        let mut sol_data = SolData::from_bytes(&data).unwrap();

        assert!(sol_data.set(1, TileProperties::SOLID));
        assert_eq!(sol_data.get(1).unwrap(), TileProperties::SOLID);

        // Out of bounds
        assert!(!sol_data.set(10, TileProperties::SOLID));
    }

    #[test]
    fn test_sol_data_apply_fix() {
        let data: Vec<u8> = vec![0b0000_0001]; // SOLID
        let mut sol_data = SolData::from_bytes(&data).unwrap();

        // Apply additional properties
        sol_data.apply_fix(0, TileProperties::BLOCK_LIGHT);

        let props = sol_data.get(0).unwrap();
        assert!(props.contains(TileProperties::SOLID));
        assert!(props.contains(TileProperties::BLOCK_LIGHT));
    }

    #[test]
    fn test_sol_data_properties_combination() {
        let data: Vec<u8> = vec![
            (TileProperties::SOLID | TileProperties::BLOCK_LIGHT).to_byte(),
            (TileProperties::TRANSPARENT | TileProperties::TRAP).to_byte(),
        ];

        let sol_data = SolData::from_bytes(&data).unwrap();

        let props0 = sol_data.get(0).unwrap();
        assert!(props0.contains(TileProperties::SOLID));
        assert!(props0.contains(TileProperties::BLOCK_LIGHT));
        assert!(!props0.contains(TileProperties::TRANSPARENT));

        let props1 = sol_data.get(1).unwrap();
        assert!(props1.contains(TileProperties::TRANSPARENT));
        assert!(props1.contains(TileProperties::TRAP));
        assert!(!props1.contains(TileProperties::SOLID));
    }
}
