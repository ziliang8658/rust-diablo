/// SDL2 Texture Cache
///
/// Manages SDL2 textures with caching to avoid recreating textures every frame.
/// Converts RGBA pixel data to SDL2 textures with proper blending for transparency.
///
/// # Architecture
///
/// - Textures are cached by micro_index (unique tile identifier)
/// - RGBA data is uploaded using the same ABGR8888 path as Engine::draw_rgba_texture
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
pub struct TextureCache {
    cache: HashMap<usize, Texture<'static>>,
}

impl TextureCache {
    /// Create a new texture cache
    pub fn new() -> Self {
        Self {
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
        texture_creator: &TextureCreator<WindowContext>,
        micro_index: usize,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<&Texture<'static>> {
        // Check if texture already exists
        if !self.cache.contains_key(&micro_index) {
            // Create and upload exactly like Engine::draw_rgba_texture, then keep
            // the texture around for subsequent frames.
            let mut texture = texture_creator
                .create_texture_static(PixelFormatEnum::ABGR8888, width, height)
                .context("Failed to create SDL2 texture")?;

            texture
                .update(None, rgba_data, (width * 4) as usize)
                .context("Failed to update SDL2 texture")?;

            // Set blend mode to support transparency.
            texture.set_blend_mode(BlendMode::Blend);

            // Cache the texture
            let texture = unsafe { std::mem::transmute::<Texture<'_>, Texture<'static>>(texture) };
            self.cache.insert(micro_index, texture);
        }

        // Return cached texture
        Ok(self.cache.get(&micro_index).unwrap())
    }

    /// Get a cached texture without creating or uploading one.
    pub fn get_texture(&self, micro_index: usize) -> Option<&Texture<'static>> {
        self.cache.get(&micro_index)
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
