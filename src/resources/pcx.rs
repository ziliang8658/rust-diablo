/// PCX Image Format Loader
/// 
/// Handles loading and parsing of PCX (Paintbrush) image files.
/// PCX is a 256-color indexed image format with RLE compression.
/// 
/// File structure:
/// - Header (128 bytes): Image metadata
/// - Pixel data (RLE compressed): 256-color indexed image
/// - Palette (768 bytes, optional): RGB palette at end of file

use anyhow::{Context, Result};
use crate::resources::palette::Palette;

/// PCX file header (128 bytes)
#[derive(Debug, Clone)]
pub struct PcxHeader {
    pub manufacturer: u8,      // 0x0A (PCX identifier)
    pub version: u8,           // Version number
    pub encoding: u8,          // Encoding (1 = RLE)
    pub bits_per_pixel: u8,   // Bits per pixel (8 = 256 colors)
    pub xmin: u16,            // Image left boundary
    pub ymin: u16,            // Image top boundary
    pub xmax: u16,            // Image right boundary
    pub ymax: u16,            // Image bottom boundary
    pub hdpi: u16,            // Horizontal resolution
    pub vdpi: u16,            // Vertical resolution
    pub colormap: [u8; 48],   // 16-color palette (usually unused)
    pub reserved: u8,         // Reserved byte
    pub n_planes: u8,         // Number of color planes (1 = single plane)
    pub bytes_per_line: u16,  // Bytes per line (must be even)
    pub palette_info: u16,   // Palette info
    pub hscreen_size: u16,    // Horizontal screen size
    pub vscreen_size: u16,   // Vertical screen size
    pub filler: [u8; 54],     // Filler bytes
}

impl PcxHeader {
    /// Parse PCX header from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 128 {
            return Err(anyhow::anyhow!("PCX header too small: {} bytes", data.len()));
        }

        Ok(PcxHeader {
            manufacturer: data[0],
            version: data[1],
            encoding: data[2],
            bits_per_pixel: data[3],
            xmin: u16::from_le_bytes([data[4], data[5]]),
            ymin: u16::from_le_bytes([data[6], data[7]]),
            xmax: u16::from_le_bytes([data[8], data[9]]),
            ymax: u16::from_le_bytes([data[10], data[11]]),
            hdpi: u16::from_le_bytes([data[12], data[13]]),
            vdpi: u16::from_le_bytes([data[14], data[15]]),
            colormap: {
                let mut cmap = [0u8; 48];
                cmap.copy_from_slice(&data[16..64]);
                cmap
            },
            reserved: data[64],
            n_planes: data[65],
            bytes_per_line: u16::from_le_bytes([data[66], data[67]]),
            palette_info: u16::from_le_bytes([data[68], data[69]]),
            hscreen_size: u16::from_le_bytes([data[70], data[71]]),
            vscreen_size: u16::from_le_bytes([data[72], data[73]]),
            filler: {
                let mut fill = [0u8; 54];
                fill.copy_from_slice(&data[74..128]);
                fill
            },
        })
    }

    /// Calculate image width
    pub fn width(&self) -> u16 {
        self.xmax.wrapping_sub(self.xmin).wrapping_add(1)
    }

    /// Calculate image height
    pub fn height(&self) -> u16 {
        self.ymax.wrapping_sub(self.ymin).wrapping_add(1)
    }
}

/// PCX Image
/// 
/// Contains decoded pixel data and optional embedded palette
#[derive(Debug, Clone)]
pub struct PcxImage {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,           // 256-color indexed image
    pub palette: Option<Palette>,  // Optional embedded palette
}

impl PcxImage {
    /// Parse PCX file from bytes
    /// 
    /// # Arguments
    /// * `data` - Complete PCX file data
    /// 
    /// # Returns
    /// `Ok(PcxImage)` if parsing succeeds, `Err` if format is invalid
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 128 {
            return Err(anyhow::anyhow!("PCX file too small: {} bytes", data.len()));
        }

