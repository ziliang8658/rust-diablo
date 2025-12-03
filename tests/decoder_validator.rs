/// Decoder Validator
/// 
/// Validates that decoded tiles match their expected geometric characteristics.
/// 
/// This validator checks that decoded tile data conforms to the expected
/// shape and alignment for each TileType, without requiring comparison
/// to original C++ output.
/// 
/// # Validation Strategy
/// 
/// Each TileType has unique geometric characteristics:
/// - Square: All rows width=32
/// - LeftTriangle: Variable-width rows, left-aligned
/// - RightTriangle: Variable-width rows, right-aligned
/// - LeftTrapezoid: Lower triangle + upper rectangle (left-aligned)
/// - RightTrapezoid: Lower triangle + upper rectangle (right-aligned)
/// - TransparentSquare: 32×32 with transparent pixels

use rust_diablo::tiles::types::TileType;

/// Decoder validator
pub struct DecoderValidator;

impl DecoderValidator {
    /// Count non-zero pixels in a row
    fn count_row_pixels(pixels: &[u8], row: usize, width: usize) -> usize {
        let start = row * width;
        let end = start + width;
        if end > pixels.len() {
            return 0;
        }
        pixels[start..end].iter().filter(|&&p| p != 0).count()
    }
    
    /// Count leading transparent pixels in a row
    fn count_left_transparent(pixels: &[u8], row: usize, width: usize) -> usize {
        let start = row * width;
        let end = start + width;
        if end > pixels.len() {
            return 0;
        }
        pixels[start..end].iter().take_while(|&&p| p == 0).count()
    }
    
    /// Validate Square tile
    pub fn validate_square(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 32 {
            return Err(format!("Square size wrong: {}, expected 1024", pixels.len()));
        }
        
        for row in 0..32 {
            let count = Self::count_row_pixels(pixels, row, 32);
            if count != 32 {
                return Err(format!(
                    "Square row {} not full: {} pixels (expected 32)",
                    row, count
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate LeftTriangle tile
    pub fn validate_left_triangle(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 31 {
            return Err(format!("LeftTriangle size wrong: {}, expected 992", pixels.len()));
        }
        
        // Expected widths for all 31 rows
        let expected_widths = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,  // rows 0-15
            30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2,      // rows 16-30
        ];
        
        for (row, &expected_width) in expected_widths.iter().enumerate() {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "LeftTriangle row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
            
            // Verify left-aligned (no leading zeros)
            let left_trans = Self::count_left_transparent(pixels, row, 32);
            if left_trans != 0 {
                return Err(format!(
                    "LeftTriangle row {} not left-aligned: {} left transparent pixels",
                    row, left_trans
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate RightTriangle tile
    pub fn validate_right_triangle(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 31 {
            return Err(format!("RightTriangle size wrong: {}, expected 992", pixels.len()));
        }
        
        // Expected widths for all 31 rows
        let expected_widths = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,  // rows 0-15
            30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2,      // rows 16-30
        ];
        
        for (row, &expected_width) in expected_widths.iter().enumerate() {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "RightTriangle row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
            
            // Verify right-aligned (leading zeros = 32 - width)
            let left_trans = Self::count_left_transparent(pixels, row, 32);
            let expected_left_trans = 32 - expected_width;
            if left_trans != expected_left_trans {
                return Err(format!(
                    "RightTriangle row {} not right-aligned: {} left transparent (expected {})",
                    row, left_trans, expected_left_trans
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate LeftTrapezoid tile
    pub fn validate_left_trapezoid(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 32 {
            return Err(format!("LeftTrapezoid size wrong: {}, expected 1024", pixels.len()));
        }
        
        // Lower half (rows 0-15): LeftTriangle lower half
        let expected_widths_lower = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ];
        
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "LeftTrapezoid row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
        }
        
        // Upper half (rows 16-31): 32×16 rectangle
        for row in 16..32 {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != 32 {
                return Err(format!(
                    "LeftTrapezoid row {} not full: {} pixels (expected 32)",
                    row, actual
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate RightTrapezoid tile
    pub fn validate_right_trapezoid(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 32 {
            return Err(format!("RightTrapezoid size wrong: {}, expected 1024", pixels.len()));
        }
        
        // Lower half (rows 0-15): RightTriangle lower half
        let expected_widths_lower = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ];
        
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "RightTrapezoid row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
            
            // Verify right-aligned
            let left_trans = Self::count_left_transparent(pixels, row, 32);
            let expected_left_trans = 32 - expected_width;
            if left_trans != expected_left_trans {
                return Err(format!(
                    "RightTrapezoid row {} not right-aligned",
                    row
                ));
            }
        }
        
        // Upper half (rows 16-31): 32×16 rectangle
        for row in 16..32 {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != 32 {
                return Err(format!(
                    "RightTrapezoid row {} not full: {} pixels",
                    row, actual
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate TransparentSquare tile
    pub fn validate_transparent_square(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 32 {
            return Err(format!("TransparentSquare size wrong: {}, expected 1024",
                pixels.len()));
        }
        
        // TransparentSquare should have at least some transparent pixels
        let transparent_count = pixels.iter().filter(|&&p| p == 0).count();
        if transparent_count == 0 {
            return Err("TransparentSquare has no transparent pixels".to_string());
        }
        
        Ok(())
    }
    
    /// Unified validation interface
    pub fn validate(tile_type: TileType, pixels: &[u8]) -> Result<(), String> {
        match tile_type {
            TileType::Square => Self::validate_square(pixels),
            TileType::LeftTriangle => Self::validate_left_triangle(pixels),
            TileType::RightTriangle => Self::validate_right_triangle(pixels),
            TileType::LeftTrapezoid => Self::validate_left_trapezoid(pixels),
            TileType::RightTrapezoid => Self::validate_right_trapezoid(pixels),
            TileType::TransparentSquare => Self::validate_transparent_square(pixels),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validator_square() {
        let pixels = vec![1u8; 1024];
        assert!(DecoderValidator::validate_square(&pixels).is_ok());
    }
    
    #[test]
    fn test_validator_square_wrong_size() {
        let pixels = vec![1u8; 100];
        assert!(DecoderValidator::validate_square(&pixels).is_err());
    }
    
    #[test]
    fn test_validator_left_triangle() {
        // Create valid LeftTriangle pattern
        let mut pixels = vec![0u8; 32 * 31];
        let widths = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
            30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2,
        ];
        
        for (row, &width) in widths.iter().enumerate() {
            for col in 0..width {
                pixels[row * 32 + col] = 1;
            }
        }
        
        assert!(DecoderValidator::validate_left_triangle(&pixels).is_ok());
    }
}














