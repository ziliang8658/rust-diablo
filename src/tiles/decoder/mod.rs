use crate::tiles::types::TileType;
/// Tile Decoder Module
///
/// This module implements decoders for all 6 TileType formats used in Diablo dungeons.
///
/// # Architecture
///
/// Each TileType has its own encoding format:
/// - Square: 32×32 raw pixels (1024 bytes)
/// - TransparentSquare: 32×32 RLE-encoded pixels
/// - LeftTriangle: 32×31 variable-width rows (left-aligned)
/// - RightTriangle: 32×31 variable-width rows (right-aligned)
/// - LeftTrapezoid: Bottom triangle + top rectangle (left-aligned)
/// - RightTrapezoid: Bottom triangle + top rectangle (right-aligned)
///
/// # References
///
/// Original code: `Source/levels/reencode_dun_cels.cpp`
/// - ReencodeDungeonCelsSquare()
/// - ReencodeDungeonCelsTransparentSquare()
/// - ReencodeDungeonCelsLeftTriangle()
/// - ReencodeDungeonCelsRightTriangle()
/// - ReencodeDungeonCelsLeftTrapezoid()
/// - ReencodeDungeonCelsRightTrapezoid()
use anyhow::Result;

mod square;
mod transparent;
mod trapezoid;
mod triangle;

pub use square::decode_square;
pub use transparent::decode_transparent_square;
pub use trapezoid::{decode_left_trapezoid, decode_right_trapezoid};
pub use triangle::{decode_left_triangle, decode_right_triangle};

/// Unified decoder interface
///
/// Dispatches to the appropriate decoder based on TileType.
///
/// # Arguments
/// * `tile_type` - Type of tile to decode
/// * `raw_data` - Raw encoded data from CEL file
///
/// # Returns
/// Decoded pixel data (indexed color, 0 = transparent)
///
/// # Output Format
/// - Square/TransparentSquare/Trapezoid: 32×32 = 1024 bytes
/// - Triangle: 32×31 = 992 bytes
///
/// Output is row-major order, left-to-right, bottom-to-top.
/// Each row is padded to 32 bytes width (transparent pixels = 0).
///
/// # Reference
/// Original code: `Source/levels/gendung.cpp::ReencodeDungeonCels()`
pub fn decode_tile(tile_type: TileType, raw_data: &[u8]) -> Result<Vec<u8>> {
    match tile_type {
        TileType::Square => decode_square(raw_data),
        TileType::TransparentSquare => decode_transparent_square(raw_data),
        TileType::LeftTriangle => decode_left_triangle(raw_data),
        TileType::RightTriangle => decode_right_triangle(raw_data),
        TileType::LeftTrapezoid => decode_left_trapezoid(raw_data),
        TileType::RightTrapezoid => decode_right_trapezoid(raw_data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_tile_square() {
        let raw_data = vec![42u8; 1024];
        let decoded = decode_tile(TileType::Square, &raw_data).unwrap();
        assert_eq!(decoded.len(), 1024);
        assert_eq!(decoded[0], 42);
    }

    #[test]
    fn test_decode_tile_invalid_type() {
        // Test with insufficient data
        let raw_data = vec![1u8; 10];
        let result = decode_tile(TileType::Square, &raw_data);
        assert!(result.is_err());
    }
}
