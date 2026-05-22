# Step 6.4.1 实施计划：光照系统

> **创建日期:** 2025-12-03  
> **状态:** 规划中  
> **预计开发时间:** 3天  
> **代码量:** ~500-600行

---

## 📋 IMPLEMENTATION CHECKLIST

本检查清单严格按照 RIPER-5 协议，将光照系统实现拆分为原子操作，确保无需临时决策。

---

### 🔹 Day 1: 光照系统核心基础设施

#### 1.1 创建光照模块结构 (30分钟)

1. 创建 `src/lighting/` 目录
2. 创建 `src/lighting/mod.rs` 文件
3. 在 `src/lib.rs` 中添加 `pub mod lighting;`
4. 创建 `src/lighting/light_source.rs` 文件
5. 创建 `src/lighting/light_table.rs` 文件
6. 创建 `src/lighting/crawl.rs` 文件
7. 在 `src/lighting/mod.rs` 中添加模块导出:
   ```rust
   pub mod light_source;
   pub mod light_table;
   pub mod crawl;
   
   pub use light_source::{LightSource, LightType};
   pub use light_table::LightTables;
   pub use crawl::crawl_light;
   ```

#### 1.2 实现光源结构 (1小时)

8. 在 `light_source.rs` 中定义 `LightType` 枚举:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum LightType {
       Player,
       Torch,
       Spell,
       Monster,
   }
   ```
9. 定义 `LightSource` 结构体:
   ```rust
   #[derive(Debug, Clone)]
   pub struct LightSource {
       pub id: usize,
       pub light_type: LightType,
       pub position: (usize, usize),
       pub radius: u8,
       pub active: bool,
   }
   ```
10. 实现 `LightSource::player()` 构造函数
    - 设置 `radius = 10`（原版玩家光照半径）
    - 返回初始化的结构体
11. 实现 `LightSource::torch()` 构造函数
    - 设置 `radius = 8`（火把半径）
12. 编写光源测试:
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_player_light_creation() {
            let light = LightSource::player((50, 50));
            assert_eq!(light.radius, 10);
            assert_eq!(light.position, (50, 50));
            assert!(light.active);
        }
    }
    ```

#### 1.3 扩展 Palette 类 (30分钟)

13. 打开 `src/resources/palette.rs`
14. 在 `Palette` impl块中添加 `find_nearest_color()` 方法:
    ```rust
    pub fn find_nearest_color(&self, target: (u8, u8, u8)) -> u8 {
        let mut best_idx = 0;
        let mut best_dist = u32::MAX;
        
        for (idx, color) in self.colors.iter().enumerate() {
            let dist = color_distance(*color, target);
            if dist < best_dist {
                best_dist = dist;
                best_idx = idx;
            }
        }
        
        best_idx as u8
    }
    ```
15. 实现 `color_distance()` 辅助函数:
    ```rust
    fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> u32 {
        let dr = (c1.0 as i32 - c2.0 as i32).abs();
        let dg = (c1.1 as i32 - c2.1 as i32).abs();
        let db = (c1.2 as i32 - c2.2 as i32).abs();
        (dr * dr + dg * dg + db * db) as u32
    }
    ```
16. 编写测试验证最近颜色查找

#### 1.4 实现颜色变换表 (2小时)

17. 在 `light_table.rs` 中定义 `LightTables` 结构体:
    ```rust
    pub struct LightTables {
        tables: [[u8; 256]; 16],
    }
    ```
18. 实现 `from_palette()` 方法:
    - 循环 `level in 0..16`
    - 计算 `brightness = level as f32 / 15.0`
    - 对每个颜色索引应用亮度
    - 查找最接近的调色板颜色
19. 实现 `get()` 方法:
    ```rust
    pub fn get(&self, light_level: u8, color_index: u8) -> u8 {
        let level = light_level.min(15) as usize;
        self.tables[level][color_index as usize]
    }
    ```
