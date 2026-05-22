# Step 6.4 实施计划

> **创建日期:** 2025-12-03  
> **状态:** 规划中  
> **预计开始:** 待定

---

## 📋 IMPLEMENTATION CHECKLIST

本检查清单按照 RIPER-5 协议要求,将整个 Step 6.4 的实现拆分为最小的原子操作单元,确保实施过程无需临时决策。

---

### 🔹 Phase 1: 光照系统 - 核心基础设施 (Day 1-2)

#### 1.1 创建光照模块结构 (30分钟)

1. 创建 `src/lighting/` 目录
2. 创建 `src/lighting/mod.rs` 文件
3. 在 `src/lib.rs` 中添加 `pub mod lighting;`
4. 创建 `src/lighting/light_source.rs` 文件
5. 创建 `src/lighting/light_table.rs` 文件
6. 创建 `src/lighting/vision.rs` 文件
7. 创建 `src/lighting/crawl.rs` 文件
8. 在 `src/lighting/mod.rs` 中添加模块导出

#### 1.2 实现光源结构 (1小时)

9. 在 `light_source.rs` 中定义 `LightType` 枚举
   - 添加 `Player` 变体
   - 添加 `Torch` 变体
   - 添加 `Spell` 变体
   - 添加 `Monster` 变体
   - 添加 `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`
10. 在 `light_source.rs` 中定义 `LightSource` 结构体
    - 添加 `id: usize` 字段
    - 添加 `light_type: LightType` 字段
    - 添加 `position: (usize, usize)` 字段
    - 添加 `radius: u8` 字段
    - 添加 `active: bool` 字段
    - 添加 `#[derive(Debug, Clone)]`
11. 实现 `LightSource::player()` 构造函数
    - 设置 `light_type = LightType::Player`
    - 设置 `radius = 10` (原版玩家光照半径)
    - 设置 `active = true`
    - 返回 `Self`
12. 实现 `LightSource::torch()` 构造函数
    - 设置 `light_type = LightType::Torch`
    - 设置 `radius = 8`
    - 设置 `active = true`
    - 返回 `Self`

#### 1.3 实现颜色变换表 (2小时)

13. 在 `light_table.rs` 中定义 `LightTables` 结构体
    - 添加 `tables: [[u8; 256]; 16]` 字段
14. 实现 `LightTables::from_palette()` 函数
    - 参数: `palette: &Palette`
    - 创建 `tables` 数组,初始化为0
    - 循环 `level in 0..16`
      - 计算 `brightness = level as f32 / 15.0`
      - 循环 `color_idx in 0..256`
        - 获取原始颜色: `color = palette.colors[color_idx]`
        - 调用 `darken_color(color, brightness)`
        - 查找最接近的调色板索引: `palette.find_nearest_color(darkened)`
        - 设置 `tables[level][color_idx] = nearest_idx`
    - 返回 `Self { tables }`
15. 实现 `LightTables::get()` 函数
    - 参数: `light_level: u8, color_index: u8`
    - 限制 `level = light_level.min(15) as usize`
    - 返回 `self.tables[level][color_index as usize]`
16. 实现 `LightTables::darken_color()` 私有函数
    - 参数: `color: (u8, u8, u8), brightness: f32`
    - 计算 `r = (color.0 as f32 * brightness) as u8`
    - 计算 `g = (color.1 as f32 * brightness) as u8`
    - 计算 `b = (color.2 as f32 * brightness) as u8`
    - 返回 `(r, g, b)`

#### 1.4 扩展 Palette 类 (30分钟)

17. 打开 `src/resources/palette.rs`
18. 在 `Palette` 结构体中添加 `find_nearest_color()` 方法
    - 参数: `target: (u8, u8, u8)`
    - 初始化 `best_idx = 0`, `best_dist = u32::MAX`
    - 循环 `(idx, color) in self.colors.iter().enumerate()`
      - 计算颜色距离: `dist = color_distance(*color, target)`
      - 如果 `dist < best_dist`
        - 更新 `best_dist = dist`
        - 更新 `best_idx = idx`
    - 返回 `best_idx as u8`
19. 实现 `color_distance()` 辅助函数
    - 参数: `c1: (u8, u8, u8), c2: (u8, u8, u8)`
    - 计算 `dr = (c1.0 as i32 - c2.0 as i32).abs()`
    - 计算 `dg = (c1.1 as i32 - c2.1 as i32).abs()`
    - 计算 `db = (c1.2 as i32 - c2.2 as i32).abs()`
    - 返回 `(dr * dr + dg * dg + db * db) as u32`

