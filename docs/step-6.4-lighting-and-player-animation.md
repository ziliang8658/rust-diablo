# Step 6.4: 光照系统与8方向玩家动画

> **设计日期:** 2025-12-03  
> **预估代码量:** 800-1000行  
> **预估开发时间:** 4-5天

---

## 📋 目标概述

Step 6.4 将实现两个核心系统:

1. **光照系统** - 实现原版Diablo的光照渲染效果
2. **8方向玩家动画系统** - 实现玩家角色的8方向动画显示

### 为什么这两个系统要一起实现?

这两个系统虽然看似独立,但实际上有很强的协同关系:

1. **视觉完整性** - 光照让场景有氛围,8方向动画让角色有生命
2. **测试需求** - 实现光照后,需要玩家移动来测试光照跟随效果
3. **代码量平衡** - 光照系统 ~500行,玩家动画 ~400行,总计约900行,符合每步500-1000行的规划
4. **渐进开发** - 两个系统相对独立,可以并行开发和测试

---

## 🎯 Part 1: 光照系统

### 1.1 核心目标

消除 Step 6.3 遗留的视觉问题:

1. ❌ **菱形边界感较重** → ✅ 通过光照渐变柔化边缘
2. ❌ **小黑三角(86.2%空块)** → ✅ 通过环境光照亮暗部
3. ❌ **整体画面平淡** → ✅ 增强黑暗地牢的压抑感

### 1.2 原版光照系统架构

**参考代码:**
- `Source/lighting.cpp` - 光照计算核心
- `Source/engine/render/scrollrt.cpp::DrawCell()` - 光照应用
- `Source/engine/render/dun_render.cpp` - RenderLine系列函数

#### 核心数据结构

```cpp
// Source/lighting.h
int dLight[MAXDUNX][MAXDUNY];  // 光照强度数组 (0-15)
uint8_t LightTables[MAXLIGHT][256];  // 颜色变换表

struct Light {
    Point position;   // 光源位置
    uint8_t radius;   // 光照半径
    bool off;         // 是否关闭
};

Light VisionList[MAXVISION];  // 光源列表
```

#### 光照计算流程

```
1. DoLighting() - 主循环
   ├─> InitVision() - 初始化视野
   ├─> InitLight() - 初始化光源
   ├─> MakeLightTable() - 生成颜色变换表
   └─> DoVision() - 计算视野
       └─> DoCrawl() - 光线追踪算法

2. 每帧更新:
   ├─> UpdateVision() - 更新视野范围
   └─> ApplyLighting() - 应用光照到渲染
       └─> dLight[x][y] → LightTables[level][color]
```

---

### 1.3 Rust实现设计

#### 模块结构

```
src/
├── lighting/
│   ├── mod.rs              # 光照系统入口
│   ├── light_source.rs     # 光源结构
│   ├── light_table.rs      # 颜色变换表
│   ├── vision.rs           # 视野系统
│   └── crawl.rs            # 光线追踪
│
└── world/
    └── mod.rs              # 集成光照渲染
```

#### 核心数据结构

```rust
// src/lighting/mod.rs

/// 光照系统 - 管理场景光照
pub struct LightingSystem {
    /// 光照强度数组 (MicroTile 级别)
    /// 值范围: 0 (完全黑暗) - 15 (完全明亮)
    pub light_grid: [[u8; MAXDUNY]; MAXDUNX],
    
    /// 光源列表
    pub light_sources: Vec<LightSource>,
    
    /// 颜色变换表 (16档亮度 × 256色)
    pub light_tables: LightTables,
    
    /// 环境光等级 (0-15)
    pub ambient_light: u8,
}

impl LightingSystem {
    pub fn new() -> Self { ... }
    
    /// 添加光源 (玩家、火把等)
    pub fn add_light(&mut self, source: LightSource) { ... }
    
    /// 移除光源
    pub fn remove_light(&mut self, id: usize) { ... }
    
    /// 更新光照 (每帧调用)
    pub fn update(&mut self) { ... }
    
    /// 计算视野和光照
    fn calculate_lighting(&mut self) { ... }
    
    /// 获取指定位置的光照等级
    pub fn get_light_level(&self, x: usize, y: usize) -> u8 { ... }
    
    /// 应用光照到颜色 (palette index → darkened palette index)
    pub fn apply_lighting(&self, color_index: u8, light_level: u8) -> u8 { ... }
}
```

#### 光源结构