20. 实现 `darken_color()` 私有方法:
    ```rust
    fn darken_color(color: (u8, u8, u8), brightness: f32) -> (u8, u8, u8) {
        (
            (color.0 as f32 * brightness) as u8,
            (color.1 as f32 * brightness) as u8,
            (color.2 as f32 * brightness) as u8,
        )
    }
    ```
21. 编写测试验证颜色变换:
    - Level 0 应该全黑
    - Level 15 应该保持原色
    - 中间等级平滑过渡

#### 1.5 实现 LightingSystem 核心 (2小时)

22. 在 `lighting/mod.rs` 中定义常量:
    ```rust
    pub const MAXDUNX: usize = 112;
    pub const MAXDUNY: usize = 112;
    ```
23. 定义 `LightingSystem` 结构体:
    ```rust
    pub struct LightingSystem {
        pub light_grid: [[u8; MAXDUNY]; MAXDUNX],
        pub light_sources: Vec<LightSource>,
        pub light_tables: LightTables,
        pub ambient_light: u8,
        next_light_id: usize,
    }
    ```
24. 实现 `new()` 构造函数:
    - 参数: `palette: &Palette`
    - 初始化 `light_grid` 为全0
    - 初始化 `light_sources` 为空Vec
    - 调用 `LightTables::from_palette(palette)`
    - 设置 `ambient_light = 3`
    - 设置 `next_light_id = 1`
25. 实现 `add_light()` 方法:
    ```rust
    pub fn add_light(&mut self, mut source: LightSource) -> usize {
        source.id = self.next_light_id;
        self.next_light_id += 1;
        self.light_sources.push(source.clone());
        source.id
    }
    ```
26. 实现 `remove_light()` 方法:
    ```rust
    pub fn remove_light(&mut self, id: usize) {
        self.light_sources.retain(|s| s.id != id);
    }
    ```
27. 实现 `update_light_position()` 方法:
    ```rust
    pub fn update_light_position(&mut self, id: usize, new_pos: (usize, usize)) {
        if let Some(source) = self.light_sources.iter_mut().find(|s| s.id == id) {
            source.position = new_pos;
        }
    }
    ```
28. 实现 `get_light_level()` 方法:
    ```rust
    pub fn get_light_level(&self, x: usize, y: usize) -> u8 {
        if x < MAXDUNX && y < MAXDUNY {
            self.light_grid[x][y].max(self.ambient_light)
        } else {
            self.ambient_light
        }
    }
    ```
29. 实现 `apply_lighting()` 方法:
    ```rust
    pub fn apply_lighting(&self, color_index: u8, light_level: u8) -> u8 {
        self.light_tables.get(light_level, color_index)
    }
    ```

#### 1.6 Day 1 测试和验收 (1小时)

30. 运行所有测试: `cargo test lighting`
31. 修复编译错误
32. 验证所有单元测试通过
33. 运行 `cargo clippy` 检查warnings
34. 运行 `cargo fmt` 格式化代码
35. 提交代码: `git commit -m "Step 6.4.1 Day1: 光照系统核心结构"`

---

### 🔹 Day 2: 光线追踪算法

#### 2.1 实现 Bresenham 直线算法 (1.5小时)

36. 在 `crawl.rs` 中添加导入:
    ```rust
    use super::LightSource;
    use super::{MAXDUNX, MAXDUNY};
    ```
37. 实现 `is_line_clear()` 函数签名:
    ```rust
    fn is_line_clear(
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        block_map: &[[bool; MAXDUNY]; MAXDUNX],
    ) -> bool
    ```
38. 实现 Bresenham 算法核心:
    - 初始化 `x = x0 as i32`, `y = y0 as i32`
    - 计算 `dx = (x1 as i32 - x0 as i32).abs()`
    - 计算 `dy = (y1 as i32 - y0 as i32).abs()`
    - 计算步进方向 `sx`, `sy`
    - 初始化 `err = dx - dy`
39. 实现循环:
    ```rust
    loop {
        if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
            if block_map[x as usize][y as usize] {
                return false;
            }
        }
        
        if x == x1 as i32 && y == y1 as i32 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
    true
    ```