#### 1.5 实现光照系统核心 (2小时)

20. 在 `lighting/mod.rs` 中定义常量
    - `pub const MAXDUNX: usize = 112;`
    - `pub const MAXDUNY: usize = 112;`
21. 定义 `LightingSystem` 结构体
    - 添加 `light_grid: [[u8; MAXDUNY]; MAXDUNX]`
    - 添加 `light_sources: Vec<LightSource>`
    - 添加 `light_tables: LightTables`
    - 添加 `ambient_light: u8`
    - 添加 `next_light_id: usize`
22. 实现 `LightingSystem::new()` 构造函数
    - 参数: `palette: &Palette`
    - 创建 `light_grid = [[0u8; MAXDUNY]; MAXDUNX]`
    - 创建 `light_sources = Vec::new()`
    - 创建 `light_tables = LightTables::from_palette(palette)`
    - 设置 `ambient_light = 3` (默认环境光)
    - 设置 `next_light_id = 1`
    - 返回 `Self`
23. 实现 `LightingSystem::add_light()` 方法
    - 参数: `mut source: LightSource`
    - 设置 `source.id = self.next_light_id`
    - 递增 `self.next_light_id += 1`
    - 添加到列表: `self.light_sources.push(source)`
    - 返回 `source.id`
24. 实现 `LightingSystem::remove_light()` 方法
    - 参数: `id: usize`
    - 使用 `self.light_sources.retain(|s| s.id != id)`
25. 实现 `LightingSystem::update_light_position()` 方法
    - 参数: `id: usize, new_pos: (usize, usize)`
    - 查找光源: `if let Some(source) = self.light_sources.iter_mut().find(|s| s.id == id)`
    - 更新位置: `source.position = new_pos`
26. 实现 `LightingSystem::get_light_level()` 方法
    - 参数: `x: usize, y: usize`
    - 边界检查: `if x < MAXDUNX && y < MAXDUNY`
    - 返回 `self.light_grid[x][y].max(self.ambient_light)`
    - 否则返回 `self.ambient_light`
27. 实现 `LightingSystem::apply_lighting()` 方法
    - 参数: `color_index: u8, light_level: u8`
    - 返回 `self.light_tables.get(light_level, color_index)`

#### 1.6 实现光线追踪算法 (3小时)

28. 在 `crawl.rs` 中导入必要的模块
    - `use super::LightSource;`
    - `use super::{MAXDUNX, MAXDUNY};`
29. 实现 `crawl_light()` 函数
    - 参数: `grid: &mut [[u8; MAXDUNY]; MAXDUNX]`
    - 参数: `source: &LightSource`
    - 参数: `block_map: &[[bool; MAXDUNY]; MAXDUNX]`
    - 获取 `(cx, cy) = source.position`
    - 获取 `radius = source.radius as i32`
30. 实现光照清空逻辑
    - 循环 `dy in -radius..=radius`
      - 循环 `dx in -radius..=radius`
        - 计算 `x = (cx as i32 + dx)`
        - 计算 `y = (cy as i32 + dy)`
        - 边界检查: `if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32`
        - 设置 `grid[x as usize][y as usize] = 0`
31. 实现光线发射逻辑
    - 循环 `dy in -radius..=radius`
      - 循环 `dx in -radius..=radius`
        - 计算 `dist = ((dx * dx + dy * dy) as f32).sqrt()`
        - 跳过超出范围: `if dist > radius as f32 { continue }`
        - 计算 `x = (cx as i32 + dx)`
        - 计算 `y = (cy as i32 + dy)`
        - 边界检查: `if x < 0 || x >= MAXDUNX as i32 || y < 0 || y >= MAXDUNY as i32 { continue }`
        - 调用 `is_line_clear(cx, cy, x as usize, y as usize, block_map)`
        - 如果清晰:
          - 计算 `intensity = 15.0 * (1.0 - dist / radius as f32)`
          - 限制 `intensity = intensity.max(0.0).min(15.0)`
          - 设置 `grid[x as usize][y as usize] = intensity as u8`
32. 实现 `is_line_clear()` 辅助函数
    - 参数: `x0: usize, y0: usize`
    - 参数: `x1: usize, y1: usize`
    - 参数: `block_map: &[[bool; MAXDUNY]; MAXDUNX]`
    - 返回类型: `bool`
