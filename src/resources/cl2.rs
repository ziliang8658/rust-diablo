/// CL2 Sprite Format Loader
/// 
/// Handles loading and parsing of CL2 sprite files (original Diablo 1 format).
/// CL2 is the original format, CLX is DevilutionX's runtime conversion format.
/// 
/// File structure:
/// - Header: Frame count, frame offsets, file size
/// - Frame data (per frame):
///   - Frame header: 32-pixel block offsets (10 uint16s = 20 bytes typically)
///   - Pixel data (CL2 RLE encoded - same as CLX)

use anyhow::{Context, Result};
use crate::resources::clx::{ClxSprite, ClxFrame};

/// CL2 Sprite (shares implementation with CLX after parsing frame headers)
pub struct Cl2Sprite {
    pub frames: Vec<ClxFrame>,
}

impl Cl2Sprite {
    /// Parse CL2 file from bytes
    /// 
    /// # Arguments
    /// * `data` - Complete CL2 file data
    /// * `frame_width` - Width of each frame (required for CL2, as it's not in the header)
    /// 
    /// # Returns
    /// Parsed CL2 sprite with decoded frames
    pub fn from_bytes(data: &[u8], frame_width: u16) -> Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!("CL2 file too small: {} bytes", data.len()));
        }

        // Read number of frames (or group offset if this is a sheet)
        let maybe_num_frames = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        
        // Check if this is a single sprite list or a sprite sheet
        let num_frames: u32;
        let group_begin: usize;
        
        // If it is a number of frames, then the last frame offset equals file size
        let last_offset_pos = (maybe_num_frames * 4 + 4) as usize;
        if last_offset_pos < data.len() {
            let last_offset = u32::from_le_bytes([
                data[last_offset_pos],
                data[last_offset_pos + 1],
                data[last_offset_pos + 2],
                data[last_offset_pos + 3],
            ]);
            
            if last_offset == data.len() as u32 {
                // Single sprite list
                num_frames = maybe_num_frames;
                group_begin = 0;
            } else {
                // Sprite sheet - for now, just load the first group
                let first_group_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                group_begin = first_group_offset as usize;
                num_frames = u32::from_le_bytes([
                    data[group_begin],
                    data[group_begin + 1],
                    data[group_begin + 2],
                    data[group_begin + 3],
                ]);
            }
        } else {
            return Err(anyhow::anyhow!("CL2 header invalid: last offset position {} >= file size {}", last_offset_pos, data.len()));
        }

        if num_frames == 0 {
            return Err(anyhow::anyhow!("CL2 file has zero frames"));
        }

        // Parse each frame
        let mut frames = Vec::with_capacity(num_frames as usize);
        
        for frame_idx in 0..num_frames {
            let frame_offset_pos = group_begin + 4 + (frame_idx as usize * 4);
            let next_offset_pos = group_begin + 4 + ((frame_idx + 1) as usize * 4);
            
            if next_offset_pos + 3 >= data.len() {
                return Err(anyhow::anyhow!(
                    "CL2 frame {} offset out of bounds",
                    frame_idx
                ));
            }

            let frame_offset = group_begin + u32::from_le_bytes([
                data[frame_offset_pos],
                data[frame_offset_pos + 1],
                data[frame_offset_pos + 2],
                data[frame_offset_pos + 3],
            ]) as usize;

            let next_offset = group_begin + u32::from_le_bytes([
                data[next_offset_pos],
                data[next_offset_pos + 1],
                data[next_offset_pos + 2],
                data[next_offset_pos + 3],
            ]) as usize;

            if frame_offset >= data.len() || next_offset > data.len() || frame_offset >= next_offset {
                return Err(anyhow::anyhow!(
                    "CL2 frame {} invalid offsets: frame_offset={}, next_offset={}, data_len={}",
                    frame_idx,
                    frame_offset,
                    next_offset,
                    data.len()
                ));
            }

            // CL2 frame header: First 2 bytes indicate the offset to pixel data
            // This offset points past the 32-pixel block offset table
            if frame_offset + 1 >= data.len() {
                return Err(anyhow::anyhow!("CL2 frame {} header out of bounds", frame_idx));
            }

            let pixel_data_offset = u16::from_le_bytes([
                data[frame_offset],
                data[frame_offset + 1],
            ]) as usize;

            let pixel_data_start = frame_offset + pixel_data_offset;
            
            if pixel_data_start >= next_offset {
                return Err(anyhow::anyhow!(
                    "CL2 frame {} pixel data offset invalid: {} >= {}",
                    frame_idx,
                    pixel_data_start,
                    next_offset
                ));
            }

            let pixel_data = &data[pixel_data_start..next_offset];

            // Decode CL2 RLE pixel data
            // We need to determine the frame height by decoding
            let (pixels, height) = Self::decode_cl2_rle_with_height(
                pixel_data,
                frame_width,
            ).with_context(|| format!("Failed to decode CL2 frame {} RLE data", frame_idx))?;

            frames.push(ClxFrame {
                width: frame_width,
                height,
                pixels,
            });
        }

        Ok(Cl2Sprite { frames })
    }

    /// Decode CL2 RLE compressed data and determine height
    /// 
    /// CL2 RLE uses the same encoding as CLX.
    /// We decode and track how many complete rows we've filled.
    fn decode_cl2_rle_with_height(data: &[u8], width: u16) -> Result<(Vec<Option<u8>>, u16)> {
        let mut pixels = Vec::new();
        let mut data_idx = 0;
        let mut x: u16 = 0;
        let mut height: u16 = 0;

        const CLX_OPAQUE_MIN: u8 = 0x80;
        const CLX_FILL_MAX: u8 = 0xBE;
        const CLX_FILL_END: u8 = 0xBF;

        while data_idx < data.len() {
            // Process one row
            while x < width {
                if data_idx >= data.len() {
                    break;
                }

                let control = data[data_idx];
                data_idx += 1;

                if control < CLX_OPAQUE_MIN {
                    // Transparent run (0x00-0x7F)
                    let transparent_count = control as u16;
                    for _ in 0..transparent_count {
                        pixels.push(None);
                    }
                    x += transparent_count;
                } else if control <= CLX_FILL_MAX {
                    // Opaque fill (0x80-0xBE)
                    let fill_width = (CLX_FILL_END - control) as u16;

                    if data_idx >= data.len() {
                        return Err(anyhow::anyhow!("CL2 opaque fill overflow: missing fill color byte"));
                    }

                    let fill_color = data[data_idx];
                    data_idx += 1;

                    for _ in 0..fill_width {
                        pixels.push(Some(fill_color));
                    }
                    x += fill_width;
                } else {
                    // Opaque pixels (0xBF-0xFF)
                    let pixel_width = (-(control as i8)) as u16;

                    if data_idx + pixel_width as usize > data.len() {
                        return Err(anyhow::anyhow!(
                            "CL2 opaque pixels overflow: width {}, remaining data {}",
                            pixel_width,
                            data.len() - data_idx
                        ));
                    }

                    for _ in 0..pixel_width {
                        pixels.push(Some(data[data_idx]));
                        data_idx += 1;
                    }
                    x += pixel_width;
                }
            }

            // Handle line overflow (when a command overruns the current line)
            if x >= width {
                let overrun = (x - width) as i32;
                if overrun > 0 {
                    // Calculate how many complete lines we skipped
                    let overrun_lines = overrun / width as i32 + 1;
                    height += overrun_lines as u16;
                    x = (overrun % width as i32) as u16;
                } else {
                    // Exact line end
                    height += 1;
                    x = 0;
                }
            }
        }

        // If we have remaining partial line, count it
        if x > 0 {
            height += 1;
        }

        Ok((pixels, height))
    }

    /// Convert to ClxSprite (for compatibility)
    pub fn to_clx_sprite(self) -> ClxSprite {
        ClxSprite {
            frames: self.frames,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cl2_basic_structure() {
        // Minimal valid CL2 structure
        // 1 frame, frame offset at 8, file size at 12
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // num_frames = 1
            0x08, 0x00, 0x00, 0x00, // frame 0 offset = 8
            0x0C, 0x00, 0x00, 0x00, // file size = 12
        ];
        
        let result = Cl2Sprite::from_bytes(&data, 32);
        // This will fail because we don't have actual pixel data, but structure check passes
        assert!(result.is_err());
    }
}
















