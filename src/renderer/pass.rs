#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPass {
    DungeonFloor,
    DungeonWallContent,
    EntityOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPassPipeline {
    passes: [Option<RenderPass>; Self::MAX_PASSES],
    len: usize,
}

impl RenderPassPipeline {
    const MAX_PASSES: usize = 3;

    pub const fn empty() -> Self {
        Self {
            passes: [None, None, None],
            len: 0,
        }
    }

    pub fn push(&mut self, pass: RenderPass) {
        debug_assert!(self.len < Self::MAX_PASSES);
        if self.len >= Self::MAX_PASSES {
            return;
        }

        self.passes[self.len] = Some(pass);
        self.len += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = RenderPass> + '_ {
        self.passes[..self.len].iter().filter_map(|pass| *pass)
    }

    pub fn contains(&self, pass: RenderPass) -> bool {
        self.iter().any(|candidate| candidate == pass)
    }

    pub fn requires_dungeon_scan(&self) -> bool {
        self.contains(RenderPass::DungeonFloor) || self.contains(RenderPass::DungeonWallContent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_preserves_pass_order() {
        let mut pipeline = RenderPassPipeline::empty();

        pipeline.push(RenderPass::DungeonFloor);
        pipeline.push(RenderPass::DungeonWallContent);
        pipeline.push(RenderPass::EntityOverlay);

        let passes: Vec<_> = pipeline.iter().collect();
        assert_eq!(
            passes,
            vec![
                RenderPass::DungeonFloor,
                RenderPass::DungeonWallContent,
                RenderPass::EntityOverlay,
            ]
        );
    }
}
