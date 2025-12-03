/// TransparentSquare Tile Decoder
///
/// Decodes 32×32 transparent square tiles with RLE (Run-Length Encoding).
///
/// # Format
///
/// TransparentSquare tiles use RLE encoding to save space for tiles with
/// many transparent pixels. Each row is encoded independently.
///
/// # RLE Encoding
///
/// Each run starts with a control byte (int8_t):
/// - **Positive value (n > 0)**: Next n bytes are actual pixel data (copy them)
/// - **Negative value (n < 0)**: Next -n pixels are transparent (skip them)
/// - **Zero (n = 0)**: Zero transparent pixels (rare edge case)
///
/// Runs do NOT cross row boundaries (each row is encoded separately).
///
/// # Example
/// ```text
/// Control byte: 5    → Copy next 5 bytes as actual pixels
/// Control byte: -10  → Skip 10 pixels (transparent)
/// Control byte: 3    → Copy next 3 bytes
/// Control byte: -14  → Skip 14 pixels (to reach end of row)
/// ```
///
/// # Reference
///
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsTransparentSquare()` Line 176-204
use anyhow::{bail, Result};

/// Decode a TransparentSquare tile (32×32 RLE-encoded)
///
/// # Arguments
/// * `raw_data` - RLE-encoded data
///
/// # Returns
/// Decoded pixels (1024 bytes = 32×32, 0 = transparent)
///
/// # RLE Decoding Algorithm
/// ```text
/// For each of 32 rows:
///   remaining_width = 32
///   while remaining_width > 0:
///     control_byte = read_i8()
///     if control_byte > 0:
///       // Actual pixels
///       copy next control_byte pixels
///       remaining_width -= control_byte
///     else:
///       // Transparent pixels
///       skip -control_byte pixels (fill with 0)
///       remaining_width -= (-control_byte)
/// ```
///
/// # Reference
/// Original code: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsTransparentSquare()` Line 176-204
pub fn decode_transparent_square(raw_data: &[u8]) -> Result<Vec<u8>> {
    const OUTPUT_SIZE: usize = 32 * 32;
    const ROW_WIDTH: usize = 32;
    const NUM_ROWS: usize = 32;

    let mut output = vec![0u8; OUTPUT_SIZE];
    let mut src = 0;
    let mut dst = 0;

    // Decode 32 rows independently
    // Reference: Source/levels/reencode_dun_cels.cpp:183-202
    for row in 0..NUM_ROWS {
        let mut draw_width = ROW_WIDTH;

        // Decode current row until we reach row_width pixels
        // Reference: Line 186
        while draw_width > 0 {
            if src >= raw_data.len() {
                bail!(
                    "TransparentSquare: RLE data incomplete at row {} (dst={}, remaining_width={})",
                    row,
                    dst,
                    draw_width
                );
            }

            // Read control byte as signed int8
            // Reference: Line 187
            let val = raw_data[src] as i8;
            src += 1;

            if val > 0 {
                // Positive: val actual pixel bytes follow
                // Reference: Line 188-192
                let count = val as usize;

                if count > draw_width {
                    bail!(
                        "TransparentSquare: RLE run too long at row {} (count={}, remaining={})",
                        row,
                        count,
                        draw_width
                    );
                }

                if src + count > raw_data.len() {
                    bail!(
                        "TransparentSquare: insufficient data for RLE run (need {} bytes)",
                        count
                    );
                }

                output[dst..dst + count].copy_from_slice(&raw_data[src..src + count]);
                src += count;
                dst += count;
                draw_width -= count;
            } else {
                // Negative or zero: -val transparent pixels (already initialized to 0)
                // Reference: Line 193-197
                let count = (-val) as usize;

                if count > draw_width {
                    bail!(
                        "TransparentSquare: RLE skip too long at row {} (count={}, remaining={})",
                        row,
                        count,
                        draw_width
                    );
                }

                // Transparent pixels are already 0 (vec initialized with 0), just advance pointer
                dst += count;
                draw_width -= count;
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_transparent_square_simple() {
        // Create simple RLE data: all rows are "5 actual pixels, 27 transparent"
        let mut raw = Vec::new();

        for _ in 0..32 {
            raw.push(5i8 as u8); // 5 actual pixels
            raw.extend_from_slice(&[1, 2, 3, 4, 5]);
            raw.push((-27i8) as u8); // 27 transparent pixels
        }

        let output = decode_transparent_square(&raw).unwrap();

        // Verify output size
        assert_eq!(output.len(), 1024);

        // Verify first row
        assert_eq!(&output[0..5], &[1, 2, 3, 4, 5]);
        assert_eq!(&output[5..32], &[0u8; 27]); // Transparent

        // Verify last row
        assert_eq!(&output[31 * 32..31 * 32 + 5], &[1, 2, 3, 4, 5]);
        assert_eq!(&output[31 * 32 + 5..32 * 32], &[0u8; 27]); // Transparent
    }

    #[test]
    fn test_decode_transparent_square_full_row() {
        // Test full rows (no transparency)
        let mut raw = Vec::new();

        for row in 0..32 {
            raw.push(32i8 as u8); // 32 actual pixels
            for x in 0..32 {
                raw.push((row * 32 + x) as u8);
            }
        }

        let output = decode_transparent_square(&raw).unwrap();

        // Verify output
        for i in 0..1024 {
            assert_eq!(output[i], (i % 256) as u8);
        }
    }

    #[test]
    fn test_decode_transparent_square_full_transparent() {
        // Test full transparent rows
        let mut raw = Vec::new();

        for _ in 0..32 {
            raw.push((-32i8) as u8); // 32 transparent pixels
        }

        let output = decode_transparent_square(&raw).unwrap();

        // All pixels should be 0 (transparent)
        assert_eq!(output, vec![0u8; 1024]);
    }

    #[test]
    fn test_decode_transparent_square_mixed() {
        // Test mixed actual and transparent pixels
        let mut raw = Vec::new();

        for row in 0..32 {
            // Pattern: 10 actual, 5 transparent, 10 actual, 7 transparent
            raw.push(10i8 as u8);
            raw.extend(vec![row as u8; 10]);

            raw.push((-5i8) as u8); // 5 transparent

            raw.push(10i8 as u8);
            raw.extend(vec![(row + 1) as u8; 10]);

            raw.push((-7i8) as u8); // 7 transparent
        }

        let output = decode_transparent_square(&raw).unwrap();

        // Verify first row pattern
        for i in 0..10 {
            assert_eq!(output[i], 0);
        }
        for i in 10..15 {
            assert_eq!(output[i], 0); // Transparent
        }
        for i in 15..25 {
            assert_eq!(output[i], 1);
        }
        for i in 25..32 {
            assert_eq!(output[i], 0); // Transparent
        }
    }

    #[test]
    fn test_decode_transparent_square_zero_run() {
        // Test zero-length run (edge case)
        let mut raw = Vec::new();

        for _ in 0..32 {
            raw.push(0i8 as u8); // 0 transparent pixels
            raw.push(32i8 as u8); // 32 actual pixels
            raw.extend(vec![42u8; 32]);
        }

        let output = decode_transparent_square(&raw).unwrap();

        // All pixels should be 42
        assert_eq!(output, vec![42u8; 1024]);
    }

    #[test]
    fn test_decode_transparent_square_incomplete_data() {
        // Test incomplete RLE data
        let raw = vec![5i8 as u8, 1, 2]; // Says 5 bytes but only provides 2

        let result = decode_transparent_square(&raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_transparent_square_run_too_long() {
        // Test RLE run that exceeds row width
        let mut raw = Vec::new();
        raw.push(40i8 as u8); // Says 40 bytes but row is only 32 wide
        raw.extend(vec![1u8; 40]);

        let result = decode_transparent_square(&raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_rle_signed_byte_handling() {
        // Test correct handling of signed bytes
        let mut raw = Vec::new();

        for _ in 0..32 {
            // Test negative values (should be transparent)
            raw.push((-16i8) as u8); // 16 transparent
            raw.push(16i8 as u8); // 16 actual
            raw.extend(vec![255u8; 16]);
        }

        let output = decode_transparent_square(&raw).unwrap();

        // Verify first row: 16 transparent, 16 actual
        assert_eq!(&output[0..16], &[0u8; 16]);
        assert_eq!(&output[16..32], &[255u8; 16]);
    }

    #[test]
    fn test_transparent_square_statistics() {
        // Test that TransparentSquare actually has transparent pixels
        let mut raw = Vec::new();

        // Create pattern with 50% transparency
        for _ in 0..32 {
            raw.push(16i8 as u8);
            raw.extend(vec![100u8; 16]);
            raw.push((-16i8) as u8);
        }

        let output = decode_transparent_square(&raw).unwrap();

        // Count transparent pixels
        let transparent_count = output.iter().filter(|&&p| p == 0).count();
        let opaque_count = output.iter().filter(|&&p| p != 0).count();

        assert_eq!(transparent_count, 512);
        assert_eq!(opaque_count, 512);

        println!(
            "TransparentSquare: {} transparent, {} opaque pixels",
            transparent_count, opaque_count
        );
    }
}
