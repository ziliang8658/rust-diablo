/// Sprite module - Sprite and animation system
/// 
/// This module provides sprite rendering and animation:
/// - Texture loading and management
/// - Sprite rendering
/// - Frame-based animation

pub mod texture;
pub mod sprite;
pub mod animation;

pub use texture::{Texture, TextureManager};
pub use animation::{Animation, AnimationState, AnimationController};

