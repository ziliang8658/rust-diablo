use crate::engine::Direction;
/// Entity module - Game entity system
///
/// This module provides a basic entity system for game objects.
/// Each entity has a position, size, and visual properties.
use crate::math::{Point, Rect};
use crate::renderer::Color;
use crate::sprite::{Animation, AnimationController, AnimationState};

/// Entity type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Player,
    Monster,
    Item,
    Object,
}

/// Entity - Basic game object
///
/// Represents any game object that has a position and can be rendered.
/// Uses tile-based position system (like C++ ActorPosition.tile)
#[derive(Clone)]
pub struct Entity {
    pub entity_type: EntityType,
    /// Tile position in world coordinates (like C++ position.tile)
    pub tile_position: Point,
    pub size: (u32, u32),
    pub color: Color,
    pub use_sprite: bool,
    pub sprite_id: Option<String>,
    pub animation: Option<AnimationController>,

    // Tile-based movement fields
    pub direction: Direction,              // 当前方向 / 朝向
    pub walking: bool,                     // 是否正在行走
    pub walk_direction: Option<Direction>, // 当前行走方向（如果正在行走）
    pub walk_start_tile: Option<Point>,
    pub walk_target_tile: Option<Point>,
    pub walk_progress: f32,
    pub walk_duration: f32,
}

impl Entity {
    const DEFAULT_WALK_DURATION: f32 = 0.15;

    /// Create a new entity
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity
    /// * `tile_position` - Initial tile position in world coordinates
    /// * `size` - Entity size in pixels (for rendering)
    /// * `color` - Entity color
    pub fn new(
        entity_type: EntityType,
        tile_position: Point,
        size: (u32, u32),
        color: Color,
    ) -> Self {
        Self {
            entity_type,
            tile_position,
            size,
            color,
            use_sprite: false,
            sprite_id: None,
            animation: None,
            direction: Direction::None,
            walking: false,
            walk_direction: None,
            walk_start_tile: None,
            walk_target_tile: None,
            walk_progress: 0.0,
            walk_duration: 0.0,
        }
    }

