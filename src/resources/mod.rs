/// Resources module - MPQ archives, palettes, and resource loading
/// 
/// This module provides infrastructure for loading game resources from MPQ archives
/// and managing palettes for indexed color images.
/// 
/// 原生实现 MPQ 读取，完全兼容 Diablo 1 的早期 MPQ 格式

pub mod mpq;
pub mod palette;
pub mod pcx;
pub mod clx;
pub mod cl2;
pub mod trn;
pub mod dungeon_cel;
pub mod resource_manager;

// libmpq FFI 支持
pub mod libmpq_ffi;
pub mod mpq_wrapper;

// Re-export for convenience
pub use mpq::MpqManager;
pub use palette::{Color, Palette};
pub use pcx::{PcxImage, PcxHeader};
pub use clx::{ClxSprite, ClxFrame, ClxHeader, ClxFrameHeader};
pub use cl2::Cl2Sprite;
pub use trn::ColorTransform;
pub use dungeon_cel::{DungeonCelSprite, DungeonCelFrame};
pub use resource_manager::ResourceManager;

// Re-export libmpq wrapper
pub use mpq_wrapper::LibMpqArchive;


