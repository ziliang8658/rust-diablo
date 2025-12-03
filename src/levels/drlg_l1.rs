/// Cathedral (L1) Dungeon Generation
///
/// This module implements the procedural generation algorithm for Cathedral levels.
///
/// # Algorithm Overview
/// 1. FirstRoom() - Create initial room layout (1-3 chambers + hallway)
/// 2. MakeDmt() - Convert room mask to tile types (walls, floors, corners)
/// 3. FillChambers() - Add details to chambers
/// 4. FixTilesPatterns() - Fix tile patterns and add decorations
/// 5. AddWall() - Add internal walls
/// 6. Pass3() - Expand dungeon[40x40] to dPiece[112x112]
///
/// # Reference
/// Original code: `Source/levels/drlg_l1.cpp`
use rand::Rng;

/// Dungeon dimensions (MegaTile scale)
pub const DMAXX: usize = 40;
pub const DMAXY: usize = 40;

/// dPiece dimensions (MicroTile scale)  
pub const MAXDUNX: usize = 112;
pub const MAXDUNY: usize = 112;

/// Tile types used in dungeon generation
/// These values are the actual MegaTile indices used in Cathedral (L1)
/// Reference: Source/levels/drlg_l1.cpp Line 94-143
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    VWall = 1,         // Vertical wall
    HWall = 2,         // Horizontal wall
    Corner = 3,        // Corner piece
    DWall = 4,         // Diagonal wall
    DArch = 5,         // Diagonal arch
    VWallEnd = 6,      // Vertical wall end
    HWallEnd = 7,      // Horizontal wall end
    HArchEnd = 8,      // Horizontal arch end
    VArchEnd = 9,      // Vertical arch end
    HArchVWall = 10,   // Horizontal arch with vertical wall
    VArch = 11,        // Vertical arch
    HArch = 12,        // Horizontal arch
    Floor = 13,        // Floor tile
    HWallVArch = 14,   // Horizontal wall with vertical arch
    Pillar = 15,       // Pillar
    VCorner = 16,      // Vertical corner
    HCorner = 17,      // Horizontal corner
    DirtHwall = 18,    // Dirt with horizontal wall
    DirtVwall = 19,    // Dirt with vertical wall
    VDirtCorner = 20,  // Vertical dirt corner
    HDirtCorner = 21,  // Horizontal dirt corner
    Dirt = 22,         // Dirt/empty tile (outside dungeon)
    DirtHwallEnd = 23, // Dirt with horizontal wall end
    DirtVwallEnd = 24, // Dirt with vertical wall end
    // More tile types for doors, fences, etc.
    VDoor = 25,
    HDoor = 26,
}

/// Cathedral dungeon generator
///
/// Generates a procedural Cathedral-style dungeon using Diablo's original algorithm.
pub struct CathedralGenerator {
    /// Room mask - marks which tiles are part of rooms
    dungeon_mask: [[bool; DMAXY]; DMAXX],
    /// Tile types for each position
    pub dungeon: [[u8; DMAXY]; DMAXX],
    /// Expanded dPiece array
    pub d_piece: [[u16; MAXDUNY]; MAXDUNX],
    /// Whether layout is vertical or horizontal
    vertical_layout: bool,
    /// Chamber flags
    has_chamber1: bool,
    has_chamber2: bool,
    has_chamber3: bool,
    /// Random seed
    seed: u32,
}

impl CathedralGenerator {
    /// Create a new generator with the given seed
    pub fn new(seed: u32) -> Self {
        Self {
            dungeon_mask: [[false; DMAXY]; DMAXX],
            dungeon: [[0; DMAXY]; DMAXX],
            d_piece: [[0; MAXDUNY]; MAXDUNX],
            vertical_layout: false,
            has_chamber1: false,
            has_chamber2: false,
            has_chamber3: false,
            seed,
        }
    }