    /// Create a player entity
    ///
    /// # Diablo 1 玩家精灵格式
    ///
    /// **资源路径：**
    /// - Idle: `plrgfx/warrior/wmn/wmnas.cl2` (Warrior Male uNarmed Stand)
    /// - Walk: `plrgfx/warrior/wmn/wmnaw.cl2` (Warrior Male uNarmed Walk)
    ///
    /// **CL2 格式说明：**
    /// - CL2 是 Diablo 1 的精灵格式，使用 RLE 压缩
    /// - 每个 CL2 文件包含多个方向的动画帧（通常是 8 方向）
    /// - 帧宽度：96 像素（通过 frame_width 参数指定）
    /// - 帧高度：通常为 96-128 像素（根据动画类型不同）
    ///
    /// **纹理 ID 格式：**
    /// - Idle: `warrior_town_idle_0`, `warrior_town_idle_1`, ...
    /// - Walk: `warrior_town_walk_0`, `warrior_town_walk_1`, ...
    ///
    /// **动画系统：**
    /// - 当前实现：简单的帧动画（所有帧顺序播放）
    /// - 原版 Diablo：8 方向动画（根据玩家面向选择帧序列）
    /// - TODO: 后续需要实现 8 方向动画系统
    ///
    /// # 参考代码
    /// - `Source/player.h:82-94` - player_graphic 枚举
    /// - `Source/playerdat.hpp:158-213` - PlayerAnimData 结构
    /// - `Source/playerdat.hpp:129-156` - PlayerSpriteData 结构
    /// - `game.rs:467-490` - 实际的精灵加载代码
    pub fn create_player(tile_position: Point) -> Self {
        let mut entity = Self::new(
            EntityType::Player,
            tile_position,
            (96, 96), // Diablo 1 warrior sprite: 96x96 pixels (frame_width from CL2)
            Color::CYAN,
        );

        entity.direction = Direction::South;

        // Enable sprite rendering
        entity.use_sprite = true;
        entity.sprite_id = Some("warrior_town".to_string()); // 匹配 resource_manager 中的加载

        // Initialize animation controller with placeholder frames
        //
        // ⚠️ 重要：这里的 frames 只是【占位符】！
        //
        // **加载流程：**
        // 1. [此处] Entity 创建时 → 使用单帧占位符（因为 MPQ 还未加载）
        // 2. [game.rs] 运行时加载 CL2 精灵 → 获得实际帧数（idle_frame_count, walk_frame_count）
        // 3. [game.rs:370-392] 重新配置动画 → 根据实际帧数重建 Animation
        //
        // 这样设计的原因：
        // - Entity 创建时还不知道实际帧数（资源未加载）
        // - 解耦：Entity 不依赖资源加载系统
        // - 运行时才能确定：只有解析 CL2 文件后才知道有多少帧
        let mut anim_controller = AnimationController::new();

        // Idle animation - Warrior standing (从 wmnas.cl2 加载)
        // 原版数据：idleFrames 通常为 8-10 帧（根据职业不同）
        // 实际配置：见 game.rs:372-377
        for direction in Direction::walk_animation_order() {
            let idle_anim = Animation::new(
                vec![Rect::new(0, 0, 96, 96)], // 【占位符】：单帧，运行时会被替换
                0.15,                          // Frame duration: 0.15s per frame
                true,                          // Loop
            );
            anim_controller.add_directional_animation(AnimationState::Idle, direction, idle_anim);

            // Walk animation - Warrior walking (从 wmnaw.cl2 加载)
            // 原版数据：walkingFrames 为 8 帧（每个方向）
            // 实际配置：见 game.rs:379-387
            let walk_anim = Animation::new(
                vec![Rect::new(0, 0, 96, 96)], // 【占位符】：单帧，运行时会被替换
                0.1,                           // Frame duration: 0.1s = 10 FPS (walking speed)
                false,                         // Walk should finish after one step
            );
            anim_controller.add_directional_animation(AnimationState::Walk, direction, walk_anim);
        }

        // Set initial state to Idle facing south.
        anim_controller.set_state_direction(AnimationState::Idle, Direction::South);
        entity.animation = Some(anim_controller);

        entity
    }

    /// Get the bounding rectangle for this entity in world pixel coordinates
    ///
    /// Note: This converts tile position to pixel position for rendering.
    /// The actual position is stored as tile coordinates.
    pub fn bounds(&self, tile_size: u32) -> Rect {
        let pixel_x = self.tile_position.x * tile_size as i32;
        let pixel_y = self.tile_position.y * tile_size as i32;
        Rect::from_center(Point::new(pixel_x, pixel_y), self.size.0, self.size.1)
    }

    /// Update entity (tile-based movement system)
    ///
    /// Similar to C++ DoWalk: checks if walking animation is complete,
    /// and if so, updates tile_position to the new tile.
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds
    /// * `is_walkable_fn` - Optional function to check if a tile is walkable (world_x, world_y) -> bool
    pub fn update<F>(&mut self, dt: f32, is_walkable_fn: Option<F>)
    where
        F: Fn(i32, i32) -> bool,
    {
        // Update animation
        if let Some(ref mut anim) = self.animation {
            anim.update(dt);
            if self.walking {
                if let Some(progress) = anim.current_animation_progress() {
                    self.walk_progress = progress;
                } else if self.walk_duration > 0.0 {
                    self.walk_progress =
                        (self.walk_progress + dt / self.walk_duration).clamp(0.0, 1.0);
                }
            }
        }

        // If walking, check if animation is complete
        if self.walking {
            if let Some(target_tile) = self.walk_target_tile {
                let can_move = if let Some(check_fn) = &is_walkable_fn {
                    check_fn(target_tile.x, target_tile.y)
                } else {
                    true
                };

                let animation_done = self
                    .animation
                    .as_ref()
                    .map(|anim| anim.is_finished())
                    .unwrap_or(false);

                if can_move && (animation_done || self.walk_progress >= 1.0) {
                    self.tile_position = target_tile;
                    self.walking = false;
                    self.walk_direction = None;
                    self.walk_start_tile = None;
                    self.walk_target_tile = None;
                    self.walk_progress = 0.0;
                    self.walk_duration = 0.0;

                    if let Some(ref mut anim) = self.animation {
                        anim.set_state_direction(AnimationState::Idle, self.direction);
                    }
                } else if !can_move {
                    // Collision detected, cancel the walk and return to idle.
                    self.walking = false;
                    self.walk_direction = None;
                    self.walk_start_tile = None;
                    self.walk_target_tile = None;
                    self.walk_progress = 0.0;
                    self.walk_duration = 0.0;

                    if let Some(ref mut anim) = self.animation {
                        anim.set_state_direction(AnimationState::Idle, self.direction);
                    }
                }
            }
        }
    }

