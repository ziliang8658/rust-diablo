/// Debug module - Debugging tools and flags for development
///
/// This module contains debugging utilities, flags, and helpers
/// for troubleshooting rendering and other game systems.

/// Rendering layer debug flags for troubleshooting rendering issues
///
/// This allows toggling individual rendering layers to identify which layer
/// has problems. Based on the two-phase rendering approach from Diablo.
#[derive(Debug, Clone)]
pub struct RenderDebugFlags {
    /// Enable floor layer rendering (Phase 1)
    pub render_floor: bool,
    /// Enable wall/upper layer rendering (Phase 2)
    pub render_walls: bool,
    /// Enable entity rendering
    pub render_entities: bool,
    /// Show debug overlay
    pub show_debug_overlay: bool,
    /// Render only the 2x2 region around the view center (in dPiece coordinates)
    pub focus_render_2x2: bool,
    /// Log the focused render region/tile IDs (requires `focus_render_2x2`)
    pub log_render_focus: bool,
    /// Enable toon/comic post-processing on tiles
    pub toon_filter: bool,
    /// Apply Diablo DrawCell mask rules before uploading tile pixels.
    pub mask_aware_draw_cell: bool,
    /// Apply Diablo-style walking camera offset while the player is stepping.
    /// Still opt-in while the Rust tile scan is being stabilized.
    pub walking_camera_offset: bool,
    /// Log per-frame walk timing and offset diagnostics.
    pub log_walk_trace: bool,
    /// Enable x-axis wall pre-draw used to hide moving sprites behind walls.
    pub wall_predraw: bool,
}

impl Default for RenderDebugFlags {
    fn default() -> Self {
        // Default: Only render floor layer for easier debugging
        Self {
            render_floor: true,
            render_walls: false,    // Disabled by default for floor debugging
            render_entities: false, // Disabled by default for floor debugging
            show_debug_overlay: false,
            focus_render_2x2: false,
            log_render_focus: false,
            toon_filter: false,
            mask_aware_draw_cell: true,
            walking_camera_offset: false,
            log_walk_trace: false,
            wall_predraw: true,
        }
    }
}

impl RenderDebugFlags {
    /// Create debug flags with all layers enabled
    pub fn all_enabled() -> Self {
        Self {
            render_floor: true,
            render_walls: true,
            render_entities: true,
            show_debug_overlay: false,
            focus_render_2x2: false,
            log_render_focus: false,
            toon_filter: false,
            mask_aware_draw_cell: true,
            walking_camera_offset: false,
            log_walk_trace: false,
            wall_predraw: true,
        }
    }

    /// Create debug flags with all layers disabled
    pub fn all_disabled() -> Self {
        Self {
            render_floor: false,
            render_walls: false,
            render_entities: false,
            show_debug_overlay: false,
            focus_render_2x2: false,
            log_render_focus: false,
            toon_filter: false,
            mask_aware_draw_cell: true,
            walking_camera_offset: false,
            log_walk_trace: false,
            wall_predraw: true,
        }
    }

    /// Create debug flags with only floor layer enabled (for debugging)
    pub fn floor_only() -> Self {
        Self {
            render_floor: true,
            render_walls: false,
            render_entities: false,
            show_debug_overlay: false,
            focus_render_2x2: false,
            log_render_focus: false,
            toon_filter: false,
            mask_aware_draw_cell: true,
            walking_camera_offset: false,
            log_walk_trace: false,
            wall_predraw: true,
        }
    }
}
