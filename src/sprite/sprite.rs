/// Sprite - Renderable sprite object
/// 
/// Represents a sprite that can be drawn on screen

use crate::math::{Point, Rect};

/// Sprite structure
#[derive(Clone)]
pub struct Sprite {
    /// Texture ID (reference to TextureManager)
    pub texture_id: String,
    /// Position in world space
    pub position: Point,
    /// Source rectangle (for sprite sheets)
    /// None means use entire texture
    pub src_rect: Option<Rect>,
    /// Size to render (None means use texture size)
    pub size: Option<(u32, u32)>,
    /// Horizontal and vertical flip
    pub flip: (bool, bool),
    /// Rotation angle in degrees
    pub rotation: f64,
}

impl Sprite {
    /// Create a new sprite
    pub fn new(texture_id: String, position: Point) -> Self {
        Self {
            texture_id,
            position,
            src_rect: None,
            size: None,
            flip: (false, false),
            rotation: 0.0,
        }
    }

    /// Set source rectangle (for sprite sheets)
    pub fn with_src_rect(mut self, rect: Rect) -> Self {
        self.src_rect = Some(rect);
        self
    }

    /// Set render size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    /// Set flip
    pub fn with_flip(mut self, h_flip: bool, v_flip: bool) -> Self {
        self.flip = (h_flip, v_flip);
        self
    }

    /// Set rotation
    pub fn with_rotation(mut self, angle: f64) -> Self {
        self.rotation = angle;
        self
    }
}

