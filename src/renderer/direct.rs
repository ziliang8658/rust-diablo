/// DirectRenderer - Direct pixel rendering engine
///
/// Completely replicates C++'s rendering approach:
/// - 8-bit indexed color framebuffer
/// - Direct pixel writes
/// - Transparent pixels (index 0) never written
/// - No clear() between frames
///
/// # Reference
/// C++: Source/engine/render/dun_render.cpp

use super::framebuffer::Framebuffer;
use crate::tiles::TileType;
use anyhow::Result;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

/// Direct pixel renderer that mimics C++ framebuffer rendering
pub struct DirectRenderer {
    framebuffer: Framebuffer,
    palette: Box<[Color; 256]>,
}

impl DirectRenderer {
    /// Create a new DirectRenderer
    ///
    /// # Arguments
    /// * `width` - Framebuffer width
    /// * `height` - Framebuffer height
    /// * `palette` - 256-color palette
    pub fn new(width: usize, height: usize, palette: [Color; 256]) -> Self {
        Self {
            framebuffer: Framebuffer::new(width, height),
            palette: Box::new(palette),
        }
    }
    
    /// Clear framebuffer (only call once during initialization)
    ///
    /// C++ never calls clear() after the first frame.
    /// This method should only be used for initialization.
    pub fn clear(&mut self) {
        self.framebuffer.clear();
    }
    
    /// Render a left triangle tile (compact format: 512 bytes)
    ///
    /// # Arguments
    /// * `data` - Compact tile data (512 bytes)
    /// * `screen_x` - Screen X position
    /// * `screen_y` - Screen Y position (top of triangle)
    ///
    /// # Reference
    /// C++: RenderLeftTriangle() in dun_render.cpp
    pub fn render_left_triangle(&mut self, data: &[u8], screen_x: i32, screen_y: i32) {
        // [DEBUG] Log first few renders with Y range
        static mut RENDER_COUNT: usize = 0;
        // Left triangle row widths: [2,4,6,8,10,12,14,16,18,20,22,24,26,28,30,32,30,28,26,24,22,20,18,16,14,12,10,8,6,4,2]
        const WIDTHS: [usize; 31] = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
            30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2
        ];
        