33. 实现 Bresenham 算法
    - 初始化 `x = x0 as i32`, `y = y0 as i32`
    - 计算 `dx = (x1 as i32 - x0 as i32).abs()`
    - 计算 `dy = (y1 as i32 - y0 as i32).abs()`
    - 计算 `sx = if x0 < x1 { 1 } else { -1 }`
    - 计算 `sy = if y0 < y1 { 1 } else { -1 }`
    - 初始化 `err = dx - dy`
    - 循环直到 `(x, y) == (x1, y1)`
      - 边界检查
      - 检查阻挡: `if block_map[x as usize][y as usize] { return false }`
      - 更新 Bresenham 步进
    - 返回 `true`

#### 1.7 编写光照系统单元测试 (1小时)

34. 创建 `src/lighting/mod.rs` 中的 `#[cfg(test)]` 模块
35. 编写 `test_light_source_creation` 测试
    - 创建玩家光源
    - 断言 `radius == 10`
    - 断言 `active == true`
36. 编写 `test_light_tables_generation` 测试
    - 创建测试调色板
    - 生成光照表
    - 断言 level 0 映射到黑色
    - 断言 level 15 保持原色
37. 编写 `test_crawl_light_attenuation` 测试
    - 创建grid和光源
    - 调用 crawl_light
    - 断言中心亮度最高
    - 断言边缘衰减
38. 编写 `test_light_blocking` 测试
    - 创建grid、光源和墙壁
    - 调用 crawl_light
    - 断言墙壁后面是暗的
39. 运行测试: `cargo test lighting`
40. 修复所有测试失败

---

### 🔹 Phase 2: 光照系统 - 渲染集成 (Day 3)

#### 2.1 准备碰撞地图 (30分钟)

41. 打开 `src/world/collision.rs`
42. 在 `CollisionMap` 中添加 `to_block_map()` 方法
    - 返回类型: `[[bool; MAXDUNY]; MAXDUNX]`
    - 创建 `block_map = [[false; MAXDUNY]; MAXDUNX]`
    - 循环填充: 从 `self.tiles` 复制阻挡信息
    - 返回 `block_map`

#### 2.2 修改 World 结构 (30分钟)

43. 打开 `src/world/mod.rs`
44. 添加导入: `use crate::lighting::LightingSystem;`
45. 在 `World` 结构体中添加字段: `pub lighting: LightingSystem`
46. 修改 `World::new()` 构造函数
    - 添加参数: `palette: &Palette`
    - 创建 `lighting = LightingSystem::new(palette)`
    - 在返回的 `Self` 中添加 `lighting`
47. 修改所有调用 `World::new()` 的地方传递 `palette`

#### 2.3 修改瓦片渲染应用光照 (2小时)

48. 找到 `World::render_micro_tile()` 方法
49. 修改方法签名添加光照参数 (如果需要)
50. 在解码瓦片像素后,添加光照应用代码:
    - 获取 `light_level = self.lighting.get_light_level(micro_x, micro_y)`
    - 创建 `lit_pixels = Vec::with_capacity(indexed_pixels.len())`
    - 循环 `for &idx in &indexed_pixels`
      - 计算 `lit_idx = self.lighting.apply_lighting(idx, light_level)`
      - 添加 `lit_pixels.push(lit_idx)`
    - 使用 `lit_pixels` 替代 `indexed_pixels` 进行RGBA转换
51. 找到所有调用 `render_micro_tile()` 的地方,传递正确的参数

#### 2.4 添加玩家光源 (1小时)

52. 打开 `src/game.rs`
53. 在 `Game` 结构体中添加字段: `player_light_id: Option<usize>`
54. 修改 `Game::new()` 方法
    - 在创建玩家后,添加光源:
      ```rust
      let player_pos = convert_to_microtile(player.position);
      let light_id = world.lighting.add_light(LightSource::player(player_pos));
      ```
    - 设置 `player_light_id = Some(light_id)`
55. 修改 `Game::update()` 方法
    - 在玩家移动后,更新光源位置:
      ```rust
      if let Some(light_id) = self.player_light_id {
          let new_pos = convert_to_microtile(self.player.position);
          self.world.lighting.update_light_position(light_id, new_pos);
      }
      ```