```rust
// src/lighting/light_source.rs

/// 光源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Player,      // 玩家光环
    Torch,       // 火把
    Spell,       // 法术光效
    Monster,     // 发光怪物
}

/// 光源
#[derive(Debug, Clone)]
pub struct LightSource {
    pub id: usize,
    pub light_type: LightType,
    pub position: (usize, usize),  // MicroTile坐标
    pub radius: u8,                 // 光照半径 (1-15)
    pub active: bool,               // 是否激活
}

impl LightSource {
    /// 创建玩家光源
    pub fn player(position: (usize, usize)) -> Self {
        Self {
            id: 0,
            light_type: LightType::Player,
            position,
            radius: 10,  // 原版玩家光照半径
            active: true,
        }
    }
    
    /// 创建火把光源
    pub fn torch(position: (usize, usize)) -> Self {
        Self {
            id: 0,
            light_type: LightType::Torch,
            position,
            radius: 8,
            active: true,
        }
    }
}
```

#### 颜色变换表

```rust
// src/lighting/light_table.rs

/// 光照颜色变换表
/// 
/// LightTables[level][color_index] = darkened_color_index
/// 
/// - level: 0 (完全黑暗) - 15 (完全明亮)
/// - color_index: 原始调色板索引 (0-255)
/// - darkened_color_index: 变暗后的调色板索引
pub struct LightTables {
    tables: [[u8; 256]; 16],
}

impl LightTables {
    /// 从调色板生成光照表
    pub fn from_palette(palette: &Palette) -> Self {
        let mut tables = [[0u8; 256]; 16];
        
        for level in 0..16 {
            // level 0: 完全黑暗 (全部映射到黑色)
            // level 15: 完全明亮 (保持原色)
            let brightness = level as f32 / 15.0;
            
            for color_idx in 0..256 {
                let color = palette.colors[color_idx];
                let darkened = Self::darken_color(color, brightness);
                tables[level][color_idx] = palette.find_nearest_color(darkened);
            }
        }
        
        Self { tables }
    }
    
    /// 获取变暗后的颜色索引
    pub fn get(&self, light_level: u8, color_index: u8) -> u8 {
        let level = light_level.min(15) as usize;
        self.tables[level][color_index as usize]
    }
    
    /// 将颜色按亮度变暗
    fn darken_color(color: (u8, u8, u8), brightness: f32) -> (u8, u8, u8) {
        (
            (color.0 as f32 * brightness) as u8,
            (color.1 as f32 * brightness) as u8,
            (color.2 as f32 * brightness) as u8,
        )
    }
}
```

#### 光线追踪算法

```rust
// src/lighting/crawl.rs

/// 光线追踪 - 计算光源照亮的区域
/// 
/// 参考: Source/lighting.cpp::DoCrawl()
pub fn crawl_light(
    grid: &mut [[u8; MAXDUNY]; MAXDUNX],
    source: &LightSource,
    block_map: &[[bool; MAXDUNY]; MAXDUNX],  // 墙壁阻挡map
) {
    let (cx, cy) = source.position;
    let radius = source.radius as i32;
    
    // 清空之前的光照
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = (cx as i32 + dx) as usize;
            let y = (cy as i32 + dy) as usize;
            
            if x < MAXDUNX && y < MAXDUNY {
                grid[x][y] = 0;
            }
        }
    }
    
    // 从中心向外辐射光线
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            
            if dist > radius as f32 {
                continue;
            }
            
            let x = (cx as i32 + dx) as usize;
            let y = (cy as i32 + dy) as usize;
            
            if x >= MAXDUNX || y >= MAXDUNY {
                continue;
            }
            
            // 检查光线路径是否被阻挡
            if is_line_clear(cx, cy, x, y, block_map) {
                // 计算光照强度 (距离越远越暗)
                let intensity = 15.0 * (1.0 - dist / radius as f32);
                grid[x][y] = intensity.max(0.0).min(15.0) as u8;
            }
        }
    }
}

/// Bresenham直线算法 - 检查光线是否被阻挡
fn is_line_clear(
    x0: usize, y0: usize,
    x1: usize, y1: usize,
    block_map: &[[bool; MAXDUNY]; MAXDUNX],
) -> bool {
    // Bresenham's line algorithm
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    
    loop {
        // 检查当前点
        if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
            if block_map[x as usize][y as usize] {
                return false;  // 被墙壁阻挡
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
}
```

#### 集成到渲染系统

