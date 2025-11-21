/// Point - 2D coordinate structure
/// 
/// Represents a position in 2D space. Can be used for:
/// - World coordinates (game world position)
/// - Screen coordinates (pixel position on screen)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Create a new point
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Create a point at origin (0, 0)
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    /// Add two points (vector addition)
    pub fn add(&self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }

    /// Subtract two points (vector subtraction)
    pub fn sub(&self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: Point) -> f32 {
        let dx = (other.x - self.x) as f32;
        let dy = (other.y - self.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Convert to SDL2 Point
    pub fn to_sdl(&self) -> sdl2::rect::Point {
        sdl2::rect::Point::new(self.x, self.y)
    }
}

impl From<(i32, i32)> for Point {
    fn from(tuple: (i32, i32)) -> Self {
        Point::new(tuple.0, tuple.1)
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

