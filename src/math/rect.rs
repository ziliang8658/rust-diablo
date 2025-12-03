/// Rect - Rectangular area structure
///
/// Represents a rectangular region in 2D space.
/// Used for collision detection, rendering bounds, etc.
use super::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle from position and size
    pub fn from_center(center: Point, width: u32, height: u32) -> Self {
        Self {
            x: center.x - (width as i32) / 2,
            y: center.y - (height as i32) / 2,
            width,
            height,
        }
    }

    /// Get the center point of the rectangle
    pub fn center(&self) -> Point {
        Point::new(
            self.x + (self.width as i32) / 2,
            self.y + (self.height as i32) / 2,
        )
    }

    /// Get the top-left corner
    pub fn top_left(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Check if this rectangle contains a point
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width as i32
            && point.y >= self.y
            && point.y < self.y + self.height as i32
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width as i32
            && self.x + self.width as i32 > other.x
            && self.y < other.y + other.height as i32
            && self.y + self.height as i32 > other.y
    }

    /// Convert to SDL2 Rect
    pub fn to_sdl(&self) -> sdl2::rect::Rect {
        sdl2::rect::Rect::new(self.x, self.y, self.width, self.height)
    }
}