```rust
// src/world/mod.rs

impl World {
    /// 渲染带光照的瓦片
    fn render_tile_with_lighting(
        &self,
        engine: &mut Engine,
        lighting: &LightingSystem,
        micro_x: usize,
        micro_y: usize,
        tile_type: TileType,
        cel_frame: &DungeonCelFrame,
    ) -> Result<()> {
        // 1. 解码瓦片像素 (indexed colors)
        let indexed_pixels = decode_tile(tile_type, &cel_frame.raw_data)?;
        
        // 2. 获取光照等级
        let light_level = lighting.get_light_level(micro_x, micro_y);
        
        // 3. 应用光照到像素
        let lit_pixels: Vec<u8> = indexed_pixels
            .iter()
            .map(|&idx| lighting.apply_lighting(idx, light_level))
            .collect();
        
        // 4. 转换为RGBA并渲染
        let rgba_pixels = self.palette.indexed_to_rgba(&lit_pixels, true);
        
        // ... 渲染到屏幕
        
        Ok(())
    }
}
```

---

### 1.4 实现步骤

#### Phase 1: 核心光照系统 (2天)

**任务清单:**
1. ✅ 创建 `src/lighting/` 模块
2. ✅ 实现 `LightingSystem` 核心结构
3. ✅ 实现 `LightSource` 光源管理
4. ✅ 实现 `LightTables` 颜色变换表
5. ✅ 编写单元测试

**验收标准:**
- [ ] 能够创建光源
- [ ] 能够计算光照强度
- [ ] 能够生成颜色变换表
- [ ] 单元测试通过 (≥80%覆盖率)

#### Phase 2: 光线追踪 (1天)

**任务清单:**
1. ✅ 实现 `crawl_light()` 光线追踪
2. ✅ 实现 `is_line_clear()` 阻挡检测
3. ✅ 集成墙壁碰撞map
4. ✅ 测试光照衰减效果

**验收标准:**
- [ ] 光照正确衰减 (中心亮,边缘暗)
- [ ] 墙壁正确阻挡光线
- [ ] 性能可接受 (<5ms per frame)

#### Phase 3: 渲染集成 (1天)

**任务清单:**
1. ✅ 修改 `World::render_tile()` 应用光照
2. ✅ 添加环境光控制
3. ✅ 玩家光源跟随移动
4. ✅ 调试视觉效果

**验收标准:**
- [ ] 瓦片正确变暗
- [ ] 玩家周围有光环
- [ ] 边界柔化 (视觉改善)
- [ ] 小黑三角消失

---

## 🎯 Part 2: 8方向玩家动画系统

### 2.1 核心目标

实现玩家角色的8方向动画显示:

1. ✅ 玩家朝向与移动方向同步
2. ✅ 8个方向各有独立动画帧
3. ✅ 动画切换流畅自然
4. ✅ 支持站立和行走两种状态

### 2.2 原版动画系统架构

**参考代码:**
- `Source/player.cpp::LoadPlrGFX()` - 玩家精灵加载
- `Source/player.cpp::NewPlrAnim()` - 动画切换
- `Source/playerdat.hpp` - 玩家数据定义
- `Source/engine/animationinfo.h` - 动画系统

#### CL2文件格式 (8方向精灵)

```
CL2文件结构 (以 wmnaw.cl2 为例):
┌──────────────────────────────────┐
│ Header                           │
│  - num_animations: 8 (8个方向)  │
│  - frame_offsets[9]              │
├──────────────────────────────────┤
│ Animation 0: South (↓)           │
│  - frames: 8帧                   │
│  - 每帧: 96×96像素 (RLE压缩)    │
├──────────────────────────────────┤
│ Animation 1: SouthWest (↙)       │
│  - frames: 8帧                   │
├──────────────────────────────────┤
│ ... (其他6个方向)                │
└──────────────────────────────────┘

方向顺序 (Direction enum):
0: South      (↓)
1: SouthWest  (↙)
2: West       (←)
3: NorthWest  (↖)
4: North      (↑)
5: NorthEast  (↗)
6: East       (→)
7: SouthEast  (↘)
```

#### 动画系统数据流

```
1. 加载阶段:
   LoadPlrGFX() 
   └─> CL2Sprite::from_file("wmnaw.cl2", 96)
       └─> 解析8个Animation
           └─> 每个Animation有8帧

2. 运行时切换:
   Player::set_direction(Direction::North)
   └─> animation_controller.set_direction(Direction::North)
       └─> 选择对应的Animation (index 4)
           └─> 播放该方向的8帧

3. 每帧更新:
   Player::update(dt)
   └─> animation_controller.update(dt)
       └─> 当前帧 = (当前帧 + 1) % 8
```