56. 实现 `convert_to_microtile()` 辅助函数
    - 参数: `pos: Point`
    - 返回: `(usize, usize)`
    - 计算MicroTile坐标 (根据等距投影)

#### 2.5 更新光照系统 (30分钟)

57. 在 `Game::update()` 方法中添加:
    ```rust
    // 更新光照
    let block_map = self.world.collision.to_block_map();
    self.world.lighting.update(&block_map);
    ```
58. 在 `LightingSystem` 中实现 `update()` 方法
    - 参数: `block_map: &[[bool; MAXDUNY]; MAXDUNX]`
    - 清空 `self.light_grid`
    - 循环 `for source in &self.light_sources`
      - 如果 `source.active`
      - 调用 `crawl_light(&mut self.light_grid, source, block_map)`

#### 2.6 测试光照渲染 (1小时)

59. 编译项目: `cargo build`
60. 运行游戏
61. 验证:
    - 玩家周围有光环
    - 移动时光源跟随
    - 墙壁阻挡光线
    - 远离玩家的区域变暗
62. 如果有问题,添加调试输出:
    - 打印光照等级
    - 打印颜色变换
    - 检查坐标转换
63. 修复所有bug

---

### 🔹 Phase 3: 8方向动画系统 - Direction扩展 (Day 3下半天)

#### 3.1 扩展 Direction 枚举 (30分钟)

64. 打开 `src/engine/direction.rs`
65. 添加 `to_animation_index()` 方法
    - 返回类型: `usize`
    - 使用 `match self`
    - `Direction::South => 0`
    - `Direction::SouthWest => 1`
    - `Direction::West => 2`
    - `Direction::NorthWest => 3`
    - `Direction::North => 4`
    - `Direction::NorthEast => 5`
    - `Direction::East => 6`
    - `Direction::SouthEast => 7`
    - `Direction::None => 0` (默认朝南)
66. 添加 `from_animation_index()` 静态方法
    - 参数: `index: usize`
    - 返回类型: `Self`
    - 使用 `match index`
    - 0-7 映射到对应方向
    - 默认返回 `Direction::South`

#### 3.2 编写 Direction 测试 (15分钟)

67. 在 `direction.rs` 的测试模块中添加:
68. 编写 `test_to_animation_index` 测试
    - 断言 `South.to_animation_index() == 0`
    - 断言 `North.to_animation_index() == 4`
    - 断言所有8个方向
69. 编写 `test_from_animation_index` 测试
    - 断言 `from_animation_index(0) == South`
    - 断言所有0-7索引
70. 编写 `test_animation_index_roundtrip` 测试
    - 循环所有方向
    - 断言 `from_animation_index(dir.to_animation_index()) == dir`
71. 运行测试: `cargo test direction`

---

### 🔹 Phase 4: 8方向动画系统 - AnimationController扩展 (Day 4上午)

#### 4.1 创建 DirectionalAnimation 结构 (1小时)

72. 打开 `src/sprite/animation.rs`
73. 在文件顶部添加导入: `use crate::engine::Direction;`
74. 定义 `DirectionalAnimation` 结构体
    - 添加 `directions: [Animation; 8]`
    - 添加 `looping: bool`
75. 实现 `DirectionalAnimation::new()`
    - 参数: `animations: [Animation; 8], looping: bool`
    - 返回 `Self { directions: animations, looping }`
76. 实现 `get_animation()` 方法
    - 参数: `direction: Direction`
    - 计算 `index = direction.to_animation_index()`
    - 返回 `&self.directions[index]`
77. 实现 `get_animation_mut()` 方法
    - 参数: `direction: Direction`
    - 计算 `index = direction.to_animation_index()`
    - 返回 `&mut self.directions[index]`

#### 4.2 扩展 AnimationController (1小时)

78. 修改 `AnimationController` 结构体
    - 修改 `animations: HashMap<AnimationState, DirectionalAnimation>`
    - 添加 `current_direction: Direction`
79. 修改 `AnimationController::new()` 构造函数
    - 添加 `current_direction: Direction::South`
80. 实现 `add_directional_animation()` 方法
    - 参数: `state: AnimationState, anim: DirectionalAnimation`
    - 插入: `self.animations.insert(state, anim)`
81. 实现 `set_direction()` 方法
    - 参数: `direction: Direction`
    - 检查: `if direction != self.current_direction`
    - 更新: `self.current_direction = direction`
    - 重置帧: `self.current_frame = 0`
    - 重置计时器: `self.frame_timer = 0.0`
