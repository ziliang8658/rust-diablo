/// Triangle Tile Decoder
/// 
/// Decodes 32×31 triangle tiles (LeftTriangle and RightTriangle).
/// 
/// # Format
/// 
/// Triangle tiles have variable-width rows with padding:
/// - Lower half (rows 0-15): Width increases from 2 to 32
/// - Upper half (rows 16-30): Width decreases from 30 to 2
/// - Every even row has 2-byte padding BEFORE the row data
/// 
/// # Row Width Pattern
/// ```text
/// Row  0: width=2  (padding=2)
/// Row  1: width=4
/// Row  2: width=6  (padding=2)
/// Row  3: width=8
/// ...
/// Row 14: width=30 (padding=2)
/// Row 15: width=32
/// Row 16: width=30 (padding=2)
/// Row 17: width=28
/// ...
/// Row 30: width=2  (padding=2)
/// ```
/// 
/// # References
/// 
/// Original code: `Source/levels/reencode_dun_cels.cpp`
/// - ReencodeDungeonCelsLeftTriangle() Line 34-72
/// - ReencodeDungeonCelsRightTriangle() Line 74-112

use anyhow::{Result, bail};

/// Decode a LeftTriangle tile (32×31 variable-width rows, left-aligned)
/// 
/// # Arguments
/// * `raw_data` - Raw encoded data with padding
/// 
/// # Returns
/// Decoded pixels (992 bytes = 32×31, row-major, padded to 32 bytes per row)
/// 
/// # Layout
/// ```text
/// Output is left-aligned, right side filled with 0 (transparent)
/// Row 0:  [##]                            padding=2, width=2
/// Row 1:  [####]                          width=4
/// Row 2:  [######]                        padding=2, width=6
/// ...
/// Row 15: [##############################] width=32
/// Row 16: [############################]  padding=2, width=30
/// ...
/// Row 30: [##]                            padding=2, width=2
/// ```
/// 
/// # Reference
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsLeftTriangle()` Line 34-72
pub fn decode_left_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    const OUTPUT_SIZE: usize = 32 * 31;
    let mut output = vec![0u8; OUTPUT_SIZE];
    let mut src = 0;
    let mut dst = 0;
    
    // Lower half (rows 0-15): 16 rows, paired (8 pairs)
    // Reference: Source/levels/reencode_dun_cels.cpp:41-54
    for i in 0..8 {
        // Skip 2-byte padding before first row of pair
        // Reference: Line 43
        src += 2;
        
        if src > raw_data.len() {
            bail!("LeftTriangle: insufficient data at pair {} (src={})", i, src);
        }
        
        // First row of pair (width: 2, 6, 10, 14, 18, 22, 26, 30)
        // Reference: Line 45-47
        let width1 = 2 + i * 4;
        if src + width1 > raw_data.len() {
            bail!("LeftTriangle: insufficient data for row {} (need {})", i*2, width1);
        }
        output[dst..dst + width1].copy_from_slice(&raw_data[src..src + width1]);
        src += width1;
        dst += 32;
        
        // Second row of pair (width: 4, 8, 12, 16, 20, 24, 28, 32)
        // Reference: Line 48-50
        let width2 = 4 + i * 4;
        if src + width2 > raw_data.len() {
            bail!("LeftTriangle: insufficient data for row {} (need {})", i*2+1, width2);
        }
        output[dst..dst + width2].copy_from_slice(&raw_data[src..src + width2]);
        src += width2;
        dst += 32;
    }
    
    // Upper half (rows 16-30): 15 rows, mostly paired (7 pairs + 1 single)
    // Reference: Source/levels/reencode_dun_cels.cpp:56-69
    let mut width = 32;
    
    // 7 pairs (rows 16-29)
    for _ in 0..7 {
        // Skip padding
        // Reference: Line 58
        src += 2;
        
        if src > raw_data.len() {
            bail!("LeftTriangle: insufficient data in upper half (src={})", src);
        }
        
        // First row (width: 30, 26, 22, 18, 14, 10, 6)
        // Reference: Line 60-62
        width -= 2;
        if src + width > raw_data.len() {
            bail!("LeftTriangle: insufficient data for upper row (need {})", width);
        }
        output[dst..dst + width].copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += 32;
        
        // Second row (width: 28, 24, 20, 16, 12, 8, 4)
        // Reference: Line 63-65
        width -= 2;
        if src + width > raw_data.len() {
            bail!("LeftTriangle: insufficient data for upper row (need {})", width);
        }
        output[dst..dst + width].copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += 32;
    }
    
    // Last row (row 30, width=2)
    // Reference: Line 67-69
    src += 2;  // Skip padding
    width -= 2;
    if src + width > raw_data.len() {
        bail!("LeftTriangle: insufficient data for last row (need {})", width);
    }
    output[dst..dst + width].copy_from_slice(&raw_data[src..src + width]);
    
    Ok(output)
}

/// Decode a RightTriangle tile (32×31 variable-width rows, right-aligned)
/// 
/// # Arguments
/// * `raw_data` - Raw encoded data with padding
/// 
/// # Returns
/// Decoded pixels (992 bytes = 32×31, row-major, padded to 32 bytes per row)
/// 
/// # Layout
/// ```text
/// Output is right-aligned, left side filled with 0 (transparent)
/// Row 0:                              [##] offset=30, width=2
/// Row 1:                          [####]   offset=28, width=4
/// ...
/// Row 15: [##############################] offset=0, width=32
/// ...
/// Row 30:                              [##] offset=30, width=2
/// ```
/// 
/// # Reference
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsRightTriangle()` Line 74-112
pub fn decode_right_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    const OUTPUT_SIZE: usize = 32 * 31;
    let mut output = vec![0u8; OUTPUT_SIZE];
    let mut src = 0;
    let mut dst = 0;
    
    // Lower half (rows 0-15): 16 rows, paired (8 pairs)
    // Reference: Source/levels/reencode_dun_cels.cpp:81-94
    for i in 0..8 {
        // First row of pair (width: 2, 6, 10, 14, 18, 22, 26, 30)
        // Padding AFTER the row data (not before)
        // Reference: Line 83-86
        let width1 = 2 + i * 4;
        if src + width1 > raw_data.len() {
            bail!("RightTriangle: insufficient data for row {} (need {})", i*2, width1);
        }
        let offset1 = 32 - width1;
        output[dst + offset1..dst + offset1 + width1]
            .copy_from_slice(&raw_data[src..src + width1]);
        src += width1 + 2;  // +2 for padding AFTER
        dst += 32;
        
        // Second row of pair (width: 4, 8, 12, 16, 20, 24, 28, 32)
        // No padding after second row
        // Reference: Line 87-90
        let width2 = 4 + i * 4;
        if src + width2 > raw_data.len() {
            bail!("RightTriangle: insufficient data for row {} (need {})", i*2+1, width2);
        }
        let offset2 = 32 - width2;
        output[dst + offset2..dst + offset2 + width2]
            .copy_from_slice(&raw_data[src..src + width2]);
        src += width2;
        dst += 32;
    }
    
    // Upper half (rows 16-30): 15 rows, mostly paired (7 pairs + 1 single)
    // Reference: Source/levels/reencode_dun_cels.cpp:96-109
    let mut width = 32;
    
    // 7 pairs (rows 16-29)
    for _ in 0..7 {
        // First row (width: 30, 26, 22, 18, 14, 10, 6)
        // Reference: Line 98-101
        width -= 2;
        if src + width > raw_data.len() {
            bail!("RightTriangle: insufficient data for upper row (need {})", width);
        }
        let offset = 32 - width;
        output[dst + offset..dst + offset + width]
            .copy_from_slice(&raw_data[src..src + width]);
        src += width + 2;  // +2 for padding AFTER
        dst += 32;
        
        // Second row (width: 28, 24, 20, 16, 12, 8, 4)
        // Reference: Line 102-105
        width -= 2;
        if src + width > raw_data.len() {
            bail!("RightTriangle: insufficient data for upper row (need {})", width);
        }
        let offset = 32 - width;
        output[dst + offset..dst + offset + width]
            .copy_from_slice(&raw_data[src..src + width]);
        src += width;
        dst += 32;
    }
    
    // Last row (row 30, width=2)
    // Reference: Line 107-109
    width -= 2;
    if src + width > raw_data.len() {
        bail!("RightTriangle: insufficient data for last row (need {})", width);
    }
    let offset = 32 - width;
    output[dst + offset..dst + offset + width]
        .copy_from_slice(&raw_data[src..src + width]);
    
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
    
    #[test]
    fn test_decode_left_triangle_widths() {
        // Create minimal valid test data
        let mut raw = Vec::new();
        
        // Lower half (8 pairs)
        for i in 0..8 {
            raw.extend_from_slice(&[0xFF, 0xFF]);  // padding
            let width1 = 2 + i * 4;
            raw.extend(vec![1u8; width1]);
            let width2 = 4 + i * 4;
            raw.extend(vec![1u8; width2]);
        }
        
        // Upper half (7 pairs + 1 single)
        let mut width = 32;
        for _ in 0..7 {
            raw.extend_from_slice(&[0xFF, 0xFF]);  // padding
            width -= 2;
            raw.extend(vec![1u8; width]);
            width -= 2;
            raw.extend(vec![1u8; width]);
        }
        raw.extend_from_slice(&[0xFF, 0xFF]);  // padding for last row
        width -= 2;
        raw.extend(vec![1u8; width]);
        
        let output = decode_left_triangle(&raw).unwrap();
        
        // Verify output size
        assert_eq!(output.len(), 32 * 31);
        
        // Verify row widths (lower half)
        let expected_widths_lower = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ];
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let row_data = &output[row * 32..(row + 1) * 32];
            let actual = count_non_zero(row_data);
            assert_eq!(
                actual, expected_width,
                "Row {} width wrong: {} (expected {})",
                row, actual, expected_width
            );
            
            // Verify left-aligned (no leading zeros)
            let leading = count_leading_zeros(row_data);
            assert_eq!(leading, 0, "Row {} not left-aligned", row);
        }
        
        // Verify row widths (upper half)
        let expected_widths_upper = [
            30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2,
        ];
        for (i, &expected_width) in expected_widths_upper.iter().enumerate() {
            let row = 16 + i;
            let row_data = &output[row * 32..(row + 1) * 32];
            let actual = count_non_zero(row_data);
            assert_eq!(
                actual, expected_width,
                "Row {} width wrong: {} (expected {})",
                row, actual, expected_width
            );
        }
    }
    
    #[test]
    fn test_decode_right_triangle_widths() {
        // Create minimal valid test data
        let mut raw = Vec::new();
        
        // Lower half (8 pairs)
        for i in 0..8 {
            let width1 = 2 + i * 4;
            raw.extend(vec![1u8; width1]);
            raw.extend_from_slice(&[0xFF, 0xFF]);  // padding AFTER
            let width2 = 4 + i * 4;
            raw.extend(vec![1u8; width2]);
        }
        
        // Upper half (7 pairs + 1 single)
        let mut width = 32;
        for _ in 0..7 {
            width -= 2;
            raw.extend(vec![1u8; width]);
            raw.extend_from_slice(&[0xFF, 0xFF]);  // padding AFTER
            width -= 2;
            raw.extend(vec![1u8; width]);
        }
        width -= 2;
        raw.extend(vec![1u8; width]);
        
        let output = decode_right_triangle(&raw).unwrap();
        
        // Verify output size
        assert_eq!(output.len(), 32 * 31);
        
        // Verify row widths and right-alignment
        let expected_widths_lower = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ];
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let row_data = &output[row * 32..(row + 1) * 32];
            let actual = count_non_zero(row_data);
            assert_eq!(
                actual, expected_width,
                "Row {} width wrong: {} (expected {})",
                row, actual, expected_width
            );
            
            // Verify right-aligned (leading zeros = 32 - width)
            let leading = count_leading_zeros(row_data);
            let expected_leading = 32 - expected_width;
            assert_eq!(
                leading, expected_leading,
                "Row {} not right-aligned (leading zeros: {}, expected: {})",
                row, leading, expected_leading
            );
        }
    }
    
    #[test]
    fn test_decode_left_triangle_insufficient_data() {
        let raw = vec![0u8; 10];  // Too short
        let result = decode_left_triangle(&raw);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_decode_right_triangle_insufficient_data() {
        let raw = vec![0u8; 10];  // Too short
        let result = decode_right_triangle(&raw);
        assert!(result.is_err());
    }
}














