use crate::math::{Point, Rect};

#[derive(Debug, Clone)]
pub struct Camera {
    // Camera position (world coordinates of top-left corner of the viewport)
    pub position: Point,

    // Viewport size (screen pixels)
    pub viewport_width: u32,
    pub viewport_height: u32,

    // Map bounds (limits for camera position)
    pub bounds: Rect,
}

impl Camera {
    /// Create a new camera
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            position: Point::zero(),
            viewport_width,
            viewport_height,
            bounds: Rect::new(0, 0, 0, 0), // Default to no bounds, will be set later
        }
    }

    /// Set the map bounds that limit camera movement
    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    /// Follow a target point (usually player center)
    /// Keeps the target in the center of the viewport
    /// For now, we don't consider panel - just use the full viewport
    pub fn follow(&mut self, target: Point) {
        // Calculate desired position (target in center of viewport)
        let mut x = target.x - (self.viewport_width as i32 / 2);
        let mut y = target.y - (self.viewport_height as i32 / 2);

        // Clamp to bounds if bounds are set (width > 0)
        if self.bounds.width > 0 && self.bounds.height > 0 {
            let min_x = self.bounds.x;
            let min_y = self.bounds.y;
            let max_x = self.bounds.x + self.bounds.width as i32 - self.viewport_width as i32;
            let max_y = self.bounds.y + self.bounds.height as i32 - self.viewport_height as i32;

            // Only clamp if map is larger than viewport
            if max_x >= min_x {
                x = x.clamp(min_x, max_x);
            } else {
                // Map is smaller than viewport, center it
                x = min_x - (self.viewport_width as i32 - self.bounds.width as i32) / 2;
            }

            if max_y >= min_y {
                y = y.clamp(min_y, max_y);
            } else {
                y = min_y - (self.viewport_height as i32 - self.bounds.height as i32) / 2;
            }
        }

        self.position = Point::new(x, y);
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Point) -> Point {
        Point {
            x: world_pos.x - self.position.x,
            y: world_pos.y - self.position.y,
        }
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Point) -> Point {
        Point {
            x: screen_pos.x + self.position.x,
            y: screen_pos.y + self.position.y,
        }
    }

    /// Check if a world point is visible in the camera
    pub fn is_visible(&self, world_pos: Point) -> bool {
        let screen_pos = self.world_to_screen(world_pos);
        screen_pos.x >= 0
            && screen_pos.x < self.viewport_width as i32
            && screen_pos.y >= 0
            && screen_pos.y < self.viewport_height as i32
    }

    /// Check if a world rectangle is visible (intersects with viewport)
    pub fn is_rect_visible(&self, world_rect: Rect) -> bool {
        let viewport_rect = Rect::new(
            self.position.x,
            self.position.y,
            self.viewport_width,
            self.viewport_height,
        );

        viewport_rect.intersects(&world_rect)
    }

    /// Update camera (can be used for smooth movement/lerp in future)
    pub fn update(&mut self, _dt: f32) {
        // Currently instant movement, so nothing to update over time
    }
}