40. 编写测试验证直线检测:
    - 测试无阻挡的直线
    - 测试有墙壁阻挡的直线

#### 2.2 实现光线追踪核心 (2小时)

41. 实现 `crawl_light()` 函数签名:
    ```rust
    pub fn crawl_light(
        grid: &mut [[u8; MAXDUNY]; MAXDUNX],
        source: &LightSource,
        block_map: &[[bool; MAXDUNY]; MAXDUNX],
    )
    ```
42. 获取光源参数:
    ```rust
    let (cx, cy) = source.position;
    let radius = source.radius as i32;
    ```
43. 清空之前的光照:
    ```rust
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = (cx as i32 + dx);
            let y = (cy as i32 + dy);
            
            if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
                grid[x as usize][y as usize] = 0;
            }
        }
    }
    ```
44. 实现光线发射:
    ```rust
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            
            if dist > radius as f32 {
                continue;
            }
            
            let x = (cx as i32 + dx);
            let y = (cy as i32 + dy);
            
            if x < 0 || x >= MAXDUNX as i32 || y < 0 || y >= MAXDUNY as i32 {
                continue;
            }
            
            if is_line_clear(cx, cy, x as usize, y as usize, block_map) {
                let intensity = 15.0 * (1.0 - dist / radius as f32);
                grid[x as usize][y as usize] = intensity.max(0.0).min(15.0) as u8;
            }
        }
    }
    ```
45. 编写测试验证光照衰减:
    - 中心亮度最高
    - 距离越远越暗
    - 半径外无光
46. 编写测试验证光线阻挡:
    - 墙壁前有光
    - 墙壁后无光

#### 2.3 实现 LightingSystem::update() (1小时)

47. 在 `LightingSystem` impl块中添加 `update()` 方法:
    ```rust
    pub fn update(&mut self, block_map: &[[bool; MAXDUNY]; MAXDUNX]) {
        // 清空光照网格
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                self.light_grid[x][y] = 0;
            }
        }
        
        // 计算所有激活的光源
        for source in &self.light_sources {
            if source.active {
                crawl_light(&mut self.light_grid, source, block_map);
            }
        }
    }
    ```
48. 编写集成测试:
    - 添加多个光源
    - 调用update
    - 验证光照网格正确

#### 2.4 Day 2 测试和验收 (1.5小时)

49. 运行所有测试: `cargo test`
50. 测试光照衰减效果
51. 测试墙壁阻挡效果
52. 性能测试: `cargo bench bench_crawl_light`
    - 目标: < 1ms per light source
53. 修复所有bug
54. 代码审查和优化
55. 提交代码: `git commit -m "Step 6.4.1 Day2: 光线追踪算法"`

---

### 🔹 Day 3: 渲染集成和调试

#### 3.1 准备碰撞地图 (30分钟)

56. 打开 `src/world/collision.rs`
57. 在 `CollisionMap` impl块中添加 `to_block_map()` 方法:
    ```rust
    pub fn to_block_map(&self) -> [[bool; crate::lighting::MAXDUNY]; crate::lighting::MAXDUNX] {
        let mut block_map = [[false; crate::lighting::MAXDUNY]; crate::lighting::MAXDUNX];
        
        for x in 0..crate::lighting::MAXDUNX {
            for y in 0..crate::lighting::MAXDUNY {
                if x < self.width && y < self.height {
                    block_map[x][y] = self.tiles[y][x];
                }
            }
        }
        
        block_map
    }
    ```
58. 测试碰撞地图转换

#### 3.2 修改 World 结构 (30分钟)

59. 打开 `src/world/mod.rs`
60. 添加导入: `use crate::lighting::LightingSystem;`
61. 在 `World` 结构体中添加字段:
    ```rust
    pub lighting: LightingSystem,
    ```
62. 修改 `World::new()` 构造函数:
    - 添加参数: `palette: &Palette`
    - 创建 `lighting = LightingSystem::new(palette)`
    - 在返回的Self中添加 `lighting`