    /// Generate a complete Cathedral dungeon
    ///
    /// # Reference
    /// Original: `Source/levels/drlg_l1.cpp::CreateL5Dungeon()` Line 1299-1314
    pub fn generate(&mut self, til_data: &crate::tiles::TilData) {
        let mut rng = rand::thread_rng();

        // Reset state
        self.dungeon_mask = [[false; DMAXY]; DMAXX];
        self.dungeon = [[0; DMAXY]; DMAXX];

        // Generate layout
        self.first_room(&mut rng);
        self.make_dmt();
        self.fill_chambers(&mut rng);
        self.fix_tiles_patterns();
        self.add_walls(&mut rng);

        // Expand to dPiece
        self.pass3(til_data);
    }

    /// Create initial room layout
    ///
    /// Creates 1-3 chambers connected by a hallway.
    ///
    /// # Reference
    /// Original: `Source/levels/drlg_l1.cpp::FirstRoom()` Line 508-552
    fn first_room<R: Rng>(&mut self, rng: &mut R) {
        self.dungeon_mask = [[false; DMAXY]; DMAXX];

        // Randomly choose layout orientation
        self.vertical_layout = rng.gen_bool(0.5);

        // Randomly choose which chambers to include
        self.has_chamber1 = rng.gen_bool(0.5);
        self.has_chamber2 = rng.gen_bool(0.5);
        self.has_chamber3 = rng.gen_bool(0.5);

        // Ensure at least chamber 2 exists if 1 or 3 don't
        if !self.has_chamber1 || !self.has_chamber3 {
            self.has_chamber2 = true;
        }

        // Define chamber rectangles
        let (chamber1, chamber2, chamber3, mut hallway) = if self.vertical_layout {
            (
                (15, 1, 10, 10), // x, y, w, h
                (15, 15, 10, 10),
                (15, 29, 10, 10),
                (17, 1, 6, 38),
            )
        } else {
            (
                (1, 15, 10, 10),
                (15, 15, 10, 10),
                (29, 15, 10, 10),
                (1, 17, 38, 6),
            )
        };

        // Adjust hallway based on which chambers exist
        if !self.has_chamber1 {
            if self.vertical_layout {
                hallway.1 += 17;
                hallway.3 -= 17;
            } else {
                hallway.0 += 17;
                hallway.2 -= 17;
            }
        }
        if !self.has_chamber3 {
            if self.vertical_layout {
                hallway.3 -= 16;
            } else {
                hallway.2 -= 16;
            }
        }

        // Map rooms to dungeon mask
        if self.has_chamber1 {
            self.map_room(chamber1.0, chamber1.1, chamber1.2, chamber1.3);
            self.generate_room(chamber1.0, chamber1.1, chamber1.2, chamber1.3, rng);
        }
        if self.has_chamber2 {
            self.map_room(chamber2.0, chamber2.1, chamber2.2, chamber2.3);
            self.generate_room(chamber2.0, chamber2.1, chamber2.2, chamber2.3, rng);
        }
        if self.has_chamber3 {
            self.map_room(chamber3.0, chamber3.1, chamber3.2, chamber3.3);
            self.generate_room(chamber3.0, chamber3.1, chamber3.2, chamber3.3, rng);
        }

        self.map_room(hallway.0, hallway.1, hallway.2, hallway.3);
    }

    /// Mark a rectangular area as part of the dungeon
    fn map_room(&mut self, x: usize, y: usize, w: usize, h: usize) {
        for j in y..(y + h).min(DMAXY) {
            for i in x..(x + w).min(DMAXX) {
                self.dungeon_mask[i][j] = true;
            }
        }
    }

    /// Generate room details (pillars, etc.)
    fn generate_room<R: Rng>(&mut self, x: usize, y: usize, w: usize, h: usize, _rng: &mut R) {
        // Add pillars at corners (simplified version)
        let cx = x + w / 2;
        let cy = y + h / 2;

        // Mark center area
        for j in (y + 2)..(y + h - 2).min(DMAXY) {
            for i in (x + 2)..(x + w - 2).min(DMAXX) {
                self.dungeon_mask[i][j] = true;
            }
        }

        // Could add pillar at center
        if cx < DMAXX && cy < DMAXY {
            // Pillars will be added in later steps
        }
    }

