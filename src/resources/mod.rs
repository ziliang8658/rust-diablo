pub mod cl2;
pub mod clx;
pub mod dungeon_cel;
/// Resources module - MPQ archives, palettes, and resource loading
///
/// This module provides infrastructure for loading game resources from MPQ archives
/// and managing palettes for indexed color images.
///
/// 原生实现 MPQ 读取，完全兼容 Diablo 1 的早期 MPQ 格式
pub mod mpq;
pub mod palette;
pub mod pcx;
pub mod resource_manager;
pub mod trn;

// libmpq FFI 支持
pub mod libmpq_ffi;
pub mod mpq_wrapper;

// Re-export for convenience
pub use cl2::{Cl2DirectionalSpriteSheet, Cl2Sprite};
pub use clx::{ClxFrame, ClxFrameHeader, ClxHeader, ClxSprite};
pub use dungeon_cel::{DungeonCelFrame, DungeonCelSprite};
pub use mpq::MpqManager;
pub use palette::{Color, Palette};
pub use pcx::{PcxHeader, PcxImage};
pub use resource_manager::ResourceManager;
pub use trn::ColorTransform;

// Re-export libmpq wrapper
pub use mpq_wrapper::LibMpqArchive;
