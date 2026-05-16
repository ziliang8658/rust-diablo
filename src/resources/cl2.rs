use crate::resources::clx::{ClxFrame, ClxSprite};
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

/// CL2 Sprite (shares implementation with CLX after parsing frame headers)
#[derive(Debug, Clone)]
pub struct Cl2Sprite {
    pub frames: Vec<ClxFrame>,
}

/// Directional CL2 sheet used by player animations.
///
/// The outer vector stores one animation per direction. Each inner `Cl2Sprite`
/// is a normal frame list for that direction.
#[derive(Debug, Clone)]
pub struct Cl2DirectionalSpriteSheet {
    pub directions: Vec<Cl2Sprite>,
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
        if Self::is_single_list(data)? {
            let frames = Self::parse_frame_list(data, frame_width)?;
            Ok(Cl2Sprite { frames })
        } else {
            let offsets = Self::read_sheet_offsets(data)?;
            let start = offsets[0];
            let frames = Self::parse_frame_list(&data[start..], frame_width)?;
            Ok(Cl2Sprite { frames })
        }
    }

    fn parse_frame_list(data: &[u8], frame_width: u16) -> Result<Vec<ClxFrame>> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!("CL2 file too small: {} bytes", data.len()));
        }

        // Read number of frames from the list header.
        // For directional sheets we call this function on one group at a time.
        let num_frames = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let group_begin = 0usize;

        if num_frames == 0 {
            return Err(anyhow::anyhow!("CL2 file has zero frames"));
        }

        // Parse each frame
        let mut frames = Vec::with_capacity(num_frames);

        for frame_idx in 0..num_frames {
            let frame_offset_pos = group_begin + 4 + (frame_idx * 4);
            let next_offset_pos = group_begin + 4 + ((frame_idx + 1) * 4);

            if next_offset_pos + 3 >= data.len() {
                return Err(anyhow::anyhow!(
                    "CL2 frame {} offset out of bounds",
                    frame_idx
                ));
            }

            let frame_offset = group_begin
                + u32::from_le_bytes([
                    data[frame_offset_pos],
                    data[frame_offset_pos + 1],
                    data[frame_offset_pos + 2],
                    data[frame_offset_pos + 3],
                ]) as usize;

            let next_offset = group_begin
                + u32::from_le_bytes([
                    data[next_offset_pos],
                    data[next_offset_pos + 1],
                    data[next_offset_pos + 2],
                    data[next_offset_pos + 3],
                ]) as usize;

            if frame_offset >= data.len() || next_offset > data.len() || frame_offset >= next_offset
            {
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
                return Err(anyhow::anyhow!(
                    "CL2 frame {} header out of bounds",
                    frame_idx
                ));
            }

            let pixel_data_offset =
                u16::from_le_bytes([data[frame_offset], data[frame_offset + 1]]) as usize;

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
            let (pixels, height) = Self::decode_cl2_rle_with_height(pixel_data, frame_width)
                .with_context(|| format!("Failed to decode CL2 frame {} RLE data", frame_idx))?;

            frames.push(ClxFrame {
                width: frame_width,
                height,
                pixels,
            });
        }

        Ok(frames)
    }

    fn read_sheet_offsets(data: &[u8]) -> Result<Vec<usize>> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!("CL2 file too small: {} bytes", data.len()));
        }

        let first_group_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if first_group_offset < 8 || first_group_offset % 4 != 0 {
            return Err(anyhow::anyhow!(
                "CL2 sheet header invalid: first group offset {}",
                first_group_offset
            ));
        }

        // DevilutionX's Cl2ToClx treats the first DWORD as the byte offset of
        // the first direction group. Since the table is 4-byte offsets with no
        // trailing sentinel, first_group_offset / 4 is the direction count.
        let group_count = first_group_offset / 4;
        if group_count == 0 {
            return Err(anyhow::anyhow!("CL2 sheet has zero direction groups"));
        }

        let mut offsets = Vec::with_capacity(group_count);
        for index in 0..group_count {
            let offset_pos = index * 4;
            if offset_pos + 4 > data.len() {
                return Err(anyhow::anyhow!(
                    "CL2 sheet offset {} out of bounds for file size {}",
                    index,
                    data.len()
                ));
            }

            let group_offset = u32::from_le_bytes([
                data[offset_pos],
                data[offset_pos + 1],
                data[offset_pos + 2],
                data[offset_pos + 3],
            ]) as usize;

            if group_offset >= data.len() {
                return Err(anyhow::anyhow!(
                    "CL2 sheet group {} offset {} out of bounds for file size {}",
                    index,
                    group_offset,
                    data.len()
                ));
            }

            offsets.push(group_offset);
        }

        Ok(offsets)
    }

    fn is_single_list(data: &[u8]) -> Result<bool> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!("CL2 file too small: {} bytes", data.len()));
        }

        let maybe_num_frames = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let last_offset_pos = (maybe_num_frames as usize * 4) + 4;

        if last_offset_pos + 4 > data.len() {
            return Ok(false);
        }

        let last_offset = u32::from_le_bytes([
            data[last_offset_pos],
            data[last_offset_pos + 1],
            data[last_offset_pos + 2],
            data[last_offset_pos + 3],
        ]);

        Ok(last_offset == data.len() as u32)
    }
}

impl Cl2DirectionalSpriteSheet {
    /// Parse a directional CL2 sheet from bytes.
    ///
    /// If the file is a plain single-direction list, it is returned as a
    /// single entry so callers can still use fallback behavior.
    pub fn from_bytes(data: &[u8], frame_width: u16) -> Result<Self> {
        if Cl2Sprite::is_single_list(data)? {
            return Ok(Self {
                directions: vec![Cl2Sprite {
                    frames: Cl2Sprite::parse_frame_list(data, frame_width)?,
                }],
            });
        }

        let offsets = Cl2Sprite::read_sheet_offsets(data)?;
        if offsets.is_empty() {
            return Err(anyhow::anyhow!("CL2 sheet has no direction groups"));
        }

        let mut directions = Vec::with_capacity(offsets.len());
        for index in 0..offsets.len() {
            let start = offsets[index];

            if start >= data.len() {
                return Err(anyhow::anyhow!(
                    "CL2 sheet direction {} invalid offset: {} of {}",
                    index,
                    start,
                    data.len()
                ));
            }

            let frames = Cl2Sprite::parse_frame_list(&data[start..], frame_width)
                .with_context(|| format!("Failed to parse CL2 direction {}", index))?;
            directions.push(Cl2Sprite { frames });
        }

        Ok(Self { directions })
    }

    /// Return the number of directions in this sheet.
    pub fn direction_count(&self) -> usize {
        self.directions.len()
    }

    /// Get a direction by index.
    pub fn direction(&self, index: usize) -> Option<&Cl2Sprite> {
        self.directions.get(index)
    }
}

impl Cl2Sprite {
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
                        return Err(anyhow::anyhow!(
                            "CL2 opaque fill overflow: missing fill color byte"
                        ));
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
