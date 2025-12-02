/// Texture - Image texture management
/// 
/// Handles loading and storing SDL2 textures

use std::collections::HashMap;
use anyhow::{Context, Result};
use sdl2::render::{Texture as SdlTexture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::image::LoadTexture;

/// Texture wrapper
pub struct Texture<'a> {
    pub(crate) texture: SdlTexture<'a>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl<'a> Texture<'a> {
    /// Create a new texture from SDL texture
    pub fn new(texture: SdlTexture<'a>, width: u32, height: u32) -> Self {
        Self { texture, width, height }
    }
    
    /// Get texture width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get texture height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the underlying SDL texture
    pub fn sdl_texture(&self) -> &SdlTexture<'a> {
        &self.texture
    }
}

/// TextureManager - Manages all game textures
pub struct TextureManager<'a> {
    pub(crate) textures: HashMap<String, Texture<'a>>,
    texture_creator: &'a TextureCreator<WindowContext>,
}

impl<'a> TextureManager<'a> {
    /// Create a new texture manager
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            textures: HashMap::new(),
            texture_creator,
        }
    }

    /// Load a texture from file
    /// 
    /// Note: This method uses the TextureCreator's load_texture which internally
    /// uses the renderer associated with the TextureCreator.
    /// 
    /// IMPORTANT: The TextureCreator must be created from a valid canvas/renderer,
    /// and the renderer must remain valid for the lifetime of the TextureCreator.
    pub fn load(&mut self, id: &str, path: &str) -> Result<()> {
        // Use load_texture from the LoadTexture trait
        // This requires the renderer to be valid
        // Note: texture_creator.load_texture() internally calls the renderer's load_texture
        let texture = self.texture_creator
            .load_texture(path)
            .map_err(|e| anyhow::anyhow!("Failed to load texture {}: {}", path, e))?;
        
        let query = texture.query();
        let width = query.width;
        let height = query.height;
        
        self.textures.insert(
            id.to_string(),
            Texture { texture, width, height }
        );
        
        Ok(())
    }

    /// Get a texture by ID
    pub fn get(&self, id: &str) -> Option<&Texture<'a>> {
        self.textures.get(id)
    }

    /// Check if a texture exists
    pub fn contains(&self, id: &str) -> bool {
        self.textures.contains_key(id)
    }
    
    /// Add a pre-created texture
    pub fn add(&mut self, id: &str, texture: Texture<'a>) {
        self.textures.insert(id.to_string(), texture);
    }
}

