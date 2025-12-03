/// Square Tile Decoder
/// 
/// Decodes 32×32 square tiles (pure rectangular tiles).
/// 
/// # Format
/// 
/// Square tiles are stored as a flat array of 1024 bytes (32×32).
/// No padding, no encoding, just raw indexed color pixels.
/// 
/// # Reference
/// 
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsSquare()`
/// Line 19-32:
/// ```cpp
/// void ReencodeDungeonCelsSquare(uint8_t *pDst, const uint8_t *pSrc)
/// {
///     constexpr size_t NumPixels = DunFrameWidth * DunFrameHeight;
///     memcpy(pDst, pSrc, NumPixels);
/// }
/// ```

use anyhow::{Result, bail};

/// Decode a Square tile (32×32 raw pixels)
/// 
/// # Arguments
/// * `raw_data` - Raw pixel data (must be >= 1024 bytes)
/// 
/// # Returns
/// Decoded pixels (1024 bytes, indexed color 0-255, 0 = transparent)
/// 
/// # Layout
/// ```text
/// 32×32 pure pixel array, no padding
/// Total size: 1024 bytes
/// ```
/// 
/// # Reference
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsSquare()` Line 19-32
pub fn decode_square(raw_data: &[u8]) -> Result<Vec<u8>> {
    const EXPECTED_SIZE: usize = 32 * 32;
    
    if raw_data.len() < EXPECTED_SIZE {
        bail!(
            "Square tile data too short: expected {} bytes, got {}",
            EXPECTED_SIZE,
            raw_data.len()
        );
    }
    
    // Direct copy - Square tiles are stored unencoded
    // Reference: Source/levels/reencode_dun_cels.cpp:28
    Ok(raw_data[..EXPECTED_SIZE].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decode_square_valid() {
        // Create 32×32 test data
        let mut raw = vec![0u8; 1024];
        
        // Fill with test pattern
        for i in 0..1024 {
            raw[i] = (i % 256) as u8;
        }
        
        let decoded = decode_square(&raw).unwrap();
        
        assert_eq!(decoded.len(), 1024);
        assert_eq!(decoded[0], 0);
        assert_eq!(decoded[1], 1);
        assert_eq!(decoded[255], 255);
        assert_eq!(decoded[256], 0);  // Wraps around
    }
    
    #[test]
    fn test_decode_square_checkerboard() {
        // Create checkerboard pattern
        let mut raw = vec![0u8; 1024];
        for y in 0..32 {
            for x in 0..32 {
                raw[y * 32 + x] = if (x + y) % 2 == 0 { 1 } else { 2 };
            }
        }
        
        let decoded = decode_square(&raw).unwrap();
        
        // Verify pattern
        assert_eq!(decoded[0], 1);  // (0,0) -> 1
        assert_eq!(decoded[1], 2);  // (0,1) -> 2
        assert_eq!(decoded[32], 2); // (1,0) -> 2
        assert_eq!(decoded[33], 1); // (1,1) -> 1
    }
    
    #[test]
    fn test_decode_square_too_short() {
        let raw = vec![0u8; 100];  // Too short
        let result = decode_square(&raw);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_decode_square_extra_data() {
        // Extra data should be ignored
        let raw = vec![42u8; 2000];
        let decoded = decode_square(&raw).unwrap();
        assert_eq!(decoded.len(), 1024);
        assert_eq!(decoded[0], 42);
    }
}














