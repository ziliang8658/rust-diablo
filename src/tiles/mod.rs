pub mod decoder;
pub mod min;
pub mod sol;
pub mod texture_manager;
pub mod til;
/// Tile system module
///
/// This module handles tile data loading and management for Diablo's level rendering system.
/// It includes:
/// - Tile type definitions (Square, Triangle, Trapezoid, etc.)
/// - MIN file loading (MicroTile indices)
/// - TIL file loading (MegaTile definitions)
/// - SOL file loading (Tile properties)
///
/// # Overview
///
/// Diablo uses a hierarchical tile system:
///
/// ```text
/// MegaTile (地图生成层, 40x40)
///   ↓ (每个MegaTile包含2x2个MicroTile)
/// MicroTile (渲染层, 112x112)
///   ↓ (每个MicroTile引用一个CEL帧)
/// CEL Frame (图形数据)
/// ```
///
/// ## Data Files
///
/// - **MIN files** (.min): MicroTile index data
///   - Each entry is a u16 value encoding tile type and CEL frame index
///   - Used for low-level tile rendering
///
/// - **TIL files** (.til): MegaTile definitions
///   - Each MegaTile contains 4 MicroTile indices (2x2 grid)
///   - Used for map generation
///
/// - **SOL files** (.sol): Tile properties
///   - Each entry is a u8 value with property flags
///   - Defines walkability, light blocking, etc.
///
/// # References
/// - Original code: `Source/levels/gendung.h`, `Source/levels/gendung.cpp`
/// - Original code: `Source/levels/dun_tile.hpp`
// Sub-modules
pub mod types;

// Re-exports for convenience
pub use types::{
    DungeonType, LevelCelBlock, TileProperties, TileType, DUN_FRAME_HEIGHT,
    DUN_FRAME_TRIANGLE_HEIGHT, DUN_FRAME_WIDTH,
};

pub use decoder::{
    decode_left_trapezoid, decode_left_triangle, decode_right_trapezoid, decode_right_triangle,
    decode_square, decode_tile, decode_transparent_square,
};
pub use min::{MinData, PieceMicros};
pub use sol::{SolData, MAXTILES};
pub use til::{MegaTile, TilData};

use crate::resources::ClxFrame;
use anyhow::{anyhow, Error, Result};

pub(crate) fn load_first_candidate<T>(
    candidates: &[&'static str],
    mut load: impl FnMut(&'static str) -> Result<T>,
    kind: &'static str,
) -> Result<T> {
    let mut last_err: Option<Error> = None;
    for &path in candidates {
        match load(path) {
            Ok(result) => return Ok(result),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("Failed to load {kind} file")))
}

/// Dungeon tileset textures
///
/// Manages CEL tile textures for dungeon rendering.
/// Each frame in the CEL corresponds to a tile that can be rendered.
///
/// # Structure
///
/// The tileset contains:
/// - Decoded tile frames from the CEL file
/// - Texture IDs for each frame (format: "tile_{index}")
///
/// # Usage
///
/// ```no_run
/// use rust_diablo::tiles::DungeonTileset;
///
/// // Create tileset from loaded CEL frames
/// let tileset = DungeonTileset::from_cel_frames(cel_frames);
///
/// // Get texture ID for a frame index (1-based from LevelCelBlock)
/// if let Some(texture_id) = tileset.get_texture_id(42) {
///     println!("Texture ID: {}", texture_id);
/// }
/// ```
pub struct DungeonTileset {
    /// Tile texture frames (indexed by LevelCelBlock.frame())
    pub frames: Vec<ClxFrame>,

    /// Texture IDs in the engine's texture manager
    /// Format: "tile_{frame_index}"
    pub texture_ids: Vec<String>,
}

impl DungeonTileset {
    /// Create tileset from CEL frames
    ///
    /// # Arguments
    /// * `frames` - Decoded CLX frames from CEL file
    ///
    /// # Returns
    /// New DungeonTileset instance
    pub fn from_cel_frames(frames: Vec<ClxFrame>) -> Self {
        let texture_ids = (0..frames.len()).map(|i| format!("tile_{}", i)).collect();

        Self {
            frames,
            texture_ids,
        }
    }

    /// Get texture ID for a given frame index (1-based from LevelCelBlock)
    ///
    /// LevelCelBlock stores 1-based frame indices, this method converts
    /// to 0-based and returns the corresponding texture ID.
    ///
    /// # Arguments
    /// * `frame_index` - Frame index from LevelCelBlock (1-based, 0 = empty)
    ///
    /// # Returns
    /// Texture ID string, or None if frame_index is 0
    ///
    /// # Note
    /// If frame_index exceeds available textures, it wraps around (modulo).
    /// This allows using a small set of test tiles for larger tile sets.
    pub fn get_texture_id(&self, frame_index: u16) -> Option<&str> {
        if frame_index == 0 || self.texture_ids.is_empty() {
            return None;
        }

        // Frame index is 1-based, convert to 0-based
        let index = (frame_index - 1) as usize;

        // Wrap around if index exceeds available textures (for test tiles)
        let wrapped_index = index % self.texture_ids.len();

        // DEBUG: Print first few lookups
        static mut DEBUG_COUNT: usize = 0;
        unsafe {
            DEBUG_COUNT += 1;
            if DEBUG_COUNT <= 10 {
                println!(
                    "  [get_texture_id] frame_index={} -> index={} -> wrapped={}/{}",
                    frame_index,
                    index,
                    wrapped_index,
                    self.texture_ids.len()
                );
                if let Some(tex_id) = self.texture_ids.get(wrapped_index) {
                    println!("    -> texture_id='{}'", tex_id);
                }
            }
        }

        self.texture_ids.get(wrapped_index).map(|s| s.as_str())
    }

    /// Get the total number of frames
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Get frame dimensions
    ///
    /// # Arguments
    /// * `frame_index` - Frame index from LevelCelBlock (1-based)
    ///
    /// # Returns
    /// (width, height) tuple, or None if frame_index is invalid
    ///
    /// # Note
    /// If frame_index exceeds available frames, it wraps around (modulo).
    /// This matches the behavior of get_texture_id().
    pub fn get_frame_size(&self, frame_index: u16) -> Option<(u32, u32)> {
        if frame_index == 0 || self.frames.is_empty() {
            return None;
        }

        let index = (frame_index - 1) as usize;

        // Wrap around if index exceeds available frames (matches get_texture_id behavior)
        let wrapped_index = index % self.frames.len();

        self.frames
            .get(wrapped_index)
            .map(|f| (f.width as u32, f.height as u32))
    }

    /// Check if tileset is empty
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}
