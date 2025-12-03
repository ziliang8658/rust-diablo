/// CLX Sprite Format Loader
/// 
/// Handles loading and parsing of CLX sprite files.
/// CLX is a format used for Diablo 1 sprites with CL2 RLE compression.
/// 
/// File structure:
/// - Header: Frame count, frame offsets, file size
/// - Frame data (per frame):
///   - Frame header (6 bytes): header size, width, height
///   - Pixel data (CL2 RLE encoded)

use anyhow::{Context, Result};

/// CLX file header
#[derive(Debug, Clone)]
pub struct ClxHeader {
    pub num_frames: u32,
    pub frame_offsets: Vec<u32>,
    pub file_size: u32,
}

impl ClxHeader {
    /// Parse CLX header from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!("CLX header too small: {} bytes", data.len()));
        }

        let num_frames = u32::from_le_bytes([
            data[0], data[1], data[2], data[3]
        ]);

        if num_frames == 0 {
            return Err(anyhow::anyhow!("CLX file has zero frames"));
        }

        // Read frame offsets (num_frames + 1 offsets, last one is file size)
        let num_offsets = (num_frames + 1) as usize;
        let header_size = 4 + (num_offsets * 4);

        if data.len() < header_size {
            return Err(anyhow::anyhow!(
                "CLX header incomplete: need {} bytes, got {}",
                header_size,
                data.len()
            ));
        }

        let mut frame_offsets = Vec::with_capacity(num_offsets);
        for i in 0..num_offsets {
            let offset = 4 + (i * 4);
            let offset_value = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            frame_offsets.push(offset_value);
        }

        let file_size = frame_offsets[num_frames as usize];

        Ok(ClxHeader {
            num_frames,
            frame_offsets,
            file_size,
        })
    }
}

/// CLX frame header (6 bytes)
#[derive(Debug, Clone)]
pub struct ClxFrameHeader {
    pub header_size: u16,  // Usually 6
    pub width: u16,
    pub height: u16,
}

impl ClxFrameHeader {
    /// Parse CLX frame header from bytes
    pub fn from_bytes(data: &[u8], offset: usize) -> Result<Self> {
        if offset + 6 > data.len() {
            return Err(anyhow::anyhow!(
                "CLX frame header out of bounds: offset {}, data len {}",
                offset,
                data.len()
            ));
        }

        Ok(ClxFrameHeader {
            header_size: u16::from_le_bytes([data[offset], data[offset + 1]]),
            width: u16::from_le_bytes([data[offset + 2], data[offset + 3]]),
            height: u16::from_le_bytes([data[offset + 4], data[offset + 5]]),
        })
    }
}

/// CLX Frame
/// 
/// Contains decoded pixel data for a single frame
/// None values represent transparent pixels
#[derive(Debug, Clone)]
pub struct ClxFrame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<Option<u8>>,  // None = transparent pixel
}

impl ClxFrame {
    /// Convert frame to RGBA texture data
    /// 
    /// # Arguments
    /// * `palette` - Palette to use for color conversion
    /// 
    /// # Returns
    /// RGBA pixel data (width * height * 4 bytes)
    /// 
    /// Note: CL2/CLX format stores pixels bottom-to-top, so we flip the Y-axis here
    pub fn to_rgba(&self, palette: &crate::resources::palette::Palette) -> Vec<u8> {
        let width = self.width as usize;
        let height = self.height as usize;
        let mut rgba = vec![0u8; width * height * 4];

        // CL2/CLX stores pixels from bottom to top, so we need to flip Y-axis
        for y in 0..height {
            for x in 0..width {
                let src_idx = y * width + x;
                // Flip Y: write to (height - 1 - y) instead of y
                let dst_idx = ((height - 1 - y) * width + x) * 4;
                
                if src_idx < self.pixels.len() {
                    if let Some(pixel_idx) = self.pixels[src_idx] {
                        let color = palette.to_rgb(pixel_idx);
                        rgba[dst_idx] = color.r;
                        rgba[dst_idx + 1] = color.g;
                        rgba[dst_idx + 2] = color.b;
                        rgba[dst_idx + 3] = 255; // Opaque
                    } else {
                        // Transparent pixel (already initialized to 0)
                    }
                }
            }
        }

        rgba
    }
}