82. 修改 `current_frame_rect()` 方法
    - 获取当前动画: `let anim = self.animations.get(&self.current_state)?`
    - 获取方向动画: `let dir_anim = anim.get_animation(self.current_direction)`
    - 检查帧范围: `if self.current_frame < dir_anim.frames.len()`
    - 返回: `Some(dir_anim.frames[self.current_frame])`
83. 修改 `update()` 方法
    - 获取当前动画的帧数 (通过current_direction)
    - 更新帧循环逻辑

#### 4.3 编写 AnimationController 测试 (30分钟)

84. 在 `animation.rs` 测试模块中添加:
85. 编写 `test_directional_animation_creation` 测试
    - 创建8个Animation
    - 创建DirectionalAnimation
    - 断言directions.len() == 8
86. 编写 `test_direction_switching` 测试
    - 创建AnimationController
    - 设置Direction::South
    - 断言current_direction == South
    - 切换到North
    - 断言current_frame == 0
87. 运行测试: `cargo test animation`

---

### 🔹 Phase 5: 8方向动画系统 - CL2加载器扩展 (Day 4下午)

#### 5.1 扩展 CL2Sprite (1小时)

88. 打开 `src/resources/cl2.rs`
89. 添加 `load_directional()` 静态方法
    - 参数: `data: &[u8], frame_width: usize`
    - 返回类型: `Result<Vec<Vec<Frame>>>`
90. 实现 CL2 header 解析
    - 固定 `num_animations = 8`
    - 读取offset表 (9个offset,最后一个是文件结尾)
    - 创建 `offsets: Vec<u32>`
91. 循环解析8个方向
    - `for i in 0..8`
    - 获取 `start = offsets[i]`
    - 获取 `end = offsets[i + 1]`
    - 切片 `anim_data = &data[start..end]`
    - 调用现有的decode方法解析帧
    - 添加到 `directional_frames`
92. 返回 `Ok(directional_frames)`

#### 5.2 测试 CL2 加载 (30分钟)

93. 创建 `tests/cl2_directional_test.rs`
94. 编写 `test_load_directional_cl2` 测试
    - 加载测试文件 (如果有)
    - 断言返回8个方向
    - 断言每个方向有帧
95. 如果没有测试文件,创建mock数据
96. 运行测试: `cargo test cl2`

---

### 🔹 Phase 6: 8方向动画系统 - 游戏集成 (Day 4下午-Day 5上午)

#### 6.1 修改 ResourceManager 加载8方向CL2 (1小时)

97. 打开 `src/resources/resource_manager.rs`
98. 添加方法 `load_player_cl2_directional()`
    - 参数: `file_path: &str, frame_width: usize`
    - 调用 `self.mpq.read_file(file_path)`
    - 调用 `CL2Sprite::load_directional(&data, frame_width)`
    - 返回 `Result<Vec<Vec<Frame>>>`
99. 在 `ResourceManager` 中添加字段存储玩家动画
    - `player_idle_8dir: Option<Vec<Vec<Frame>>>`
    - `player_walk_8dir: Option<Vec<Vec<Frame>>>`

#### 6.2 修改 game.rs 加载玩家精灵 (1.5小时)

100. 打开 `src/game.rs`
101. 在 `Game::new()` 中加载8方向精灵:
     ```rust
     // Load Idle (wm nas.cl2 - warrior male unarmed stand)
     let idle_8dir = resource_manager.load_player_cl2_directional(
         "plrgfx/warrior/wmn/wmnas.cl2",
         96
     )?;
     
     // Load Walk (wmnaw.cl2 - warrior male unarmed walk)
     let walk_8dir = resource_manager.load_player_cl2_directional(
         "plrgfx/warrior/wmn/wmnaw.cl2",
         96
     )?;
     ```
102. 创建8个方向的Idle Animation
     - 循环 `for dir_frames in &idle_8dir`
     - 将每个方向的帧转换为 `Vec<Rect>`
     - 创建 `Animation::new(frames, 0.15, true)`
     - 收集到数组 `[Animation; 8]`
103. 创建8个方向的Walk Animation
     - 同样的流程,帧率0.1秒
104. 创建 `DirectionalAnimation`
     - `let idle = DirectionalAnimation::new(idle_animations, true)`
     - `let walk = DirectionalAnimation::new(walk_animations, true)`
