use crate::debug::RenderDebugFlags;
use sdl2::keyboard::{Keycode, Mod};
use tracing::info;

pub fn print_startup_help() {
    info!("Rust Diablo - Step 6.1: Tiles System Demo");
    info!("Controls: WASD/Arrow Keys move, ESC quits, F1 Cathedral, F2 Town");
    info!(
        "Devtools: F4 floor, F5 walls, F6 entities, F7 overlay, Shift+F7 focus, Ctrl+F7 focus log"
    );
    info!(
        "Devtools: F8 all layers, F9 floor only, F10 toon, F11 walk camera, Shift+F11 walk trace"
    );
    info!("Devtools: F12 wall pre-draw, Shift+F12 mask-aware DrawCell");
}

pub fn handle_render_debug_key(
    flags: &mut RenderDebugFlags,
    keycode: Keycode,
    keymod: Mod,
) -> bool {
    let Some(action) = RenderDebugAction::from_key(keycode, keymod) else {
        return false;
    };

    action.apply(flags);
    info!("{} {}", action.label(), action.state(flags));
    log_render_debug_status(flags);
    true
}

pub fn log_render_debug_status(flags: &RenderDebugFlags) {
    info!(
        floor = flags.render_floor,
        walls = flags.render_walls,
        entities = flags.render_entities,
        overlay = flags.show_debug_overlay,
        focus = flags.focus_render_2x2,
        focus_log = flags.log_render_focus,
        toon = flags.toon_filter,
        mask = flags.mask_aware_draw_cell,
        walk_camera = flags.walking_camera_offset,
        walk_trace = flags.log_walk_trace,
        wall_predraw = flags.wall_predraw,
        "render debug status"
    );
}

#[derive(Clone, Copy)]
enum RenderDebugAction {
    ToggleFloor,
    ToggleWalls,
    ToggleEntities,
    ToggleOverlay,
    ToggleFocusRender,
    ToggleFocusLog,
    ResetAll,
    FloorOnly,
    ToggleToon,
    ToggleWalkCamera,
    ToggleWalkTrace,
    ToggleWallPredraw,
    ToggleMaskAwareDrawCell,
}

impl RenderDebugAction {
    fn from_key(keycode: Keycode, keymod: Mod) -> Option<Self> {
        let shift = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
        let ctrl = keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD);