/// CLX Sprite
/// 
/// Contains multiple frames (for animation)
#[derive(Debug, Clone)]
pub struct ClxSprite {
    pub frames: Vec<ClxFrame>,
}

impl ClxSprite {
    /// Parse CLX file from bytes
    /// 
    /// # Arguments
    /// * `data` - Complete CLX file data
    /// 
    /// # Returns
    /// `Ok(ClxSprite)` if parsing succeeds, `Err` if format is invalid
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // Parse header
        let header = ClxHeader::from_bytes(data)
            .context("Failed to parse CLX header")?;

        if data.len() < header.file_size as usize {
            return Err(anyhow::anyhow!(
                "CLX file size mismatch: header says {}, actual {}",
                header.file_size,
                data.len()
            ));
        }

        let mut frames = Vec::with_capacity(header.num_frames as usize);

        // Parse each frame
        for frame_idx in 0..header.num_frames {
            let frame_offset = header.frame_offsets[frame_idx as usize] as usize;
            let next_offset = header.frame_offsets[(frame_idx + 1) as usize] as usize;

            if frame_offset >= data.len() {
                return Err(anyhow::anyhow!(
                    "CLX frame {} offset {} out of bounds",
                    frame_idx,
                    frame_offset
                ));
            }

            if next_offset > data.len() {
                return Err(anyhow::anyhow!(
                    "CLX frame {} next offset {} out of bounds",
                    frame_idx,
                    next_offset
                ));
            }

            // Parse frame header
            let frame_header = ClxFrameHeader::from_bytes(data, frame_offset)
                .with_context(|| format!("Failed to parse CLX frame {} header", frame_idx))?;

            // Calculate pixel data offset (after frame header)
            let pixel_data_offset = frame_offset + frame_header.header_size as usize;

            if pixel_data_offset >= next_offset {
                return Err(anyhow::anyhow!(
                    "CLX frame {} pixel data offset {} >= next offset {}",
                    frame_idx,
                    pixel_data_offset,
                    next_offset
                ));
            }

            let pixel_data = &data[pixel_data_offset..next_offset];

            // Decode CL2 RLE pixel data
            let pixels = Self::decode_cl2_rle(pixel_data, frame_header.width, frame_header.height)
                .with_context(|| format!("Failed to decode CLX frame {} RLE data", frame_idx))?;

            frames.push(ClxFrame {
                width: frame_header.width,
                height: frame_header.height,
                pixels,
            });
        }

