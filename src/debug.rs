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
    /// Show debug info overlay
    pub show_debug_info: bool,
}

impl Default for RenderDebugFlags {
    fn default() -> Self {
        // Default: Only render floor layer for easier debugging
        Self {
            render_floor: true,
            render_walls: false,      // Disabled by default for floor debugging
            render_entities: false,     // Disabled by default for floor debugging
            show_debug_info: false,
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
            show_debug_info: false,
        }
    }
    
    /// Create debug flags with all layers disabled
    pub fn all_disabled() -> Self {
        Self {
            render_floor: false,
            render_walls: false,
            render_entities: false,
            show_debug_info: false,
        }
    }
    
    /// Create debug flags with only floor layer enabled (for debugging)
    pub fn floor_only() -> Self {
        Self {
            render_floor: true,
            render_walls: false,
            render_entities: false,
            show_debug_info: false,
        }
    }
}