        match keycode {
            Keycode::F4 => Some(Self::ToggleFloor),
            Keycode::F5 => Some(Self::ToggleWalls),
            Keycode::F6 => Some(Self::ToggleEntities),
            Keycode::F7 if shift => Some(Self::ToggleFocusRender),
            Keycode::F7 if ctrl => Some(Self::ToggleFocusLog),
            Keycode::F7 => Some(Self::ToggleOverlay),
            Keycode::F8 => Some(Self::ResetAll),
            Keycode::F9 => Some(Self::FloorOnly),
            Keycode::F10 => Some(Self::ToggleToon),
            Keycode::F11 if shift => Some(Self::ToggleWalkTrace),
            Keycode::F11 => Some(Self::ToggleWalkCamera),
            Keycode::F12 if shift => Some(Self::ToggleMaskAwareDrawCell),
            Keycode::F12 => Some(Self::ToggleWallPredraw),
            _ => None,
        }
    }

    fn apply(self, flags: &mut RenderDebugFlags) {
        match self {
            Self::ToggleFloor => flags.render_floor = !flags.render_floor,
            Self::ToggleWalls => flags.render_walls = !flags.render_walls,
            Self::ToggleEntities => flags.render_entities = !flags.render_entities,
            Self::ToggleOverlay => flags.show_debug_overlay = !flags.show_debug_overlay,
            Self::ToggleFocusRender => flags.focus_render_2x2 = !flags.focus_render_2x2,
            Self::ToggleFocusLog => flags.log_render_focus = !flags.log_render_focus,
            Self::ResetAll => *flags = RenderDebugFlags::all_enabled(),
            Self::FloorOnly => *flags = RenderDebugFlags::floor_only(),
            Self::ToggleToon => flags.toon_filter = !flags.toon_filter,
            Self::ToggleWalkCamera => flags.walking_camera_offset = !flags.walking_camera_offset,
            Self::ToggleWalkTrace => flags.log_walk_trace = !flags.log_walk_trace,
            Self::ToggleWallPredraw => flags.wall_predraw = !flags.wall_predraw,
            Self::ToggleMaskAwareDrawCell => {
                flags.mask_aware_draw_cell = !flags.mask_aware_draw_cell
            }
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::ToggleFloor => "Floor layer",
            Self::ToggleWalls => "Wall layer",
            Self::ToggleEntities => "Entity layer",
            Self::ToggleOverlay => "Debug overlay",
            Self::ToggleFocusRender => "Focus render",
            Self::ToggleFocusLog => "Focus render log",
            Self::ResetAll => "All render layers",
            Self::FloorOnly => "Floor-only mode",
            Self::ToggleToon => "Toon filter",
            Self::ToggleWalkCamera => "Walk camera offset",
            Self::ToggleWalkTrace => "Walk trace",
            Self::ToggleWallPredraw => "Wall pre-draw",
            Self::ToggleMaskAwareDrawCell => "Mask-aware DrawCell",
        }
    }

    fn state(self, flags: &RenderDebugFlags) -> &'static str {
        match self {
            Self::ToggleFloor => on_off(flags.render_floor),
            Self::ToggleWalls => on_off(flags.render_walls),
            Self::ToggleEntities => on_off(flags.render_entities),
            Self::ToggleOverlay => on_off(flags.show_debug_overlay),
            Self::ToggleFocusRender => on_off(flags.focus_render_2x2),
            Self::ToggleFocusLog => on_off(flags.log_render_focus),
            Self::ResetAll => "reset",
            Self::FloorOnly => "enabled",
            Self::ToggleToon => on_off(flags.toon_filter),
            Self::ToggleWalkCamera => on_off(flags.walking_camera_offset),
            Self::ToggleWalkTrace => on_off(flags.log_walk_trace),
            Self::ToggleWallPredraw => on_off(flags.wall_predraw),
            Self::ToggleMaskAwareDrawCell => on_off(flags.mask_aware_draw_cell),
        }
    }
}

fn on_off(value: bool) -> &'static str {
    if value {
        "ON"
    } else {
        "OFF"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_render_layers_from_hotkeys() {
        let mut flags = RenderDebugFlags::all_enabled();

        assert!(handle_render_debug_key(&mut flags, Keycode::F4, Mod::NOMOD));
        assert!(!flags.render_floor);

        assert!(handle_render_debug_key(&mut flags, Keycode::F5, Mod::NOMOD));
        assert!(!flags.render_walls);

        assert!(handle_render_debug_key(&mut flags, Keycode::F6, Mod::NOMOD));
        assert!(!flags.render_entities);
    }

    #[test]
    fn honors_modifier_specific_hotkeys() {
        let mut flags = RenderDebugFlags::all_enabled();

        assert!(handle_render_debug_key(
            &mut flags,
            Keycode::F7,
            Mod::LSHIFTMOD
        ));
        assert!(flags.focus_render_2x2);
        assert!(!flags.show_debug_overlay);

        assert!(handle_render_debug_key(
            &mut flags,
            Keycode::F7,
            Mod::LCTRLMOD
        ));
        assert!(flags.log_render_focus);
    }

    #[test]
    fn ignores_non_devtools_keys() {
        let mut flags = RenderDebugFlags::all_enabled();

        assert!(!handle_render_debug_key(&mut flags, Keycode::A, Mod::NOMOD));
        assert!(flags.render_floor);
        assert!(flags.render_walls);
    }
}