63. 修改所有调用 `World::new()` 的地方传递 `palette`
    - 搜索: `World::new(`
    - 添加palette参数

#### 3.3 修改瓦片渲染应用光照 (2小时)

64. 找到 `World::render_micro_tile()` 方法
65. 在解码瓦片像素后,添加光照应用:
    ```rust
    // 获取光照等级
    let light_level = self.lighting.get_light_level(micro_x, micro_y);
    
    // 应用光照到索引像素
    let lit_pixels: Vec<u8> = indexed_pixels
        .iter()
        .map(|&idx| self.lighting.apply_lighting(idx, light_level))
        .collect();
    
    // 使用lit_pixels替代indexed_pixels进行RGBA转换
    let rgba_pixels = self.palette.indexed_to_rgba(&lit_pixels, true);
    ```
66. 测试编译
67. 修复类型错误
68. 测试渲染（应该全黑,因为还没有光源）

#### 3.4 添加玩家光源 (1小时)

69. 打开 `src/game.rs`
70. 在 `Game` 结构体中添加字段:
    ```rust
    player_light_id: Option<usize>,
    ```
71. 实现坐标转换辅助函数:
    ```rust
    fn convert_to_microtile(pos: Point) -> (usize, usize) {
        // 假设tile_size = 64
        let x = (pos.x / 64).max(0) as usize;
        let y = (pos.y / 64).max(0) as usize;
        (x, y)
    }
    ```
72. 修改 `Game::new()` 方法:
    - 在创建玩家后添加:
    ```rust
    let player_pos = convert_to_microtile(player.position);
    let light_id = world.lighting.add_light(LightSource::player(player_pos));
    ```
    - 设置 `player_light_id = Some(light_id)`
73. 修改 `Game::update()` 方法:
    - 在玩家移动后添加:
    ```rust
    // 更新玩家光源位置
    if let Some(light_id) = self.player_light_id {
        let new_pos = convert_to_microtile(self.player.position);
        self.world.lighting.update_light_position(light_id, new_pos);
    }
    
    // 更新光照系统
    let block_map = self.world.collision.to_block_map();
    self.world.lighting.update(&block_map);
    ```

#### 3.5 测试和调试 (2小时)

74. 编译项目: `cargo build`
75. 修复所有编译错误
76. 运行游戏
77. 验证基本功能:
    - [ ] 游戏启动
    - [ ] 地图渲染
    - [ ] 玩家可见
78. 验证光照效果:
    - [ ] 玩家周围有光环
    - [ ] 移动时光源跟随
    - [ ] 墙壁阻挡光线
    - [ ] 远离玩家的区域变暗
79. 调试问题:
    - 添加日志输出光照等级
    - 检查坐标转换
    - 验证颜色变换
80. 性能测试:
    - 测试FPS
    - 测试光照计算时间
    - 目标: FPS ≥ 30
81. 视觉效果调整:
    - 调整环境光等级 (2-4)
    - 调整玩家光照半径 (8-12)
    - 截图保存效果
82. 最终测试:
    - 运行所有单元测试
    - 运行集成测试
    - 手动测试清单
83. 代码清理:
    - 删除调试打印
    - 删除注释代码
    - `cargo fmt`
    - `cargo clippy`
84. 提交代码: `git commit -m "Step 6.4.1 Day3: 光照渲染集成完成"`

---

## ✅ 验收清单

### 功能验收

- [ ] 85. 光照系统正确初始化
- [ ] 86. 玩家光源正确创建
- [ ] 87. 光源ID管理正确
- [ ] 88. 颜色变换表生成正确
- [ ] 89. 光线追踪算法正确
- [ ] 90. 墙壁阻挡检测正确
- [ ] 91. 光照应用到渲染
- [ ] 92. 光源跟随玩家移动

### 视觉验收

