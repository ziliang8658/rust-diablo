/// Entity module - Game entity system
/// 
/// This module provides a basic entity system for game objects.
/// Each entity has a position, size, and visual properties.

use crate::math::{Point, Rect};
use crate::renderer::Color;
use crate::sprite::{Animation, AnimationController, AnimationState};
use crate::engine::Direction;
use crate::world::collision::CollisionMap;

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
#[derive(Clone)]
pub struct Entity {
    pub entity_type: EntityType,
    pub position: Point,
    pub size: (u32, u32),
    pub color: Color,
    pub velocity: Point,
    pub use_sprite: bool,
    pub sprite_id: Option<String>,
    pub animation: Option<AnimationController>,
    
    // Step 4.1: 新增字段
    pub direction: Direction,     // 当前方向
    pub speed: f32,                // 移动速度（像素/秒）
    
    // 浮点精度位置（用于累积小数移动，避免舍入误差）
    pub position_f: (f32, f32),
}

impl Entity {
    /// Create a new entity
    pub fn new(entity_type: EntityType, position: Point, size: (u32, u32), color: Color) -> Self {
        Self {
            entity_type,
            position,
            size,
            color,
            velocity: Point::zero(),
            use_sprite: false,
            sprite_id: None,
            animation: None,
            direction: Direction::None,
            speed: 100.0, // 默认速度：100像素/秒
            position_f: (position.x as f32, position.y as f32),
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
    pub fn create_player(position: Point) -> Self {
        let mut entity = Self::new(
            EntityType::Player,
            position,
            (96, 96),  // Diablo 1 warrior sprite: 96x96 pixels (frame_width from CL2)
            Color::CYAN,
        );
        
        // Enable sprite rendering
        entity.use_sprite = true;
        entity.sprite_id = Some("warrior_town".to_string()); // 匹配 resource_manager 中的加载
        entity.speed = 200.0; // 玩家速度：200像素/秒
        
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
        let idle_anim = Animation::new(
            vec![Rect::new(0, 0, 96, 96)],  // 【占位符】：单帧，运行时会被替换
            0.15,   // Frame duration: 0.15s per frame
            true    // Loop
        );
        anim_controller.add_animation(AnimationState::Idle, idle_anim);
        
        // Walk animation - Warrior walking (从 wmnaw.cl2 加载)
        // 原版数据：walkingFrames 为 8 帧（每个方向）
        // 实际配置：见 game.rs:379-387
        let walk_anim = Animation::new(
            vec![Rect::new(0, 0, 96, 96)],  // 【占位符】：单帧，运行时会被替换
            0.1,    // Frame duration: 0.1s = 10 FPS (walking speed)
            true    // Loop
        );
        anim_controller.add_animation(AnimationState::Walk, walk_anim);
        
        // Set initial state to Idle
        anim_controller.set_state(AnimationState::Idle);
        entity.animation = Some(anim_controller);
        
        entity
    }

    /// Get the bounding rectangle for this entity
    pub fn bounds(&self) -> Rect {
        Rect::from_center(self.position, self.size.0, self.size.1)
    }

    /// Update entity position based on velocity
    pub fn update(&mut self, dt: f32, collision_info: Option<(&CollisionMap, u32)>) {
        // Apply velocity with delta time
        // Velocity is in pixels/second, so multiply by dt to get pixels/frame
        
        // Calculate potential new position
        let move_x = self.velocity.x as f32 * dt;
        let move_y = self.velocity.y as f32 * dt;
        
        let mut new_pos_f = self.position_f;
        new_pos_f.0 += move_x;
        new_pos_f.1 += move_y;
        
        let new_pos = Point::new(
            new_pos_f.0.round() as i32,
            new_pos_f.1.round() as i32
        );
        
        // Check collision if map is provided
        if let Some((map, tile_size)) = collision_info {
            // Only validate if we actually moved
            if new_pos != self.position {
                let validated_pos = map.validate_move(self.position, new_pos, self.size, tile_size);
                
                // If position was adjusted (collision occurred), update float position to match
                if validated_pos != new_pos {
                    // Update float position to match the validated integer position
                    // This prevents "stuck" float values drifting from visual position
                    self.position_f.0 = validated_pos.x as f32;
                    self.position_f.1 = validated_pos.y as f32;
                    self.position = validated_pos;
                } else {
                    // No collision, apply movement
                    self.position_f = new_pos_f;
                    self.position = new_pos;
                }
            } else {
                // Integer position didn't change (sub-pixel movement)
                // We must still update position_f to accumulate the movement
                // Since integer pos didn't change, we are safe from collision
                self.position_f = new_pos_f;
            }
        } else {
            // No collision check, just apply movement
            self.position_f = new_pos_f;
            self.position = new_pos;
        }
        
        // Update animation
        if let Some(ref mut anim) = self.animation {
            anim.update(dt);
        }
    }

    /// Set velocity
    pub fn set_velocity(&mut self, velocity: Point) {
        self.velocity = velocity;
    }

    /// Move in a direction
    pub fn move_by(&mut self, delta: Point) {
        self.position = self.position + delta;
        self.position_f.0 = self.position.x as f32;
        self.position_f.1 = self.position.y as f32;
    }

    /// Check if this entity collides with another
    pub fn collides_with(&self, other: &Entity) -> bool {
        self.bounds().intersects(&other.bounds())
    }
    
    // Step 4.1: 新增方法
    
    /// Set direction and update velocity accordingly
    /// 
    /// This will automatically calculate velocity based on direction and speed.
    /// The velocity vector is normalized for diagonal directions.
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
        self.update_velocity_from_direction();
    }
    
    /// Update velocity based on current direction and speed
    /// 
    /// Uses normalized direction vector to ensure consistent speed in all directions.
    /// Note: Velocity is stored as i32 but represents pixels per second.
    /// The actual movement is calculated in update() by multiplying with dt.
    pub fn update_velocity_from_direction(&mut self) {
        let (vx, vy) = self.direction.to_unit_vector();
        
        // Store velocity as pixels/second (will be multiplied by dt in update())
        // We keep it as i32 for simplicity, understanding that we lose some precision
        // but this is acceptable for pixel-based movement
        self.velocity = Point::new(
            (vx * self.speed) as i32,
            (vy * self.speed) as i32,
        );
    }
}
