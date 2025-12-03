/// Animation - Frame-based animation system
///
/// Manages sprite sheet animations with multiple frames
use crate::math::Rect;
use std::collections::HashMap;

/// Animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationState {
    Idle,   // Standing still
    Walk,   // Walking
    Attack, // Attacking
    Hit,    // Being hit
    Death,  // Death animation
    Cast,   // Casting spell (future)
}

/// Animation - Frame-based animation
#[derive(Clone)]
pub struct Animation {
    /// List of frame rectangles in the sprite sheet
    pub frames: Vec<Rect>,
    /// Current frame index
    pub current_frame: usize,
    /// Duration of each frame in seconds
    pub frame_duration: f32,
    /// Time elapsed in current frame
    elapsed: f32,
    /// Whether animation loops
    pub looping: bool,
    /// Whether animation is finished (for non-looping)
    pub finished: bool,
}

impl Animation {
    /// Create a new animation
    pub fn new(frames: Vec<Rect>, frame_duration: f32, looping: bool) -> Self {
        Self {
            frames,
            current_frame: 0,
            frame_duration,
            elapsed: 0.0,
            looping,
            finished: false,
        }
    }

    /// Create a single-frame "animation" (static sprite)
    pub fn single_frame(frame: Rect) -> Self {
        Self::new(vec![frame], 1.0, true)
    }

    /// Update animation with delta time
    ///
    /// Returns true if the frame changed
    pub fn update(&mut self, dt: f32) -> bool {
        if self.finished && !self.looping {
            return false;
        }

        self.elapsed += dt;
        let mut frame_changed = false;

        while self.elapsed >= self.frame_duration {
            self.elapsed -= self.frame_duration;
            let old_frame = self.current_frame;
            self.current_frame += 1;

            if self.current_frame >= self.frames.len() {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                    self.finished = true;
                }
            }

            frame_changed = old_frame != self.current_frame;
        }

        frame_changed
    }

    /// Set frame duration
    pub fn set_frame_duration(&mut self, duration: f32) {
        self.frame_duration = duration;
    }

    /// Check if animation is finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Get the current frame rectangle
    pub fn current_frame_rect(&self) -> Rect {
        self.frames[self.current_frame]
    }

    /// Reset animation to first frame
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.finished = false;
    }
}

/// AnimationController - Manages multiple animation states
#[derive(Clone)]
pub struct AnimationController {
    animations: HashMap<AnimationState, Animation>,
    current_state: AnimationState,
    previous_state: AnimationState,
}

impl AnimationController {
    /// Create a new animation controller
    pub fn new() -> Self {
        Self {
            animations: std::collections::HashMap::new(),
            current_state: AnimationState::Idle,
            previous_state: AnimationState::Idle,
        }
    }

    /// Add an animation state
    pub fn add_animation(&mut self, state: AnimationState, animation: Animation) {
        self.animations.insert(state, animation);
    }

    /// Set current animation state
    ///
    /// If the state is different from current, it will switch to the new state
    /// and reset the animation.
    pub fn set_state(&mut self, state: AnimationState) {
        if state != self.current_state {
            self.previous_state = self.current_state;
            self.current_state = state;
            if let Some(anim) = self.animations.get_mut(&state) {
                anim.reset();
            }
        }
    }

    /// Get previous animation state
    pub fn previous_state(&self) -> AnimationState {
        self.previous_state
    }

    /// Get current animation state
    pub fn current_state(&self) -> AnimationState {
        self.current_state
    }

    /// Get current frame index of the current animation
    ///
    /// Returns the frame index (0-based) of the currently playing animation.
    /// Useful for synchronizing visual effects with animation frames.
    pub fn current_frame_index(&self) -> Option<usize> {
        self.animations
            .get(&self.current_state)
            .map(|anim| anim.current_frame)
    }

    /// Update current animation
    pub fn update(&mut self, dt: f32) {
        if let Some(anim) = self.animations.get_mut(&self.current_state) {
            anim.update(dt);
        }
    }

    /// Get current frame rectangle
    pub fn current_frame_rect(&self) -> Option<Rect> {
        self.animations
            .get(&self.current_state)
            .map(|anim| anim.current_frame_rect())
    }

    /// Check if current animation is finished
    pub fn is_finished(&self) -> bool {
        self.animations
            .get(&self.current_state)
            .map(|anim| anim.is_finished())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_update() {
        let frames = vec![
            Rect {
                x: 0,
                y: 0,
                width: 32,
                height: 32,
            },
            Rect {
                x: 32,
                y: 0,
                width: 32,
                height: 32,
            },
        ];
        let mut anim = Animation::new(frames, 0.1, true);

        // 初始帧应该是 0
        assert_eq!(anim.current_frame, 0);

        // 更新但不足一帧时间
        assert!(!anim.update(0.05));
        assert_eq!(anim.current_frame, 0);

        // 更新超过一帧时间
        assert!(anim.update(0.06));
        assert_eq!(anim.current_frame, 1);

        // 循环回到开始
        assert!(anim.update(0.1));
        assert_eq!(anim.current_frame, 0);
    }

    #[test]
    fn test_animation_non_looping() {
        let frames = vec![
            Rect {
                x: 0,
                y: 0,
                width: 32,
                height: 32,
            },
            Rect {
                x: 32,
                y: 0,
                width: 32,
                height: 32,
            },
        ];
        let mut anim = Animation::new(frames, 0.1, false);

        anim.update(0.1);
        assert_eq!(anim.current_frame, 1);
        assert!(!anim.is_finished());

        anim.update(0.1);
        assert_eq!(anim.current_frame, 1);
        assert!(anim.is_finished());

        // 完成后不再更新
        assert!(!anim.update(0.1));
    }

    #[test]
    fn test_animation_controller_state_switch() {
        let mut controller = AnimationController::new();

        let idle_anim = Animation::new(
            vec![Rect {
                x: 0,
                y: 0,
                width: 32,
                height: 32,
            }],
            0.1,
            true,
        );
        let walk_anim = Animation::new(
            vec![
                Rect {
                    x: 0,
                    y: 32,
                    width: 32,
                    height: 32,
                },
                Rect {
                    x: 32,
                    y: 32,
                    width: 32,
                    height: 32,
                },
            ],
            0.1,
            true,
        );

        controller.add_animation(AnimationState::Idle, idle_anim);
        controller.add_animation(AnimationState::Walk, walk_anim);

        // 初始状态应该是 Idle
        assert_eq!(controller.current_state(), AnimationState::Idle);

        // 切换到 Walk
        controller.set_state(AnimationState::Walk);
        assert_eq!(controller.current_state(), AnimationState::Walk);
        assert_eq!(controller.previous_state(), AnimationState::Idle);

        // 重复设置同一状态不应该改变 previous_state
        controller.set_state(AnimationState::Walk);
        assert_eq!(controller.previous_state(), AnimationState::Idle);
    }

    #[test]
    fn test_animation_reset() {
        let frames = vec![
            Rect {
                x: 0,
                y: 0,
                width: 32,
                height: 32,
            },
            Rect {
                x: 32,
                y: 0,
                width: 32,
                height: 32,
            },
        ];
        let mut anim = Animation::new(frames, 0.1, true);

        anim.update(0.1);
        assert_eq!(anim.current_frame, 1);

        anim.reset();
        assert_eq!(anim.current_frame, 0);
        assert_eq!(anim.elapsed, 0.0);
        assert!(!anim.finished);
    }
}