---

### 2.3 Rust实现设计

#### 扩展 Direction 系统

```rust
// src/engine/direction.rs

impl Direction {
    /// 转换为CL2动画索引 (0-7)
    /// 
    /// 对应原版 `Source/player.h::GetPlayerAnimData()`
    pub fn to_animation_index(&self) -> usize {
        match self {
            Direction::South => 0,      // ↓
            Direction::SouthWest => 1,  // ↙
            Direction::West => 2,       // ←
            Direction::NorthWest => 3,  // ↖
            Direction::North => 4,      // ↑
            Direction::NorthEast => 5,  // ↗
            Direction::East => 6,       // →
            Direction::SouthEast => 7,  // ↘
            Direction::None => 0,       // 默认朝南
        }
    }
    
    /// 从动画索引创建Direction
    pub fn from_animation_index(index: usize) -> Self {
        match index {
            0 => Direction::South,
            1 => Direction::SouthWest,
            2 => Direction::West,
            3 => Direction::NorthWest,
            4 => Direction::North,
            5 => Direction::NorthEast,
            6 => Direction::East,
            7 => Direction::SouthEast,
            _ => Direction::South,
        }
    }
}
```

#### 扩展动画控制器

```rust
// src/sprite/animation.rs

/// 动画控制器 - 支持8方向
pub struct AnimationController {
    /// 动画集合 (状态 → 8方向动画)
    animations: HashMap<AnimationState, DirectionalAnimation>,
    
    /// 当前动画状态
    current_state: AnimationState,
    
    /// 当前方向
    current_direction: Direction,
    
    /// 当前帧
    current_frame: usize,
    
    /// 帧计时器
    frame_timer: f32,
}

/// 8方向动画
pub struct DirectionalAnimation {
    /// 8个方向的动画 (South, SW, W, NW, N, NE, E, SE)
    directions: [Animation; 8],
    
    /// 是否循环
    looping: bool,
}

impl DirectionalAnimation {
    /// 创建8方向动画
    pub fn new(animations: [Animation; 8], looping: bool) -> Self {
        Self {
            directions: animations,
            looping,
        }
    }
    
    /// 获取指定方向的动画
    pub fn get_animation(&self, direction: Direction) -> &Animation {
        let index = direction.to_animation_index();
        &self.directions[index]
    }
}

impl AnimationController {
    /// 设置方向 (切换到对应方向的动画)
    pub fn set_direction(&mut self, direction: Direction) {
        if direction != self.current_direction {
            self.current_direction = direction;
            self.current_frame = 0;  // 重置帧
            self.frame_timer = 0.0;
        }
    }
    
    /// 获取当前帧的纹理区域
    pub fn current_frame_rect(&self) -> Option<Rect> {
        let anim = self.animations.get(&self.current_state)?;
        let dir_anim = anim.get_animation(self.current_direction);
        
        if self.current_frame < dir_anim.frames.len() {
            Some(dir_anim.frames[self.current_frame])
        } else {
            None
        }
    }
}
```

#### CL2加载器扩展

```rust
// src/resources/cl2.rs

impl CL2Sprite {
    /// 加载8方向CL2精灵
    /// 
    /// 返回: Vec<Vec<Frame>> - 8个方向,每个方向多帧
    pub fn load_directional(
        data: &[u8],
        frame_width: usize,
    ) -> Result<Vec<Vec<Frame>>> {
        // 1. 解析header
        let num_animations = 8;  // 固定8方向
        
        // 2. 读取offset表
        let offsets = Self::read_offsets(data, num_animations)?;
        
        // 3. 解析每个方向的动画
        let mut directional_frames = Vec::new();
        
        for i in 0..num_animations {
            let start = offsets[i];
            let end = offsets[i + 1];
            let anim_data = &data[start..end];
            
            // 解析该方向的所有帧
            let frames = Self::decode_animation(anim_data, frame_width)?;
            directional_frames.push(frames);
        }
        
        Ok(directional_frames)
    }
}
```

#### 集成到Entity

