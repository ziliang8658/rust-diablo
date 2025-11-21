/// Engine module - Low-level rendering and window management
/// 
/// This module handles:
/// - SDL2 initialization
/// - Window creation and management
/// - Rendering surface management
/// - Event pump access

// Sub-modules
pub mod direction;

// Re-exports
pub use direction::Direction;

use anyhow::Result;
use sdl2::Sdl;
use sdl2::render::{TextureCreator, Texture as SdlTexture};
use sdl2::video::WindowContext;
use sdl2::image::LoadTexture;
use crate::math::{Point, Rect};
use crate::renderer::Color;
use crate::sprite::TextureManager;

/// Engine structure that manages SDL2 and rendering
pub struct Engine {
    sdl_context: Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    canvas: sdl2::render::WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
    texture_manager: TextureManager<'static>,
}

impl Engine {
    /// Create a new engine instance
    /// 
    /// Initializes SDL2, creates a window, and sets up rendering.
    /// Window size matches Diablo's original resolution: 640x480
    pub fn new() -> Result<Self> {
        // Initialize SDL2
        let sdl_context = sdl2::init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize SDL2: {}", e))?;
        
        // Initialize SDL2_image
        // This must be called before loading any textures with image formats
        sdl2::image::init(sdl2::image::InitFlag::PNG)
            .map_err(|e| anyhow::anyhow!("Failed to initialize SDL2_image: {}", e))?;
        
        // Initialize video subsystem
        let video_subsystem = sdl_context
            .video()
            .map_err(|e| anyhow::anyhow!("Failed to initialize SDL2 video subsystem: {}", e))?;
        
        // Create window
        // Diablo's original resolution was 640x480
        let window = video_subsystem
            .window("Rust Diablo", 640, 480)
            .position_centered()
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create window: {}", e))?;
        
        // Create canvas for rendering
        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create canvas: {}", e))?;
        
        // Create texture creator (for texture rendering)
        let texture_creator = canvas.texture_creator();
        
        // Create texture manager
        // Note: We use unsafe to extend lifetime, as texture_creator lives as long as canvas
        let texture_manager = unsafe {
            std::mem::transmute::<TextureManager, TextureManager<'static>>(
                TextureManager::new(&texture_creator)
            )
        };
        