        let mut src_pos = 0;
        for (row, &width) in WIDTHS.iter().enumerate() {
            // C++ renders bottom-to-top (dst -= dstPitch)
            // But position.y passed to RenderTileFrame is the TOP
            // Triangle is 31 rows (0-30)
            // Row 0 is at the bottom (screen_y + 30)
            // Row 30 is at the top (screen_y)
            // So: y = screen_y + (30 - row)
            const TRIANGLE_HEIGHT: i32 = 31;
            let y = screen_y + (TRIANGLE_HEIGHT - 1 - row as i32);
            
            // Bounds check Y
            if y < 0 || y >= self.framebuffer.height() as i32 {
                src_pos += width;
                continue;
            }
            
            // Render row (left-aligned)



            let px = screen_x + x as i32;
                
                // Bounds check X
                if px >= 0 && px < self.framebuffer.width() as i32 && src_pos + x < data.len() {
                    self.framebuffer.set_pixel(
                        px as usize,
                        y as usize,
                        data[src_pos + x],
                    );
                }
            }
        }
    }
    
    /// Render a right triangle tile (compact format: 512 bytes)
    ///
    /// # Arguments
    /// * `data` - Compact tile data (512 bytes)
    /// * `screen_x` - Screen X position (left edge)
    /// * `screen_y` - Screen Y position (top of triangle)
    ///
    /// # Reference
    /// C++: RenderRightTriangle() in dun_render.cpp
    pub fn render_right_triangle(&mut self, data: &[u8], screen_x: i32, screen_y: i32) {
        // Right triangle row widths (same as left)
        const WIDTHS: [usize; 31] = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
            30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2
        ];
        
        let mut src_pos = 0;
        for (row, &width) in WIDTHS.iter().enumerate() {
            // Same Y coordinate logic as left triangle
            const TRIANGLE_HEIGHT: i32 = 31;
            let y = screen_y + (TRIANGLE_HEIGHT - 1 - row as i32);
            
            if y < 0 || y >= self.framebuffer.height() as i32 {
                src_pos += width;
                continue;
            }
            
            // Render row (right-aligned)
            let x_offset = 32 - width;
            for x in 0..width {
                let px = screen_x + (x_offset + x) as i32;
                
                if px >= 0 && px < self.framebuffer.width() as i32 && src_pos + x < data.len() {
                    self.framebuffer.set_pixel(
                        px as usize,
                        y as usize,
                        data[src_pos + x],
                    );
                }
            }
            
            src_pos += width;
        }
    }
    
    /// Render a square tile (32×32, compact format: 1024 bytes)
    ///
    /// # Arguments
    /// * `data` - Compact tile data (1024 bytes)
    /// * `screen_x` - Screen X position
    /// * `screen_y` - Screen Y position (BOTTOM of square, like C++)
    ///
    /// # Reference
    /// C++: RenderSquare() in dun_render.cpp Line 260-267
    /// C++ renders from bottom-to-top: dst -= dstPitch
    pub fn render_square(&mut self, data: &[u8], screen_x: i32, screen_y: i32) {
        const WIDTH: usize = 32;
        const HEIGHT: usize = 32;
        
        for row in 0..HEIGHT {
            // C++ renders bottom-to-top (dst -= dstPitch)
            // position.y is the TOP of the tile
            // Row 0 is at bottom (screen_y + 31)
            // Row 31 is at top (screen_y)
            let y = screen_y + (HEIGHT as i32 - 1 - row as i32);
            
            if y < 0 || y >= self.framebuffer.height() as i32 {
                continue;
            }
            
            for col in 0..WIDTH {
                let x = screen_x + col as i32;
                
                if x >= 0 && x < self.framebuffer.width() as i32 {
                    let idx = row * WIDTH + col;
                    if idx < data.len() {
                        self.framebuffer.set_pixel(
                            x as usize,
                            y as usize,
                            data[idx],
                        );
                    }
                }
            }
        }
    }
    
    /// Render a transparent square tile (32×32)
    ///
    /// # Reference
    /// C++: RenderTransparentSquare() in dun_render.cpp
    pub fn render_transparent_square(&mut self, data: &[u8], screen_x: i32, screen_y: i32) {
        // TransparentSquare is the same as Square in terms of data format
        // The difference is in how it's used (foliage, etc.)
        self.render_square(data, screen_x, screen_y);
    }
    
    /// Render a tile based on its type
    ///
    /// # Arguments
    /// * `data` - Compact tile data
    /// * `tile_type` - Type of tile
    /// * `screen_x` - Screen X position
    /// * `screen_y` - Screen Y position
    pub fn render_tile(&mut self, data: &[u8], tile_type: TileType, screen_x: i32, screen_y: i32) {
        match tile_type {
            TileType::LeftTriangle => self.render_left_triangle(data, screen_x, screen_y),
            TileType::RightTriangle => self.render_right_triangle(data, screen_x, screen_y),
            //TileType::Square => self.render_square(data, screen_x, screen_y),
            //TileType::TransparentSquare => self.render_transparent_square(data, screen_x, screen_y),
            //TileType::LeftTrapezoid | TileType::RightTrapezoid => {
                // Trapezoids are rendered as triangles in the compact format
                // We'll handle these later if needed
                //self.render_square(data, screen_x, screen_y);
            //}
            _ => {}
        }
    }
    
    /// Get RGBA data from framebuffer
    ///
    /// # Returns
    /// RGBA byte array (4 bytes per pixel)
    pub fn to_rgba(&self) -> Vec<u8> {
        self.framebuffer.to_rgba(&self.palette)
    }
    
    /// Get framebuffer dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.framebuffer.width(), self.framebuffer.height())
    }
    
    /// Get framebuffer width
    pub fn width(&self) -> usize {
        self.framebuffer.width()
    }
    
    /// Get framebuffer height
    pub fn height(&self) -> usize {
        self.framebuffer.height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_direct_renderer_create() {
        let palette = [Color::RGB(0, 0, 0); 256];
        let renderer = DirectRenderer::new(800, 600, palette);
        assert_eq!(renderer.width(), 800);
        assert_eq!(renderer.height(), 600);
    }
    
    #[test]
    fn test_render_left_triangle() {
        let palette = [Color::RGB(0, 0, 0); 256];
        let mut renderer = DirectRenderer::new(100, 100, palette);
        
        // Create a simple triangle (all pixels = 1)
        let mut data = vec![1u8; 512];
        data[0] = 0;  // First pixel transparent
        
        renderer.render_left_triangle(&data, 10, 10);
        
        // Check that pixels were written
        // (Can't easily verify without exposing framebuffer)
    }
    
    #[test]
    fn test_render_square() {
        let palette = [Color::RGB(0, 0, 0); 256];
        let mut renderer = DirectRenderer::new(100, 100, palette);
        
        let data = vec![1u8; 1024];
        renderer.render_square(&data, 10, 10);
    }
}

