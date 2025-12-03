/// Tile type definitions and properties for Diablo level rendering
///
/// This module defines tile types and properties used in the level generation and rendering.
///
/// # References
/// - Original code: `Source/levels/dun_tile.hpp`
use bitflags::bitflags;

/// Dungeon type enumeration
///
/// Corresponds to the different dungeon levels in Diablo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonType {
    /// Town (Tristram)
    Town,
    /// Cathedral (L1, levels 1-4)
    Cathedral,
    /// Catacombs (L2, levels 5-8)
    Catacombs,
    /// Caves (L3, levels 9-12)
    Caves,
    /// Hell (L4, levels 13-16)
    Hell,
}

/// Level tile type
///
/// The tile type determines data encoding and the shape.
/// Each tile type has its own encoding but they all encode data in the order
/// of bottom-to-top (bottom row first).
///
/// # Tile Shapes
///
/// - Square: 🮆 A 32x32 square
/// - TransparentSquare: 🮆 A 32x32 square with transparency (RLE encoded)
/// - LeftTriangle: 🭮 Left-pointing 32x31 triangle
/// - RightTriangle: 🭬 Right-pointing 32x31 triangle
/// - LeftTrapezoid: 🭓 Left-pointing 32x32 trapezoid
/// - RightTrapezoid: 🭞 Right-pointing 32x32 trapezoid
///
/// # Reference
/// Original code: `Source/levels/dun_tile.hpp::TileType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TileType {
    /// 🮆 A 32x32 square. Stored as an array of pixels.
    Square = 0,

    /// 🮆 A 32x32 square with transparency. RLE encoded.
    ///
    /// Each run starts with an int8_t value.
    /// If positive, it is followed by this many pixels.
    /// If negative, it indicates `-value` fully transparent pixels, which are omitted.
    ///
    /// Runs do not cross row boundaries.
    TransparentSquare = 1,

    /// 🭮 Left-pointing 32x31 triangle. Encoded as 31 varying-width rows with 2 padding bytes before every even row.
    ///
    /// The smallest rows (bottom and top) are 2px wide, the largest row is 32px wide (middle row).
    LeftTriangle = 2,

    /// 🭬 Right-pointing 32x31 triangle. Encoded as 31 varying-width rows with 2 padding bytes after every even row.
    ///
    /// The smallest rows (bottom and top) are 2px wide, the largest row is 32px wide (middle row).
    RightTriangle = 3,

    /// 🭓 Left-pointing 32x32 trapezoid: a 32x16 rectangle and the 16x16 bottom part of `LeftTriangle`.
    ///
    /// Begins with triangle part, which uses the `LeftTriangle` encoding,
    /// and is followed by a flat array of pixels for the top rectangle part.
    LeftTrapezoid = 4,

    /// 🭞 Right-pointing 32x32 trapezoid: 32x16 rectangle and the 16x16 bottom part of `RightTriangle`.
    ///
    /// Begins with the triangle part, which uses the `RightTriangle` encoding,
    /// and is followed by a flat array of pixels for the top rectangle part.
    RightTrapezoid = 5,
}

impl TileType {
    /// Create TileType from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TileType::Square),
            1 => Some(TileType::TransparentSquare),
            2 => Some(TileType::LeftTriangle),
            3 => Some(TileType::RightTriangle),
            4 => Some(TileType::LeftTrapezoid),
            5 => Some(TileType::RightTrapezoid),
            _ => None,
        }
    }

    /// Convert TileType to u8 value
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Square
    }
}

/// Specifies the current MIN block of the level CEL file, as used during rendering of the level tiles.
///
/// This is a 16-bit value that encodes both the tile type (high 3 bits) and the frame index (low 12 bits).
///
/// # Bit Layout
/// ```
/// Bits 15-12: Reserved
/// Bits 14-12: Tile type (0-5)
/// Bits 11-0:  Frame index (1-based index in pDungeonCels)
/// ```
///
/// # Reference
/// Original code: `Source/levels/dun_tile.hpp::LevelCelBlock`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelCelBlock {
    pub data: u16,
}

impl LevelCelBlock {
    /// Create a new LevelCelBlock from raw data
    pub fn new(data: u16) -> Self {
        Self { data }
    }

    /// Check if this block has a valid value
    pub fn has_value(&self) -> bool {
        self.data != 0
    }

    /// Get the tile type
    pub fn tile_type(&self) -> TileType {
        let type_value = ((self.data & 0x7000) >> 12) as u8;
        TileType::from_u8(type_value).unwrap_or(TileType::Square)
    }

    /// Get the frame index (1-based)
    ///
    /// Returns the 1-based index of the frame in `pDungeonCels`.
    pub fn frame(&self) -> u16 {
        self.data & 0xFFF
    }

    /// Create a LevelCelBlock from tile type and frame index
    pub fn from_parts(tile_type: TileType, frame: u16) -> Self {
        let type_bits = (tile_type as u16) << 12;
        let frame_bits = frame & 0xFFF;
        Self {
            data: type_bits | frame_bits,
        }
    }
}

impl Default for LevelCelBlock {
    fn default() -> Self {
        Self { data: 0 }
    }
}