        Ok(Self {
            sdl_context,
            video_subsystem,
            canvas,
            texture_creator,
            texture_manager,
        })
    }

    /// Get SDL context reference (for creating event pump in Game)
    pub fn sdl_context(&self) -> &Sdl {
        &self.sdl_context
    }

    /// Get texture creator reference
    pub fn texture_creator(&self) -> &TextureCreator<WindowContext> {
        &self.texture_creator
    }

    /// Get texture manager reference
    pub fn texture_manager(&self) -> &TextureManager<'static> {
        &self.texture_manager
    }

    /// Get mutable texture manager reference
    pub fn texture_manager_mut(&mut self) -> &mut TextureManager<'static> {
        &mut self.texture_manager
    }

    /// Load a texture using the texture creator directly
    /// This ensures we use the texture_creator from Engine, not from TextureManager
    /// The texture_creator is created from canvas and should have a valid renderer
    pub fn load_texture(&mut self, id: &str, path: &str) -> Result<()> {
        // Use texture_creator directly from Engine
        // This ensures the renderer is valid
        let texture = self.texture_creator
            .load_texture(path)
            .map_err(|e| anyhow::anyhow!("Failed to load texture {}: {}", path, e))?;
        
        let query = texture.query();
        let width = query.width;
        let height = query.height;
        
        // Store in texture manager
        unsafe {
            use crate::sprite::Texture;
            let texture_wrapper = Texture {
                texture: std::mem::transmute::<SdlTexture, SdlTexture<'static>>(texture),
                width,
                height,
            };
            let texture_manager_ptr = &mut self.texture_manager as *mut TextureManager<'static>;
            (*texture_manager_ptr).textures.insert(id.to_string(), texture_wrapper);
        }
        
        Ok(())
    }

    /// Clear the rendering surface with a color
    pub fn clear(&mut self) -> Result<()> {
        self.clear_with_color(Color::BLACK)
    }

    /// Clear the rendering surface with a specific color
    pub fn clear_with_color(&mut self, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas.clear();
        Ok(())
    }

    /// Draw a filled rectangle
    pub fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas.fill_rect(rect.to_sdl())
            .map_err(|e| anyhow::anyhow!("Failed to draw rectangle: {}", e))?;
        Ok(())
    }

    /// Draw a rectangle outline
    pub fn draw_rect_outline(&mut self, rect: Rect, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas.draw_rect(rect.to_sdl())
            .map_err(|e| anyhow::anyhow!("Failed to draw rectangle outline: {}", e))?;
        Ok(())
    }

    /// Draw a line
    pub fn draw_line(&mut self, start: Point, end: Point, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas.draw_line(start.to_sdl(), end.to_sdl())
            .map_err(|e| anyhow::anyhow!("Failed to draw line: {}", e))?;
        Ok(())
    }

    /// Draw a point (single pixel)
    pub fn draw_point(&mut self, point: Point, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas.draw_point(point.to_sdl())
            .map_err(|e| anyhow::anyhow!("Failed to draw point: {}", e))?;
        Ok(())
    }

    /// Draw a texture at a position
    pub fn draw_texture(
        &mut self, 
        texture: &SdlTexture, 
        src: Option<Rect>, 
        dst: Rect
    ) -> Result<()> {
        let src_sdl = src.map(|r| r.to_sdl());
        let dst_sdl = dst.to_sdl();
        
        self.canvas.copy(texture, src_sdl, dst_sdl)
            .map_err(|e| anyhow::anyhow!("Failed to draw texture: {}", e))?;
        Ok(())
    }

    /// Draw a texture by ID from the texture manager
    /// This method avoids borrow conflicts by internally accessing the texture manager
    /// 
    /// Safety: We use unsafe to extend the texture reference lifetime because textures
    /// are valid for the entire lifetime of the Engine (they're owned by texture_creator
    /// which lives as long as the Engine).
    pub fn draw_texture_by_id(
        &mut self,
        texture_id: &str,
        src: Option<Rect>,
        dst: Rect,
    ) -> Result<bool> {
        // Get texture reference and convert to static lifetime using raw pointer
        // Safety: Textures are valid for the entire lifetime of Engine because
        // they're owned by texture_creator which lives as long as Engine.
        let texture_ptr_opt: Option<*const SdlTexture> = {
            let texture_mgr = self.texture_manager();
            texture_mgr.get(texture_id).map(|t| {
                // Get raw pointer to the texture
                t.sdl_texture() as *const SdlTexture
            })
        };
        
        if let Some(texture_ptr) = texture_ptr_opt {
            // Safety: The texture is valid because:
            // 1. It's owned by texture_creator which lives as long as Engine
            // 2. We only use this reference for the duration of the copy call
            // 3. The texture won't be dropped while Engine exists
            unsafe {
                let texture = &*texture_ptr;
                let src_sdl = src.map(|r| r.to_sdl());
                let dst_sdl = dst.to_sdl();
                self.canvas.copy(texture, src_sdl, dst_sdl)
                    .map_err(|e| anyhow::anyhow!("Failed to draw texture: {}", e))?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Draw a texture with rotation and flip
    pub fn draw_texture_ex(
        &mut self,
        texture: &SdlTexture,
        src: Option<Rect>,
        dst: Rect,
        rotation: f64,
        flip_h: bool,
        flip_v: bool,
    ) -> Result<()> {
        let src_sdl = src.map(|r| r.to_sdl());
        let dst_sdl = dst.to_sdl();
        
        self.canvas.copy_ex(
            texture,
            src_sdl,
            dst_sdl,
            rotation,
            None,
            flip_h,
            flip_v,
        ).map_err(|e| anyhow::anyhow!("Failed to draw texture: {}", e))?;
        
        Ok(())
    }

    /// Present the rendered frame to the screen
    pub fn present(&mut self) {
        self.canvas.present();
    }

    /// Get the canvas for drawing operations
    /// 
    /// This allows other modules to draw directly to the canvas.
    #[allow(dead_code)]
    pub fn canvas_mut(&mut self) -> &mut sdl2::render::WindowCanvas {
        &mut self.canvas
    }
}

impl Drop for Engine {
    /// Cleanup when engine is dropped
    /// 
    /// SDL2 will automatically clean up resources when the context is dropped,
    /// but we can add explicit cleanup here if needed in the future.
    fn drop(&mut self) {
        // SDL2 cleanup happens automatically
    }
}