```rust
// src/entity/mod.rs

impl Entity {
    /// 创建玩家 (8方向动画版本)
    pub fn create_player_8dir(position: Point) -> Self {
        let mut entity = Self::new(
            EntityType::Player,
            position,
            (96, 96),
            Color::CYAN,
        );
        
        entity.use_sprite = true;
        entity.sprite_id = Some("warrior_town".to_string());
        entity.speed = 200.0;
        
        // 初始化8方向动画控制器
        // 注意: 实际帧数据在game.rs中加载后填充
        let mut anim_controller = AnimationController::new_directional();
        
        // Idle: 8个方向各1帧 (占位符)
        let idle_8dir = DirectionalAnimation::new(
            [
                Animation::new(vec![Rect::new(0, 0, 96, 96)], 0.15, true),  // S
                Animation::new(vec![Rect::new(0, 0, 96, 96)], 0.15, true),  // SW
                Animation::new(vec![Rect::new(0, 0, 96, 96)], 0.15, true),  // W
                Animation::new(vec![Rect::new(0, 0, 96, 96)], 0.15, true),  // NW
                Animation::new(vec![Rect::new(0, 0, 96, 96)], 0.15, true),  // N
                Animation::new(vec![Rect::new(0, 0, 96, 96)], 0.15, true),  // NE
                Animation::new(vec![Rect::new(0, 0, 96, 96)], 0.15, true),  // E
                Animation::new(vec![Rect::new(0, 0, 96, 96)], 0.15, true),  // SE
            ],
            true,
        );
        anim_controller.add_directional_animation(AnimationState::Idle, idle_8dir);
        
        // Walk: 8个方向各8帧 (占位符)
        let walk_8dir = DirectionalAnimation::new(
            [
                Animation::new(vec![Rect::new(0, 0, 96, 96); 8], 0.1, true),  // S
                Animation::new(vec![Rect::new(0, 0, 96, 96); 8], 0.1, true),  // SW
                Animation::new(vec![Rect::new(0, 0, 96, 96); 8], 0.1, true),  // W
                Animation::new(vec![Rect::new(0, 0, 96, 96); 8], 0.1, true),  // NW
                Animation::new(vec![Rect::new(0, 0, 96, 96); 8], 0.1, true),  // N
                Animation::new(vec![Rect::new(0, 0, 96, 96); 8], 0.1, true),  // NE
                Animation::new(vec![Rect::new(0, 0, 96, 96); 8], 0.1, true),  // E
                Animation::new(vec![Rect::new(0, 0, 96, 96); 8], 0.1, true),  // SE
            ],
            true,
        );
        anim_controller.add_directional_animation(AnimationState::Walk, walk_8dir);
        
        anim_controller.set_state(AnimationState::Idle);
        anim_controller.set_direction(Direction::South);  // 默认朝南
        
        entity.animation = Some(anim_controller);
        
        entity
    }
    
    /// 更新方向和动画
    pub fn update_with_direction(&mut self, dt: f32, new_direction: Direction) {
        // 更新方向
        if new_direction != self.direction {
            self.set_direction(new_direction);
            
            // 切换动画状态
            if new_direction.is_moving() {
                if let Some(ref mut anim) = self.animation {
                    anim.set_state(AnimationState::Walk);
                }
            } else {
                if let Some(ref mut anim) = self.animation {
                    anim.set_state(AnimationState::Idle);
                }
            }
            
            // 同步动画方向
            if let Some(ref mut anim) = self.animation {
                anim.set_direction(self.direction);
            }
        }
        
        // 更新位置和动画
        self.update(dt, None);
    }
}
```

---

### 2.4 实现步骤

#### Phase 1: Direction系统扩展 (半天)

**任务清单:**
1. ✅ 扩展 `Direction::to_animation_index()`
2. ✅ 扩展 `Direction::from_animation_index()`
3. ✅ 编写单元测试

**验收标准:**
- [ ] 8个方向正确映射到索引0-7
- [ ] 往返转换一致
- [ ] 单元测试通过

#### Phase 2: 动画控制器扩展 (1天)

**任务清单:**
1. ✅ 实现 `DirectionalAnimation`
2. ✅ 扩展 `AnimationController` 支持方向
3. ✅ 实现方向切换逻辑
4. ✅ 编写单元测试

**验收标准:**
- [ ] 能够存储8方向动画
- [ ] 能够切换方向
- [ ] 动画状态和方向独立控制
- [ ] 单元测试通过

#### Phase 3: CL2加载器扩展 (半天)

**任务清单:**
1. ✅ 实现 `CL2Sprite::load_directional()`
2. ✅ 测试加载 `wmnaw.cl2`
3. ✅ 验证8个方向各有8帧

**验收标准:**
- [ ] 成功加载8方向CL2
- [ ] 帧数正确 (8方向×8帧=64帧)
- [ ] 帧尺寸正确 (96×96)

