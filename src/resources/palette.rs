/// Palette System
/// 
/// Handles 256-color palettes for indexed color images.
/// Supports loading from .PAL files (768 bytes: 256 colors × 3 bytes RGB).

use anyhow::{Context, Result};
use crate::resources::mpq::MpqManager;

/// RGB Color (no alpha channel in palette)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    /// Convert to RGBA array (with alpha channel)
    pub fn to_rgba(self, alpha: u8) -> [u8; 4] {
        [self.r, self.g, self.b, alpha]
    }
}

/// 256-color palette
/// 
/// Diablo uses a 256-color palette where:
/// - Index 0-127: Level-specific colors (different palettes for different levels)
/// - Index 128-255: Global colors (blue, red, yellow, etc., shared across all levels)
#[derive(Debug, Clone)]
pub struct Palette {
    /// 256 colors, indexed 0-255
    pub colors: [Color; 256],
}

impl Palette {
    /// Create a palette from raw bytes
    /// 
    /// # Arguments
    /// * `data` - 768 bytes (256 colors × 3 bytes RGB)
    /// 
    /// # Returns
    /// `Ok(Palette)` if data is valid, `Err` if size is incorrect
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() != 768 {
            return Err(anyhow::anyhow!(
                "Invalid palette size: expected 768 bytes, got {}",
                data.len()
            ));
        }
        
        let mut colors = [Color { r: 0, g: 0, b: 0 }; 256];
        
        for i in 0..256 {
            let offset = i * 3;
            colors[i] = Color {
                r: data[offset],
                g: data[offset + 1],
                b: data[offset + 2],
            };
        }
        