bitflags! {
    /// Tile properties
    ///
    /// These flags define various properties of tiles, such as:
    /// - Solid: Cannot be walked through
    /// - BlockLight: Blocks line of sight
    /// - BlockMissile: Blocks projectiles
    /// - Transparent: Has transparency
    /// - Trap: Is a trap tile
    ///
    /// # Reference
    /// Original code: `Source/levels/dun_tile.hpp::TileProperties`
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TileProperties: u8 {
        /// No special properties
        const NONE = 0;
        /// Cannot be walked through
        const SOLID = 1 << 0;
        /// Blocks line of sight
        const BLOCK_LIGHT = 1 << 1;
        /// Blocks projectiles
        const BLOCK_MISSILE = 1 << 2;
        /// Has transparency
        const TRANSPARENT = 1 << 3;
        /// Left side is transparent
        const TRANSPARENT_LEFT = 1 << 4;
        /// Right side is transparent
        const TRANSPARENT_RIGHT = 1 << 5;
        /// Is a trap tile
        const TRAP = 1 << 7;
    }
}

impl Default for TileProperties {
    fn default() -> Self {
        TileProperties::NONE
    }
}

impl TileProperties {
    /// Create TileProperties from a byte
    pub fn from_byte(byte: u8) -> Self {
        TileProperties::from_bits_truncate(byte)
    }

    /// Convert TileProperties to a byte
    pub fn to_byte(self) -> u8 {
        self.bits()
    }

    /// Check if this tile is solid (not walkable)
    pub fn is_solid(self) -> bool {
        self.contains(TileProperties::SOLID)
    }

    /// Check if this tile blocks light
    pub fn blocks_light(self) -> bool {
        self.contains(TileProperties::BLOCK_LIGHT)
    }

    /// Check if this tile blocks missiles
    pub fn blocks_missile(self) -> bool {
        self.contains(TileProperties::BLOCK_MISSILE)
    }
}

/// Width of a tile rendering primitive
pub const DUN_FRAME_WIDTH: i32 = 32; // TILE_WIDTH / 2

/// Height of a tile rendering primitive (except triangles)
pub const DUN_FRAME_HEIGHT: i32 = 32; // TILE_HEIGHT

/// Height of triangle tile types
pub const DUN_FRAME_TRIANGLE_HEIGHT: i32 = 31;

/// Size of re-encoded triangle frame data (544 - 32)
pub const REENCODED_TRIANGLE_FRAME_SIZE: usize = 512;

/// Size of re-encoded trapezoid frame data (800 - 16)
pub const REENCODED_TRAPEZOID_FRAME_SIZE: usize = 784;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_type_from_u8() {
        assert_eq!(TileType::from_u8(0), Some(TileType::Square));
        assert_eq!(TileType::from_u8(1), Some(TileType::TransparentSquare));
        assert_eq!(TileType::from_u8(2), Some(TileType::LeftTriangle));
        assert_eq!(TileType::from_u8(3), Some(TileType::RightTriangle));
        assert_eq!(TileType::from_u8(4), Some(TileType::LeftTrapezoid));
        assert_eq!(TileType::from_u8(5), Some(TileType::RightTrapezoid));
        assert_eq!(TileType::from_u8(6), None);
    }

    #[test]
    fn test_tile_type_to_u8() {
        assert_eq!(TileType::Square.to_u8(), 0);
        assert_eq!(TileType::TransparentSquare.to_u8(), 1);
        assert_eq!(TileType::LeftTriangle.to_u8(), 2);
        assert_eq!(TileType::RightTriangle.to_u8(), 3);
        assert_eq!(TileType::LeftTrapezoid.to_u8(), 4);
        assert_eq!(TileType::RightTrapezoid.to_u8(), 5);
    }

    #[test]
    fn test_level_cel_block_decoding() {
        // Test data: type = 1 (TransparentSquare), frame = 0xABC
        let block = LevelCelBlock::new(0x1ABC);

        assert!(block.has_value());
        assert_eq!(block.tile_type(), TileType::TransparentSquare);
        assert_eq!(block.frame(), 0xABC);
    }

    #[test]
    fn test_level_cel_block_encoding() {
        let block = LevelCelBlock::from_parts(TileType::LeftTriangle, 0x123);

        assert_eq!(block.tile_type(), TileType::LeftTriangle);
        assert_eq!(block.frame(), 0x123);
        assert_eq!(block.data, 0x2123); // Type 2 << 12 | frame 0x123
    }

    #[test]
    fn test_level_cel_block_zero() {
        let block = LevelCelBlock::new(0);

        assert!(!block.has_value());
        assert_eq!(block.frame(), 0);
    }

    #[test]
    fn test_tile_properties() {
        let props = TileProperties::SOLID | TileProperties::BLOCK_LIGHT;

        assert!(props.contains(TileProperties::SOLID));
        assert!(props.contains(TileProperties::BLOCK_LIGHT));
        assert!(!props.contains(TileProperties::TRANSPARENT));
        assert!(!props.contains(TileProperties::TRAP));
    }

    #[test]
    fn test_tile_properties_from_byte() {
        let props = TileProperties::from_byte(0b0000_0011); // SOLID | BLOCK_LIGHT

        assert!(props.contains(TileProperties::SOLID));
        assert!(props.contains(TileProperties::BLOCK_LIGHT));
        assert!(!props.contains(TileProperties::BLOCK_MISSILE));
    }

    #[test]
    fn test_tile_properties_to_byte() {
        let props = TileProperties::SOLID | TileProperties::BLOCK_MISSILE;
        assert_eq!(props.to_byte(), 0b0000_0101); // Bits 0 and 2
    }

    #[test]
    fn test_tile_constants() {
        assert_eq!(DUN_FRAME_WIDTH, 32);
        assert_eq!(DUN_FRAME_HEIGHT, 32);
        assert_eq!(DUN_FRAME_TRIANGLE_HEIGHT, 31);
        assert_eq!(REENCODED_TRIANGLE_FRAME_SIZE, 512);
        assert_eq!(REENCODED_TRAPEZOID_FRAME_SIZE, 784);
    }
}