#### Phase 4: 游戏集成 (1天)

**任务清单:**
1. ✅ 修改 `game.rs` 加载8方向精灵
2. ✅ 修改输入处理同步方向
3. ✅ 测试8方向移动和动画
4. ✅ 调试和优化

**验收标准:**
- [ ] 玩家移动时方向正确
- [ ] 动画切换流畅
- [ ] 停止时切换到Idle
- [ ] 8个方向视觉正确

---

## 📊 综合实施计划

### 开发时间表

| 天数 | Part 1: 光照系统 | Part 2: 8方向动画 | 集成测试 |
|-----|------------------|------------------|---------|
| Day 1 | 核心光照系统 | - | - |
| Day 2 | 光线追踪 | Direction扩展 | - |
| Day 3 | 渲染集成 | 动画控制器扩展 | - |
| Day 4 | - | CL2加载+游戏集成 | 联调 |
| Day 5 | - | - | 测试+文档 |

### 代码量估算

| 模块 | 文件 | 行数 |
|-----|------|------|
| **Part 1: 光照系统** | | |
| `lighting/mod.rs` | 核心系统 | 200 |
| `lighting/light_source.rs` | 光源 | 80 |
| `lighting/light_table.rs` | 颜色表 | 120 |
| `lighting/crawl.rs` | 光线追踪 | 100 |
| **Part 2: 8方向动画** | | |
| `engine/direction.rs` | 方向扩展 | 50 |
| `sprite/animation.rs` | 动画控制器 | 150 |
| `resources/cl2.rs` | CL2加载 | 80 |
| `entity/mod.rs` | Entity集成 | 60 |
| **集成** | | |
| `world/mod.rs` | 光照渲染 | 80 |
| `game.rs` | 游戏集成 | 100 |
| **测试** | | |
| 单元测试 | 各模块测试 | 150 |
| **总计** | | **~1170行** |

---

## 🧪 测试方案

### Part 1: 光照系统测试

#### 单元测试

```rust
// tests/lighting_test.rs

#[test]
fn test_light_source_creation() {
    let source = LightSource::player((50, 50));
    assert_eq!(source.position, (50, 50));
    assert_eq!(source.radius, 10);
    assert!(source.active);
}

#[test]
fn test_light_tables_generation() {
    let palette = Palette::load_default().unwrap();
    let tables = LightTables::from_palette(&palette);
    
    // Level 0: 全黑
    assert_eq!(tables.get(0, 128), 0);  // 应该映射到黑色
    
    // Level 15: 全亮
    assert_eq!(tables.get(15, 128), 128);  // 保持原色
}

#[test]
fn test_crawl_light_attenuation() {
    let mut grid = [[0u8; MAXDUNY]; MAXDUNX];
    let source = LightSource::player((50, 50));
    let block_map = [[false; MAXDUNY]; MAXDUNX];
    
    crawl_light(&mut grid, &source, &block_map);
    
    // 中心亮度最高
    assert!(grid[50][50] > 10);
    
    // 边缘亮度衰减
    assert!(grid[60][50] < grid[55][50]);
}

#[test]
fn test_light_blocking() {
    let mut grid = [[0u8; MAXDUNY]; MAXDUNX];
    let source = LightSource::player((50, 50));
    let mut block_map = [[false; MAXDUNY]; MAXDUNX];
    
    // 放置墙壁
    block_map[55][50] = true;
    
    crawl_light(&mut grid, &source, &block_map);
    
    // 墙壁后面应该是暗的
    assert!(grid[60][50] < 5);
}
```

#### 集成测试

```rust
// tests/lighting_integration.rs

#[test]
fn test_player_light_follows_movement() {
    let mut game = Game::new().unwrap();
    
    // 初始位置
    let initial_pos = game.player.position;
    let initial_light = game.lighting.light_grid[50][50];
    
    // 移动玩家
    game.player.position = Point::new(initial_pos.x + 10, initial_pos.y);
    game.lighting.update();
    
    // 光源应该跟随
    assert!(game.lighting.light_grid[60][50] > initial_light);
}
```

### Part 2: 8方向动画测试

#### 单元测试

