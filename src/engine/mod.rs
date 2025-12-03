/// Engine module - Low-level rendering and window management
///
/// This module handles:
/// - SDL2 initialization
/// - Window creation and management
/// - Rendering surface management
/// - Event pump access
// Sub-modules
pub mod direction;
pub mod isometric;
pub mod texture_cache;
pub mod tile_renderer;

// Re-exports
pub use direction::Direction;
pub use isometric::*;
pub use texture_cache::TextureCache;

use crate::math::{Point, Rect};
use crate::renderer::Color;
use crate::sprite::TextureManager;
use anyhow::{bail, Result};
use sdl2::image::LoadTexture;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture as SdlTexture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::Sdl;

/// Engine structure that manages SDL2 and rendering
pub struct Engine {
    sdl_context: Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    canvas: sdl2::render::WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
    texture_manager: TextureManager<'static>,

    /// Tile texture cache for dungeon tile rendering
    /// Uses separate cache from general texture_manager for better organization
    tile_texture_cache: Option<TextureCache<'static>>,
}

impl Engine {
    /// Create a new engine instance
    ///
    /// Initializes SDL2, creates a window, and sets up rendering.
    /// Window size matches Diablo's original resolution: 640x480
    pub fn new() -> Result<Self> {
        // Initialize SDL2
        let sdl_context =
            sdl2::init().map_err(|e| anyhow::anyhow!("Failed to initialize SDL2: {}", e))?;

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

        // Set texture filtering to nearest (pixel-perfect) to avoid seams
        // Reference: C++ uses SDL_HINT_RENDER_SCALE_QUALITY="0" (nearest) or "1" (linear)
        // Source/utils/display.cpp::ReinitializeTexture() Line 708-709
        // This must be set BEFORE creating the renderer/canvas
        sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");

        // Create canvas for rendering
        let canvas = window
            .into_canvas()
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create canvas: {}", e))?;

        // Create texture creator (for texture rendering)
        let texture_creator = canvas.texture_creator();

        // Create texture manager
        // Note: We use unsafe to extend lifetime, as texture_creator lives as long as canvas
        let texture_manager = unsafe {
            std::mem::transmute::<TextureManager, TextureManager<'static>>(TextureManager::new(
                &texture_creator,
            ))
        };

        // Create tile texture cache for dungeon tile rendering
        // Safety: tile_texture_cache and texture_creator both live for the duration of Engine
        let tile_texture_cache = Some(unsafe {
            std::mem::transmute::<TextureCache, TextureCache<'static>>(TextureCache::new(
                &texture_creator,
            ))
        });

        Ok(Self {
            sdl_context,
            video_subsystem,
            canvas,
            texture_creator,
            texture_manager,
            tile_texture_cache,
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

    /// Get mutable tile texture cache reference
    ///
    /// Used for caching dungeon tile textures during rendering.
    pub fn tile_texture_cache_mut(&mut self) -> &mut TextureCache<'static> {
        self.tile_texture_cache
            .as_mut()
            .expect("Tile texture cache should always be initialized")
    }

    /// Create SDL texture from CLX frame
    ///
    /// Converts a ClxFrame's indexed pixel data into an SDL texture using a palette.
    /// Used for loading dungeon tile textures from CEL files.
    ///
    /// # Arguments
    /// * `frame` - CLX frame containing indexed color data
    /// * `palette` - Palette to use for color conversion
    ///
    /// # Returns
    /// SDL texture ready for rendering
    pub fn create_texture_from_clx_frame(
        &mut self,
        frame: &crate::resources::ClxFrame,
        palette: &crate::resources::Palette,
    ) -> Result<SdlTexture> {
        let width = frame.width as u32;
        let height = frame.height as u32;

        // Create texture
        let mut texture = self
            .texture_creator
            .create_texture_static(PixelFormatEnum::RGBA8888, width, height)
            .map_err(|e| anyhow::anyhow!("Failed to create texture: {}", e))?;

        // Convert indexed pixel data to RGBA using palette
        let mut rgba_data = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let src_idx = (y * width + x) as usize;
                let dst_idx = src_idx * 4;

                if src_idx < frame.pixels.len() {
                    if let Some(palette_index) = frame.pixels[src_idx] {
                        // Get color from palette
                        let color = palette.to_rgb(palette_index);
                        rgba_data[dst_idx] = color.r;
                        rgba_data[dst_idx + 1] = color.g;
                        rgba_data[dst_idx + 2] = color.b;
                        rgba_data[dst_idx + 3] = 255; // Opaque
                    } else {
                        // Transparent pixel
                        rgba_data[dst_idx] = 0;
                        rgba_data[dst_idx + 1] = 0;
                        rgba_data[dst_idx + 2] = 0;
                        rgba_data[dst_idx + 3] = 0;
                    }
                }
            }
        }

        // Update texture
        texture
            .update(None, &rgba_data, (width * 4) as usize)
            .map_err(|e| anyhow::anyhow!("Failed to update texture: {}", e))?;

        // Set blend mode for transparency
        texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        Ok(texture)
    }

    /// Create a texture from raw RGBA data
    ///
    /// # Arguments
    /// * `rgba_data` - RGBA pixel data (4 bytes per pixel)
    /// * `width` - Texture width
    /// * `height` - Texture height
    ///
    /// # Returns
    /// SDL texture ready for rendering
    pub fn create_texture_from_rgba(
        &mut self,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<SdlTexture> {
        if rgba_data.len() != (width * height * 4) as usize {
            return Err(anyhow::anyhow!(
                "RGBA data size mismatch: expected {}, got {}",
                width * height * 4,
                rgba_data.len()
            ));
        }

        let mut texture = self
            .texture_creator
            .create_texture_static(PixelFormatEnum::RGBA8888, width, height)
            .map_err(|e| anyhow::anyhow!("Failed to create texture: {}", e))?;

        texture
            .update(None, rgba_data, (width * 4) as usize)
            .map_err(|e| anyhow::anyhow!("Failed to update texture: {}", e))?;

        texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        Ok(texture)
    }

    /// Draw RGBA texture data directly to screen
    ///
    /// Creates a temporary texture from RGBA data and draws it immediately.
    /// For frequently used tiles, consider pre-caching.
    ///
    /// # Arguments
    /// * `_texture_id` - Unused (for future caching)
    /// * `rgba_data` - RGBA pixel data (4 bytes per pixel)
    /// * `width` - Texture width
    /// * `height` - Texture height
    /// * `dst_rect` - Destination rectangle on screen
    pub fn draw_rgba_texture(
        &mut self,
        _texture_id: &str,
        rgba_data: &[u8],
        width: u32,
        height: u32,
        dst_rect: crate::math::Rect,
    ) -> Result<bool> {
        if rgba_data.len() != (width * height * 4) as usize {
            return Err(anyhow::anyhow!(
                "RGBA data size mismatch: expected {}, got {}",
                width * height * 4,
                rgba_data.len()
            ));
        }

        let sdl_rect =
            sdl2::rect::Rect::new(dst_rect.x, dst_rect.y, dst_rect.width, dst_rect.height);

        // Create new texture
        // Note: Texture filtering is set at renderer level via SDL_HINT_RENDER_SCALE_QUALITY
        // which was set in Engine::new() before creating the canvas
        let mut sdl_texture = self
            .texture_creator
            .create_texture_static(PixelFormatEnum::ABGR8888, width, height)
            .map_err(|e| anyhow::anyhow!("Failed to create texture: {}", e))?;

        sdl_texture
            .update(None, rgba_data, (width * 4) as usize)
            .map_err(|e| anyhow::anyhow!("Failed to update texture: {}", e))?;

        // For Solid tiles (floors), use None blend mode (opaque) like C++ BlitPixelsDirect
        // For Transparent tiles, use Blend mode
        // Reference: Source/engine/render/dun_render.cpp::RenderLineOpaque() Line 130-135
        // C++ uses BlitPixelsDirect (no blending) for Solid mask type
        // This prevents transparent edge pixels from blending and creating visible seams
        sdl_texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        // Draw dungeon tiles with vertical flip only
        // CLX special CEL files (trees, etc.) store pixels bottom-to-top, need flip_v
        // Reference: CLX format is bottom-to-top, SDL Y-axis is top-to-bottom
        self.canvas
            .copy_ex(
                &sdl_texture,
                None,
                sdl_rect,
                0.0,
                None,
                false, // flip_h - no horizontal flip
                true,  // flip_v - vertical flip for CLX bottom-to-top format
            )
            .map_err(|e| anyhow::anyhow!("Failed to copy texture: {}", e))?;

        Ok(true)
    }

    /// Register multiple CEL tile textures in batch
    ///
    /// Creates and registers SDL textures for all frames in a CEL tileset.
    /// This method avoids borrow checker issues by handling all textures in one call.
    ///
    /// # Arguments
    /// * `frames` - Vector of CLX frames from CEL file
    /// * `palette` - Palette to use for color conversion
    /// * `id_prefix` - Prefix for texture IDs (e.g. "tile_" results in "tile_0", "tile_1", etc.)
    ///
    /// # Returns
    /// Number of textures successfully registered
    pub fn register_cel_textures(
        &mut self,
        frames: &[crate::resources::ClxFrame],
        palette: &crate::resources::Palette,
        id_prefix: &str,
    ) -> Result<usize> {
        use crate::sprite::Texture;

        let mut registered_count = 0;

        for (i, frame) in frames.iter().enumerate() {
            let texture_id = format!("{}{}", id_prefix, i);
            let width = frame.width as u32;
            let height = frame.height as u32;

            // Create texture directly (inline to avoid borrow checker issues)
            let texture_result = (|| -> Result<SdlTexture> {
                let mut texture = self
                    .texture_creator
                    .create_texture_static(PixelFormatEnum::RGBA8888, width, height)
                    .map_err(|e| anyhow::anyhow!("Failed to create texture: {}", e))?;

                // Convert indexed pixel data to RGBA using palette
                let mut rgba_data = vec![0u8; (width * height * 4) as usize];
                for y in 0..height {
                    for x in 0..width {
                        let src_idx = (y * width + x) as usize;
                        let dst_idx = src_idx * 4;

                        if src_idx < frame.pixels.len() {
                            if let Some(palette_index) = frame.pixels[src_idx] {
                                let color = palette.to_rgb(palette_index);
                                rgba_data[dst_idx] = color.r;
                                rgba_data[dst_idx + 1] = color.g;
                                rgba_data[dst_idx + 2] = color.b;
                                rgba_data[dst_idx + 3] = 255;
                            } else {
                                // Transparent pixel
                                rgba_data[dst_idx] = 0;
                                rgba_data[dst_idx + 1] = 0;
                                rgba_data[dst_idx + 2] = 0;
                                rgba_data[dst_idx + 3] = 0;
                            }
                        }
                    }
                }

                texture
                    .update(None, &rgba_data, (width * 4) as usize)
                    .map_err(|e| anyhow::anyhow!("Failed to update texture: {}", e))?;

                texture.set_blend_mode(sdl2::render::BlendMode::Blend);

                Ok(texture)
            })();

            match texture_result {
                Ok(texture) => {
                    let query = texture.query();

                    // Register texture in texture manager
                    unsafe {
                        let texture_wrapper = Texture {
                            texture: std::mem::transmute::<SdlTexture, SdlTexture<'static>>(
                                texture,
                            ),
                            width: query.width,
                            height: query.height,
                        };
                        self.texture_manager
                            .textures
                            .insert(texture_id, texture_wrapper);
                    }

                    registered_count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to create texture for frame {}: {}", i, e);
                }
            }
        }

        Ok(registered_count)
    }

    /// Load PNG tiles directly as textures (without palette conversion)
    ///
    /// This preserves the original RGB colors from PNG files.
    ///
    /// # Arguments
    /// * `folder_path` - Path to folder containing tile_0.png, tile_1.png, etc.
    /// * `id_prefix` - Prefix for texture IDs (e.g. "tile_")
    ///
    /// # Returns
    /// Number of textures registered
    pub fn register_png_tile_textures(
        &mut self,
        folder_path: &str,
        id_prefix: &str,
    ) -> Result<usize> {
        use std::path::Path;

        let folder = Path::new(folder_path);
        let mut registered_count = 0;
        let mut index = 0;

        loop {
            let tile_path = folder.join(format!("tile_{}.png", index));

            if !tile_path.exists() {
                break;
            }

            let texture_id = format!("{}{}", id_prefix, index);

            // Load PNG directly as SDL texture
            match self.load_texture(&texture_id, tile_path.to_str().unwrap()) {
                Ok(()) => {
                    registered_count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load tile {}: {}", index, e);
                }
            }

            index += 1;
        }

        if registered_count == 0 {
            bail!("No tile PNG files found in folder: {}", folder_path);
        }

        Ok(registered_count)
    }

    /// Load a texture using the texture creator directly
    /// This ensures we use the texture_creator from Engine, not from TextureManager
    /// The texture_creator is created from canvas and should have a valid renderer
    pub fn load_texture(&mut self, id: &str, path: &str) -> Result<()> {
        // Use texture_creator directly from Engine
        // This ensures the renderer is valid
        let texture = self
            .texture_creator
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
            (*texture_manager_ptr)
                .textures
                .insert(id.to_string(), texture_wrapper);
        }

        Ok(())
    }

    /// Load a texture from RGBA pixel data
    ///
    /// This method creates a texture from raw RGBA pixel data (4 bytes per pixel).
    /// Used for loading PCX and CLX resources converted to RGBA format.
    pub fn load_texture_from_rgba(
        &mut self,
        id: &str,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<()> {
        // Verify data size
        let expected_size = (width * height * 4) as usize;
        if rgba_data.len() != expected_size {
            return Err(anyhow::anyhow!(
                "Invalid RGBA data size: expected {} bytes, got {} bytes",
                expected_size,
                rgba_data.len()
            ));
        }

        // Create a streaming texture
        let mut texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, width, height)
            .map_err(|e| anyhow::anyhow!("Failed to create texture: {}", e))?;

        // Enable alpha blending for transparency
        texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        // Write RGBA data to texture
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..height as usize {
                    let src_offset = y * width as usize * 4;
                    let dst_offset = y * pitch;
                    let row_size = width as usize * 4;
                    buffer[dst_offset..dst_offset + row_size]
                        .copy_from_slice(&rgba_data[src_offset..src_offset + row_size]);
                }
            })
            .map_err(|e| anyhow::anyhow!("Failed to write texture data: {}", e))?;

        // Store in texture manager
        unsafe {
            use crate::sprite::Texture;
            let texture_wrapper = Texture {
                texture: std::mem::transmute::<SdlTexture, SdlTexture<'static>>(texture),
                width,
                height,
            };
            let texture_manager_ptr = &mut self.texture_manager as *mut TextureManager<'static>;
            (*texture_manager_ptr)
                .textures
                .insert(id.to_string(), texture_wrapper);
        }

        Ok(())
    }

    /// Clear the rendering surface with a color
    pub fn clear(&mut self) -> Result<()> {
        // 🔧 FIX: Use dark gray instead of pure black to make tile seams less visible
        // Pure black background makes transparent pixels in triangles show as black gaps
        // TODO: Replace with proper lighting system in Step 6.4
        self.clear_with_color(Color::new(16, 16, 16))
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
        self.canvas
            .fill_rect(rect.to_sdl())
            .map_err(|e| anyhow::anyhow!("Failed to draw rectangle: {}", e))?;
        Ok(())
    }

    /// Draw a rectangle outline
    pub fn draw_rect_outline(&mut self, rect: Rect, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas
            .draw_rect(rect.to_sdl())
            .map_err(|e| anyhow::anyhow!("Failed to draw rectangle outline: {}", e))?;
        Ok(())
    }

    /// Draw a line
    pub fn draw_line(&mut self, start: Point, end: Point, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas
            .draw_line(start.to_sdl(), end.to_sdl())
            .map_err(|e| anyhow::anyhow!("Failed to draw line: {}", e))?;
        Ok(())
    }

    /// Draw a point (single pixel)
    pub fn draw_point(&mut self, point: Point, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color.to_sdl());
        self.canvas
            .draw_point(point.to_sdl())
            .map_err(|e| anyhow::anyhow!("Failed to draw point: {}", e))?;
        Ok(())
    }

    /// Draw a filled diamond (isometric tile shape)
    ///
    /// Draws a filled diamond centered at the given point.
    /// The diamond has width TILE_WIDTH (64px) and height TILE_HEIGHT (32px).
    ///
    /// # Arguments
    /// * `center` - Center point of the diamond
    /// * `color` - Fill color
    ///
    /// # Diamond vertices (for 64x32 tile):
    /// ```text
    ///       top (cx, cy-16)
    ///        *
    ///       / \
    ///      /   \
    /// left     right
    /// (cx-32,cy) (cx+32,cy)
    ///      \   /
    ///       \ /
    ///        *
    ///     bottom (cx, cy+16)
    /// ```
    ///
    /// # Reference
    /// Original code: `Source/automap.cpp::DrawDiamond()` (Line 165-171)
    pub fn draw_diamond(&mut self, center: Point, color: Color) -> Result<()> {
        use crate::engine::isometric::{TILE_HEIGHT, TILE_WIDTH};

        let half_width = (TILE_WIDTH / 2) as i32;
        let half_height = (TILE_HEIGHT / 2) as i32;

        self.canvas.set_draw_color(color.to_sdl());

        // 上半部分：从顶点到中线
        for i in 0..=half_height {
            let y = center.y - half_height + i;
            let width = (i * 2) as i32;
            let x_start = center.x - width;
            let x_end = center.x + width;

            // 使用draw_line绘制水平线（更高效）
            if x_start <= x_end {
                self.canvas
                    .draw_line((x_start, y), (x_end, y))
                    .map_err(|e| anyhow::anyhow!("Failed to draw diamond line: {}", e))?;
            }
        }

        // 下半部分：从中线到底点
        for i in 1..=half_height {
            let y = center.y + i;
            let width = half_width - (i * 2);
            let x_start = center.x - width;
            let x_end = center.x + width;

            if x_start <= x_end {
                self.canvas
                    .draw_line((x_start, y), (x_end, y))
                    .map_err(|e| anyhow::anyhow!("Failed to draw diamond line: {}", e))?;
            }
        }

        Ok(())
    }

    /// Draw a diamond outline (isometric tile shape)
    ///
    /// Draws only the outline of a diamond using 4 lines.
    /// This matches the original DrawDiamond implementation.
    ///
    /// # Arguments
    /// * `center` - Center point of the diamond
    /// * `color` - Line color
    ///
    /// # Reference
    /// Original code: `Source/automap.cpp::DrawDiamond()` (Line 165-171)
    pub fn draw_diamond_outline(&mut self, center: Point, color: Color) -> Result<()> {
        use crate::engine::isometric::{TILE_HEIGHT, TILE_WIDTH};

        let half_width = (TILE_WIDTH / 2) as i32; // 32
        let half_height = (TILE_HEIGHT / 2) as i32; // 16

        // 4个顶点
        let top = Point::new(center.x, center.y - half_height);
        let bottom = Point::new(center.x, center.y + half_height);
        let left = Point::new(center.x - half_width, center.y);
        let right = Point::new(center.x + half_width, center.y);

        // 4条边
        self.draw_line(left, top, color)?; // 左上边
        self.draw_line(left, bottom, color)?; // 左下边
        self.draw_line(top, right, color)?; // 右上边
        self.draw_line(bottom, right, color)?; // 右下边

        Ok(())
    }

    /// Draw a texture at a position
    pub fn draw_texture(
        &mut self,
        texture: &SdlTexture,
        src: Option<Rect>,
        dst: Rect,
    ) -> Result<()> {
        let src_sdl = src.map(|r| r.to_sdl());
        let dst_sdl = dst.to_sdl();

        self.canvas
            .copy(texture, src_sdl, dst_sdl)
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
        // DEBUG: Print first few texture lookups
        static mut LOOKUP_COUNT: usize = 0;
        unsafe {
            LOOKUP_COUNT += 1;
            if LOOKUP_COUNT <= 10 {
                println!("  [draw_texture_by_id] Looking for: '{}'", texture_id);
            }
        }

        // Get texture reference and convert to static lifetime using raw pointer
        // Safety: Textures are valid for the entire lifetime of Engine because
        // they're owned by texture_creator which lives as long as Engine.
        let texture_ptr_opt: Option<*const SdlTexture> = {
            let texture_mgr = self.texture_manager();

            // DEBUG: Check if texture exists
            unsafe {
                if LOOKUP_COUNT <= 10 {
                    if texture_mgr.contains(texture_id) {
                        println!("    ✓ Texture found in manager");
                    } else {
                        println!("    ✗ Texture NOT found in manager");
                        println!(
                            "    Available textures: {:?}",
                            texture_mgr.textures.keys().take(5).collect::<Vec<_>>()
                        );
                    }
                }
            }

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
                self.canvas
                    .copy(texture, src_sdl, dst_sdl)
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

        self.canvas
            .copy_ex(texture, src_sdl, dst_sdl, rotation, None, flip_h, flip_v)
            .map_err(|e| anyhow::anyhow!("Failed to draw texture: {}", e))?;

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
