/// Animation - Frame-based animation system
///
/// Manages sprite sheet animations with multiple frames
use crate::engine::Direction;
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
        if self.frames.is_empty() {
            return false;
        }

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

    /// Total animation duration in seconds.
    pub fn duration(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.frames.len() as f32 * self.frame_duration
    }

    /// Normalized progress through the animation in the range 0.0..=1.0.
    pub fn progress(&self) -> f32 {
        let total_duration = self.duration();
        if total_duration <= 0.0 {
            return 0.0;
        }
        if self.finished && !self.looping {
            return 1.0;
        }

        let current_duration =
            self.current_frame as f32 * self.frame_duration + self.elapsed.min(self.frame_duration);
        (current_duration / total_duration).clamp(0.0, 1.0)
    }

    /// Get the current frame rectangle
    pub fn current_frame_rect(&self) -> Rect {
        if self.frames.is_empty() {
            return Rect::new(0, 0, 0, 0);
        }
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
    animations: HashMap<(AnimationState, Direction), Animation>,
    current_state: AnimationState,
    current_direction: Direction,
    previous_state: AnimationState,
    previous_direction: Direction,
}

impl AnimationController {
    /// Create a new animation controller
    pub fn new() -> Self {
        Self {
            animations: std::collections::HashMap::new(),
            current_state: AnimationState::Idle,
            current_direction: Direction::South,
            previous_state: AnimationState::Idle,
            previous_direction: Direction::South,
        }
    }

    /// Add a directional animation state.
    pub fn add_directional_animation(
        &mut self,
        state: AnimationState,
        direction: Direction,
        animation: Animation,
    ) {
        self.animations.insert((state, direction), animation);
    }

    /// Set current animation state
    ///
    /// If the state is different from current, it will switch to the new state
    /// and reset the animation.
    pub fn set_state(&mut self, state: AnimationState) {
        self.set_state_direction(state, self.current_direction);
    }

    /// Set current animation state and direction together.
    pub fn set_state_direction(&mut self, state: AnimationState, direction: Direction) {
        if state != self.current_state || direction != self.current_direction {
            self.previous_state = self.current_state;
            self.previous_direction = self.current_direction;
            self.current_state = state;
            self.current_direction = direction;
            if let Some(anim) = self.current_animation_mut() {
                anim.reset();
            }
        }
    }

    /// Set only the current direction while keeping the current state.
    pub fn set_direction(&mut self, direction: Direction) {
        if direction != self.current_direction {
            self.previous_direction = self.current_direction;
            self.current_direction = direction;
            if let Some(anim) = self.current_animation_mut() {
                anim.reset();
            }
        }
    }

    /// Get previous animation state
    pub fn previous_state(&self) -> AnimationState {
        self.previous_state
    }

    /// Get previous animation direction.
    pub fn previous_direction(&self) -> Direction {
        self.previous_direction
    }

    /// Get current animation state
    pub fn current_state(&self) -> AnimationState {
        self.current_state
    }

    /// Get current animation direction.
    pub fn current_direction(&self) -> Direction {
        self.current_direction
    }

    /// Get the current animation duration.
    pub fn current_animation_duration(&self) -> Option<f32> {
        self.current_animation().map(|anim| anim.duration())
    }

    /// Set frame duration for every direction in one animation state.
    pub fn set_frame_duration_for_state(&mut self, state: AnimationState, duration: f32) {
        for ((animation_state, _), animation) in self.animations.iter_mut() {
            if *animation_state == state {
                animation.set_frame_duration(duration);
            }
        }
    }

    /// Get the current animation progress.
    pub fn current_animation_progress(&self) -> Option<f32> {
        self.current_animation().map(|anim| anim.progress())
    }

    fn resolve_key(&self) -> Option<(AnimationState, Direction)> {
        let exact = (self.current_state, self.current_direction);
        if self.animations.contains_key(&exact) {
            return Some(exact);
        }

        let south = (self.current_state, Direction::South);
        if self.animations.contains_key(&south) {
            return Some(south);
        }

        self.animations
            .keys()
            .copied()
            .find(|(state, _)| *state == self.current_state)
    }

    fn current_animation(&self) -> Option<&Animation> {
        let key = self.resolve_key()?;
        self.animations.get(&key)
    }

    fn current_animation_mut(&mut self) -> Option<&mut Animation> {
        let key = self.resolve_key()?;
        self.animations.get_mut(&key)
    }

    /// Get current frame index of the current animation
    ///
    /// Returns the frame index (0-based) of the currently playing animation.
    /// Useful for synchronizing visual effects with animation frames.
    pub fn current_frame_index(&self) -> Option<usize> {
        self.current_animation().map(|anim| anim.current_frame)
    }

    /// Update current animation
    pub fn update(&mut self, dt: f32) {
        if let Some(anim) = self.current_animation_mut() {
            anim.update(dt);
        }
    }

    /// Get current frame rectangle
    pub fn current_frame_rect(&self) -> Option<Rect> {
        self.current_animation()
            .map(|anim| anim.current_frame_rect())
    }

    /// Check if current animation is finished
    pub fn is_finished(&self) -> bool {
        self.current_animation()
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

    #[test]
    fn test_animation_progress() {
        let frames = vec![
            Rect::new(0, 0, 10, 10),
            Rect::new(10, 0, 10, 10),
            Rect::new(20, 0, 10, 10),
            Rect::new(30, 0, 10, 10),
        ];
        let mut anim = Animation::new(frames, 0.25, false);
        assert_eq!(anim.progress(), 0.0);

        anim.update(0.125);
        assert!(anim.progress() > 0.0);

        anim.update(1.0);
        assert!(anim.is_finished());
        assert_eq!(anim.progress(), 1.0);
    }
}
