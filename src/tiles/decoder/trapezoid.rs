/// Trapezoid Tile Decoder
/// 
/// Decodes 32×32 trapezoid tiles (LeftTrapezoid and RightTrapezoid).
/// 
/// # Format
/// 
/// Trapezoid tiles consist of two parts:
/// 1. Lower half (rows 0-15): Triangle part (variable-width rows)
/// 2. Upper half (rows 16-31): Rectangle part (fixed 32-width rows)
/// 
/// The triangle part uses the same encoding as Triangle tiles (with padding).
/// The rectangle part is a flat array of 32×16 = 512 bytes.
/// 
/// # References
/// 
/// Original code: `Source/levels/reencode_dun_cels.cpp`
/// - ReencodeDungeonCelsLeftTrapezoid() Line 114-143
/// - ReencodeDungeonCelsRightTrapezoid() Line 145-174

use anyhow::{Result, bail};

/// Decode a LeftTrapezoid tile (lower triangle + upper rectangle, left-aligned)
/// 
/// # Arguments
/// * `raw_data` - Raw encoded data
/// 
/// # Returns
/// Decoded pixels (1024 bytes = 32×32, row-major)
/// 
/// # Layout
/// ```text
/// Rows 0-15:  LeftTriangle lower half (variable-width, left-aligned)
/// Rows 16-31: 32×16 rectangle (fixed width, full 32 pixels)
/// ```
/// 
/// # Reference
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsLeftTrapezoid()` Line 114-143
pub fn decode_left_trapezoid(raw_data: &[u8]) -> Result<Vec<u8>> {
    const OUTPUT_SIZE: usize = 32 * 32;
    let mut output = vec![0u8; OUTPUT_SIZE];
    let mut src = 0;
    let mut dst = 0;
    
    // Lower half (rows 0-15): LeftTriangle lower half
    // Reference: Source/levels/reencode_dun_cels.cpp:121-134
    for i in 0..8 {
        // Skip padding before first row of pair
        // Reference: Line 123
        src += 2;
        
        if src > raw_data.len() {
            bail!("LeftTrapezoid: insufficient data at pair {} (src={})", i, src);
        }
        
        // First row (width: 2, 6, 10, 14, 18, 22, 26, 30)
        // Reference: Line 125-127
        let width1 = 2 + i * 4;
        if src + width1 > raw_data.len() {
            bail!("LeftTrapezoid: insufficient data for row {} (need {})", i*2, width1);
        }
        output[dst..dst + width1].copy_from_slice(&raw_data[src..src + width1]);
        src += width1;
        dst += 32;
        
        // Second row (width: 4, 8, 12, 16, 20, 24, 28, 32)
        // Reference: Line 128-130
        let width2 = 4 + i * 4;
        if src + width2 > raw_data.len() {
            bail!("LeftTrapezoid: insufficient data for row {} (need {})", i*2+1, width2);
        }
        output[dst..dst + width2].copy_from_slice(&raw_data[src..src + width2]);
        src += width2;
        dst += 32;
    }
    
    // Upper half (rows 16-31): 32×16 pure rectangle
    // Reference: Source/levels/reencode_dun_cels.cpp:136-141
    const RECT_SIZE: usize = 32 * 16;
    if src + RECT_SIZE > raw_data.len() {
        bail!(
            "LeftTrapezoid: insufficient data for rectangle part (need {} bytes, have {})",
            RECT_SIZE,
            raw_data.len() - src
        );
    }
    output[dst..dst + RECT_SIZE].copy_from_slice(&raw_data[src..src + RECT_SIZE]);
    
    Ok(output)
}

/// Decode a RightTrapezoid tile (lower triangle + upper rectangle, right-aligned)
/// 
/// # Arguments
/// * `raw_data` - Raw encoded data
/// 
/// # Returns
/// Decoded pixels (1024 bytes = 32×32, row-major)
/// 
/// # Layout
/// ```text
/// Rows 0-15:  RightTriangle lower half (variable-width, right-aligned)
/// Rows 16-31: 32×16 rectangle (fixed width, full 32 pixels)
/// ```
/// 
/// # Reference
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsRightTrapezoid()` Line 145-174
pub fn decode_right_trapezoid(raw_data: &[u8]) -> Result<Vec<u8>> {
    const OUTPUT_SIZE: usize = 32 * 32;
    let mut output = vec![0u8; OUTPUT_SIZE];
    let mut src = 0;
    let mut dst = 0;
    
    // Lower half (rows 0-15): RightTriangle lower half
    // Reference: Source/levels/reencode_dun_cels.cpp:152-165
    for i in 0..8 {
        // First row (width: 2, 6, 10, 14, 18, 22, 26, 30)
        // Padding AFTER the row data
        // Reference: Line 154-157
        let width1 = 2 + i * 4;
        if src + width1 > raw_data.len() {
            bail!("RightTrapezoid: insufficient data for row {} (need {})", i*2, width1);
        }
        let offset1 = 32 - width1;
        output[dst + offset1..dst + offset1 + width1]
            .copy_from_slice(&raw_data[src..src + width1]);
        src += width1 + 2;  // +2 for padding AFTER
        dst += 32;
        
        // Second row (width: 4, 8, 12, 16, 20, 24, 28, 32)
        // Reference: Line 158-161
        let width2 = 4 + i * 4;
        if src + width2 > raw_data.len() {
            bail!("RightTrapezoid: insufficient data for row {} (need {})", i*2+1, width2);
        }
        let offset2 = 32 - width2;
        output[dst + offset2..dst + offset2 + width2]
            .copy_from_slice(&raw_data[src..src + width2]);
        src += width2;
        dst += 32;
    }
    
    // Upper half (rows 16-31): 32×16 pure rectangle
    // Reference: Source/levels/reencode_dun_cels.cpp:167-172
    const RECT_SIZE: usize = 32 * 16;
    if src + RECT_SIZE > raw_data.len() {
        bail!(
            "RightTrapezoid: insufficient data for rectangle part (need {} bytes, have {})",
            RECT_SIZE,
            raw_data.len() - src
        );
    }
    output[dst..dst + RECT_SIZE].copy_from_slice(&raw_data[src..src + RECT_SIZE]);
    
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
    fn test_decode_left_trapezoid_structure() {
        // Create minimal valid test data
        let mut raw = Vec::new();
        
        // Lower half: LeftTriangle lower half (8 pairs)
        for i in 0..8 {
            raw.extend_from_slice(&[0xFF, 0xFF]);  // padding
            let width1 = 2 + i * 4;
            raw.extend(vec![1u8; width1]);
            let width2 = 4 + i * 4;
            raw.extend(vec![1u8; width2]);
        }
        
        // Upper half: 32×16 rectangle
        raw.extend(vec![2u8; 32 * 16]);
        
        let output = decode_left_trapezoid(&raw).unwrap();
        
        // Verify output size
        assert_eq!(output.len(), 32 * 32);
        
        // Verify lower half (rows 0-15): variable-width, left-aligned
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
            
            // Verify left-aligned
            if expected_width > 0 {
                let leading = count_leading_zeros(row_data);
                assert_eq!(leading, 0, "Row {} not left-aligned", row);
            }
        }
        
        // Verify upper half (rows 16-31): full width 32
        for row in 16..32 {
            let row_data = &output[row * 32..(row + 1) * 32];
            let actual = count_non_zero(row_data);
            assert_eq!(actual, 32, "Row {} not full width", row);
            assert_eq!(row_data[0], 2, "Row {} should have value 2", row);
        }
    }
    
    #[test]
    fn test_decode_right_trapezoid_structure() {
        // Create minimal valid test data
        let mut raw = Vec::new();
        
        // Lower half: RightTriangle lower half (8 pairs)
        for i in 0..8 {
            let width1 = 2 + i * 4;
            raw.extend(vec![1u8; width1]);
            raw.extend_from_slice(&[0xFF, 0xFF]);  // padding AFTER
            let width2 = 4 + i * 4;
            raw.extend(vec![1u8; width2]);
        }
        
        // Upper half: 32×16 rectangle
        raw.extend(vec![2u8; 32 * 16]);
        
        let output = decode_right_trapezoid(&raw).unwrap();
        
        // Verify output size
        assert_eq!(output.len(), 32 * 32);
        
        // Verify lower half (rows 0-15): variable-width, right-aligned
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
                "Row {} not right-aligned (leading: {}, expected: {})",
                row, leading, expected_leading
            );
        }
        
        // Verify upper half (rows 16-31): full width 32
        for row in 16..32 {
            let row_data = &output[row * 32..(row + 1) * 32];
            let actual = count_non_zero(row_data);
            assert_eq!(actual, 32, "Row {} not full width", row);
            assert_eq!(row_data[0], 2, "Row {} should have value 2", row);
        }
    }
    
    #[test]
    fn test_decode_left_trapezoid_insufficient_data() {
        let raw = vec![0u8; 10];  // Too short
        let result = decode_left_trapezoid(&raw);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_decode_right_trapezoid_insufficient_data() {
        let raw = vec![0u8; 10];  // Too short
        let result = decode_right_trapezoid(&raw);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_trapezoid_transition() {
        // Test that the transition from triangle to rectangle is correct
        let mut raw = Vec::new();
        
        // Lower half (LeftTriangle pattern)
        for i in 0..8 {
            raw.extend_from_slice(&[0xFF, 0xFF]);
            raw.extend(vec![10u8; 2 + i * 4]);
            raw.extend(vec![20u8; 4 + i * 4]);
        }
        
        // Upper half (rectangle)
        raw.extend(vec![30u8; 32 * 16]);
        
        let output = decode_left_trapezoid(&raw).unwrap();
        
        // Row 15 should be full width (last row of triangle)
        let row15 = &output[15 * 32..16 * 32];
        assert_eq!(count_non_zero(row15), 32);
        assert_eq!(row15[0], 20);  // Should have value 20
        
        // Row 16 should be full width (first row of rectangle)
        let row16 = &output[16 * 32..17 * 32];
        assert_eq!(count_non_zero(row16), 32);
        assert_eq!(row16[0], 30);  // Should have value 30
    }
}














