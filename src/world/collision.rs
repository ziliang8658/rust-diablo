use crate::math::{Point, Rect};
use crate::renderer::Color;

/// Tile type in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Empty,
    Floor,
    Wall,
    Door,     // 预留
    Obstacle, // 预留
    Grass,    // 新增：草地
    Water,    // 新增：水（不可行走）
}

impl TileType {
    /// Get the color for this tile type
    pub fn color(&self) -> Color {
        match self {
            TileType::Empty => Color::BLACK,
            TileType::Floor => Color::DARK_GRAY,
            TileType::Wall => Color::GRAY,
            TileType::Door => Color::new(139, 69, 19), // Brown
            TileType::Obstacle => Color::RED,
            TileType::Grass => Color::GREEN,
            TileType::Water => Color::BLUE,
        }
    }

    /// Get the sprite ID for this tile type
    pub fn sprite_id(&self) -> Option<&'static str> {
        match self {
            TileType::Floor => Some("tile_floor"),
            TileType::Wall => Some("tile_wall"),
            TileType::Grass => Some("tile_grass"),
            TileType::Water => Some("tile_water"),
            _ => None,
        }
    }

    /// Check if the tile is walkable
    pub fn is_walkable(&self) -> bool {
        match self {
            TileType::Empty | TileType::Floor | TileType::Door | TileType::Grass => true,
            TileType::Wall | TileType::Obstacle | TileType::Water => false,
        }
    }

    /// Check if the tile is solid (blocks movement)
    pub fn is_solid(&self) -> bool {
        !self.is_walkable()
    }
}

/// Collision Map - Handles collision detection against the static world
pub struct CollisionMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TileType>>,
}

impl CollisionMap {
    /// Create a new collision map
    pub fn new(width: usize, height: usize, tiles: Vec<Vec<TileType>>) -> Self {
        Self {
            width,
            height,
            tiles,
        }
    }

    /// Update the tiles data
    pub fn update_tiles(&mut self, tiles: Vec<Vec<TileType>>) {
        self.tiles = tiles;
    }

    /// Check if a tile coordinate is walkable
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return false;
        }

        let x = x as usize;
        let y = y as usize;

        if x >= self.width || y >= self.height {
            return false;
        }

        self.tiles[y][x].is_walkable()
    }

    /// Check if a world position is walkable (point collision)
    pub fn is_world_pos_walkable(&self, x: f32, y: f32, tile_size: u32) -> bool {
        let tile_x = (x / tile_size as f32) as i32;
        let tile_y = (y / tile_size as f32) as i32;
        self.is_walkable(tile_x, tile_y)
    }

    /// Check if a rectangle area is walkable (AABB collision)
    /// This checks all tiles that the rectangle overlaps with.
    pub fn is_rect_walkable(&self, rect: Rect, tile_size: u32) -> bool {
        // Calculate tile range covered by the rectangle
        // Using a small epsilon to avoid floating point issues at exact boundaries
        // is not needed here since we use integer Rect, but we need to be careful.

        let min_tile_x = rect.x / tile_size as i32;
        let min_tile_y = rect.y / tile_size as i32;

        // For max tile, we check the right/bottom edge
        // If rect is 32x32 at 0,0, it covers 0-31, 0-31.
        // right edge is at 32.
        let max_tile_x = (rect.x + rect.width as i32 - 1) / tile_size as i32;
        let max_tile_y = (rect.y + rect.height as i32 - 1) / tile_size as i32;

        for y in min_tile_y..=max_tile_y {
            for x in min_tile_x..=max_tile_x {
                if !self.is_walkable(x, y) {
                    return false;
                }
            }
        }

        true
    }

    /// Validate and adjust movement to prevent collisions
    /// Returns the allowed new position.
    ///
    /// Simple implementation: checks if the target position is valid.
    /// If valid, returns target. If invalid, returns start position.
    ///
    /// Future improvement: slide along walls.
    pub fn validate_move(&self, from: Point, to: Point, size: (u32, u32), tile_size: u32) -> Point {
        // Construct target bounding box (center aligned)
        // Note: entity position is usually center. Rect::from_center is useful here if available.
        // Assuming `to` is the top-left or center?
        // Let's check Entity::bounds(). It uses Rect::from_center.
        // So `to` is the center position.

        let rect = Rect::from_center(to, size.0, size.1);

        if self.is_rect_walkable(rect, tile_size) {
            to
        } else {
            // Try to slide? For now, just block.
            // Can we move in X only?
            let rect_x = Rect::from_center(Point::new(to.x, from.y), size.0, size.1);
            if self.is_rect_walkable(rect_x, tile_size) {
                return Point::new(to.x, from.y);
            }

            // Can we move in Y only?
            let rect_y = Rect::from_center(Point::new(from.x, to.y), size.0, size.1);
            if self.is_rect_walkable(rect_y, tile_size) {
                return Point::new(from.x, to.y);
            }

            from
        }
    }

    /// Convert collision map to block map for lighting system
    ///
    /// # Returns
    /// 2D array where true = blocking (wall), false = non-blocking (floor)
    pub fn to_block_map(&self) -> [[bool; crate::lighting::MAXDUNY]; crate::lighting::MAXDUNX] {
        let mut block_map = [[false; crate::lighting::MAXDUNY]; crate::lighting::MAXDUNX];

        for y in 0..self.height.min(crate::lighting::MAXDUNY) {
            for x in 0..self.width.min(crate::lighting::MAXDUNX) {
                // Wall and solid tiles block light
                block_map[x][y] = self.tiles[y][x].is_solid();
            }
        }

        block_map
    }
}
