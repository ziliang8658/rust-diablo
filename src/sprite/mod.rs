pub mod animation;
pub mod sprite;
/// Sprite module - Sprite and animation system
///
/// This module provides sprite rendering and animation:
/// - Texture loading and management
/// - Sprite rendering
/// - Frame-based animation
pub mod texture;

pub use animation::{Animation, AnimationController, AnimationState};
pub use texture::{Texture, TextureManager};
