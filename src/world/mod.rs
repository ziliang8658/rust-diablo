/// World module - Game world and level system
/// 
/// This module manages the game world, including:
/// - Grid-based map
/// - Entity management
/// - World rendering

use crate::math::{Point, Rect};
use crate::renderer::{Color, Camera};
use crate::entity::Entity;
use crate::engine::Engine;
use anyhow::Result;

pub mod collision;
pub use collision::{TileType, CollisionMap};

/// World - Game world representation
pub struct World {
    /// Grid dimensions (in tiles)
    pub width: usize,
    pub height: usize,
    /// Tile size in pixels
    pub tile_size: u32,
    /// Collision map (holds tile data)
    pub collision_map: CollisionMap,
    /// Entities in the world
    entities: Vec<Entity>,
}

impl World {
    /// Create a new world
    pub fn new(width: usize, height: usize, tile_size: u32) -> Self {
        // Initialize with a simple floor pattern
        let mut tiles = vec![vec![TileType::Empty; width]; height];
        
        // Create a map with mixed terrain
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    tiles[y][x] = TileType::Wall;
                } else {
                    // Simple noise-like generation for variety
                    // Use x, y to determine tile type for determinism (or use rand if imported)
                    let noise = ((x as f64 * 0.1).sin() + (y as f64 * 0.1).cos() + ((x * y) as f64 * 0.01).sin()) * 10.0;
                    
                    if noise > 15.0 {
                         tiles[y][x] = TileType::Water;
                    } else if noise < -15.0 {
                         tiles[y][x] = TileType::Grass;
                    } else {
                         tiles[y][x] = TileType::Floor;
                    }
                    
                    // Add random grass patches
                    if (x * 7 + y * 13) % 11 == 0 {
                        tiles[y][x] = TileType::Grass;
                    }
                    
                    // Add some obstacles
                    if (x * 3 + y * 5) % 29 == 0 && x > 5 && x < width-5 && y > 5 && y < height-5 {
                         tiles[y][x] = TileType::Wall;
                    }
                }
            }
        }
        
        // Ensure center is clear for player
        let center_x = width / 2;
        let center_y = height / 2;
        for y in center_y-2..=center_y+2 {
            for x in center_x-2..=center_x+2 {
                tiles[y][x] = TileType::Floor;
            }
        }
        
        // Create collision map
        let collision_map = CollisionMap::new(width, height, tiles);

        Self {
            width,
            height,
            tile_size,
            collision_map,
            entities: Vec::new(),
        }
    }

    /// Add an entity to the world
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Get a mutable reference to an entity by index
    pub fn get_entity_mut(&mut self, index: usize) -> Option<&mut Entity> {
        self.entities.get_mut(index)
    }

    /// Get all entities
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Get mutable reference to all entities
    pub fn entities_mut(&mut self) -> &mut [Entity] {
        &mut self.entities
    }

    /// Get tile at grid position
    pub fn get_tile(&self, x: usize, y: usize) -> Option<TileType> {
        if x < self.width && y < self.height {
            Some(self.collision_map.tiles[y][x])
        } else {
            None
        }
    }

    /// Convert world position to grid coordinates
    pub fn world_to_grid(&self, pos: Point) -> (usize, usize) {
        let x = (pos.x / self.tile_size as i32).max(0) as usize;
        let y = (pos.y / self.tile_size as i32).max(0) as usize;
        (x, y)
    }

    /// Convert grid coordinates to world position (center of tile)
    pub fn grid_to_world(&self, x: usize, y: usize) -> Point {
        Point::new(
            (x as i32 * self.tile_size as i32) + (self.tile_size as i32 / 2),
            (y as i32 * self.tile_size as i32) + (self.tile_size as i32 / 2),
        )
    }

    /// Update all entities
    pub fn update(&mut self, dt: f32) {
        let tile_size = self.tile_size;
        let map = &self.collision_map;
        
        for entity in &mut self.entities {
            entity.update(dt, Some((map, tile_size)));
        }
    }

    /// Render the world
    pub fn render(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
        // Calculate visible tile range
        let start_tile_x = (camera.position.x / self.tile_size as i32).max(0) as usize;
        let start_tile_y = (camera.position.y / self.tile_size as i32).max(0) as usize;
        
        let end_tile_x = ((camera.position.x + camera.viewport_width as i32) / self.tile_size as i32 + 1).min(self.width as i32) as usize;
        let end_tile_y = ((camera.position.y + camera.viewport_height as i32) / self.tile_size as i32 + 1).min(self.height as i32) as usize;
        
        for y in start_tile_y..end_tile_y {
            for x in start_tile_x..end_tile_x {
                let tile = self.collision_map.tiles[y][x];
                let world_pos = Point::new(
                    x as i32 * self.tile_size as i32,
                    y as i32 * self.tile_size as i32,
                );
                
                let screen_pos = camera.world_to_screen(world_pos);
                
                let rect = Rect::new(
                    screen_pos.x,
                    screen_pos.y,
                    self.tile_size,
                    self.tile_size,
                );
                
                // Try to render with texture
                let mut texture_drawn = false;
                if let Some(sprite_id) = tile.sprite_id() {
                     // Note: draw_texture_by_id returns Result<bool> where bool is true if texture found
                     // We unwrap_or(false) to handle potential error by treating as not found
                     if engine.draw_texture_by_id(sprite_id, None, rect).unwrap_or(false) {
                         texture_drawn = true;
                     }
                }
                
                // Fallback to color if no texture
                if !texture_drawn {
                    engine.draw_rect(rect, tile.color())?;
                    // Draw grid lines only if no texture (cleaner look)
                    engine.draw_rect_outline(rect, Color::new(60, 60, 60))?;
                }
            }
        }

        // Draw entities
        for entity in &self.entities {
            // Skip entities not visible in camera
            if !camera.is_rect_visible(entity.bounds()) {
                continue;
            }
            
            let screen_pos = camera.world_to_screen(entity.position);
            let dst_rect = Rect::from_center(screen_pos, entity.size.0, entity.size.1);
            
            if entity.use_sprite {
                // Render sprite if available
                if let Some(ref sprite_id) = entity.sprite_id {
                    // Get current animation frame if available
                    let src_rect = entity.animation
                        .as_ref()
                        .and_then(|anim| anim.current_frame_rect());
                    
                    // Try to draw texture by ID with animation frame, fallback to colored rectangle if not found
                    if !engine.draw_texture_by_id(sprite_id, src_rect, dst_rect)? {
                        // Fallback to colored rectangle if texture not found
                        engine.draw_rect(dst_rect, entity.color)?;
                    }
                } else {
                    engine.draw_rect(dst_rect, entity.color)?;
                }
            } else {
                // Render as colored rectangle
                engine.draw_rect(dst_rect, entity.color)?;
            }
        }

        Ok(())
    }
}
