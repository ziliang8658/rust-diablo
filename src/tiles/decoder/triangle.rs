/// Triangle Tile Decoder
///
/// Decodes 32×31 triangle tiles (LeftTriangle and RightTriangle).
///
/// # Format
///
/// Triangle tiles consist of 31 rows with varying widths:
/// - Lower half (rows 0-15): 16 rows, width increases from 2 to 32
/// - Upper half (rows 16-30): 15 rows, width decreases from 30 to 2
///
/// The encoding uses padding bytes (2 bytes) before/after rows depending on alignment.
///
/// # References
///
/// Original code: `Source/levels/reencode_dun_cels.cpp`
/// - ReencodeDungeonCelsLeftTriangle() Line 39-58
/// - ReencodeDungeonCelsRightTriangle() Line 75-92
use anyhow::{bail, Result};

const DUN_FRAME_WIDTH: usize = 32;
const TRIANGLE_HEIGHT: usize = 31;
const OUTPUT_SIZE: usize = DUN_FRAME_WIDTH * TRIANGLE_HEIGHT; // 32 * 31 = 992

/// Decode a LeftTriangle tile (left-aligned triangle)
///
/// # Arguments
/// * `raw_data` - Raw encoded data
///
/// # Returns
/// Decoded pixels (992 bytes = 32×31, row-major)
///
/// # Layout
/// ```text
/// Rows 0-15:  Lower half (width: 2, 4, 6, 8, ..., 30, 32) - left-aligned
/// Rows 16-30: Upper half (width: 30, 28, 26, ..., 4, 2) - left-aligned
/// ```
///
/// # Reference
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsLeftTriangle()` Line 39-58
pub fn decode_left_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; OUTPUT_SIZE];
    let mut src = 0;
    let mut dst = 0;

    // Lower half (rows 0-15): 8 pairs, width increases from 2 to 32
    // Reference: Source/levels/reencode_dun_cels.cpp:23-37 (ReencodeDungeonCelsLeftTriangleLower)
    let mut width = 0;
    for i in 0..8 {
        // Skip padding before first row of pair
        // Reference: Line 27
        src += 2;
        if src > raw_data.len() {
            bail!(
                "LeftTriangle: insufficient data at lower pair {} (src={})",
                i,
                src
            );
        }

        // First row of pair
        // Reference: Line 28-31
        width += 2;
        if src + width > raw_data.len() {
            bail!(
                "LeftTriangle: insufficient data for lower row {} (need {})",
                i * 2,
                width
            );
        }
        output[dst..dst + width].copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += DUN_FRAME_WIDTH;

        // Second row of pair
        // Reference: Line 32-35
        width += 2;
        if src + width > raw_data.len() {
            bail!(
                "LeftTriangle: insufficient data for lower row {} (need {})",
                i * 2 + 1,
                width
            );
        }
        output[dst..dst + width].copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += DUN_FRAME_WIDTH;
    }

    // Upper half (rows 16-30): 7 pairs + 1 single, width decreases from 30 to 2
    // Reference: Source/levels/reencode_dun_cels.cpp:42-57
    let mut width = DUN_FRAME_WIDTH; // Start at 32
    for i in 0..7 {
        // Skip padding before first row of pair
        // Reference: Line 44
        src += 2;
        if src > raw_data.len() {
            bail!(
                "LeftTriangle: insufficient data at upper pair {} (src={})",
                i,
                src
            );
        }

        // First row of pair
        // Reference: Line 45-48
        width -= 2;
        if src + width > raw_data.len() {
            bail!(
                "LeftTriangle: insufficient data for upper row {} (need {})",
                16 + i * 2,
                width
            );
        }
        output[dst..dst + width].copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += DUN_FRAME_WIDTH;

        // Second row of pair
        // Reference: Line 49-52
        width -= 2;
        if src + width > raw_data.len() {
            bail!(
                "LeftTriangle: insufficient data for upper row {} (need {})",
                16 + i * 2 + 1,
                width
            );
        }
        output[dst..dst + width].copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += DUN_FRAME_WIDTH;
    }

    // Last single row (row 30)
    // Reference: Source/levels/reencode_dun_cels.cpp:54-57
    src += 2; // Skip padding
    if src > raw_data.len() {
        bail!("LeftTriangle: insufficient data for last row padding");
    }
    width -= 2;
    if src + width > raw_data.len() {
        bail!("LeftTriangle: insufficient data for last row (need {})", width);
    }
    output[dst..dst + width].copy_from_slice(&raw_data[src..src + width]);

    Ok(output)
}

/// Decode a RightTriangle tile (right-aligned triangle)
///
/// # Arguments
/// * `raw_data` - Raw encoded data
///
/// # Returns
/// Decoded pixels (992 bytes = 32×31, row-major)
///
/// # Layout
/// ```text
/// Rows 0-15:  Lower half (width: 2, 4, 6, 8, ..., 30, 32) - right-aligned
/// Rows 16-30: Upper half (width: 30, 28, 26, ..., 4, 2) - right-aligned
/// ```
///
/// # Reference
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsRightTriangle()` Line 75-92
pub fn decode_right_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; OUTPUT_SIZE];
    let mut src = 0;
    let mut dst = 0;

    // Lower half (rows 0-15): 8 pairs, width increases from 2 to 32
    // Reference: Source/levels/reencode_dun_cels.cpp:60-73 (ReencodeDungeonCelsRightTriangleLower)
    let mut width = 0;
    for i in 0..8 {
        // First row of pair
        // Padding AFTER the row data
        // Reference: Line 64-66
        width += 2;
        if src + width > raw_data.len() {
            bail!(
                "RightTriangle: insufficient data for lower row {} (need {})",
                i * 2,
                width
            );
        }
        let offset = DUN_FRAME_WIDTH - width;
        output[dst + offset..dst + offset + width].copy_from_slice(&raw_data[src..src + width]);
        src += width + 2; // +2 for padding AFTER
        dst += DUN_FRAME_WIDTH;

        // Second row of pair
        // Reference: Line 68-71
        width += 2;
        if src + width > raw_data.len() {
            bail!(
                "RightTriangle: insufficient data for lower row {} (need {})",
                i * 2 + 1,
                width
            );
        }
        let offset = DUN_FRAME_WIDTH - width;
        output[dst + offset..dst + offset + width].copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += DUN_FRAME_WIDTH;
    }

    // Upper half (rows 16-30): 7 pairs + 1 single, width decreases from 30 to 2
    // Reference: Source/levels/reencode_dun_cels.cpp:78-91
    let mut width = DUN_FRAME_WIDTH; // Start at 32
    for i in 0..7 {
        // First row of pair
        // Padding AFTER the row data
        // Reference: Line 80-83
        width -= 2;
        if src + width > raw_data.len() {
            bail!(
                "RightTriangle: insufficient data for upper row {} (need {})",
                16 + i * 2,
                width
            );
        }
        let offset = DUN_FRAME_WIDTH - width;
        output[dst + offset..dst + offset + width].copy_from_slice(&raw_data[src..src + width]);
        src += width + 2; // +2 for padding AFTER
        dst += DUN_FRAME_WIDTH;

        // Second row of pair
        // Reference: Line 84-87
        width -= 2;
        if src + width > raw_data.len() {
            bail!(
                "RightTriangle: insufficient data for upper row {} (need {})",
                16 + i * 2 + 1,
                width
            );
        }
        let offset = DUN_FRAME_WIDTH - width;
        output[dst + offset..dst + offset + width].copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += DUN_FRAME_WIDTH;
    }

    // Last single row (row 30)
    // Reference: Source/levels/reencode_dun_cels.cpp:89-91
    width -= 2;
    if src + width > raw_data.len() {
        bail!("RightTriangle: insufficient data for last row (need {})", width);
    }
    let offset = DUN_FRAME_WIDTH - width;
    output[dst + offset..dst + offset + width].copy_from_slice(&raw_data[src..src + width]);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Count non-zero pixels in a row
    fn count_non_zero(row: &[u8]) -> usize {
        row.iter().filter(|&&p| p != 0).count()
    }

    /// Helper: Count leading zeros
    fn count_leading_zeros(row: &[u8]) -> usize {
        row.iter().take_while(|&&p| p == 0).count()
    }

    /// Helper: Count trailing zeros
    fn count_trailing_zeros(row: &[u8]) -> usize {
        row.iter().rev().take_while(|&&p| p == 0).count()
    }

    #[test]
    fn test_decode_left_triangle_structure() {
        // Create minimal valid test data
        let mut raw = Vec::new();

        // Lower half: 8 pairs (16 rows)
        let mut width = 0;
        for i in 0..8 {
            raw.extend_from_slice(&[0xFF, 0xFF]); // padding
            width += 2;
            raw.extend(vec![1u8; width]);
            width += 2;
            raw.extend(vec![1u8; width]);
        }

        // Upper half: 7 pairs + 1 single (15 rows)
        let mut width = DUN_FRAME_WIDTH;
        for i in 0..7 {
            raw.extend_from_slice(&[0xFF, 0xFF]); // padding
            width -= 2;
            raw.extend(vec![2u8; width]);
            width -= 2;
            raw.extend(vec![2u8; width]);
        }
        raw.extend_from_slice(&[0xFF, 0xFF]); // padding for last row
        width -= 2;
        raw.extend(vec![2u8; width]);

        let output = decode_left_triangle(&raw).unwrap();

        // Verify output size
        assert_eq!(output.len(), OUTPUT_SIZE);

        // Verify lower half (rows 0-15): variable-width, left-aligned
        let expected_widths_lower = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32];
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let row_data = &output[row * DUN_FRAME_WIDTH..(row + 1) * DUN_FRAME_WIDTH];
            let actual = count_non_zero(row_data);
            assert_eq!(
                actual, expected_width,
                "Row {} width wrong: {} (expected {})",
                row, actual, expected_width
            );

            // Verify left-aligned
            if expected_width > 0 {
                let leading = count_leading_zeros(row_data);
                assert_eq!(leading, 0, "Row {} not left-aligned", row);
            }
        }

        // Verify upper half (rows 16-30): variable-width, left-aligned
        let expected_widths_upper = [30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2];
        for (idx, &expected_width) in expected_widths_upper.iter().enumerate() {
            let row = 16 + idx;
            let row_data = &output[row * DUN_FRAME_WIDTH..(row + 1) * DUN_FRAME_WIDTH];
            let actual = count_non_zero(row_data);
            assert_eq!(
                actual, expected_width,
                "Row {} width wrong: {} (expected {})",
                row, actual, expected_width
            );

            // Verify left-aligned
            if expected_width > 0 {
                let leading = count_leading_zeros(row_data);
                assert_eq!(leading, 0, "Row {} not left-aligned", row);
            }
        }
    }

    #[test]
    fn test_decode_right_triangle_structure() {
        // Create minimal valid test data
        let mut raw = Vec::new();

        // Lower half: 8 pairs (16 rows)
        let mut width = 0;
        for i in 0..8 {
            width += 2;
            raw.extend(vec![1u8; width]);
            raw.extend_from_slice(&[0xFF, 0xFF]); // padding AFTER
            width += 2;
            raw.extend(vec![1u8; width]);
        }

        // Upper half: 7 pairs + 1 single (15 rows)
        let mut width = DUN_FRAME_WIDTH;
        for i in 0..7 {
            width -= 2;
            raw.extend(vec![2u8; width]);
            raw.extend_from_slice(&[0xFF, 0xFF]); // padding AFTER
            width -= 2;
            raw.extend(vec![2u8; width]);
        }
        width -= 2;
        raw.extend(vec![2u8; width]);

        let output = decode_right_triangle(&raw).unwrap();

        // Verify output size
        assert_eq!(output.len(), OUTPUT_SIZE);

        // Verify lower half (rows 0-15): variable-width, right-aligned
        let expected_widths_lower = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32];
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let row_data = &output[row * DUN_FRAME_WIDTH..(row + 1) * DUN_FRAME_WIDTH];
            let actual = count_non_zero(row_data);
            assert_eq!(
                actual, expected_width,
                "Row {} width wrong: {} (expected {})",
                row, actual, expected_width
            );

            // Verify right-aligned (leading zeros = 32 - width)
            let leading = count_leading_zeros(row_data);
            let expected_leading = DUN_FRAME_WIDTH - expected_width;
            assert_eq!(
                leading, expected_leading,
                "Row {} not right-aligned (leading: {}, expected: {})",
                row, leading, expected_leading
            );
        }

        // Verify upper half (rows 16-30): variable-width, right-aligned
        let expected_widths_upper = [30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2];
        for (idx, &expected_width) in expected_widths_upper.iter().enumerate() {
            let row = 16 + idx;
            let row_data = &output[row * DUN_FRAME_WIDTH..(row + 1) * DUN_FRAME_WIDTH];
            let actual = count_non_zero(row_data);
            assert_eq!(
                actual, expected_width,
                "Row {} width wrong: {} (expected {})",
                row, actual, expected_width
            );

            // Verify right-aligned
            let leading = count_leading_zeros(row_data);
            let expected_leading = DUN_FRAME_WIDTH - expected_width;
            assert_eq!(
                leading, expected_leading,
                "Row {} not right-aligned (leading: {}, expected: {})",
                row, leading, expected_leading
            );
        }
    }

    #[test]
    fn test_decode_left_triangle_insufficient_data() {
        let raw = vec![0u8; 10]; // Too short
        let result = decode_left_triangle(&raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_right_triangle_insufficient_data() {
        let raw = vec![0u8; 10]; // Too short
        let result = decode_right_triangle(&raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_triangle_middle_row() {
        // Test that the middle row (row 15) is full width (32)
        let mut raw = Vec::new();

        // Lower half
        let mut width = 0;
        for i in 0..8 {
            raw.extend_from_slice(&[0xFF, 0xFF]);
            width += 2;
            raw.extend(vec![10u8; width]);
            width += 2;
            raw.extend(vec![10u8; width]);
        }

        // Upper half
        let mut width = DUN_FRAME_WIDTH;
        for i in 0..7 {
            raw.extend_from_slice(&[0xFF, 0xFF]);
            width -= 2;
            raw.extend(vec![20u8; width]);
            width -= 2;
            raw.extend(vec![20u8; width]);
        }
        raw.extend_from_slice(&[0xFF, 0xFF]);
        width -= 2;
        raw.extend(vec![20u8; width]);

        let output = decode_left_triangle(&raw).unwrap();

        // Row 15 should be full width (last row of lower half)
        let row15 = &output[15 * DUN_FRAME_WIDTH..16 * DUN_FRAME_WIDTH];
        assert_eq!(count_non_zero(row15), 32);
        assert_eq!(row15[0], 10); // Should have value 10

        // Row 16 should be width 30 (first row of upper half)
        let row16 = &output[16 * DUN_FRAME_WIDTH..17 * DUN_FRAME_WIDTH];
        assert_eq!(count_non_zero(row16), 30);
        assert_eq!(row16[0], 20); // Should have value 20
    }
}