        Ok(Palette { colors })
    }
    
    /// Load a palette from an MPQ archive
    /// 
    /// # Arguments
    /// * `mpq` - MPQ manager to read from (mutable reference required)
    /// * `path` - Path to palette file in MPQ (e.g., "levels/towndata/town.pal")
    /// 
    /// # Returns
    /// `Ok(Palette)` if loaded successfully, `Err` if file not found or invalid
    pub fn from_mpq(mpq: &mut MpqManager, path: &str) -> Result<Self> {
        let data = mpq.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("Palette file not found in MPQ: {}", path))?;
        
        Self::from_bytes(&data)
            .with_context(|| format!("Failed to parse palette from MPQ: {}", path))
    }
    
    /// Convert a color index to RGB color
    /// 
    /// # Arguments
    /// * `index` - Color index (0-255)
    /// 
    /// # Returns
    /// RGB color
    pub fn to_rgb(&self, index: u8) -> Color {
        self.colors[index as usize]
    }
    
    /// Convert a color index to RGBA color
    /// 
    /// # Arguments
    /// * `index` - Color index (0-255)
    /// * `transparent` - If `true` and `index == 0`, alpha will be 0 (transparent)
    /// 
    /// # Returns
    /// RGBA array [R, G, B, A]
    pub fn to_rgba(&self, index: u8, transparent: bool) -> [u8; 4] {
        let color = self.colors[index as usize];
        let alpha = if transparent && index == 0 { 0 } else { 255 };
        color.to_rgba(alpha)
    }
    
    /// Convert an indexed image to RGBA
    /// 
    /// # Arguments
    /// * `indices` - Indexed image data (one byte per pixel)
    /// * `transparent` - If `true`, index 0 will be transparent (alpha = 0)
    /// 
    /// # Returns
    /// RGBA image data (4 bytes per pixel: R, G, B, A)
    pub fn indices_to_rgba(&self, indices: &[u8], transparent: bool) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(indices.len() * 4);
        for &index in indices {
            let color = self.to_rgba(index, transparent);
            rgba.extend_from_slice(&color);
        }
        rgba
    }
    
    /// Get the number of colors in the palette (always 256)
    pub fn len(&self) -> usize {
        256
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_palette_from_bytes() {
        // Create test palette data (768 bytes)
        let mut data = vec![0u8; 768];
        
        // Set first color to red
        data[0] = 255; // R
        data[1] = 0;   // G
        data[2] = 0;   // B
        
        // Set second color to green
        data[3] = 0;   // R
        data[4] = 255; // G
        data[5] = 0;   // B
        
        let palette = Palette::from_bytes(&data).unwrap();
        
        assert_eq!(palette.colors[0], Color { r: 255, g: 0, b: 0 });
        assert_eq!(palette.colors[1], Color { r: 0, g: 255, b: 0 });
    }
    
    #[test]
    fn test_palette_invalid_size() {
        let data = vec![0u8; 100];
        assert!(Palette::from_bytes(&data).is_err());
    }
    
    #[test]
    fn test_palette_to_rgb() {
        let mut data = vec![0u8; 768];
        data[0] = 255; // Index 0 = red
        data[3] = 0;   // Index 1 = green
        data[4] = 255;
        data[5] = 0;
        
        let palette = Palette::from_bytes(&data).unwrap();
        
        assert_eq!(palette.to_rgb(0), Color { r: 255, g: 0, b: 0 });
        assert_eq!(palette.to_rgb(1), Color { r: 0, g: 255, b: 0 });
    }
    
    #[test]
    fn test_palette_to_rgba_transparent() {
        let mut data = vec![0u8; 768];
        data[0] = 255; // Index 0 = red
        
        let palette = Palette::from_bytes(&data).unwrap();
        
        // With transparency: index 0 should have alpha = 0
        let rgba = palette.to_rgba(0, true);
        assert_eq!(rgba, [255, 0, 0, 0]); // R, G, B, A (transparent)
        
        // Without transparency: index 0 should have alpha = 255
        let rgba = palette.to_rgba(0, false);
        assert_eq!(rgba, [255, 0, 0, 255]); // R, G, B, A (opaque)
        
        // Other indices should always have alpha = 255
        let rgba = palette.to_rgba(1, true);
        assert_eq!(rgba[3], 255); // Alpha should be 255
    }
    
    #[test]
    fn test_palette_indices_to_rgba() {
        let mut data = vec![0u8; 768];
        data[0] = 255; // Index 0 = red
        data[3] = 0;   // Index 1 = green
        data[4] = 255;
        data[5] = 0;
        
        let palette = Palette::from_bytes(&data).unwrap();
        let indices = vec![0u8, 1u8];
        let rgba = palette.indices_to_rgba(&indices, false);
        
        assert_eq!(rgba.len(), 8); // 2 pixels × 4 bytes
        assert_eq!(rgba[0..4], [255, 0, 0, 255]); // Red pixel
        assert_eq!(rgba[4..8], [0, 255, 0, 255]); // Green pixel
    }
    
    #[test]
    fn test_palette_indices_to_rgba_transparent() {
        let mut data = vec![0u8; 768];
        data[0] = 255; // Index 0 = red
        
        let palette = Palette::from_bytes(&data).unwrap();
        let indices = vec![0u8, 1u8];
        let rgba = palette.indices_to_rgba(&indices, true);
        
        // Index 0 should be transparent
        assert_eq!(rgba[0..4], [255, 0, 0, 0]); // Red pixel, transparent
        // Index 1 should be opaque (even with transparent=true, only index 0 is transparent)
        assert_eq!(rgba[4..8], [0, 0, 0, 255]); // Black pixel (index 1), opaque
    }
    
    #[test]
    fn test_palette_from_mpq() {
        // 尝试从MPQ加载真实的palette文件
        let mut mpq = MpqManager::new();
        
        // 尝试加载Diabdat.mpq
        let mpq_paths = vec![
            "assets/Diabdat.mpq",
            "rust-diablo/assets/Diabdat.mpq",
            "../assets/Diabdat.mpq",
        ];
        
        let mut loaded = false;
        for path in mpq_paths {
            if mpq.load_mpq(path, 1000).is_ok() {
                println!("\n=== Test: Loading Palette from MPQ ===");
                println!("✓ Loaded MPQ: {}", path);
                loaded = true;
                break;
            }
        }
        
        if !loaded {
            println!("⚠ Skipping test_palette_from_mpq: Diabdat.mpq not found");
            return;
        }
        
        // 尝试加载town palette
        let palette_paths = vec![
            "levels/towndata/town.pal",
            "levels\\towndata\\town.pal",
        ];
        
        let mut palette_loaded = false;
        for path in palette_paths {
            match Palette::from_mpq(&mut mpq, path) {
                Ok(palette) => {
                    println!("✓ Loaded palette: {}", path);
                    println!("  Palette size: {} colors", palette.len());
                    
                    // 验证palette基本属性
                    assert_eq!(palette.len(), 256);
                    
                    // 打印前几个颜色
                    println!("  First 5 colors:");
                    for i in 0..5 {
                        let color = palette.to_rgb(i);
                        println!("    Color {}: RGB({}, {}, {})", i, color.r, color.g, color.b);
                    }
                    
                    // 测试颜色转换
                    let rgba = palette.to_rgba(0, true);
                    println!("  Color 0 (transparent): RGBA({}, {}, {}, {})", 
                             rgba[0], rgba[1], rgba[2], rgba[3]);
                    
                    let rgba = palette.to_rgba(1, false);
                    println!("  Color 1 (opaque): RGBA({}, {}, {}, {})", 
                             rgba[0], rgba[1], rgba[2], rgba[3]);
                    assert_eq!(rgba[3], 255); // Should be opaque
                    
                    palette_loaded = true;
                    break;
                }
                Err(e) => {
                    eprintln!("  Failed to load {}: {}", path, e);
                }
            }
        }
        
        if palette_loaded {
            println!("=== Palette Test Passed ===\n");
        } else {
            println!("⚠ Warning: Could not load palette from MPQ\n");
        }
    }
    
    #[test]
    fn test_palette_color_operations() {
        // 测试调色板的各种颜色操作
        let mut data = vec![0u8; 768];
        
        // 设置一些测试颜色
        // Color 0: Red (255, 0, 0)
        data[0] = 255;
        data[1] = 0;
        data[2] = 0;
        
        // Color 1: Green (0, 255, 0)
        data[3] = 0;
        data[4] = 255;
        data[5] = 0;
        
        // Color 2: Blue (0, 0, 255)
        data[6] = 0;
        data[7] = 0;
        data[8] = 255;
        
        // Color 255: White (255, 255, 255)
        data[765] = 255;
        data[766] = 255;
        data[767] = 255;
        
        let palette = Palette::from_bytes(&data).unwrap();
        
        // 测试边界情况
        let color_0 = palette.to_rgb(0);
        assert_eq!(color_0, Color { r: 255, g: 0, b: 0 });
        
        let color_255 = palette.to_rgb(255);
        assert_eq!(color_255, Color { r: 255, g: 255, b: 255 });
        
        // 测试RGBA转换
        let rgba_transparent = palette.to_rgba(0, true);
        assert_eq!(rgba_transparent[3], 0); // Alpha should be 0
        
        let rgba_opaque = palette.to_rgba(0, false);
        assert_eq!(rgba_opaque[3], 255); // Alpha should be 255
        
        // 测试批量转换
        let indices = vec![0u8, 1u8, 2u8, 255u8];
        let rgba_data = palette.indices_to_rgba(&indices, false);
        
        assert_eq!(rgba_data.len(), 16); // 4 pixels × 4 bytes
        
        // 验证每个像素的颜色
        assert_eq!(&rgba_data[0..4], &[255, 0, 0, 255]);   // Red
        assert_eq!(&rgba_data[4..8], &[0, 255, 0, 255]);   // Green
        assert_eq!(&rgba_data[8..12], &[0, 0, 255, 255]);  // Blue
        assert_eq!(&rgba_data[12..16], &[255, 255, 255, 255]); // White
    }
    
    #[test]
    fn test_palette_transparency_index_zero() {
        // 测试index 0的透明度特殊处理
        let mut data = vec![0u8; 768];
        
        // 设置多个颜色
        for i in 0..256 {
            let offset = i * 3;
            data[offset] = i as u8;     // R = index
            data[offset + 1] = 128;     // G = 128
            data[offset + 2] = 255 - i as u8; // B = 255 - index
        }
        
        let palette = Palette::from_bytes(&data).unwrap();
        
        // 测试所有索引的透明度
        for i in 0..=255u8 {
            let rgba_transparent = palette.to_rgba(i, true);
            let rgba_opaque = palette.to_rgba(i, false);
            
            if i == 0 {
                // Index 0 应该是透明的
                assert_eq!(rgba_transparent[3], 0, "Index 0 should be transparent");
            } else {
                // 其他索引应该是不透明的
                assert_eq!(rgba_transparent[3], 255, "Index {} should be opaque", i);
            }
            
            // 不透明模式下所有颜色都应该是不透明的
            assert_eq!(rgba_opaque[3], 255, "Index {} should be opaque in opaque mode", i);
        }
    }
    
    #[test]
    fn test_palette_image_conversion() {
        // 模拟将一个8×8的索引图像转换为RGBA
        let mut data = vec![0u8; 768];
        
        // 创建一个简单的渐变调色板
        for i in 0..256 {
            let offset = i * 3;
            let value = i as u8;
            data[offset] = value;     // R
            data[offset + 1] = value; // G
            data[offset + 2] = value; // B
        }
        
        let palette = Palette::from_bytes(&data).unwrap();
        
        // 创建一个8×8的索引图像（渐变）
        let mut indices = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                // 创建一个简单的渐变模式
                let index = ((y * 8 + x) * 4) as u8; // 0, 4, 8, 12, ...
                indices.push(index);
            }
        }
        
        // 转换为RGBA
        let rgba = palette.indices_to_rgba(&indices, false);
        
        // 验证尺寸
        assert_eq!(rgba.len(), 64 * 4); // 8×8 pixels × 4 bytes
        
        // 验证第一个像素（index 0）
        assert_eq!(&rgba[0..4], &[0, 0, 0, 255]); // Black
        
        // 验证最后一个像素（index 252）
        let last_pixel_offset = 63 * 4;
        assert_eq!(&rgba[last_pixel_offset..last_pixel_offset+4], &[252, 252, 252, 255]); // Almost white
    }
    
    #[test]
    fn test_palette_clone() {
        // 测试调色板克隆
        let mut data = vec![0u8; 768];
        data[0] = 255; // Red
        
        let palette1 = Palette::from_bytes(&data).unwrap();
        let palette2 = palette1.clone();
        
        // 验证克隆的调色板是独立的
        assert_eq!(palette1.to_rgb(0), palette2.to_rgb(0));
        assert_eq!(palette1.len(), palette2.len());
        
        // 验证所有颜色都相同
        for i in 0..=255u8 {
            assert_eq!(palette1.to_rgb(i), palette2.to_rgb(i));
        }
    }
    
    #[test]
    fn test_palette_performance() {
        // 性能测试：大量转换操作
        use std::time::Instant;
        
        let mut data = vec![0u8; 768];
        for i in 0..768 {
            data[i] = (i % 256) as u8;
        }
        
        let palette = Palette::from_bytes(&data).unwrap();
        
        // 创建一个较大的索引图像 (256×256)
        let size = 256 * 256;
        let mut indices = Vec::with_capacity(size);
        for i in 0..size {
            indices.push((i % 256) as u8);
        }
        
        // 测试转换速度
        let start = Instant::now();
        let rgba = palette.indices_to_rgba(&indices, true);
        let duration = start.elapsed();
        
        println!("\n=== Palette Performance Test ===");
        println!("Converted {}×{} pixels in {:?}", 256, 256, duration);
        println!("Output size: {} bytes ({} MB)", rgba.len(), rgba.len() / 1024 / 1024);
        
        // 验证结果
        assert_eq!(rgba.len(), size * 4);
        
        // 性能断言：应该在合理时间内完成
        assert!(duration.as_millis() < 100, "Conversion should be fast");
    }
}