- [ ] 93. 玩家周围有可见光环
- [ ] 94. 光照半径约10瓦片
- [ ] 95. 光照平滑衰减
- [ ] 96. 中心亮,边缘暗
- [ ] 97. 墙壁正确阻挡光线
- [ ] 98. 墙壁后面是暗的
- [ ] 99. 环境光照亮暗部
- [ ] 100. 菱形边界感消失
- [ ] 101. 小黑三角消失

### 性能验收

- [ ] 102. 单光源计算 < 1ms
- [ ] 103. 光照更新 < 5ms/frame
- [ ] 104. FPS ≥ 30
- [ ] 105. 无卡顿
- [ ] 106. 内存使用合理

### 测试验收

- [ ] 107. 所有单元测试通过
- [ ] 108. 测试覆盖率 ≥ 80%
- [ ] 109. 性能测试达标
- [ ] 110. 无编译warnings
- [ ] 111. 无clippy warnings

### 代码质量验收

- [ ] 112. 代码格式化正确
- [ ] 113. 注释完整
- [ ] 114. 错误处理完善
- [ ] 115. 无unsafe代码
- [ ] 116. 模块化清晰
- [ ] 117. 命名规范

---

## 📊 进度追踪

| Day | 任务数 | 已完成 | 进度 |
|-----|-------|-------|------|
| Day 1 | 35 | 0 | 0% |
| Day 2 | 20 | 0 | 0% |
| Day 3 | 29 | 0 | 0% |
| **总计** | **84** | **0** | **0%** |

---

## 📝 每日检查点

### Day 1 结束验收

- [ ] 光照模块结构创建完成
- [ ] LightSource实现完成
- [ ] LightTables实现完成
- [ ] LightingSystem核心完成
- [ ] 所有单元测试通过
- [ ] 代码提交

### Day 2 结束验收

- [ ] Bresenham算法实现完成
- [ ] crawl_light()实现完成
- [ ] LightingSystem::update()完成
- [ ] 光照衰减测试通过
- [ ] 光线阻挡测试通过
- [ ] 性能测试达标
- [ ] 代码提交

### Day 3 结束验收

- [ ] 碰撞地图转换完成
- [ ] World结构集成完成
- [ ] 瓦片渲染应用光照
- [ ] 玩家光源添加完成
- [ ] 游戏中可见光照效果
- [ ] 所有验收标准通过
- [ ] 实施总结文档完成
- [ ] 代码提交

---

## 🚨 风险和应对

### 风险1: 性能不达标

**表现:** 光照计算导致FPS下降

**应对:**
1. 优化光线追踪算法
2. 减少光源数量
3. 限制光照计算范围
4. 使用缓存

### 风险2: 颜色变换不正确

**表现:** 颜色变暗后出现色带

**应对:**
1. 检查find_nearest_color算法
2. 增加调色板精度
3. 使用dithering算法

### 风险3: 光线穿透墙壁

**表现:** 墙壁后面被照亮

**应对:**
1. 检查Bresenham算法
2. 增加采样点
3. 参考原版bleeding-up算法

---

## 📖 参考代码

| 功能 | 原版文件 | 行号 | Rust实现 |
|-----|---------|------|----------|
| 光照数组 | `lighting.cpp` | 15-20 | `lighting/mod.rs` |
| 光源管理 | `lighting.cpp::DoLighting()` | 56-89 | `LightingSystem::update()` |
| 光线追踪 | `lighting.cpp::DoCrawl()` | 124-156 | `crawl.rs::crawl_light()` |
| 颜色变换表 | `lighting.cpp::MakeLightTable()` | 201-234 | `light_table.rs` |
| 光照渲染 | `scrollrt.cpp::DrawCell()` | 588-611 | `world/mod.rs` |

---

**文档版本:** 1.0  
**创建日期:** 2025-12-03  
**状态:** 待执行  
**关联文档:**
- [Step 6.4 总体设计](../step-6.4/step-6.4-lighting-and-player-animation.md)
- [Step 6.4 Phase拆分方案](../step-6.4/step-6.4-phase-split.md)
- [master_plan.md](master_plan.md)