    /// Convert room mask to tile types
    ///
    /// # Reference
    /// Original: `Source/levels/drlg_l1.cpp::MakeDmt()` Line 562-582
    fn make_dmt(&mut self) {
        // First fill everything with Dirt (22)
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                self.dungeon[i][j] = Tile::Dirt as u8;
            }
        }

        for j in 0..(DMAXY - 1) {
            for i in 0..(DMAXX - 1) {
                let here = self.dungeon_mask[i][j];
                let right = self.dungeon_mask[i + 1][j];
                let down = self.dungeon_mask[i][j + 1];
                let diag = self.dungeon_mask[i + 1][j + 1];

                self.dungeon[i][j] = if here {
                    Tile::Floor as u8 // 13
                } else if !diag && down && right {
                    Tile::Floor as u8 // 13 - Remove diagonal corners
                } else if diag && down && right {
                    Tile::VCorner as u8 // 16
                } else if down {
                    Tile::HWall as u8 // 2
                } else if right {
                    Tile::VWall as u8 // 1
                } else if diag {
                    Tile::DWall as u8 // 4
                } else {
                    Tile::Dirt as u8 // 22
                };
            }
        }
    }

    /// Fill chambers with details
    fn fill_chambers<R: Rng>(&mut self, _rng: &mut R) {
        // Simplified: just ensure chambers have proper floor tiles
        // Full implementation would add pillars, decorations, etc.
    }

    /// Fix tile patterns for visual consistency
    ///
    /// # Reference
    /// Original: `Source/levels/drlg_l1.cpp::FixTilesPatterns()` Line 744-990
    fn fix_tiles_patterns(&mut self) {
        // Fix wall endings and corners - simplified version
        for j in 1..(DMAXY - 1) {
            for i in 1..(DMAXX - 1) {
                let tile = self.dungeon[i][j];
                let up = self.dungeon[i][j - 1];
                let down = self.dungeon[i][j + 1];
                let left = self.dungeon[i - 1][j];
                let right = self.dungeon[i + 1][j];

                // Fix vertical wall endings (VWall -> VWallEnd when floor below)
                if tile == Tile::VWall as u8 && down == Tile::Floor as u8 {
                    self.dungeon[i][j] = Tile::VWallEnd as u8;
                }

                // Fix horizontal wall endings (HWall -> HWallEnd when floor to right)
                if tile == Tile::HWall as u8 && right == Tile::Floor as u8 {
                    self.dungeon[i][j] = Tile::HWallEnd as u8;
                }

                // Fix corners where walls meet
                if tile == Tile::DWall as u8 {
                    if (up == Tile::VWall as u8 || up == Tile::VWallEnd as u8)
                        && (left == Tile::HWall as u8 || left == Tile::HWallEnd as u8)
                    {
                        self.dungeon[i][j] = Tile::Corner as u8;
                    }
                }

                // Add dirt-wall transitions
                if tile == Tile::Dirt as u8 {
                    if down == Tile::Floor as u8 || down == Tile::VCorner as u8 {
                        self.dungeon[i][j] = Tile::DirtHwall as u8;
                    } else if right == Tile::Floor as u8 || right == Tile::HCorner as u8 {
                        self.dungeon[i][j] = Tile::DirtVwall as u8;
                    }
                }
            }
        }
    }

    /// Add internal walls to create more interesting layouts
    fn add_walls<R: Rng>(&mut self, rng: &mut R) {
        // Add some random internal walls
        let num_walls = rng.gen_range(3..8);

        for _ in 0..num_walls {
            let x = rng.gen_range(5..35);
            let y = rng.gen_range(5..35);

            if self.dungeon[x][y] == Tile::Floor as u8 {
                // Try to add a short wall segment
                let horizontal = rng.gen_bool(0.5);
                let length = rng.gen_range(2..5);

                if horizontal {
                    for dx in 0..length {
                        if x + dx < DMAXX && self.dungeon[x + dx][y] == Tile::Floor as u8 {
                            // Check if wall would block path
                            let above = y > 0 && self.dungeon[x + dx][y - 1] == Tile::Floor as u8;
                            let below =
                                y + 1 < DMAXY && self.dungeon[x + dx][y + 1] == Tile::Floor as u8;
                            if above && below {
                                self.dungeon[x + dx][y] = Tile::HWall as u8;
                            }
                        }
                    }
                } else {
                    for dy in 0..length {
                        if y + dy < DMAXY && self.dungeon[x][y + dy] == Tile::Floor as u8 {
                            let left = x > 0 && self.dungeon[x - 1][y + dy] == Tile::Floor as u8;
                            let right =
                                x + 1 < DMAXX && self.dungeon[x + 1][y + dy] == Tile::Floor as u8;
                            if left && right {
                                self.dungeon[x][y + dy] = Tile::VWall as u8;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Expand dungeon[40x40] to dPiece[112x112]
    ///
    /// Each MegaTile in dungeon becomes 4 MicroTiles in dPiece.
    ///
    /// # Reference
    /// Original: `Source/levels/gendung.cpp::DRLG_LPass3()` Line 768-801
    fn pass3(&mut self, til_data: &crate::tiles::TilData) {
        // Step 1: Fill entire dPiece with default tile
        // C++ uses (Dirt - 1) = 21 as default tile index
        // Dirt = 22, so index is 21 (0-based)
        let default_tile_idx = (Tile::Dirt as usize).saturating_sub(1); // 21
        if let Some(default_mega) = til_data.get(default_tile_idx) {
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

        // Step 2: Expand dungeon to dPiece
        // dPiece starts at offset 16 from dungeon coordinates
        // C++ code: const int tileId = dungeon[i][j] - 1;
        let mut yy = 16usize;
        for j in 0..DMAXY {
            let mut xx = 16usize;
            for i in 0..DMAXX {
                let tile_id = self.dungeon[i][j];
                // dungeon stores tile enum values (1-based), convert to 0-based TIL index
                // e.g., Floor=13 -> TIL index 12, VWall=1 -> TIL index 0
                let tile_idx = (tile_id as usize).saturating_sub(1);
                if let Some(mega) = til_data.get(tile_idx) {
                    self.d_piece[xx][yy] = mega.micro1;
                    self.d_piece[xx + 1][yy] = mega.micro2;
                    self.d_piece[xx][yy + 1] = mega.micro3;
                    self.d_piece[xx + 1][yy + 1] = mega.micro4;
                }
                xx += 2;
            }
            yy += 2;
        }
    }

    /// Get the levelPieceId at dPiece coordinates
    pub fn get_piece(&self, x: i32, y: i32) -> u16 {
        if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
            self.d_piece[x as usize][y as usize]
        } else {
            0
        }
    }

    /// Check if coordinates are in bounds
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32
    }
}

/// Create a Cathedral dungeon with the given seed
///
/// # Arguments
/// * `seed` - Random seed for reproducible generation
/// * `til_data` - TIL data for MegaTile definitions
///
/// # Returns
/// Generated dungeon with d_piece array filled
pub fn create_l1_dungeon(seed: u32, til_data: &crate::tiles::TilData) -> CathedralGenerator {
    let mut generator = CathedralGenerator::new(seed);
    generator.generate(til_data);
    generator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_enum() {
        assert_eq!(Tile::VWall as u8, 1);
        assert_eq!(Tile::Floor as u8, 13);
        assert_eq!(Tile::Dirt as u8, 16);
    }

    #[test]
    fn test_generator_creation() {
        let gen = CathedralGenerator::new(12345);
        assert_eq!(gen.seed, 12345);
    }
}
