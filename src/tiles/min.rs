pub use crate::tiles::types::DungeonType;
use crate::tiles::types::LevelCelBlock;
/// MIN file format loader
///
/// MIN files contain micro tile (MicroTile) definitions.
/// Each "Piece" (levelPieceId) corresponds to a stack of micro tiles:
/// - Town: 16 blocks
/// - Cathedral (L1): 10 blocks
/// - Catacombs (L2): 10 blocks
/// - Caves (L3): 10 blocks
/// - Hell (L4): 12 blocks
///
/// # References
/// - Original code: `Source/levels/gendung.cpp::SetDungeonMicros()` Line 509-549
use anyhow::Result;

/// A collection of micro tiles that make up a single map piece
#[derive(Debug, Clone)]
pub struct PieceMicros {
    pub mt: Vec<LevelCelBlock>,
}

/// MIN file data container
pub struct MinData {
    /// Piece definitions (each contains 10-16 micro blocks)
    pub pieces: Vec<PieceMicros>,
    /// Number of blocks per piece (10, 12, or 16)
    pub blocks_per_piece: usize,
}

impl MinData {
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            blocks_per_piece: 10,
        }
    }

    pub fn len(&self) -> usize {
        self.pieces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&PieceMicros> {
        self.pieces.get(index)
    }

    /// Load MIN file from raw bytes with specified dungeon type
    pub fn from_bytes(data: &[u8], dungeon_type: DungeonType) -> Result<Self> {
        let blocks_per_piece = match dungeon_type {
            DungeonType::Town => 16,
            DungeonType::Hell => 12,
            _ => 10,
        };

        if data.len() % 2 != 0 {
            return Err(anyhow::anyhow!("MIN file size must be even"));
        }

        let total_u16s = data.len() / 2;
        let num_pieces = total_u16s / blocks_per_piece;
        let mut pieces = Vec::with_capacity(num_pieces);

        let mut offset = 0;
        for _ in 0..num_pieces {
            let mut mt = Vec::with_capacity(blocks_per_piece);
            for _ in 0..blocks_per_piece {
                if offset + 2 > data.len() {
                    break;
                }
                let val = u16::from_le_bytes([data[offset], data[offset + 1]]);
                mt.push(LevelCelBlock::new(val));
                offset += 2;
            }
            pieces.push(PieceMicros { mt });
        }

        // Apply reordering to match DPieceMicros structure (C++ SetDungeonMicros logic)
        // C++ code: pieces[blocks - 2 + (block & 1) - (block & 0xE)]
        // This reorders so that block 0,1 contain floor data (from original block 14,15 for Town)
        for piece in pieces.iter_mut() {
            let raw_mt = piece.mt.clone();
            for block in 0..blocks_per_piece {
                // C++ formula: blocks - 2 + (block & 1) - (block & 0xE)
                let src_idx = (blocks_per_piece as isize - 2 + (block as isize & 1)
                    - (block as isize & 0xE)) as usize;
                if src_idx < raw_mt.len() {
                    piece.mt[block] = raw_mt[src_idx];
                }
            }
        }

        Ok(Self {
            pieces,
            blocks_per_piece,
        })
    }

    pub fn from_mpq(
        mpq_manager: &mut crate::resources::MpqManager,
        path: &str,
        dungeon_type: DungeonType,
    ) -> Result<Self> {
        let unix_path = path.replace('\\', "/");
        let windows_path = path.replace('/', "\\");

        let data = mpq_manager
            .find_file(&unix_path)
            .or_else(|| mpq_manager.find_file(&windows_path))
            .ok_or_else(|| anyhow::anyhow!("MIN file not found: {}", path))?;
        Self::from_bytes(&data, dungeon_type)
    }

    pub fn load_for_dungeon(
        mpq_manager: &mut crate::resources::MpqManager,
        dungeon_type: DungeonType,
    ) -> Result<Self> {
        let candidates: &[&str] = match dungeon_type {
            DungeonType::Town => &[
                // Match DevilutionX C++ preference order (Source/levels/gendung.cpp loads nlevels first)
                "nlevels/towndata/town.min",
                "levels/towndata/town.min",
            ],
            DungeonType::Cathedral => &["levels/l1data/l1.min"],
            DungeonType::Catacombs => &["levels/l2data/l2.min"],
            DungeonType::Caves => &["levels/l3data/l3.min"],
            DungeonType::Hell => &["levels/l4data/l4.min"],
        };

        let mut last_err: Option<anyhow::Error> = None;
        for path in candidates {
            match Self::from_mpq(mpq_manager, path, dungeon_type) {
                Ok(result) => {
                    println!("Loaded MIN: {}", path);
                    return Ok(result);
                }
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Failed to load MIN file")))
    }
}

impl Default for MinData {
    fn default() -> Self {
        Self::new()
    }
}
