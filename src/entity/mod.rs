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
    /// Player sprite sheet: 256x64 pixels, 4 frames horizontally
    /// - Frame 1 (0-63): Idle monkey (standing)
    /// - Frame 2 (64-127): Monkey on cloud (walk 1)
    /// - Frame 3 (128-191): Monkey on cloud (walk 2)
    /// - Frame 4 (192-255): Monkey on cloud (walk 3)
    pub fn create_player(position: Point) -> Self {
        let mut entity = Self::new(
            EntityType::Player,
            position,
            (64, 64),  // Player sprite is 64x64 pixels
            Color::CYAN,
        );
        
        // Enable sprite rendering
        entity.use_sprite = true;
        entity.sprite_id = Some("player".to_string());
        entity.speed = 200.0; // 玩家速度：200像素/秒
        
        // Initialize animation controller with monkey animations
        let mut anim_controller = AnimationController::new();
        
        // Idle animation - Frame 1: Standing monkey
        let idle_anim = Animation::new(
            vec![Rect::new(0, 0, 64, 64)],  // Frame 1: Idle monkey
            0.2,    // Frame duration (not used for single frame)
            true    // Loop
        );
        anim_controller.add_animation(AnimationState::Idle, idle_anim);
        
        // Walk animation - Frames 2-4: Monkey on cloud (walking)
        // Animation cycles through frames 2, 3, 4 for smooth walking effect
        let walk_anim = Animation::new(
            vec![
                Rect::new(64, 0, 64, 64),   // Frame 2: Monkey on cloud (walk 1)
                Rect::new(128, 0, 64, 64),  // Frame 3: Monkey on cloud (walk 2)
                Rect::new(192, 0, 64, 64),  // Frame 4: Monkey on cloud (walk 3)
                Rect::new(128, 0, 64, 64),  // Frame 3 again for smooth loop back
            ],
            0.12,   // Frame duration: 0.12s = ~8 FPS animation (smooth walking)
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
