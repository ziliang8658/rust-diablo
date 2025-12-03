/// SDL2 Texture Cache
///
/// Manages SDL2 textures with caching to avoid recreating textures every frame.
/// Converts RGBA pixel data to SDL2 textures with proper blending for transparency.
///
/// # Architecture
///
/// - Textures are cached by micro_index (unique tile identifier)
/// - RGBA data is converted to SDL2's ABGR8888 format
/// - Blend mode is set to support transparency
///
/// # Reference
///
/// Original code: `Source/engine/render/dun_render.cpp` - various Render*() functions
/// - Line 305-418: RenderTile() - main tile rendering
/// - Tiles are rendered directly to screen buffer in C++ (software rendering)
/// - Rust uses SDL2 hardware-accelerated textures for better performance
use anyhow::{Context, Result};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::collections::HashMap;

/// SDL2 texture cache
///
/// Manages creation and caching of SDL2 textures from RGBA pixel data.
/// Textures are cached by a unique identifier to avoid redundant uploads to GPU.
pub struct TextureCache<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    cache: HashMap<usize, Texture<'a>>,
}

impl<'a> TextureCache<'a> {
    /// Create a new texture cache
    ///
    /// # Arguments
    /// * `texture_creator` - SDL2 texture creator (must outlive the cache)
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            texture_creator,
            cache: HashMap::new(),
        }
    }

    /// Get or create a texture from RGBA data
    ///
    /// If the texture already exists in cache, returns the cached version.
    /// Otherwise, creates a new texture and caches it.
    ///
    /// # Arguments
    /// * `micro_index` - Unique identifier for this tile
    /// * `rgba_data` - RGBA pixel data (4 bytes per pixel: R, G, B, A)
    /// * `width` - Texture width in pixels
    /// * `height` - Texture height in pixels
    ///
    /// # Returns
    /// Reference to the SDL2 texture
    ///
    /// # Reference
    /// Original: Source/engine/render/dun_render.cpp::RenderTile() Line 305-418
    pub fn get_or_create_texture(
        &mut self,
        micro_index: usize,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<&Texture<'a>> {
        // Check if texture already exists
        if !self.cache.contains_key(&micro_index) {
            // Create new texture
            let mut texture = self
                .texture_creator
                .create_texture_streaming(PixelFormatEnum::ABGR8888, width, height)
                .context("Failed to create SDL2 texture")?;

            // Set blend mode to support transparency
            // Reference: SDL2 uses BlendMode::Blend for standard alpha blending
            texture.set_blend_mode(BlendMode::Blend);

            // Upload pixel data to texture
            texture
                .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    // Convert RGBA to ABGR (SDL2 format)
                    // Reference: SDL2 ABGR8888 format layout
                    for y in 0..height as usize {
                        for x in 0..width as usize {
                            let src_idx = (y * width as usize + x) * 4;
                            let dst_idx = y * pitch + x * 4;

                            if src_idx + 3 < rgba_data.len() && dst_idx + 3 < buffer.len() {
                                // RGBA -> ABGR conversion
                                buffer[dst_idx + 0] = rgba_data[src_idx + 3]; // A
                                buffer[dst_idx + 1] = rgba_data[src_idx + 2]; // B
                                buffer[dst_idx + 2] = rgba_data[src_idx + 1]; // G
                                buffer[dst_idx + 3] = rgba_data[src_idx + 0]; // R
                            }
                        }
                    }
                })
                .map_err(|e| anyhow::anyhow!("Failed to lock texture for pixel upload: {}", e))?;

            // Cache the texture
            self.cache.insert(micro_index, texture);
        }

        // Return cached texture
        Ok(self.cache.get(&micro_index).unwrap())
    }

    /// Clear all cached textures
    ///
    /// Useful for freeing GPU memory when changing levels or tilesets.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// (cached_count, capacity)
    pub fn stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }
}

#[cfg(test)]
mod tests {
    // Note: Full tests require SDL2 initialization and are in integration tests
    // See tests/texture_cache_tests.rs for full tests
}
