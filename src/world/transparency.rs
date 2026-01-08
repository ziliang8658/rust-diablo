/// Transparency System
///
/// Manages tile transparency values for blending different terrain types.
///
/// # Reference
/// Original: Source/levels/gendung.cpp Line 58-59, 592-657

use crate::world::dungeon_map::{MAXDUNX, MAXDUNY};

/// Transparency list - 256 bool values indicating which transparency values are active
///
/// # Reference
/// Original: Source/levels/gendung.cpp Line 59
/// C++: std::array<bool, 256> TransList;
#[derive(Clone)]
pub struct TransList {
    /// 256 bool values, indexed by transparency value (0-255)
    list: [bool; 256],
}

impl TransList {
    /// Create new TransList with all values false
    ///
    /// # Reference
    /// Original: Source/levels/gendung.cpp Line 595
    /// C++: TransList = {};
    pub fn new() -> Self {
        Self { list: [false; 256] }
    }

    /// Set a transparency value as active
    pub fn set(&mut self, index: u8, active: bool) {
        self.list[index as usize] = active;
    }

    /// Check if a transparency value is active
    pub fn get(&self, index: i8) -> bool {
        if index < 0 {
            return false;
        }
        self.list[index as usize]
    }

    /// Check if a transparency value is active (u8 version)
    pub fn get_u8(&self, index: u8) -> bool {
        self.list[index as usize]
    }

    /// Initialize TransList based on TransVal data
    ///
    /// Scans the entire TransVal array and marks all non-zero
    /// transparency indices as active in TransList.
    ///
    /// # Reference
    /// This is inferred from C++ behavior - TransList needs to know
    /// which transparency values are actually used in the level.
    ///
    /// # Arguments
    /// * `trans_val` - The TransVal structure containing transparency values for each tile
    pub fn init_from_trans_val(&mut self, trans_val: &TransVal) {
        // Reset all to false
        self.list = [false; 256];

        // Scan all positions in trans_val
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                let val = trans_val.values[x][y];
                if val > 0 {
                    self.list[val as usize] = true;
                }
            }
        }

        // Debug output
        let active_count = self.list.iter().filter(|&&b| b).count();
        println!("TransList initialized: {} active indices", active_count);

        // List active indices (for debugging)
        let active_indices: Vec<usize> = self
            .list
            .iter()
            .enumerate()
            .filter(|(_, &active)| active)
            .map(|(i, _)| i)
            .collect();
        if !active_indices.is_empty() {
            println!("Active transparency indices: {:?}", active_indices);
        }
    }
}

impl Default for TransList {
    fn default() -> Self {
        Self::new()
    }
}

/// Transparency value map - stores transparency index for each dungeon tile
///
/// # Reference
/// Original: Source/levels/gendung.cpp Line 62
/// C++: int8_t dTransVal[MAXDUNX][MAXDUNY];
#[derive(Clone)]
pub struct TransVal {
    /// Transparency value for each tile (0 = no transparency, 1-255 = transparency index)
    values: [[i8; MAXDUNY]; MAXDUNX],
}

impl TransVal {
    /// Create new TransVal with all values 0
    ///
    /// # Reference
    /// Original: Source/levels/gendung.cpp Line 594
    /// C++: memset(dTransVal, 0, sizeof(dTransVal));
    pub fn new() -> Self {
        Self {
            values: [[0; MAXDUNY]; MAXDUNX],
        }
    }