        Ok(ClxSprite { frames })
    }

    /// Decode CL2 RLE compressed data
    /// 
    /// CL2 RLE rules:
    /// 1. Transparent run (0x00-0x7F):
    ///    - Value = number of transparent pixels
    ///    - Skip these pixels (don't write)
    /// 
    /// 2. Opaque fill (0x80-0xBF):
    ///    - Width = 0xBF - control byte
    ///    - Next byte is fill color value
    ///    - Repeat color value 'width' times
    /// 
    /// 3. Opaque pixels (0xC0-0xFF):
    ///    - Width = control byte - 0xBF
    ///    - Next 'width' bytes are raw pixel values
    /// 
    /// # Arguments
    /// * `data` - RLE compressed data
    /// * `width` - Frame width
    /// * `height` - Frame height
    /// 
    /// # Returns
    /// Decoded pixel data (None = transparent)
    fn decode_cl2_rle(data: &[u8], width: u16, height: u16) -> Result<Vec<Option<u8>>> {
        let mut pixels = vec![None; (width as usize) * (height as usize)];
        let mut pixel_idx = 0;
        let mut data_idx = 0;

        const CLX_OPAQUE_MIN: u8 = 0x80;
        const CLX_FILL_MAX: u8 = 0xBE;  // Fixed: was 0xBF
        const CLX_FILL_END: u8 = 0xBF;

        for _ in 0..height {
            let mut x = 0;
            while x < width {
                if data_idx >= data.len() {
                    return Err(anyhow::anyhow!(
                        "CL2 RLE decode overflow at line {}, pixel {}",
                        pixel_idx / width as usize,
                        x
                    ));
                }

                let control = data[data_idx];
                data_idx += 1;

                if control < CLX_OPAQUE_MIN {
                    // Transparent run (0x00-0x7F)
                    let transparent_count = control as usize;
                    
                    // Check bounds
                    if pixel_idx + transparent_count > pixels.len() {
                        return Err(anyhow::anyhow!(
                            "CL2 transparent run overflow: count {}, remaining {}",
                            transparent_count,
                            pixels.len() - pixel_idx
                        ));
                    }

                    // Skip transparent pixels (already None)
                    pixel_idx += transparent_count;
                    x += transparent_count as u16;
                } else if control <= CLX_FILL_MAX {
                    // Opaque fill (0x80-0xBE)
                    let fill_width = (CLX_FILL_END - control) as usize;

                    if data_idx >= data.len() {
                        return Err(anyhow::anyhow!(
                            "CL2 opaque fill overflow: missing fill color byte"
                        ));
                    }

                    let fill_color = data[data_idx];
                    data_idx += 1;

                    // Check bounds
                    if pixel_idx + fill_width > pixels.len() {
                        return Err(anyhow::anyhow!(
                            "CL2 opaque fill overflow: width {}, remaining {}",
                            fill_width,
                            pixels.len() - pixel_idx
                        ));
                    }

                    if x as usize + fill_width > width as usize {
                        return Err(anyhow::anyhow!(
                            "CL2 opaque fill exceeds line width: width {}, remaining {}",
                            fill_width,
                            width as usize - x as usize
                        ));
                    }

                    for _ in 0..fill_width {
                        pixels[pixel_idx] = Some(fill_color);
                        pixel_idx += 1;
                        x += 1;
                    }
                } else {
                    // Opaque pixels (0xBF-0xFF)
                    // Width = -static_cast<int8_t>(control)
                    let pixel_width = (-(control as i8)) as usize;

                    // Check bounds
                    if data_idx + pixel_width > data.len() {
                        return Err(anyhow::anyhow!(
                            "CL2 opaque pixels overflow: width {}, remaining data {}",
                            pixel_width,
                            data.len() - data_idx
                        ));
                    }

                    if pixel_idx + pixel_width > pixels.len() {
                        return Err(anyhow::anyhow!(
                            "CL2 opaque pixels overflow: width {}, remaining pixels {}",
                            pixel_width,
                            pixels.len() - pixel_idx
                        ));
                    }

                    if x as usize + pixel_width > width as usize {
                        return Err(anyhow::anyhow!(
                            "CL2 opaque pixels exceeds line width: width {}, remaining {}",
                            pixel_width,
                            width as usize - x as usize
                        ));
                    }

                    for _ in 0..pixel_width {
                        pixels[pixel_idx] = Some(data[data_idx]);
                        pixel_idx += 1;
                        x += 1;
                        data_idx += 1;
                    }
                }
            }
        }

        if pixel_idx != pixels.len() {
            return Err(anyhow::anyhow!(
                "CL2 RLE decode size mismatch: expected {} pixels, got {}",
                pixels.len(),
                pixel_idx
            ));
        }

        Ok(pixels)
    }

    /// Get number of frames
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    /// Get frame by index
    pub fn get_frame(&self, index: usize) -> Option<&ClxFrame> {
        self.frames.get(index)
    }

    /// Convert frame to RGBA texture data
    /// 
    /// # Arguments
    /// * `frame_index` - Frame index (0-based)
    /// * `palette` - Palette to use for color conversion
    /// 
    /// # Returns
    /// RGBA pixel data
    pub fn frame_to_rgba(
        &self,
        frame_index: usize,
        palette: &crate::resources::palette::Palette,
    ) -> Result<Vec<u8>> {
        let frame = self.get_frame(frame_index)
            .ok_or_else(|| anyhow::anyhow!("Frame index {} out of bounds", frame_index))?;

        Ok(frame.to_rgba(palette))
    }

    /// Load CLX from MPQ archive
    /// 
    /// # Arguments
    /// * `mpq` - MPQ manager to read from
    /// * `path` - Path to CLX file in MPQ
    /// 
    /// # Returns
    /// `Ok(ClxSprite)` if loaded successfully
    pub fn from_mpq(mpq: &mut crate::resources::mpq::MpqManager, path: &str) -> Result<Self> {
        let data = mpq.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("CLX file not found in MPQ: {}", path))?;

        Self::from_bytes(&data)
            .with_context(|| format!("Failed to parse CLX from MPQ: {}", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clx_header_parse() {
        // Create a minimal valid CLX header (1 frame)
        let mut header_data = vec![0u8; 12];
        header_data[0] = 1; // num_frames = 1
        header_data[4] = 12; // frame 0 offset = 12 (header size)
        header_data[8] = 20; // frame 1 offset = 20 (file size)

        let header = ClxHeader::from_bytes(&header_data).unwrap();
        assert_eq!(header.num_frames, 1);
        assert_eq!(header.frame_offsets.len(), 2);
        assert_eq!(header.frame_offsets[0], 12);
        assert_eq!(header.frame_offsets[1], 20);
        assert_eq!(header.file_size, 20);
    }

    #[test]
    fn test_clx_frame_header_parse() {
        let mut frame_data = vec![0u8; 10];
        frame_data[0] = 6;  // header_size = 6
        frame_data[1] = 0;
        frame_data[2] = 10; // width = 10
        frame_data[3] = 0;
        frame_data[4] = 20; // height = 20
        frame_data[5] = 0;

        let header = ClxFrameHeader::from_bytes(&frame_data, 0).unwrap();
        assert_eq!(header.header_size, 6);
        assert_eq!(header.width, 10);
        assert_eq!(header.height, 20);
    }

    #[test]
    fn test_cl2_rle_decode_transparent() {
        // Test transparent run: 0x05 = 5 transparent pixels
        let data = vec![0x05];
        let result = ClxSprite::decode_cl2_rle(&data, 5, 1).unwrap();
        assert_eq!(result.len(), 5);
        assert!(result.iter().all(|p| p.is_none()));
    }

    #[test]
    fn test_cl2_rle_decode_opaque_fill() {
        // Test opaque fill: 0xBE = 0xBF - 1, width = 1, next byte is color
        let data = vec![0xBE, 0x42];
        let result = ClxSprite::decode_cl2_rle(&data, 1, 1).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Some(0x42));
    }

    #[test]
    fn test_cl2_rle_decode_opaque_pixels() {
        // Test opaque pixels: 0xC3 = 0xBF + 4, width = 4, next 4 bytes are pixels
        let data = vec![0xC3, 0x10, 0x20, 0x30, 0x40];
        let result = ClxSprite::decode_cl2_rle(&data, 4, 1).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Some(0x10));
        assert_eq!(result[1], Some(0x20));
        assert_eq!(result[2], Some(0x30));
        assert_eq!(result[3], Some(0x40));
    }

    #[test]
    fn test_cl2_rle_decode_mixed() {
        // Test mixed: transparent + opaque fill + opaque pixels
        // 0xC2 = 0xBF + 3, so width = 3, need 3 pixel bytes
        let data = vec![
            0x02,           // 2 transparent pixels
            0xBE, 0x10,     // 1 opaque fill (color 0x10)
            0xC2, 0x20, 0x30, 0x40, // 3 opaque pixels (0xC2 = 0xBF + 3)
        ];
        let result = ClxSprite::decode_cl2_rle(&data, 6, 1).unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0], None);  // transparent
        assert_eq!(result[1], None);  // transparent
        assert_eq!(result[2], Some(0x10)); // opaque fill
        assert_eq!(result[3], Some(0x20)); // opaque pixel
        assert_eq!(result[4], Some(0x30)); // opaque pixel
        assert_eq!(result[5], Some(0x40)); // opaque pixel
    }

    #[test]
    fn test_cl2_rle_decode_simple_mixed() {
        // Test simpler mixed case: transparent + opaque fill
        let data = vec![
            0x01,           // 1 transparent pixel
            0xBE, 0x42,     // 1 opaque fill (color 0x42)
        ];
        let result = ClxSprite::decode_cl2_rle(&data, 2, 1).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], None);  // transparent
        assert_eq!(result[1], Some(0x42)); // opaque fill
    }
}