        // Parse header
        let header = PcxHeader::from_bytes(&data[0..128])
            .context("Failed to parse PCX header")?;

        // Validate header
        if header.manufacturer != 0x0A {
            return Err(anyhow::anyhow!(
                "Invalid PCX manufacturer: expected 0x0A, got 0x{:02X}",
                header.manufacturer
            ));
        }

        if header.bits_per_pixel != 8 {
            return Err(anyhow::anyhow!(
                "Unsupported bits per pixel: {} (only 8-bit supported)",
                header.bits_per_pixel
            ));
        }

        let width = header.width();
        let height = header.height();

        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!(
                "Invalid image dimensions: {}x{}",
                width, height
            ));
        }

        // Calculate pixel data size (excluding header and optional palette)
        let pixel_data_start = 128;
        let pixel_data_end = if data.len() >= 769 {
            // Check if palette exists (separator byte + 768 bytes)
            let palette_start = data.len() - 769;
            if data[palette_start] == 0x0C {
                // Palette exists
                palette_start
            } else {
                data.len()
            }
        } else {
            data.len()
        };

        if pixel_data_end <= pixel_data_start {
            return Err(anyhow::anyhow!("No pixel data in PCX file"));
        }

        let pixel_data = &data[pixel_data_start..pixel_data_end];

        // Decode RLE compressed pixel data
        let pixels = Self::decode_rle(pixel_data, width, height)
            .context("Failed to decode PCX RLE data")?;

        // Try to read embedded palette
        let palette = if data.len() >= 769 && data[data.len() - 769] == 0x0C {
            let palette_data = &data[data.len() - 768..];
            Some(Palette::from_bytes(palette_data)
                .context("Failed to parse embedded PCX palette")?)
        } else {
            None
        };

        Ok(PcxImage {
            width,
            height,
            pixels,
            palette,
        })
    }

    /// Decode PCX RLE compressed data
    /// 
    /// PCX RLE rules:
    /// - If byte <= 0xBF (191): Direct pixel value
    /// - If byte >= 0xC0 (192): 
    ///   - Run length = byte & 0x3F (lower 6 bits)
    ///   - Next byte is the repeated value
    /// 
    /// # Arguments
    /// * `data` - RLE compressed data
    /// * `width` - Image width
    /// * `height` - Image height
    /// 
    /// # Returns
    /// Decoded pixel data
    fn decode_rle(data: &[u8], width: u16, height: u16) -> Result<Vec<u8>> {
        let mut pixels = Vec::with_capacity((width as usize) * (height as usize));
        let mut data_idx = 0;
        const PCX_MAX_SINGLE_PIXEL: u8 = 0xBF;
        const PCX_RUN_LENGTH_MASK: u8 = 0x3F;

        for _ in 0..height {
            let mut x = 0;
            while x < width {
                if data_idx >= data.len() {
                    return Err(anyhow::anyhow!(
                        "RLE decode overflow at line {}, pixel {}",
                        pixels.len() / width as usize,
                        x
                    ));
                }

                let byte = data[data_idx];
                data_idx += 1;

                if byte <= PCX_MAX_SINGLE_PIXEL {
                    // Direct pixel value
                    pixels.push(byte);
                    x += 1;
                } else {
                    // RLE compression
                    let run_length = (byte & PCX_RUN_LENGTH_MASK) as usize;
                    
                    if data_idx >= data.len() {
                        return Err(anyhow::anyhow!(
                            "RLE decode overflow: missing value byte for run length {}",
                            run_length
                        ));
                    }

                    let value = data[data_idx];
                    data_idx += 1;

                    // Check if we have enough space
                    if x as usize + run_length > width as usize {
                        return Err(anyhow::anyhow!(
                            "RLE run length {} exceeds remaining width {}",
                            run_length,
                            width as usize - x as usize
                        ));
                    }

                    for _ in 0..run_length {
                        pixels.push(value);
                        x += 1;
                    }
                }
            }

            // PCX requires each line to have an even number of bytes
            // Skip padding byte if width is odd
            if width % 2 != 0 && data_idx < data.len() {
                data_idx += 1;
            }
        }

        if pixels.len() != (width as usize) * (height as usize) {
            return Err(anyhow::anyhow!(
                "RLE decode size mismatch: expected {} pixels, got {}",
                (width as usize) * (height as usize),
                pixels.len()
            ));
        }

        Ok(pixels)
    }

    /// Convert to RGBA texture data
    /// 
    /// # Arguments
    /// * `palette` - Palette to use for color conversion
    /// * `transparent` - Optional transparent color index (None = no transparency)
    /// 
    /// # Returns
    /// RGBA pixel data (width * height * 4 bytes)
    pub fn to_rgba(&self, palette: &Palette, transparent: Option<u8>) -> Vec<u8> {
        let mut rgba = Vec::with_capacity((self.width as usize) * (self.height as usize) * 4);

        for &pixel_idx in &self.pixels {
            let color = palette.to_rgb(pixel_idx);
            let alpha = if let Some(trans_idx) = transparent {
                if pixel_idx == trans_idx {
                    0
                } else {
                    255
                }
            } else {
                255
            };

            rgba.push(color.r);
            rgba.push(color.g);
            rgba.push(color.b);
            rgba.push(alpha);
        }

        rgba
    }

    /// Load PCX from MPQ archive
    /// 
    /// # Arguments
    /// * `mpq` - MPQ manager to read from
    /// * `path` - Path to PCX file in MPQ
    /// 
    /// # Returns
    /// `Ok(PcxImage)` if loaded successfully
    pub fn from_mpq(mpq: &mut crate::resources::mpq::MpqManager, path: &str) -> Result<Self> {
        let data = mpq.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("PCX file not found in MPQ: {}", path))?;

        Self::from_bytes(&data)
            .with_context(|| format!("Failed to parse PCX from MPQ: {}", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcx_header_parse() {
        // Create a minimal valid PCX header
        let mut header_data = vec![0u8; 128];
        header_data[0] = 0x0A; // Manufacturer
        header_data[1] = 5;     // Version
        header_data[2] = 1;     // Encoding (RLE)
        header_data[3] = 8;     // Bits per pixel
        // xmin = 0, ymin = 0
        // xmax = 99, ymax = 99 (100x100 image)
        header_data[8] = 99;
        header_data[9] = 0;
        header_data[10] = 99;
        header_data[11] = 0;
        header_data[65] = 1;    // n_planes

        let header = PcxHeader::from_bytes(&header_data).unwrap();
        assert_eq!(header.manufacturer, 0x0A);
        assert_eq!(header.width(), 100);
        assert_eq!(header.height(), 100);
    }

    #[test]
    fn test_pcx_rle_decode_single_pixels() {
        // Test RLE decode with single pixels (no compression)
        let data = vec![0x10, 0x20, 0x30, 0x40];
        let result = PcxImage::decode_rle(&data, 4, 1).unwrap();
        assert_eq!(result, vec![0x10, 0x20, 0x30, 0x40]);
    }

    #[test]
    fn test_pcx_rle_decode_compressed() {
        // Test RLE decode with compression
        // 0xC5 = 0xC0 | 5 = repeat 5 times
        // 0x10 = value to repeat
        let data = vec![0xC5, 0x10, 0x20, 0x30];
        let result = PcxImage::decode_rle(&data, 7, 1).unwrap();
        assert_eq!(result, vec![0x10, 0x10, 0x10, 0x10, 0x10, 0x20, 0x30]);
    }

    #[test]
    fn test_pcx_rle_decode_overflow() {
        // Test RLE decode with insufficient data
        let data = vec![0xC5]; // Missing value byte
        let result = PcxImage::decode_rle(&data, 5, 1);
        assert!(result.is_err());
    }
}

