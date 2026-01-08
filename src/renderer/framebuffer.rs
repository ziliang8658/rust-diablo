/// Framebuffer - 8-bit indexed color framebuffer
///
/// Mimics C++'s direct framebuffer access for pixel-perfect rendering.
/// Transparent pixels (index 0) are never written, preserving the underlying content.
///
/// # Reference
/// C++: Source/engine/surface.hpp - 8-bit Surface

use sdl2::pixels::Color;

/// 8-bit indexed color framebuffer
///
/// This struct replicates C++'s approach of using an 8-bit palette-indexed
/// framebuffer, where transparent pixels (palette index 0) are never written.
pub struct Framebuffer {
    /// Pixel data (8-bit palette indices)
    pixels: Vec<u8>,
    
    /// Width in pixels
    width: usize,
    
    /// Height in pixels
    height: usize,
}

impl Framebuffer {
    /// Create a new framebuffer with all pixels initialized to 0 (transparent)
    ///
    /// # Arguments
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![0; width * height],
            width,
            height,
        }
    }
    
    /// Set a pixel to a palette index
    ///
    /// # C++ Behavior
    /// In C++, transparent pixels (index 0) are never written.
    /// This preserves the underlying framebuffer content, enabling
    /// the layered rendering approach without clearing.
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `palette_index` - Palette index (0 = transparent, don't write)
    ///
    /// # Reference
    /// C++: Direct framebuffer write: `*dst = palette_index`
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, palette_index: u8) {
        // C++ transparent pixel behavior: if (palette_index != 0) { *dst = palette_index; }
        if palette_index != 0 {
            let offset = y * self.width + x;
            if offset < self.pixels.len() {
                self.pixels[offset] = palette_index;
            }
        }
        // palette_index == 0: transparent, don't write (C++ behavior)
    }
    
    /// Get a pixel's palette index
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let offset = y * self.width + x;
        self.pixels.get(offset).copied().unwrap_or(0)
    }
    
    /// Convert framebuffer to RGBA data for SDL2 texture
    ///
    /// # Arguments
    /// * `palette` - 256-color palette
    ///
    /// # Returns
    /// RGBA byte array (4 bytes per pixel)
    pub fn to_rgba(&self, palette: &[Color; 256]) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.pixels.len() * 4);
        
        for &idx in &self.pixels {
            let color = palette[idx as usize];
            rgba.push(color.r);
            rgba.push(color.g);
            rgba.push(color.b);
            rgba.push(255);  // Always opaque after conversion
        }
        
        rgba
    }
    
    /// Get width
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }
    
    /// Get height
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }
    
    /// Clear framebuffer to transparent (palette index 0)
    ///
    /// Note: In C++, the framebuffer is NEVER cleared after initialization.
    /// This method should only be called once during initialization.
    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }
    
    /// Get raw pixel data (for debugging)
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_framebuffer_create() {
        let fb = Framebuffer::new(800, 600);
        assert_eq!(fb.width(), 800);
        assert_eq!(fb.height(), 600);
        assert_eq!(fb.pixels().len(), 800 * 600);
    }
    
    #[test]
    fn test_set_pixel_opaque() {
        let mut fb = Framebuffer::new(10, 10);
        fb.set_pixel(5, 5, 42);
        assert_eq!(fb.get_pixel(5, 5), 42);
    }
    
    #[test]
    fn test_set_pixel_transparent() {
        let mut fb = Framebuffer::new(10, 10);
        fb.set_pixel(5, 5, 42);  // Set to 42
        assert_eq!(fb.get_pixel(5, 5), 42);
        
        fb.set_pixel(5, 5, 0);   // Try to set to transparent
        assert_eq!(fb.get_pixel(5, 5), 42);  // Should still be 42 (not written)
    }
    
    #[test]
    fn test_to_rgba() {
        let mut fb = Framebuffer::new(2, 2);
        let mut palette = [Color::RGB(0, 0, 0); 256];
        palette[1] = Color::RGB(255, 0, 0);  // Red
        palette[2] = Color::RGB(0, 255, 0);  // Green
        
        fb.set_pixel(0, 0, 1);
        fb.set_pixel(1, 0, 2);
        
        let rgba = fb.to_rgba(&palette);
        
        // Pixel (0,0) should be red
        assert_eq!(&rgba[0..4], &[255, 0, 0, 255]);
        // Pixel (1,0) should be green  
        assert_eq!(&rgba[4..8], &[0, 255, 0, 255]);
    }
}