```rust
// tests/direction_test.rs

#[test]
fn test_direction_to_animation_index() {
    assert_eq!(Direction::South.to_animation_index(), 0);
    assert_eq!(Direction::SouthWest.to_animation_index(), 1);
    assert_eq!(Direction::NorthEast.to_animation_index(), 5);
}

#[test]
fn test_animation_direction_switching() {
    let mut controller = AnimationController::new_directional();
    
    // 初始朝南
    controller.set_direction(Direction::South);
    assert_eq!(controller.current_direction, Direction::South);
    
    // 切换到北
    controller.set_direction(Direction::North);
    assert_eq!(controller.current_direction, Direction::North);
    assert_eq!(controller.current_frame, 0);  // 帧应该重置
}
```

#### 手动测试

```
1. 启动游戏
2. 按方向键移动玩家
3. 验证:
   - 向上移动 → 玩家朝北
   - 向右上移动 → 玩家朝东北
   - 停止移动 → 切换到Idle动画
   - 8个方向都测试一遍
```

### 性能测试

```rust
#[bench]
fn bench_light_calculation(b: &mut Bencher) {
    let mut lighting = LightingSystem::new();
    lighting.add_light(LightSource::player((50, 50)));
    
    b.iter(|| {
        lighting.update();
    });
}

// 目标: < 5ms per frame
```

---

## 📚 学习要点

### 1. 光照算法

**Bresenham直线算法:**
- 用于判断光线是否被阻挡
- 整数运算,高效
- 广泛应用于图形学

**光线追踪 (Ray Casting):**
- 从光源向外辐射
- 距离衰减公式: `intensity = 1 - (distance / radius)`
- 阻挡检测

**颜色变换表:**
- 预计算优化: 运行时只需查表,不需要计算
- 空间换时间: 16档×256色=4KB内存
- 视觉效果: 平滑的明暗过渡

### 2. 8方向动画系统

**方向映射:**
- 从逻辑方向到视觉方向
- 顺时针顺序: S → SW → W → NW → N → NE → E → SE
- 索引映射: 0-7

**动画状态机:**
- 状态: Idle, Walk, Attack, ...
- 方向: 8个方向
- 二维状态: (状态, 方向) → 动画帧序列

**帧序列:**
- 每个方向8帧 (walk)
- 总计: 8方向 × 8帧 = 64帧
- 内存: ~6MB (64帧 × 96×96像素 × 4字节RGBA)

### 3. 性能优化

**光照系统:**
- 只计算视口内的光照
- 使用预计算的颜色表
- 光源数量限制 (<20)

**动画系统:**
- 纹理图集 (Texture Atlas)
- 批量渲染
- 只渲染当前方向的帧

---

## ⚠️ 踩坑点

### 1. 光照系统

**问题1: 颜色表精度损失**
- 症状: 颜色变暗后出现色带
- 原因: 调色板256色限制
- 解决: 使用dithering算法

**问题2: 光照闪烁**
- 症状: 光照边缘闪烁
- 原因: 整数坐标四舍五入
- 解决: 使用浮点计算 + 平滑过渡

**问题3: 墙壁光照bleeding**
- 症状: 光线穿透墙壁
- 原因: Bresenham算法采样不足
- 解决: 原版使用特殊的bleeding-up算法

### 2. 8方向动画

**问题1: 方向抖动**
- 症状: 玩家频繁切换方向
- 原因: 输入噪声
- 解决: 添加方向锁定时间 (0.1s)

**问题2: 动画不同步**
- 症状: 切换方向时动画跳帧
- 原因: 帧计数器未重置
- 解决: 切换方向时 `current_frame = 0`

**问题3: 内存占用大**
- 症状: 8方向动画占用大量内存
- 原因: 64帧 × 96×96 × 4字节
- 解决: 使用纹理压缩或按需加载

---

## 📖 原版代码参考

### 光照系统

| 功能 | 原版文件 | 行号 | Rust实现 |
|-----|---------|------|----------|
| 光照数组 | `lighting.cpp` | 15-20 | `lighting/mod.rs` |
| 光源管理 | `lighting.cpp::DoLighting()` | 56-89 | `LightingSystem::update()` |
| 光线追踪 | `lighting.cpp::DoCrawl()` | 124-156 | `lighting/crawl.rs` |
| 颜色变换表 | `lighting.cpp::MakeLightTable()` | 201-234 | `light_table.rs` |
| 光照渲染 | `scrollrt.cpp::DrawCell()` | 588-611 | `world/mod.rs` |

### 8方向动画

