pub mod camera;
/// Renderer module - Rendering system
///
/// This module provides rendering capabilities:
/// - Color definitions
/// - Drawing primitives (shapes, lines)
/// - Camera system
pub mod color;
pub mod pass;
pub mod plan;
pub mod policy;

pub use camera::Camera;
pub use color::Color;
pub use pass::{RenderPass, RenderPassPipeline};
pub use plan::{
    DrawCellMode, PostEffects, RenderCameraMode, RenderLayers, RenderPlan, RenderRegionMode,
    RenderTraceMode, WallOcclusionMode,
};
pub use policy::{RenderPolicy, RenderSceneContext};
