/// Direction system for 8-directional movement
///
/// Reference: Source/engine/direction.hpp
use crate::math::Point;

/// Diablo player animation order.
///
/// This matches `Source/player.cpp::WalkSettings` and
/// `Source/engine/render/scrollrt.cpp::GetOffsetForWalking()`.
pub const WALK_ANIMATION_ORDER: [Direction; 8] = [
    Direction::South,
    Direction::SouthWest,
    Direction::West,
    Direction::NorthWest,
    Direction::North,
    Direction::NorthEast,
    Direction::East,
    Direction::SouthEast,
];

/// Screen-space walking offsets used by the original renderer.
pub const WALKING_RENDER_OFFSETS: [Point; 8] = [
    Point { x: 0, y: 32 },
    Point { x: -32, y: 16 },
    Point { x: -64, y: 0 },
    Point { x: -32, y: -16 },
    Point { x: 0, y: -32 },
    Point { x: 32, y: -16 },
    Point { x: 64, y: 0 },
    Point { x: 32, y: 16 },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    None,      // 静止
    North,     // 上
    NorthEast, // 右上
    East,      // 右
    SouthEast, // 右下
    South,     // 下
    SouthWest, // 左下
    West,      // 左
    NorthWest, // 左上
}

impl Direction {
    /// Get the fixed animation order used by Diablo player walk sheets.
    pub const fn walk_animation_order() -> [Direction; 8] {
        WALK_ANIMATION_ORDER
    }

    /// Convert this direction to the Diablo walk animation index.
    ///
    /// `Direction::None` does not represent a valid walk direction.
    pub fn to_walk_animation_index(self) -> Option<usize> {
        match self {
            Direction::South => Some(0),
            Direction::SouthWest => Some(1),
            Direction::West => Some(2),
            Direction::NorthWest => Some(3),
            Direction::North => Some(4),
            Direction::NorthEast => Some(5),
            Direction::East => Some(6),
            Direction::SouthEast => Some(7),
            Direction::None => None,
        }
    }

