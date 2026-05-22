#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPlan {
    layers: RenderLayers,
    region: RenderRegionMode,
    trace: RenderTraceMode,
    camera: RenderCameraMode,
    wall_occlusion: WallOcclusionMode,
    draw_cell: DrawCellMode,
    post_effects: PostEffects,
}

impl Default for RenderPlan {
    fn default() -> Self {
        Self {
            layers: RenderLayers::floor_only(),
            region: RenderRegionMode::Full,
            trace: RenderTraceMode::Off,
            camera: RenderCameraMode::Fixed,
            wall_occlusion: WallOcclusionMode::DiabloPredraw,
            draw_cell: DrawCellMode::DiabloMaskAware,
            post_effects: PostEffects::none(),
        }
    }
}

impl RenderPlan {
    pub const fn new(
        layers: RenderLayers,
        region: RenderRegionMode,
        trace: RenderTraceMode,
        camera: RenderCameraMode,
        wall_occlusion: WallOcclusionMode,
        draw_cell: DrawCellMode,
        post_effects: PostEffects,
    ) -> Self {
        Self {
            layers,
            region,
            trace,
            camera,
            wall_occlusion,
            draw_cell,
            post_effects,
        }
    }

    pub const fn render_floor(self) -> bool {
        self.layers.floor
    }

    pub const fn render_walls(self) -> bool {
        self.layers.walls
    }

    pub const fn render_entities(self) -> bool {
        self.layers.entities
    }

    pub const fn focus_2x2(self) -> bool {
        matches!(self.region, RenderRegionMode::Focus2x2)
    }

    pub const fn trace_focus(self) -> bool {
        matches!(self.trace, RenderTraceMode::Focus)
    }

    pub const fn uses_walking_camera_offset(self) -> bool {
        matches!(self.camera, RenderCameraMode::WalkingOffset)
    }

    pub const fn camera_label(self) -> &'static str {
        match self.camera {
            RenderCameraMode::Fixed => "OFF",
            RenderCameraMode::WalkingOffset => "ON",
        }
    }

    pub const fn uses_wall_predraw(self) -> bool {
        matches!(self.wall_occlusion, WallOcclusionMode::DiabloPredraw)
    }

    pub const fn mask_aware_draw_cell(self) -> bool {
        matches!(self.draw_cell, DrawCellMode::DiabloMaskAware)
    }

    pub const fn toon_filter(self) -> bool {
        self.post_effects.toon_filter
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderLayers {
    floor: bool,
    walls: bool,
    entities: bool,
}

impl RenderLayers {
    pub const fn new(floor: bool, walls: bool, entities: bool) -> Self {
        Self {
            floor,
            walls,
            entities,
        }
    }

    pub const fn floor_only() -> Self {
        Self::new(true, false, false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderRegionMode {
    Full,
    Focus2x2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTraceMode {
    Off,
    Focus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderCameraMode {
    Fixed,
    WalkingOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallOcclusionMode {
    Simple,
    DiabloPredraw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawCellMode {
    RawUpload,
    DiabloMaskAware,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostEffects {
    toon_filter: bool,
}

impl PostEffects {
    pub const fn new(toon_filter: bool) -> Self {
        Self { toon_filter }
    }

    pub const fn none() -> Self {
        Self { toon_filter: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_plan_matches_learning_build_render_start() {
        let plan = RenderPlan::default();

        assert!(plan.render_floor());
        assert!(!plan.render_walls());
        assert!(!plan.render_entities());
        assert!(plan.mask_aware_draw_cell());
        assert!(plan.uses_wall_predraw());
    }
}
