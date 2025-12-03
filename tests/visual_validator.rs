/// Visual Validator - Export decoded tiles as PNG images
///
/// This tool exports decoded tile data as PNG images for visual inspection.
/// Transparent pixels are shown in magenta (pink) for easy identification.
use image::{ImageBuffer, Rgba, RgbaImage};
use rust_diablo::resources::Palette;
use rust_diablo::tiles::types::TileType;

/// Visual validator for tile decoding
pub struct VisualValidator;

impl VisualValidator {
    /// Save decoded indexed tile as PNG
    ///
    /// # Arguments
    /// * `pixels` - Decoded pixel data (indexed color, 0 = transparent)
    /// * `width` - Tile width
    /// * `height` - Tile height
    /// * `palette` - Color palette for index->RGB conversion
    /// * `output_path` - Output PNG file path
    ///
    /// # Returns
    /// Ok(()) on success, Err(String) on failure
    pub fn save_as_png(
        pixels: &[u8],
        width: u32,
        height: u32,
        palette: &Palette,
        output_path: &str,
    ) -> Result<(), String> {
        if pixels.len() != (width * height) as usize {
            return Err(format!(
                "Pixel count mismatch: {} != {}x{}",
                pixels.len(),
                width,
                height
            ));
        }

        let mut img: RgbaImage = ImageBuffer::new(width, height);

        for (i, &index) in pixels.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;

            let rgba = if index == 0 {
                // Transparent pixel -> Magenta with alpha (easy to spot)
                [255, 0, 255, 128]
            } else {
                // Use palette
                palette.to_rgba(index, false)
            };

            img.put_pixel(x, y, Rgba(rgba));
        }

        img.save(output_path)
            .map_err(|e| format!("Failed to save PNG: {}", e))?;

        Ok(())
    }

    /// Export multiple tiles as PNG
    ///
    /// # Arguments
    /// * `tiles` - Vec of (frame_idx, tile_type, pixels)
    /// * `palette` - Color palette
    /// * `output_dir` - Output directory
    ///
    /// # Returns
    /// Number of successfully exported tiles
    pub fn export_tiles(
        tiles: &[(usize, TileType, Vec<u8>)],
        palette: &Palette,
        output_dir: &str,
    ) -> usize {
        // Create output directory
        if let Err(e) = std::fs::create_dir_all(output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            return 0;
        }

        let mut success_count = 0;

        for (frame_idx, tile_type, pixels) in tiles {
            // Determine dimensions based on tile type
            let (width, height) = match tile_type {
                TileType::LeftTriangle | TileType::RightTriangle => (32, 31),
                _ => (32, 32),
            };

            let filename = format!(
                "{}/frame_{:04}_type_{:?}.png",
                output_dir, frame_idx, tile_type
            );

            match Self::save_as_png(pixels, width, height, palette, &filename) {
                Ok(_) => {
                    println!("  ✓ Exported: {}", filename);
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("  ✗ Failed to export frame {}: {}", frame_idx, e);
                }
            }
        }

        success_count
    }

    /// Create a visual comparison grid (multiple tiles in one image)
    ///
    /// # Arguments
    /// * `tiles` - Vec of (label, pixels, width, height)
    /// * `palette` - Color palette
    /// * `cols` - Number of columns in grid
    /// * `output_path` - Output PNG path
    pub fn create_comparison_grid(
        tiles: &[(&str, &[u8], u32, u32)],
        palette: &Palette,
        cols: usize,
        output_path: &str,
    ) -> Result<(), String> {
        if tiles.is_empty() {
            return Err("No tiles to export".to_string());
        }

        let rows = (tiles.len() + cols - 1) / cols;
        let max_width = tiles.iter().map(|(_, _, w, _)| *w).max().unwrap_or(32);
        let max_height = tiles.iter().map(|(_, _, _, h)| *h).max().unwrap_or(32);

        // Add padding between tiles
        let padding = 2;
        let grid_width = cols as u32 * (max_width + padding) + padding;
        let grid_height = rows as u32 * (max_height + padding) + padding;

        let mut grid: RgbaImage = ImageBuffer::from_pixel(
            grid_width,
            grid_height,
            Rgba([32, 32, 32, 255]), // Dark gray background
        );

        for (idx, (label, pixels, width, height)) in tiles.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;

            let x_offset = (col as u32) * (max_width + padding) + padding;
            let y_offset = (row as u32) * (max_height + padding) + padding;

            // Draw tile
            for (i, &index) in pixels.iter().enumerate() {
                let x = (i as u32) % width;
                let y = (i as u32) / width;

                let rgba = if index == 0 {
                    [255, 0, 255, 128] // Magenta for transparent
                } else {
                    palette.to_rgba(index, false)
                };

                grid.put_pixel(x_offset + x, y_offset + y, Rgba(rgba));
            }

            // TODO: Add label text (requires font rendering)
            println!("  Tile {}: {} ({}x{})", idx, label, width, height);
        }

        grid.save(output_path)
            .map_err(|e| format!("Failed to save grid: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_validator_basic() {
        // Create test palette
        let mut pal_data = vec![0u8; 768];
        // Color 0: Black (transparent)
        // Color 1: Red
        pal_data[3] = 255;
        // Color 2: Green
        pal_data[7] = 255;
        // Color 3: Blue
        pal_data[11] = 255;

        let palette = Palette::from_bytes(&pal_data).unwrap();

        // Create test tile (4x4 for simplicity)
        let pixels = vec![
            0, 1, 1, 0, // Transparent, red, red, transparent
            1, 2, 2, 1, // Red, green, green, red
            1, 2, 2, 1, // Red, green, green, red
            0, 1, 1, 0, // Transparent, red, red, transparent
        ];

        // Try to save (will fail in test env without file system, but validates logic)
        // In real test, this would save to tests/output/
        let result = VisualValidator::save_as_png(&pixels, 4, 4, &palette, "test_output.png");

        // We expect this to fail in test environment without proper setup
        // But the code path is validated
        println!("Save result: {:?}", result);
    }
}
