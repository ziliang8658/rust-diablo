use super::{RenderPass, RenderPassPipeline, RenderPlan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPolicy {
    plan: RenderPlan,
}

impl RenderPolicy {
    pub const fn from_plan(plan: RenderPlan) -> Self {
        Self { plan }
    }

    pub const fn plan(self) -> RenderPlan {
        self.plan
    }

    pub fn pipeline(self, scene: RenderSceneContext) -> RenderPassPipeline {
        let mut pipeline = RenderPassPipeline::empty();

        if scene.has_dungeon_map() {
            if self.plan.render_floor() {
                pipeline.push(RenderPass::DungeonFloor);
            }
            if self.plan.render_walls() {
                pipeline.push(RenderPass::DungeonWallContent);
            }
        }

        if self.plan.render_entities() && (!scene.has_dungeon_map() || !self.plan.render_walls()) {
            pipeline.push(RenderPass::EntityOverlay);
        }

        pipeline
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSceneContext {
    has_dungeon_map: bool,
}

impl RenderSceneContext {
    pub const fn new(has_dungeon_map: bool) -> Self {
        Self { has_dungeon_map }
    }

    pub const fn has_dungeon_map(self) -> bool {
        self.has_dungeon_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{
        DrawCellMode, PostEffects, RenderCameraMode, RenderLayers, RenderRegionMode,
        RenderTraceMode, WallOcclusionMode,
    };

    fn plan(floor: bool, walls: bool, entities: bool) -> RenderPlan {
        RenderPlan::new(
            RenderLayers::new(floor, walls, entities),
            RenderRegionMode::Full,
            RenderTraceMode::Off,
            RenderCameraMode::Fixed,
            WallOcclusionMode::DiabloPredraw,
            DrawCellMode::DiabloMaskAware,
            PostEffects::none(),
        )
    }

    #[test]
    fn default_policy_runs_floor_pass_for_dungeon_scene() {
        let policy = RenderPolicy::from_plan(RenderPlan::default());
        let passes: Vec<_> = policy
            .pipeline(RenderSceneContext::new(true))
            .iter()
            .collect();

        assert_eq!(passes, vec![RenderPass::DungeonFloor]);
    }

    #[test]
    fn policy_embeds_entities_in_wall_pass_when_walls_are_enabled() {
        let policy = RenderPolicy::from_plan(plan(true, true, true));
        let passes: Vec<_> = policy
            .pipeline(RenderSceneContext::new(true))
            .iter()
            .collect();

        assert_eq!(
            passes,
            vec![RenderPass::DungeonFloor, RenderPass::DungeonWallContent]
        );
    }

    #[test]
    fn policy_uses_overlay_entities_without_wall_occlusion() {
        let policy = RenderPolicy::from_plan(plan(true, false, true));
        let passes: Vec<_> = policy
            .pipeline(RenderSceneContext::new(true))
            .iter()
            .collect();

        assert_eq!(
            passes,
            vec![RenderPass::DungeonFloor, RenderPass::EntityOverlay]
        );
    }

    #[test]
    fn policy_keeps_entities_available_without_dungeon_map() {
        let policy = RenderPolicy::from_plan(plan(true, true, true));
        let passes: Vec<_> = policy
            .pipeline(RenderSceneContext::new(false))
            .iter()
            .collect();

        assert_eq!(passes, vec![RenderPass::EntityOverlay]);
    }
}