105. 更新 player 的 AnimationController
     - `if let Some(ref mut anim) = player.animation`
     - `anim.add_directional_animation(AnimationState::Idle, idle)`
     - `anim.add_directional_animation(AnimationState::Walk, walk)`

#### 6.3 修改输入处理同步方向 (1小时)

106. 在 `Game::handle_input()` 方法中:
107. 计算移动方向
     - 根据WASD或方向键计算 `(vx, vy)`
     - 调用 `Direction::from_velocity(vx, vy)`
     - 获得 `new_direction`
108. 更新玩家方向
     - `self.player.set_direction(new_direction)`
109. 更新动画状态
     - 如果 `new_direction.is_moving()`
       - `player.animation.set_state(AnimationState::Walk)`
     - 否则
       - `player.animation.set_state(AnimationState::Idle)`
110. 更新动画方向
     - `player.animation.set_direction(new_direction)`

#### 6.4 修改 Entity 渲染 (30分钟)

111. 打开 `src/entity/mod.rs`
112. 确保 `Entity::update_with_direction()` 正确实现
113. 在渲染时使用 `animation.current_frame_rect()`
114. 确保纹理ID包含方向信息 (如果需要)

---

### 🔹 Phase 7: 集成测试和调试 (Day 5)

#### 7.1 编译和初步测试 (1小时)

115. 运行 `cargo build`
116. 修复所有编译错误
117. 运行 `cargo test`
118. 修复所有测试失败
119. 运行游戏
120. 验证基本功能:
     - 游戏启动
     - 地图渲染
     - 玩家可见

#### 7.2 光照系统测试 (1小时)

121. 验证玩家周围有光环
122. 移动玩家,观察光源跟随
123. 检查墙壁阻挡效果
124. 调整环境光等级观察效果
125. 测试性能 (FPS ≥ 30)
126. 如果有问题,添加日志输出:
     - `println!("Light level at ({}, {}): {}", x, y, level);`
127. 修复所有光照相关bug

#### 7.3 8方向动画测试 (1小时)

128. 测试向上移动 (W键)
     - 验证玩家朝北
     - 验证Walk动画播放
129. 测试向右上移动 (W+D键)
     - 验证玩家朝东北
     - 验证对角线动画正确
130. 测试所有8个方向
     - 上: W
     - 右上: W+D
     - 右: D
     - 右下: S+D
     - 下: S
     - 左下: S+A
     - 左: A
     - 左上: W+A
131. 测试停止移动
     - 验证切换到Idle动画
     - 验证方向保持不变
132. 测试快速切换方向
     - 验证动画不抖动
     - 验证帧切换流畅
133. 如果有问题,添加调试信息:
     - 显示当前方向
     - 显示当前帧
     - 显示动画状态

#### 7.4 性能测试 (30分钟)

134. 添加FPS显示
135. 测试光照计算性能
     - 单个光源: 应 < 1ms
     - 多个光源: 应 < 5ms
136. 测试动画更新性能
     - 应 < 1ms
137. 测试整体帧率
     - 目标: ≥ 30 FPS
     - 理想: ≥ 60 FPS
138. 如果性能不足,进行优化:
     - 减少光照计算范围
     - 缓存光照结果
     - 优化纹理上传

#### 7.5 视觉效果调整 (1小时)

139. 调整光照半径
     - 玩家: 10瓦片
     - 可根据效果微调
140. 调整环境光等级
     - 测试 0-15 各个等级
     - 选择最佳值 (建议 2-4)
141. 调整动画帧率
     - Idle: 6-8 FPS (0.12-0.15s)
     - Walk: 10-12 FPS (0.08-0.1s)
142. 调整方向切换延迟
     - 防止抖动: 0.05-0.1s
143. 截图保存效果
     - 8个方向各一张
     - 光照效果一张

---

### 🔹 Phase 8: 文档和总结 (Day 5下午)

#### 8.1 编写实现总结 (1小时)

144. 创建 `step-6.4-implementation-summary.md`
145. 记录实际完成的功能
146. 记录遇到的问题和解决方案
147. 记录性能数据
148. 记录视觉效果截图
149. 记录代码统计 (实际行数)

#### 8.2 编写测试总结 (30分钟)

150. 创建 `step-6.4-testing-summary.md`
151. 记录单元测试结果
152. 记录集成测试结果
153. 记录性能测试数据
154. 记录手动测试checklist

#### 8.3 更新项目文档 (30分钟)