    /// Calculate target tile for a given direction
    fn calculate_target_tile(&self, direction: Direction) -> Point {
        let (dx, dy) = direction.to_tile_offset();
        Point::new(self.tile_position.x + dx, self.tile_position.y + dy)
    }

    /// Move entity by tile offset
    pub fn move_by_tile(&mut self, delta: Point) {
        self.tile_position = self.tile_position + delta;
    }

    /// Check if this entity collides with another (tile-based)
    pub fn collides_with_tile(&self, other: &Entity) -> bool {
        self.tile_position == other.tile_position
    }

    /// Start walking in a direction (like C++ HandleWalkMode)
    ///
    /// # Arguments
    /// * `direction` - Direction to walk
    /// * `is_walkable_fn` - Optional function to check if a tile is walkable (world_x, world_y) -> bool
    ///
    /// # Returns
    /// `true` if walk started successfully, `false` if target tile is not walkable
    pub fn start_walk<F>(&mut self, direction: Direction, is_walkable_fn: Option<F>) -> bool
    where
        F: Fn(i32, i32) -> bool,
    {
        // Don't start new walk if already walking
        if self.walking || !direction.is_moving() {
            return false;
        }

        // Calculate target tile
        let start_tile = self.tile_position;
        let target_tile = self.calculate_target_tile(direction);

        // Check if target tile is walkable
        let can_walk = if let Some(check_fn) = &is_walkable_fn {
            check_fn(target_tile.x, target_tile.y)
        } else {
            true // No collision check, allow movement
        };

        if !can_walk {
            return false;
        }

        // Start walking
        self.direction = direction;
        self.walking = true;
        self.walk_direction = Some(direction);
        self.walk_start_tile = Some(start_tile);
        self.walk_target_tile = Some(target_tile);
        self.walk_progress = 0.0;

        // Switch to walk animation
        if let Some(ref mut anim) = self.animation {
            anim.set_state_direction(AnimationState::Walk, direction);
            self.walk_duration = anim
                .current_animation_duration()
                .unwrap_or(Self::DEFAULT_WALK_DURATION);
        } else {
            self.walk_duration = Self::DEFAULT_WALK_DURATION;
        }

        true
    }

    /// Check if entity can walk in a direction
    pub fn can_walk<F>(&self, direction: Direction, is_walkable_fn: Option<F>) -> bool
    where
        F: Fn(i32, i32) -> bool,
    {
        let target_tile = self.calculate_target_tile(direction);
        if let Some(check_fn) = &is_walkable_fn {
            check_fn(target_tile.x, target_tile.y)
        } else {
            true
        }
    }