    /// Get transparency value at position
    pub fn get(&self, x: i32, y: i32) -> i8 {
        if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
            self.values[x as usize][y as usize]
        } else {
            0
        }
    }

    /// Set transparency value at position
    pub fn set(&mut self, x: i32, y: i32, value: i8) {
        if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
            self.values[x as usize][y as usize] = value;
        }
    }

    /// Load transparency data from DUN file
    ///
    /// # Reference
    /// Original: Source/levels/gendung.cpp Line 628-645 (LoadTransparency)
    ///
    /// # DUN File Structure
    /// The DUN file contains multiple layers:
    /// - Layer 0: Mega tile indices (width × height)
    /// - Layer 1: Unknown
    /// - Layer 2-4: Various tile data at dPiece scale (2× size)
    /// - Layer 4 (transparency): Starts at offset = 2 + width*height + (2*width)*(2*height)*3
    ///
    /// # Arguments
    /// * `dun_data` - DUN file data as u16 array
    ///
    /// # Returns
    /// Ok(()) if loaded successfully
    pub fn load_from_dun(&mut self, dun_data: &[u16]) -> anyhow::Result<()> {
        if dun_data.len() < 2 {
            return Err(anyhow::anyhow!("DUN data too short"));
        }

        // Read size from header
        let width = dun_data[0] as usize;
        let height = dun_data[1] as usize;

        // Calculate offset to transparency layer (layer 4)
        // Offset = 2 (header) + width*height (layer 0) + (2*width)*(2*height)*3 (layers 1-3)
        let layer2_offset = 2 + width * height;
        let dpiece_width = width * 2;
        let dpiece_height = height * 2;
        let transparency_offset = layer2_offset + dpiece_width * dpiece_height * 3;

        if dun_data.len() < transparency_offset + dpiece_width * dpiece_height {
            return Err(anyhow::anyhow!(
                "DUN data too short for transparency layer: need {}, got {}",
                transparency_offset + dpiece_width * dpiece_height,
                dun_data.len()
            ));
        }

        // Load transparency values (dPiece scale, starting at offset 16,16)
        // Reference: C++ loads into dTransVal[16+i][16+j]
        for j in 0..dpiece_height {
            for i in 0..dpiece_width {
                let dun_index = transparency_offset + j * dpiece_width + i;
                let trans_value = dun_data[dun_index] as i8;

                // Store in dTransVal with 16-tile border offset
                let x = 16 + i as i32;
                let y = 16 + j as i32;
                self.set(x, y, trans_value);
            }
        }

        Ok(())
    }

    /// Initialize transparency values to 0
    ///
    /// # Reference
    /// Original: Source/levels/gendung.cpp Line 592-595 (DRLG_InitTrans)
    pub fn init(&mut self) {
        self.values = [[0; MAXDUNY]; MAXDUNX];
    }
}

impl Default for TransVal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trans_list_new() {
        let list = TransList::new();
        assert!(!list.get(0));
        assert!(!list.get(127));
        assert!(!list.get_u8(255));
    }

    #[test]
    fn test_trans_list_set_get() {
        let mut list = TransList::new();
        list.set(42, true);
        assert!(list.get(42));
        assert!(!list.get(43));
    }

    #[test]
    fn test_trans_val_new() {
        let val = TransVal::new();
        assert_eq!(val.get(50, 50), 0);
    }

    #[test]
    fn test_trans_val_set_get() {
        let mut val = TransVal::new();
        val.set(30, 40, 5);
        assert_eq!(val.get(30, 40), 5);
        assert_eq!(val.get(30, 41), 0);
    }

    #[test]
    fn test_trans_val_bounds() {
        let val = TransVal::new();
        assert_eq!(val.get(-1, 0), 0);  // Out of bounds
        assert_eq!(val.get(0, -1), 0);
        assert_eq!(val.get(200, 0), 0); // Beyond MAXDUNX
    }

    #[test]
    fn test_trans_list_init_from_trans_val() {
        let mut trans_val = TransVal::new();
        let mut trans_list = TransList::new();

        // Set some transparency values
        trans_val.set(20, 30, 5);
        trans_val.set(21, 30, 5);
        trans_val.set(25, 35, 10);
        trans_val.set(30, 40, 15);

        // Initialize TransList from TransVal
        trans_list.init_from_trans_val(&trans_val);

        // Verify active indices
        assert!(trans_list.get_u8(5));   // Should be active
        assert!(trans_list.get_u8(10));  // Should be active
        assert!(trans_list.get_u8(15));  // Should be active
        assert!(!trans_list.get_u8(3));  // Should NOT be active
        assert!(!trans_list.get_u8(7));  // Should NOT be active
        assert!(!trans_list.get_u8(20)); // Should NOT be active
    }
}

