/// 城镇场景模块 - Step 5.3
/// 
/// 这是城镇场景的预览版本，用于验证资源系统的完整性。
/// 完整的地图系统（.DUN, .TIL, .CEL）将在Step 6实现。
/// 
/// 设计方案：混合方案
/// - PCX图像作为静态背景
/// - 简化的碰撞检测（矩形可行走区域）
/// - 玩家在场景中移动
/// - 相机跟随玩家（有限滚动）

use crate::math::Rect as Rectangle;
use crate::engine::Engine;
use anyhow::Result;

/// 简化的城镇场景
/// 
/// 这是Step 5.3的预览实现，用于：
/// 1. 验证资源加载系统完整性
/// 2. 测试PCX背景显示
/// 3. 测试玩家在场景中移动
/// 4. 为Step 6的完整地图系统打基础
pub struct SimpleTown {
    /// 背景纹理ID（可选）
    pub background_texture_id: Option<String>,
    
    /// 背景宽度（像素）
    pub background_width: u32,
    
    /// 背景高度（像素）
    pub background_height: u32,
    
    /// 可行走区域（矩形）
    /// 玩家只能在这个区域内移动
    pub walkable_area: Rectangle,
    
    /// 障碍物列表（可选，后续可添加）
    pub obstacles: Vec<Rectangle>,
    
    /// 场景名称
    pub name: String,
}

impl SimpleTown {
    /// 创建默认的城镇场景
    /// 
    /// 默认配置：
    /// - 无背景纹理
    /// - 640x480的可行走区域（屏幕大小）
    /// - 无障碍物
    pub fn new() -> Self {
        Self {
            background_texture_id: None,
            background_width: 640,
            background_height: 480,
            // 默认可行走区域：整个屏幕，边缘留出32像素边距
            walkable_area: Rectangle::new(32, 32, 640 - 64, 480 - 64),
            obstacles: Vec::new(),
            name: "Town Preview".to_string(),
        }
    }
    
    /// 创建带背景的城镇场景
    /// 
    /// # 参数
    /// - `texture_id`: 背景纹理ID
    /// - `width`: 背景宽度
    /// - `height`: 背景高度
    pub fn with_background(texture_id: String, width: u32, height: u32) -> Self {
        let mut town = Self::new();
        town.background_texture_id = Some(texture_id);
        town.background_width = width;
        town.background_height = height;
        
        // 更新可行走区域为背景大小（留出边距）
        let margin = 50i32;
        let margin_u32 = (margin * 2) as u32;
        town.walkable_area = Rectangle::new(
            margin,
            margin,
            width.saturating_sub(margin_u32),
            height.saturating_sub(margin_u32),
        );
        
        town
    }
    
    /// 设置可行走区域
    pub fn set_walkable_area(&mut self, area: Rectangle) {
        self.walkable_area = area;
    }
    
    /// 添加障碍物
    pub fn add_obstacle(&mut self, obstacle: Rectangle) {
        self.obstacles.push(obstacle);
    }
    
    /// 检查位置是否可行走
    /// 
    /// # 参数
    /// - `x`: X坐标（像素）
    /// - `y`: Y坐标（像素）
    /// 
    /// # 返回
    /// - `true`: 可行走
    /// - `false`: 不可行走（超出边界或碰到障碍物）
    pub fn is_walkable(&self, x: f32, y: f32) -> bool {
        use crate::math::Point;
        
        let point = Point::new(x as i32, y as i32);
        
        // 检查是否在可行走区域内
        if !self.walkable_area.contains(point) {
            return false;
        }
        
        // 检查是否与障碍物碰撞
        for obstacle in &self.obstacles {
            if obstacle.contains(point) {
                return false;
            }
        }
        
        true
    }
    
    /// 获取背景纹理ID
    /// 
    /// 返回背景纹理ID（如果有）。
    /// 调用者可以用此ID在Engine中渲染背景。
    pub fn get_background_texture_id(&self) -> Option<&String> {
        self.background_texture_id.as_ref()
    }
    
    /// 计算背景渲染位置（考虑相机）
    /// 
    /// # 参数
    /// - `camera_x`: 相机X坐标
    /// - `camera_y`: 相机Y坐标
    /// 
    /// # 返回
    /// 背景的目标矩形
    pub fn get_background_dst(&self, camera_x: f32, camera_y: f32) -> Rectangle {
        Rectangle::new(
            -camera_x as i32,
            -camera_y as i32,
            self.background_width,
            self.background_height,
        )
    }
    
    /// 获取背景尺寸
    pub fn get_size(&self) -> (u32, u32) {
        (self.background_width, self.background_height)
    }
    
    /// 获取可行走区域
    pub fn get_walkable_area(&self) -> &Rectangle {
        &self.walkable_area
    }
}

impl Default for SimpleTown {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_town() {
        let town = SimpleTown::new();
        assert_eq!(town.name, "Town Preview");
        assert!(town.background_texture_id.is_none());
        assert_eq!(town.obstacles.len(), 0);
    }
    
    #[test]
    fn test_with_background() {
        let town = SimpleTown::with_background("bg_texture".to_string(), 800, 600);
        assert_eq!(town.background_texture_id, Some("bg_texture".to_string()));
        assert_eq!(town.background_width, 800);
        assert_eq!(town.background_height, 600);
    }
    
    #[test]
    fn test_is_walkable_inside() {
        let town = SimpleTown::new();
        // 中心点应该可行走
        assert!(town.is_walkable(320.0, 240.0));
    }
    
    #[test]
    fn test_is_walkable_outside() {
        let town = SimpleTown::new();
        // 超出边界应该不可行走
        assert!(!town.is_walkable(10.0, 10.0)); // 太靠近边缘
        assert!(!town.is_walkable(1000.0, 1000.0)); // 超出范围
    }
    
    #[test]
    fn test_is_walkable_with_obstacle() {
        let mut town = SimpleTown::new();
        // 添加障碍物
        town.add_obstacle(Rectangle::new(100, 100, 50, 50));
        
        // 障碍物内部不可行走
        assert!(!town.is_walkable(120.0, 120.0));
        
        // 障碍物外部可行走
        assert!(town.is_walkable(200.0, 200.0));
    }
    
    #[test]
    fn test_get_size() {
        let town = SimpleTown::with_background("bg".to_string(), 1024, 768);
        let (w, h) = town.get_size();
        assert_eq!(w, 1024);
        assert_eq!(h, 768);
    }
    
    #[test]
    fn test_set_walkable_area() {
        let mut town = SimpleTown::new();
        let new_area = Rectangle::new(50, 50, 500, 400);
        town.set_walkable_area(new_area);
        
        assert_eq!(town.walkable_area, new_area);
        
        // 测试新区域的边界
        assert!(town.is_walkable(100.0, 100.0)); // 内部
        assert!(!town.is_walkable(30.0, 30.0)); // 外部
    }
}