155. 更新 `master_plan.md`
     - 标记 Step 6.4 为已完成
     - 更新代码量统计
     - 更新完成日期
156. 更新 `FEATURE_COMPARISON.md`
     - 标记光照系统为已完成
     - 标记8方向动画为已完成
157. 更新 `README.md` (如果需要)

#### 8.4 代码清理 (30分钟)

158. 删除所有调试打印语句
159. 删除注释掉的代码
160. 格式化代码: `cargo fmt`
161. 运行 linter: `cargo clippy`
162. 修复所有 warnings
163. 最终编译: `cargo build --release`
164. 最终测试: `cargo test --release`

---

## ✅ 验收标准清单

### 光照系统验收

- [ ] 165. 玩家周围有可见光环 (半径约10瓦片)
- [ ] 166. 光照平滑衰减 (中心亮,边缘暗)
- [ ] 167. 墙壁正确阻挡光线
- [ ] 168. 光源跟随玩家移动
- [ ] 169. 环境光照亮暗部区域
- [ ] 170. 菱形边界感消失
- [ ] 171. 小黑三角消失
- [ ] 172. 光照计算性能 < 5ms/frame
- [ ] 173. 颜色变换正确 (无色带)
- [ ] 174. FPS ≥ 30

### 8方向动画验收

- [ ] 175. 支持8个方向移动
- [ ] 176. Idle和Walk两种状态
- [ ] 177. 向上移动时玩家朝北
- [ ] 178. 向右上移动时玩家朝东北
- [ ] 179. 所有8个方向视觉正确
- [ ] 180. 停止移动时切换到Idle
- [ ] 181. 方向切换流畅 (无抖动)
- [ ] 182. 动画帧率合适 (10 FPS)
- [ ] 183. 动画循环正确
- [ ] 184. 方向保持直到下次移动

### 代码质量验收

- [ ] 185. 所有单元测试通过
- [ ] 186. 测试覆盖率 ≥ 80%
- [ ] 187. 无编译警告
- [ ] 188. 无clippy警告
- [ ] 189. 代码格式化正确
- [ ] 190. 文档注释完整
- [ ] 191. 无unsafe代码 (除非必要)
- [ ] 192. 错误处理完善
- [ ] 193. 性能可接受
- [ ] 194. 可读性良好

---

## 📊 进度追踪

| Phase | 任务数 | 已完成 | 进度 |
|-------|-------|-------|------|
| Phase 1: 光照基础 | 40 | 0 | 0% |
| Phase 2: 光照渲染 | 23 | 0 | 0% |
| Phase 3: Direction扩展 | 8 | 0 | 0% |
| Phase 4: Animation扩展 | 16 | 0 | 0% |
| Phase 5: CL2加载 | 9 | 0 | 0% |
| Phase 6: 游戏集成 | 15 | 0 | 0% |
| Phase 7: 测试调试 | 43 | 0 | 0% |
| Phase 8: 文档总结 | 21 | 0 | 0% |
| **总计** | **194** | **0** | **0%** |

---

## 🚀 执行说明

### RIPER-5 协议要求

1. **严格按照清单执行** - 不得跳过任何步骤
2. **不得临时决策** - 所有决策已在PLAN阶段完成
3. **发现问题立即停止** - 返回PLAN模式修改计划
4. **完成一项标记一项** - 更新进度追踪表
5. **测试失败不得继续** - 必须修复后才能继续

### 每日检查点

**Day 1 结束:**
- [ ] Phase 1 完成 (40/40)
- [ ] 所有测试通过
- [ ] 提交代码

**Day 2 结束:**
- [ ] Phase 2 完成 (63/63)
- [ ] 光照可见
- [ ] 提交代码

**Day 3 结束:**
- [ ] Phase 3-4 完成 (87/87)
- [ ] Direction测试通过
- [ ] 提交代码

**Day 4 结束:**
- [ ] Phase 5-6 完成 (111/111)
- [ ] 8方向动画可见
- [ ] 提交代码

**Day 5 结束:**
- [ ] Phase 7-8 完成 (194/194)
- [ ] 所有验收标准通过
- [ ] 文档完成
- [ ] 最终提交

---

**文档版本:** 1.0  
**创建日期:** 2025-12-03  
**状态:** 待执行  
**关联文档:**
- [Step 6.4 设计文档](step-6.4-lighting-and-player-animation.md)
- [master_plan.md](master_plan.md)