    /// Set direction (for facing, not movement)
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
        if !self.walking {
            if let Some(ref mut anim) = self.animation {
                anim.set_state_direction(AnimationState::Idle, direction);
            }
        }
    }

    /// Return the direction that should be used for rendering.
    pub fn render_direction(&self) -> Direction {
        if self.walking {
            let direction = self.walk_direction.unwrap_or(self.direction);
            if direction.is_moving() {
                direction
            } else {
                Direction::South
            }
        } else {
            if self.direction.is_moving() {
                self.direction
            } else {
                Direction::South
            }
        }
    }

    /// Return the current walking progress in the range 0.0..=1.0.
    pub fn walking_progress(&self) -> f32 {
        if self.walking {
            self.walk_progress.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Adjust walk animation timing for live movement tuning.
    pub fn set_walk_frame_duration(&mut self, duration: f32) {
        if let Some(ref mut anim) = self.animation {
            anim.set_frame_duration_for_state(AnimationState::Walk, duration);
            if self.walking {
                self.walk_duration = anim
                    .current_animation_duration()
                    .unwrap_or(Self::DEFAULT_WALK_DURATION);
            }
        }
    }

    /// Return the original Diablo-style isometric walking offset.
    pub fn walking_render_offset(&self) -> Point {
        if !self.walking {
            return Point::zero();
        }
        let direction = self.render_direction();
        direction.walking_render_offset(self.walking_progress())
    }

    /// Return a simple 2D walking offset for flat previews or town scenes.
    pub fn walking_pixel_offset(&self, tile_size: i32) -> Point {
        if !self.walking {
            return Point::zero();
        }
        let direction = self.render_direction();
        direction.walking_pixel_offset(self.walking_progress(), tile_size)
    }

    /// Cancel any in-progress walk and return to idle.
    pub fn cancel_walk(&mut self) {
        self.walking = false;
        self.walk_direction = None;
        self.walk_start_tile = None;
        self.walk_target_tile = None;
        self.walk_progress = 0.0;
        self.walk_duration = 0.0;

        if let Some(ref mut anim) = self.animation {
            anim.set_state_direction(AnimationState::Idle, self.direction);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_walk_records_step_without_committing_tile() {
        let mut player = Entity::create_player(Point::new(10, 10));

        assert!(player.start_walk::<fn(i32, i32) -> bool>(Direction::East, None));

        assert!(player.walking);
        assert_eq!(player.direction, Direction::East);
        assert_eq!(player.walk_direction, Some(Direction::East));
        assert_eq!(player.walk_start_tile, Some(Point::new(10, 10)));
        assert_eq!(player.walk_target_tile, Some(Point::new(11, 9)));
        assert_eq!(player.tile_position, Point::new(10, 10));
        assert_eq!(player.walking_progress(), 0.0);
    }

    #[test]
    fn test_start_walk_collision_failure_does_not_change_state() {
        let mut player = Entity::create_player(Point::new(10, 10));

        let started = player.start_walk(Direction::East, Some(|_, _| false));

        assert!(!started);
        assert!(!player.walking);
        assert_eq!(player.tile_position, Point::new(10, 10));
        assert_eq!(player.walk_direction, None);
        assert_eq!(player.walk_target_tile, None);
    }

    #[test]
    fn test_update_commits_tile_after_walk_animation_finishes() {
        let mut player = Entity::create_player(Point::new(10, 10));
        assert!(player.start_walk::<fn(i32, i32) -> bool>(Direction::East, None));

        player.update::<fn(i32, i32) -> bool>(0.2, None);

        assert!(!player.walking);
        assert_eq!(player.tile_position, Point::new(11, 9));
        assert_eq!(player.direction, Direction::East);
        assert_eq!(player.walk_direction, None);
        assert_eq!(player.walk_target_tile, None);
        assert_eq!(player.walking_progress(), 0.0);
    }

    #[test]
    fn test_walking_render_offset_uses_progress() {
        let mut player = Entity::create_player(Point::new(10, 10));
        assert!(player.start_walk::<fn(i32, i32) -> bool>(Direction::East, None));

        player.update::<fn(i32, i32) -> bool>(0.05, None);

        assert!(player.walking);
        assert_eq!(player.walking_render_offset(), Point::new(32, 0));
        assert!((player.walking_progress() - 0.5).abs() < 0.001);
    }
}
