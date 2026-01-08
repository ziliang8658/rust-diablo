/// Direction system for 8-directional movement
///
/// Reference: Source/engine/direction.hpp

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

    /// 转换为 tile 偏移量（整数）
    ///
    /// # Returns
    /// (dx, dy) 元组，表示在 tile 坐标系统中的偏移量
    /// - dx: X 方向偏移（-1, 0, 或 1）
    /// - dy: Y 方向偏移（-1, 0, 或 1）
    pub fn to_tile_offset(&self) -> (i32, i32) {
        match self {
            Direction::None => (0, 0),
            Direction::North => (0, -1),
            Direction::NorthEast => (1, -1),
            Direction::East => (1, 0),
            Direction::SouthEast => (1, 1),
            Direction::South => (0, 1),
            Direction::SouthWest => (-1, 1),
            Direction::West => (-1, 0),
            Direction::NorthWest => (-1, -1),
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
}