| 功能 | 原版文件 | 行号 | Rust实现 |
|-----|---------|------|----------|
| 方向枚举 | `engine/direction.hpp` | 13-24 | `engine/direction.rs` |
| CL2加载 | `load_cl2.cpp` | 35-78 | `resources/cl2.rs` |
| 动画切换 | `player.cpp::NewPlrAnim()` | 456-489 | `entity/mod.rs` |
| 方向更新 | `player.cpp::StartWalk()` | 523-556 | `entity/update_with_direction()` |

---

## 🎯 验收标准

### Part 1: 光照系统 ✅

- [ ] **核心功能**
  - [ ] 能够创建和管理光源
  - [ ] 能够计算光照强度
  - [ ] 能够生成颜色变换表
  - [ ] 能够应用光照到渲染

- [ ] **视觉效果**
  - [ ] 玩家周围有光环 (半径10瓦片)
  - [ ] 墙壁正确阻挡光线
  - [ ] 光照平滑衰减
  - [ ] 菱形边界感消失
  - [ ] 小黑三角消失

- [ ] **性能**
  - [ ] 光照计算 < 5ms/frame
  - [ ] FPS保持 ≥30
  - [ ] 无卡顿

### Part 2: 8方向动画 ✅

- [ ] **核心功能**
  - [ ] 支持8个方向
  - [ ] Idle和Walk两种状态
  - [ ] 方向切换流畅
  - [ ] 动画循环正确

- [ ] **视觉效果**
  - [ ] 玩家朝向与移动方向一致
  - [ ] 8个方向视觉正确
  - [ ] 动画帧率合适 (10 FPS)
  - [ ] 停止时切换到Idle

- [ ] **交互**
  - [ ] 方向键控制方向
  - [ ] 对角线移动正确
  - [ ] 动画与移动同步

---

## 📝 练习题

### 基础练习

1. **光照可视化工具** (入门)
   - 创建工具显示light_grid数组
   - 用颜色表示光照强度
   - 实时显示光源位置

2. **方向指示器** (入门)
   - 在玩家头顶显示当前方向
   - 用箭头或文字表示
   - 切换方向时高亮

### 进阶练习

3. **动态光源** (进阶)
   - 添加火把光源
   - 火把会闪烁 (半径在8-10之间波动)
   - 放置多个火把测试性能

4. **光照预设** (进阶)
   - 添加光照强度预设 (白天/黄昏/夜晚)
   - F1-F3快捷键切换
   - 平滑过渡效果

### 创意练习

5. **法术光效** (创意)
   - 火球术: 移动的橙色光源
   - 闪电术: 蓝白色闪光
   - 治疗术: 绿色光环

6. **阴影系统** (高挑战)
   - 实现hard shadow (完全黑暗)
   - 墙壁投射阴影
   - 参考: Shadow Mapping算法

### 图形学练习

7. **光照平滑** (图形学)
   - 实现双线性插值
   - 光照边缘更平滑
   - 避免锯齿效果

8. **动态环境光** (图形学)
   - 根据地牢深度调整环境光
   - 1层最亮,16层最暗
   - 模拟下降到地狱的感觉

---

## 🔜 下一步计划 (Step 6.5)

完成 Step 6.4 后,下一步是 **Step 6.5: 地牢特殊对象 (dSpecial)**

**核心功能:**
- 门、楼梯、宝箱等特殊对象
- 对象交互系统
- 对象动画 (门开关等)
- 对象渲染 (叠加在地图上)

**参考代码:**
- `Source/objects.cpp` - 对象系统
- `Source/gendung.cpp::LoadSetMap()` - 特殊地图

---

## 📌 总结

Step 6.4 完成后,游戏将具备:

1. **完整的视觉体验**
   - ✅ 动态光照系统
   - ✅ 8方向角色动画
   - ✅ 平滑的明暗过渡
   - ✅ 沉浸式的地牢氛围

2. **技术里程碑**
   - ✅ 光线追踪算法
   - ✅ 颜色变换表优化
   - ✅ 8方向动画状态机
   - ✅ CL2多方向精灵加载

3. **可玩性提升**
   - ✅ 玩家移动更真实
   - ✅ 场景氛围更佳
   - ✅ 准备好添加怪物和战斗

**预计开发时间:** 4-5天  
**代码量:** ~1000行  
**测试覆盖率:** ≥80%

---

**文档版本:** 1.0  
**创建日期:** 2025-12-03  
**作者:** AI Assistant  
**关联文档:**
- [Step 6.3 完成总结](step-6.3-completion-summary.md)
- [master_plan.md](master_plan.md)
- [FEATURE_COMPARISON.md](FEATURE_COMPARISON.md)