    /// Convert a Diablo walk animation index back into a direction.
    pub fn from_walk_animation_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Direction::South),
            1 => Some(Direction::SouthWest),
            2 => Some(Direction::West),
            3 => Some(Direction::NorthWest),
            4 => Some(Direction::North),
            5 => Some(Direction::NorthEast),
            6 => Some(Direction::East),
            7 => Some(Direction::SouthEast),
            _ => None,
        }
    }

    /// Return a stable lowercase suffix for texture IDs and debug output.
    pub fn animation_suffix(self) -> &'static str {
        match self {
            Direction::None => "none",
            Direction::North => "north",
            Direction::NorthEast => "north_east",
            Direction::East => "east",
            Direction::SouthEast => "south_east",
            Direction::South => "south",
            Direction::SouthWest => "south_west",
            Direction::West => "west",
            Direction::NorthWest => "north_west",
        }
    }

    /// Return the original walking offset for this direction, scaled by progress.
    pub fn walking_render_offset(self, progress: f32) -> Point {
        let Some(index) = self.to_walk_animation_index() else {
            return Point::zero();
        };

        let progress = progress.clamp(0.0, 1.0);
        let base = WALKING_RENDER_OFFSETS[index];
        Point::new(
            (base.x as f32 * progress).round() as i32,
            (base.y as f32 * progress).round() as i32,
        )
    }

    /// Return a simple 2D pixel offset for orthogonal previews.
    pub fn walking_pixel_offset(self, progress: f32, tile_size: i32) -> Point {
        let (dx, dy) = self.to_velocity();
        let progress = progress.clamp(0.0, 1.0);
        Point::new(
            (dx * tile_size as f32 * progress).round() as i32,
            (dy * tile_size as f32 * progress).round() as i32,
        )
    }

    /// 从速度向量创建方向
    ///
    /// # Arguments
    /// * `vx` - X 方向速度（负数为左，正数为右）
    /// * `vy` - Y 方向速度（负数为上，正数为下）
    ///
    /// # Returns
    /// 对应的方向枚举
    pub fn from_velocity(vx: f32, vy: f32) -> Self {
        // 如果速度为0，返回None
        if vx == 0.0 && vy == 0.0 {
            return Direction::None;
        }

        // 使用阈值来判断方向，避免浮点数精度问题
        const THRESHOLD: f32 = 0.1;

        let vx_positive = vx > THRESHOLD;
        let vx_negative = vx < -THRESHOLD;
        let vy_positive = vy > THRESHOLD;
        let vy_negative = vy < -THRESHOLD;

        // 根据8个方向判断
        match (vx_positive, vx_negative, vy_positive, vy_negative) {
            // 纯方向
            (false, false, false, true) => Direction::North, // 上
            (true, false, false, false) => Direction::East,  // 右
            (false, false, true, false) => Direction::South, // 下
            (false, true, false, false) => Direction::West,  // 左

            // 对角方向
            (true, false, false, true) => Direction::NorthEast, // 右上
            (true, false, true, false) => Direction::SouthEast, // 右下
            (false, true, true, false) => Direction::SouthWest, // 左下
            (false, true, false, true) => Direction::NorthWest, // 左上

            // 其他情况返回None
            _ => Direction::None,
        }
    }

    /// 转换为速度向量（未归一化）
    ///
    /// # Returns
    /// (vx, vy) 元组，值为 -1.0, 0.0, 或 1.0
    pub fn to_velocity(&self) -> (f32, f32) {
        match self {
            Direction::None => (0.0, 0.0),
            Direction::North => (0.0, -1.0),
            Direction::NorthEast => (1.0, -1.0),
            Direction::East => (1.0, 0.0),
            Direction::SouthEast => (1.0, 1.0),
            Direction::South => (0.0, 1.0),
            Direction::SouthWest => (-1.0, 1.0),
            Direction::West => (-1.0, 0.0),
            Direction::NorthWest => (-1.0, -1.0),
        }
    }

    /// 转换为单位向量（归一化后的速度向量）
    ///
    /// 对角线方向会被归一化，确保所有方向的移动速度一致
    ///
    /// # Returns
    /// (vx, vy) 元组，长度为 1.0
    pub fn to_unit_vector(&self) -> (f32, f32) {
        let (vx, vy) = self.to_velocity();

        if vx == 0.0 && vy == 0.0 {
            return (0.0, 0.0);
        }

        // 计算向量长度
        let length = (vx * vx + vy * vy).sqrt();

        // 归一化
        (vx / length, vy / length)
    }

    /// 判断是否在移动
    ///
    /// # Returns
    /// true 如果不是 None，false 如果是 None
    pub fn is_moving(&self) -> bool {
        !matches!(self, Direction::None)
    }

    /// 判断是否为对角线方向
    ///
    /// # Returns
    /// true 如果是对角线方向（NE, SE, SW, NW）
    pub fn is_diagonal(&self) -> bool {
        matches!(
            self,
            Direction::NorthEast
                | Direction::SouthEast
                | Direction::SouthWest
                | Direction::NorthWest
        )
    }

    /// 转换为 Diablo 等距 tile 偏移量（整数）
    ///
    /// # Returns
    /// (dx, dy) 元组，表示在 Diablo tile 坐标系统中的偏移量。
    ///
    /// 这里不能使用普通屏幕上下左右坐标。DevilutionX 的 `Direction`
    /// 是等距地图方向，参考 `Source/engine/displacement.hpp::fromDirection()`.
    pub fn to_tile_offset(&self) -> (i32, i32) {
        match self {
            Direction::None => (0, 0),
            Direction::South => (1, 1),
            Direction::SouthWest => (0, 1),
            Direction::West => (-1, 1),
            Direction::NorthWest => (-1, 0),
            Direction::North => (-1, -1),
            Direction::NorthEast => (0, -1),
            Direction::East => (1, -1),
            Direction::SouthEast => (1, 0),
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_velocity_cardinal() {
        // 测试4个基本方向
        assert_eq!(Direction::from_velocity(0.0, -1.0), Direction::North);
        assert_eq!(Direction::from_velocity(1.0, 0.0), Direction::East);
        assert_eq!(Direction::from_velocity(0.0, 1.0), Direction::South);
        assert_eq!(Direction::from_velocity(-1.0, 0.0), Direction::West);
    }

    #[test]
    fn test_from_velocity_diagonal() {
        // 测试4个对角方向
        assert_eq!(Direction::from_velocity(1.0, -1.0), Direction::NorthEast);
        assert_eq!(Direction::from_velocity(1.0, 1.0), Direction::SouthEast);
        assert_eq!(Direction::from_velocity(-1.0, 1.0), Direction::SouthWest);
        assert_eq!(Direction::from_velocity(-1.0, -1.0), Direction::NorthWest);
    }

    #[test]
    fn test_from_velocity_none() {
        // 测试静止
        assert_eq!(Direction::from_velocity(0.0, 0.0), Direction::None);
    }

    #[test]
    fn test_to_velocity() {
        // 测试速度向量转换
        assert_eq!(Direction::North.to_velocity(), (0.0, -1.0));
        assert_eq!(Direction::NorthEast.to_velocity(), (1.0, -1.0));
        assert_eq!(Direction::East.to_velocity(), (1.0, 0.0));
        assert_eq!(Direction::None.to_velocity(), (0.0, 0.0));
    }

    #[test]
    fn test_to_unit_vector() {
        // 测试归一化向量
        let (vx, vy) = Direction::North.to_unit_vector();
        assert_eq!((vx, vy), (0.0, -1.0));

        // 对角线方向应该被归一化
        let (vx, vy) = Direction::NorthEast.to_unit_vector();
        let length = (vx * vx + vy * vy).sqrt();
        assert!((length - 1.0).abs() < 0.0001); // 长度应该为1

        // 检查对角线方向的具体值
        let sqrt_2_inv = 1.0 / 2.0_f32.sqrt();
        assert!((vx - sqrt_2_inv).abs() < 0.0001);
        assert!((vy - (-sqrt_2_inv)).abs() < 0.0001);
    }

    #[test]
    fn test_is_moving() {
        assert!(!Direction::None.is_moving());
        assert!(Direction::North.is_moving());
        assert!(Direction::NorthEast.is_moving());
    }

    #[test]
    fn test_is_diagonal() {
        assert!(!Direction::None.is_diagonal());
        assert!(!Direction::North.is_diagonal());
        assert!(Direction::NorthEast.is_diagonal());
        assert!(Direction::SouthEast.is_diagonal());
        assert!(Direction::SouthWest.is_diagonal());
        assert!(Direction::NorthWest.is_diagonal());
    }

    #[test]
    fn test_default() {
        assert_eq!(Direction::default(), Direction::None);
    }

    #[test]
    fn test_walk_animation_index_roundtrip() {
        let order = Direction::walk_animation_order();
        assert_eq!(order[0], Direction::South);
        assert_eq!(order[4], Direction::North);
        assert_eq!(Direction::None.to_walk_animation_index(), None);

        for (index, direction) in order.iter().enumerate() {
            assert_eq!(direction.to_walk_animation_index(), Some(index));
            assert_eq!(
                Direction::from_walk_animation_index(index),
                Some(*direction)
            );
        }

        assert_eq!(Direction::from_walk_animation_index(8), None);
    }

    #[test]
    fn test_animation_suffixes() {
        assert_eq!(Direction::South.animation_suffix(), "south");
        assert_eq!(Direction::NorthEast.animation_suffix(), "north_east");
        assert_eq!(Direction::None.animation_suffix(), "none");
    }

    #[test]
    fn test_walking_render_offset_scaling() {
        let full = Direction::East.walking_render_offset(1.0);
        let half = Direction::East.walking_render_offset(0.5);
        assert_eq!(full, Point::new(64, 0));
        assert_eq!(half, Point::new(32, 0));

        let none = Direction::None.walking_render_offset(0.75);
        assert_eq!(none, Point::zero());
    }

    #[test]
    fn test_diablo_tile_offsets() {
        assert_eq!(Direction::South.to_tile_offset(), (1, 1));
        assert_eq!(Direction::SouthWest.to_tile_offset(), (0, 1));
        assert_eq!(Direction::West.to_tile_offset(), (-1, 1));
        assert_eq!(Direction::NorthWest.to_tile_offset(), (-1, 0));
        assert_eq!(Direction::North.to_tile_offset(), (-1, -1));
        assert_eq!(Direction::NorthEast.to_tile_offset(), (0, -1));
        assert_eq!(Direction::East.to_tile_offset(), (1, -1));
        assert_eq!(Direction::SouthEast.to_tile_offset(), (1, 0));
        assert_eq!(Direction::None.to_tile_offset(), (0, 0));
    }
}
