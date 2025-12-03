# Step 5.3: TRN颜色转换和城镇场景预览

## 📅 开始日期
2025-11-25

## 🎯 目标

实现TRN颜色转换系统，完善资源管理器，并**实现城镇场景预览**，让玩家可以在城镇场景中自由行走。这是Step 5的最后一个子步骤，完成后将拥有完整的原版资源格式支持。

## 📋 功能清单

### 1. TRN颜色转换 (`resources/trn.rs`) 
**代码量预估：80-100行**

**核心功能：**
- TRN文件加载（256字节映射表）
- 颜色映射应用
- 应用到精灵像素
- 支持多个TRN同时使用（不同变色效果）

**参考代码：**
- `Source/engine/load_file.hpp` - 文件加载（TRN作为通用文件）
- `Source/engine/trn.cpp` - TRN加载和应用（如果有）
- `Source/plrgfx/warrior/*.trn` - 战士TRN文件示例

**技术要点：**
```rust
pub struct ColorTransform {
    pub map: [u8; 256],  // 颜色索引映射表
}

impl ColorTransform {
    // 从字节加载
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    
    // 应用到单个颜色索引
    pub fn apply(&self, color_index: u8) -> u8;
    
    // 应用到像素数组（批量）
    pub fn apply_to_pixels(&self, pixels: &mut [Option<u8>]);
    
    // 应用到RGBA像素（需要反查索引）
    pub fn apply_to_rgba(&self, rgba: &mut [u8], palette: &Palette);
}
```

**使用场景：**
- **怪物变色**：不同颜色的僵尸、骷髅（使用相同精灵+不同TRN）
- **玩家装备颜色**：不同颜色的装备
- **特殊效果**：中毒变绿、冰冻变蓝等

---

### 2. 资源管理器完善 (`resources/mod.rs`)
**代码量预估：250-300行**

**核心功能：**
- 统一资源加载接口
- 资源缓存管理（LRU缓存）
- MPQ管理器集成
- 调色板管理器
- 纹理缓存（避免重复创建SDL纹理）
- TRN缓存

**架构设计：**
```rust
pub struct ResourceManager {
    // MPQ系统
    mpq_manager: MpqManager,
    
    // 资源缓存
    palette_cache: HashMap<String, Arc<Palette>>,
    sprite_cache: HashMap<String, Arc<Vec<ClxFrame>>>,
    trn_cache: HashMap<String, Arc<ColorTransform>>,
    
    // 纹理ID映射（资源路径 -> 纹理ID列表）
    texture_id_map: HashMap<String, Vec<String>>,
}

impl ResourceManager {
    pub fn new(mpq_paths: Vec<(&str, i32)>) -> Result<Self>;
    
    // 基础资源加载
    pub fn load_palette(&mut self, path: &str) -> Result<Arc<Palette>>;
    pub fn load_trn(&mut self, path: &str) -> Result<Arc<ColorTransform>>;
    pub fn load_cl2(&mut self, path: &str, width: u16) -> Result<Arc<Vec<ClxFrame>>>;
    pub fn load_clx(&mut self, path: &str) -> Result<Arc<Vec<ClxFrame>>>;
    pub fn load_pcx(&mut self, path: &str) -> Result<PcxImage>;
    
    // 高级接口：加载并创建纹理
    pub fn load_sprite_textures(
        &mut self,
        engine: &mut Engine,
        sprite_path: &str,
        palette_path: &str,
        sprite_name: &str,
        state_name: &str,
    ) -> Result<Vec<String>>;  // 返回纹理ID列表
    
    // 应用TRN到精灵
    pub fn load_sprite_with_trn(
        &mut self,
        engine: &mut Engine,
        sprite_path: &str,
        palette_path: &str,
        trn_path: &str,
        sprite_name: &str,
        state_name: &str,
    ) -> Result<Vec<String>>;
    
    // 缓存管理
    pub fn clear_cache(&mut self);
    pub fn cache_stats(&self) -> String;
}
```

**参考代码：**
- `Source/engine/assets.cpp` - 资源管理
- `Source/engine/load_file.hpp` - 文件加载接口

---

### 3. 城镇场景系统 (`world/town.rs`, `game.rs`)
**代码量预估：150-200行**

**目标：** 创建一个可行走的城镇场景预览

**实现方案：PCX背景 + 简化碰撞地图（方案3：混合方案）**

#### 3.1 城镇背景
- 使用PCX图像作为静态背景
- 加载城镇调色板（`levels/towndata/town.pal`）
- 渲染到屏幕中央

**可能的城镇背景资源：**
- 尝试查找MPQ中的城镇背景图（ui_art/town*.pcx 或 levels/towndata/*.pcx）
- 如果没有完整背景，可以使用logo.pcx作为测试背景

#### 3.2 玩家精灵
- 使用原版战士CL2精灵
- 支持多方向（如果资源中有，先实现单方向）
- 支持多种动画状态（Idle, Walk）

#### 3.3 简化碰撞系统
```rust
pub struct SimpleTown {
    // 背景
    background_texture_id: Option<String>,
    background_width: u32,
    background_height: u32,
    
    // 可行走区域（简化碰撞地图）
    walkable_area: Rectangle,  // 定义可行走的矩形区域
    
    // 或使用简单的障碍物列表
    obstacles: Vec<Rectangle>,
}

impl SimpleTown {
    pub fn new() -> Self;
    
    // 检查位置是否可行走
    pub fn is_walkable(&self, x: f32, y: f32) -> bool;
    
    // 渲染背景
    pub fn render_background(&self, engine: &mut Engine) -> Result<()>;
}
```

#### 3.4 相机系统简化
- 背景固定在屏幕中央
- 玩家在屏幕中移动
- **或**：背景跟随玩家移动（滚动背景）

**实现选择：**
- **方案A（简单）**：背景固定，玩家在屏幕内移动
- **方案B（推荐）**：相机跟随玩家，背景有限滚动

#### 3.5 游戏集成
```rust
// 在 Game 结构体中
pub struct Game {
    // ... 现有字段
    town_scene: Option<SimpleTown>,
    current_scene: SceneType,
}

enum SceneType {
    TestWorld,   // 当前的测试场景
    TownPreview, // 新的城镇预览场景
}

impl Game {
    pub fn load_town_scene(&mut self) -> Result<()> {
        // 1. 加载城镇调色板
        let palette = self.resource_manager.load_palette("levels/towndata/town.pal")?;
        
        // 2. 尝试加载城镇背景（如果有）
        // let bg = self.resource_manager.load_pcx("levels/towndata/town.pcx")?;
        
        // 3. 加载玩家精灵（战士）
        let texture_ids = self.resource_manager.load_sprite_textures(
            &mut self.engine,
            "plrgfx/warrior/wmn/wmnas.cl2",  // 站立
            "levels/towndata/town.pal",
            "warrior",
            "idle",
        )?;
        
        // 4. 加载行走动画
        let walk_texture_ids = self.resource_manager.load_sprite_textures(
            &mut self.engine,
            "plrgfx/warrior/wmn/wmnaw.cl2",  // 行走
            "levels/towndata/town.pal",
            "warrior",
            "walk",
        )?;
        
        // 5. 创建城镇场景
        self.town_scene = Some(SimpleTown::new());
        self.current_scene = SceneType::TownPreview;
        
        Ok(())
    }
    
    pub fn switch_scene(&mut self, scene: SceneType) {
        self.current_scene = scene;
    }
}
```

---

### 4. 完整游戏集成和测试场景
**代码量预估：100-150行**

#### 4.1 场景切换系统
- 按键切换场景（如F1 = 测试世界，F2 = 城镇预览）
- 不同场景使用不同的渲染和更新逻辑

#### 4.2 TRN测试场景
创建一个测试场景展示TRN颜色转换效果：
```rust
// examples/test_trn.rs
// 加载战士精灵，应用不同的TRN，显示多个不同颜色的战士
```

#### 4.3 资源管理器测试场景
```rust
// 测试缓存系统
// 测试重复加载同一资源（应使用缓存）
// 测试加载多种资源组合
```

---

## 🏗️ 实现步骤

### 阶段1：TRN颜色转换实现（预计1-2小时）

1. **创建TRN模块** (`resources/trn.rs`)
   - 实现ColorTransform结构体
   - 实现from_bytes加载
   - 实现apply和apply_to_pixels方法

2. **编写单元测试**
   - TRN文件加载测试
   - 颜色映射应用测试
   - 边界情况测试

3. **集成到资源管理器**
   - 添加TRN缓存
   - 实现load_trn方法

4. **测试TRN加载**
   - 从MPQ加载TRN文件
   - 验证映射表正确

---

### 阶段2：资源管理器完善（预计2-3小时）

1. **创建资源管理器** (`resources/mod.rs`)
   - 实现基础结构
   - 集成MPQ管理器
   - 实现缓存系统

2. **实现统一加载接口**
   - load_palette
   - load_trn
   - load_cl2 / load_clx
   - load_pcx

3. **实现高级接口**
   - load_sprite_textures（加载精灵并创建纹理）
   - load_sprite_with_trn（应用TRN的精灵加载）

4. **重构现有代码**
   - 将Game中的资源加载迁移到ResourceManager
   - 简化Game::new()中的加载逻辑

---

### 阶段3：城镇场景实现（预计3-4小时）

1. **创建SimpleTown结构** (`world/town.rs`)
   - 定义数据结构
   - 实现碰撞检测
   - 实现渲染方法

2. **加载城镇资源**
   - 尝试查找城镇背景PCX
   - 加载城镇调色板
   - 加载玩家精灵（使用城镇调色板）

3. **实现场景切换**
   - 定义SceneType枚举
   - 实现场景切换逻辑
   - 按键切换场景（F1/F2）

4. **实现玩家在城镇中移动**
   - 使用现有的玩家移动逻辑
   - 添加碰撞检测（简化版）
   - 确保玩家不会移出可行走区域

5. **实现相机系统（可选）**
   - 相机跟随玩家
   - 背景有限滚动

---

### 阶段4：测试和优化（预计1-2小时）

1. **创建TRN测试场景**
   - 加载战士精灵
   - 应用不同TRN
   - 并排显示多个颜色的战士

2. **测试城镇场景**
   - 玩家移动流畅性
   - 场景切换正常
   - 碰撞检测正确

3. **性能测试**
   - 资源加载时间
   - 缓存命中率
   - 渲染帧率

4. **代码优化和清理**
   - 错误处理完善
   - 代码注释
   - 文档更新

---

## 📝 技术要点和设计决策

### 1. TRN颜色转换原理

**TRN文件格式：**
- **大小**：256字节
- **内容**：调色板索引映射表
- **映射规则**：`new_index = trn[old_index]`

**示例：**
```
TRN文件内容：
索引 0 -> 0   (透明色保持不变)
索引 1 -> 10  (颜色1映射到颜色10)
索引 2 -> 11  (颜色2映射到颜色11)
...
索引 255 -> 255
```

**应用流程：**
```
原始精灵像素 (索引图像)
    ↓
应用TRN映射
    ↓
新索引图像
    ↓
应用调色板
    ↓
RGBA纹理
```

**关键代码：**
```rust
// 应用TRN到精灵帧
pub fn apply_trn_to_frame(
    frame: &ClxFrame,
    trn: &ColorTransform,
) -> ClxFrame {
    let mut new_pixels = frame.pixels.clone();
    
    for pixel in new_pixels.iter_mut() {
        if let Some(index) = pixel {
            *index = trn.apply(*index);
        }
    }
    
    ClxFrame {
        width: frame.width,
        height: frame.height,
        pixels: new_pixels,
    }
}
```

---

### 2. 资源管理器设计模式

**为什么需要资源管理器？**
1. **避免重复加载**：使用缓存减少MPQ读取和解析
2. **统一接口**：所有资源加载使用相同的API
3. **生命周期管理**：控制资源的创建和销毁
4. **内存控制**：实现LRU缓存，防止内存溢出

**缓存策略：**
```rust
// 使用Arc实现共享所有权
// 多个对象可以共享同一个调色板，避免重复加载
let palette = self.palette_cache
    .entry(path.to_string())
    .or_insert_with(|| {
        Arc::new(Palette::from_bytes(&data).unwrap())
    })
    .clone();
```

**LRU缓存（可选，后续优化）：**
```rust
use lru::LruCache;

pub struct ResourceManager {
    texture_cache: LruCache<String, Vec<String>>,
    max_cache_size: usize,
}
```

---

### 3. 城镇场景设计

**为什么是"预览"版本？**
- 完整的地图系统（.DUN, .TIL, .CEL）将在**Step 6**实现
- Step 5.3只需要一个**可行走的场景**，验证资源系统完整性
- 简化实现，专注于资源加载和集成

**设计选择：方案3（混合方案）**

**优点：**
- ✅ 简单易实现
- ✅ 使用PCX背景，视觉效果好
- ✅ 简化碰撞地图，逻辑清晰
- ✅ 为Step 6打下基础

**实现细节：**
```rust
pub struct SimpleTown {
    // 背景纹理
    background_texture_id: Option<String>,
    
    // 背景大小
    bg_width: u32,
    bg_height: u32,
    
    // 可行走区域（矩形）
    walkable_area: Rectangle,
    
    // 障碍物列表（可选，后续添加）
    obstacles: Vec<Rectangle>,
}

impl SimpleTown {
    pub fn new() -> Self {
        Self {
            background_texture_id: None,
            bg_width: 0,
            bg_height: 0,
            // 定义一个中央区域可行走
            walkable_area: Rectangle::new(100, 100, 600, 400),
            obstacles: vec![],
        }
    }
    
    pub fn is_walkable(&self, x: f32, y: f32) -> bool {
        // 检查是否在可行走区域内
        if !self.walkable_area.contains(x as i32, y as i32) {
            return false;
        }
        
        // 检查是否与障碍物碰撞
        for obstacle in &self.obstacles {
            if obstacle.contains(x as i32, y as i32) {
                return false;
            }
        }
        
        true
    }
}
```

---

### 4. 相机系统简化实现

**方案A：固定背景，玩家在屏幕内移动**
```rust
// 玩家位置限制在屏幕内
let screen_rect = Rectangle::new(0, 0, 800, 600);
if !screen_rect.contains(player.x as i32, player.y as i32) {
    // 限制玩家移动
}
```

**方案B：相机跟随，背景滚动（推荐）**
```rust
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub view_width: u32,
    pub view_height: u32,
}

impl Camera {
    // 跟随玩家
    pub fn follow(&mut self, player_x: f32, player_y: f32) {
        self.x = player_x - (self.view_width as f32 / 2.0);
        self.y = player_y - (self.view_height as f32 / 2.0);
        
        // 限制相机不超出背景边界
        self.x = self.x.max(0.0).min(bg_width - view_width);
        self.y = self.y.max(0.0).min(bg_height - view_height);
    }
    
    // 世界坐标 -> 屏幕坐标
    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        (world_x - self.x, world_y - self.y)
    }
}
```

---

## 📚 参考代码出处

### TRN颜色转换
**原版代码：**
- `Source/engine/load_file.hpp` - 文件加载
- `Source/utils/file_util.h` - 文件工具
- TRN文件通常作为通用二进制文件加载

**应用位置：**
- `Source/monster.cpp` - 怪物生成时应用TRN
- `Source/player.cpp` - 玩家装备变色

**改造思路：**
- 原版在渲染时应用TRN（运行时）
- 我们在加载时应用TRN（预处理），生成不同的纹理

---

### 资源管理器
**原版代码：**
- `Source/engine/assets.cpp` - 资源管理主文件
  - `GetAssetPath()` - 资源路径解析
  - `FindMpqFile()` - MPQ文件查找
  - `LoadMPQ()` - MPQ加载
  
**改造思路：**
- 原版使用全局函数，我们使用结构体封装
- 原版使用C++智能指针，我们使用Arc<T>
- 添加缓存系统，原版可能没有完整缓存

---

### 城镇系统
**原版代码：**
- `Source/levels/town.cpp` - 城镇地图生成
- `Source/levels/towndata/town.pal` - 城镇调色板

**注意：**
- 原版使用.DUN, .TIL, .CEL格式渲染城镇
- 我们Step 5.3只实现预览版本
- 完整城镇将在Step 6实现

---

## ⚠️ 潜在踩坑点

### 1. TRN文件位置

**问题：** TRN文件在MPQ中的位置可能不明显

**查找策略：**
```rust
// 可能的TRN文件位置：
// 1. plrgfx/warrior/*.trn - 玩家装备TRN
// 2. monsters/*.trn - 怪物变色TRN
// 3. levels/*.trn - 关卡特效TRN
```

**解决方案：**
- 使用例子工具列出MPQ中所有.trn文件
- 参考原版代码的TRN文件路径

---

### 2. TRN应用时机

**问题：** 应该在加载时还是渲染时应用TRN？

**方案对比：**

| 时机 | 优点 | 缺点 |
|------|------|------|
| **加载时** | 渲染简单，性能好 | 占用更多内存（每个TRN一套纹理） |
| **渲染时** | 节省内存 | 需要特殊渲染支持，复杂 |

**决策：加载时应用（推荐）**
- SDL2不方便运行时颜色映射
- 内存占用可接受（一个精灵+TRN ~1-2MB）
- 实现简单，性能好

---

### 3. 城镇背景资源查找

**问题：** 可能找不到完整的城镇背景PCX

**解决方案：**
```rust
// 按优先级尝试加载
let background_paths = vec![
    "levels/towndata/town.pcx",
    "levels/towndata/townback.pcx",
    "ui_art/town.pcx",
    // 如果都找不到，使用logo.pcx作为测试背景
    "ui_art/logo.pcx",
];

for path in background_paths {
    if let Ok(bg) = resource_manager.load_pcx(path) {
        // 成功加载
        break;
    }
}
```

**备选方案：**
- 使用简化的瓦片背景（Step 2的网格系统）
- 使用纯色背景 + 玩家精灵

---

### 4. 玩家精灵方向问题

**问题：** 玩家精灵可能有多个方向（8方向）

**当前资源：**
- `wmnaw.cl2` - 可能是单方向或多方向

**解决方案：**
```rust
// 先实现单方向
// 如果CL2包含多个方向：
// - 帧 0-7: 方向1
// - 帧 8-15: 方向2
// ...

// Step 5.3: 只使用第一个方向
// Step 6+: 实现完整的8方向系统
```

---

### 5. 资源缓存的生命周期

**问题：** SDL纹理的生命周期与TextureCreator绑定

**挑战：**
```rust
// 错误示例：
// ResourceManager不能持有SDL纹理，因为生命周期问题
pub struct ResourceManager {
    textures: HashMap<String, Texture>,  // ❌ 不能这样
}

// 正确做法：
// ResourceManager只持有纹理ID，Engine持有纹理
pub struct ResourceManager {
    texture_id_map: HashMap<String, Vec<String>>,  // ✅
}
```

---

## 🧪 测试要求

### 单元测试

1. **TRN测试**
   - TRN文件加载
   - 颜色映射正确性
   - 应用到像素数组
   - 边界情况（索引0,255）

2. **资源管理器测试**
   - 缓存命中测试
   - 重复加载测试
   - 不同资源类型加载

### 集成测试

1. **TRN应用测试**
   - 加载精灵
   - 应用TRN
   - 验证颜色变化

2. **完整资源加载流程**
   - MPQ -> Palette -> CL2 -> TRN -> Texture

### 游戏集成测试

1. **TRN测试场景**
   - 显示多个不同颜色的战士精灵
   - 验证TRN正确应用

2. **城镇场景测试**
   - 场景正确加载
   - 玩家可以移动
   - 碰撞检测工作
   - 场景切换正常
   - 帧率稳定60 FPS

### 性能测试

1. **资源加载性能**
   - 首次加载时间
   - 缓存命中时间
   - 内存使用

2. **渲染性能**
   - 城镇场景渲染帧率
   - 多精灵渲染性能

**测试覆盖率目标：≥ 85%**

---

## 📊 验收标准

### 功能验收

| 验收项 | 标准 | 状态 |
|--------|------|------|
| TRN文件加载 | 能从MPQ加载TRN文件 | ⏳ |
| TRN颜色映射 | 映射结果正确 | ⏳ |
| 应用TRN到精灵 | 精灵颜色正确变化 | ⏳ |
| 资源管理器 | 统一接口正常工作 | ⏳ |
| 资源缓存 | 缓存命中率 > 80% | ⏳ |
| 城镇场景加载 | 城镇场景正确显示 | ⏳ |
| 玩家移动 | 玩家可以在城镇中移动 | ⏳ |
| 碰撞检测 | 玩家不会移出可行走区域 | ⏳ |
| 场景切换 | F1/F2切换场景正常 | ⏳ |
| 性能 | 60 FPS稳定运行 | ⏳ |

### 代码质量验收

| 验收项 | 标准 | 状态 |
|--------|------|------|
| 单元测试 | 所有测试通过 | ⏳ |
| 测试覆盖率 | ≥ 85% | ⏳ |
| 代码注释 | 关键函数有注释 | ⏳ |
| 错误处理 | 使用Result，错误信息清晰 | ⏳ |
| 代码风格 | 符合Rust惯例 | ⏳ |

### 文档验收

| 验收项 | 标准 | 状态 |
|--------|------|------|
| 实现文档 | 记录实现细节 | ⏳ |
| 总结文档 | 记录学习要点和踩坑点 | ⏳ |
| 功能对照表 | 更新完成状态 | ⏳ |

---

## 🎯 最终目标

完成Step 5.3后，应该能够：

1. ✅ **TRN系统完整**
   - 从MPQ加载TRN文件
   - 应用TRN到精灵，生成不同颜色的变体
   - 在游戏中展示TRN效果

2. ✅ **资源管理器完善**
   - 统一资源加载接口
   - 资源缓存工作正常
   - 内存使用可控

3. ✅ **城镇场景预览** ⭐️ 重要
   - 城镇场景正确显示
   - 玩家可以在城镇中自由行走
   - 碰撞检测工作正常
   - 场景切换流畅

4. ✅ **完整的原版资源支持**
   - MPQ ✅ (Step 5.1)
   - Palette ✅ (Step 5.1)
   - PCX ✅ (Step 5.2)
   - CLX/CL2 ✅ (Step 5.2)
   - **TRN ✅ (Step 5.3)**
   - **场景加载 ✅ (Step 5.3)**

---

## 🎓 学习要点

### 1. 为什么需要TRN系统？

**90年代游戏的内存优化技术：**
- **问题**：存储多个颜色变体需要大量内存
  - 红色僵尸、绿色僵尸、蓝色僵尸...
  - 每个都存储完整精灵 = 3倍内存
  
- **解决方案：TRN颜色映射**
  - 只存储一个精灵
  - 使用TRN文件定义颜色映射
  - 运行时（或加载时）应用映射
  - 内存占用：1个精灵 + N个TRN文件（每个256字节）

**现代游戏的做法：**
- 使用着色器（shader）实时变色
- 使用颜色遮罩纹理
- 硬件支持，更灵活

**为什么我们实现TRN？**
1. **保持原版兼容**：直接使用原版TRN文件
2. **学习价值**：理解90年代优化技术
3. **实现简单**：256字节查找表，性能极好

---

### 2. 资源管理模式

**为什么需要资源管理器？**

**问题场景：**
```rust
// 不使用资源管理器：
// 每次需要资源都重新加载
let pal = load_palette("town.pal")?;  // 加载10ms
let pal2 = load_palette("town.pal")?; // 又加载10ms（重复）

// 场景切换时资源丢失
// 返回场景时需要重新加载
```

**使用资源管理器：**
```rust
// 第一次加载
let pal = rm.load_palette("town.pal")?;  // 加载10ms
// 第二次访问，使用缓存
let pal2 = rm.load_palette("town.pal")?; // <1ms（缓存命中）
```

**设计模式：**
- **单例模式**：全局唯一的资源管理器
- **工厂模式**：统一的资源创建接口
- **缓存模式**：避免重复加载

---

### 3. 游戏场景架构

**场景系统设计：**
```
Game
├── Scene 1: TestWorld (测试世界)
│   ├── World (瓦片地图)
│   ├── Entities (实体列表)
│   └── Rendering (渲染逻辑)
│
├── Scene 2: TownPreview (城镇预览)
│   ├── SimpleTown (背景+碰撞)
│   ├── Player Entity (玩家)
│   └── Rendering (渲染逻辑)
│
└── SceneManager (场景管理)
    ├── current_scene: SceneType
    ├── switch_scene()
    └── update/render delegation
```

**为什么需要场景系统？**
- 不同场景有不同的逻辑（城镇 vs 地牢）
- 场景切换时资源管理
- 代码组织更清晰

---

### 4. 渐进式开发方法

**Step 5.3的"预览"定位：**
- **目标**：验证资源系统完整性，提供可行走的场景
- **不是目标**：完整的地图系统、NPC、任务等
- **后续Steps**：逐步完善地图、NPC、战斗等

**为什么这样设计？**
1. **降低复杂度**：一次只关注一个目标
2. **快速验证**：能看到结果，保持动力
3. **灵活调整**：发现问题早点调整
4. **学习友好**：每个步骤独立且可理解

---

## 💡 练习任务

### 基础练习（必做）

1. **TRN编辑器**
   - 创建工具读取TRN文件
   - 显示256个索引的映射关系
   - 允许修改映射并保存

2. **资源浏览器**
   - 列出MPQ中所有资源
   - 显示资源类型和大小
   - 支持搜索和过滤

3. **性能分析工具**
   - 记录资源加载时间
   - 显示缓存命中率
   - 内存使用统计

### 进阶练习（选做）

1. **多TRN对比**
   - 同时显示一个精灵的多个TRN变体
   - 并排对比颜色差异

2. **LRU缓存实现**
   - 实现完整的LRU缓存系统
   - 设置内存限制
   - 自动淘汰最少使用的资源

3. **场景编辑器**
   - 允许调整可行走区域
   - 添加/移除障碍物
   - 保存和加载场景配置

### 创意练习（挑战）

1. **动态TRN生成**
   - 根据颜色参数自动生成TRN
   - 实现色相、饱和度、亮度调整
   - 预览效果

2. **资源热重载**
   - 监听资源文件变化
   - 自动重新加载
   - 无需重启游戏

3. **城镇装饰系统**
   - 在城镇场景中放置装饰物
   - 支持拖拽摆放
   - 保存装饰布局

---

## 🐛 Debug和调试建议

### 1. TRN调试

**验证TRN加载：**
```rust
// 打印TRN映射表
fn print_trn(trn: &ColorTransform) {
    println!("TRN Mapping:");
    for i in 0..256 {
        if trn.map[i] != i as u8 {
            println!("  {} -> {}", i, trn.map[i]);
        }
    }
}
```

**验证TRN应用：**
```rust
// 对比应用TRN前后的像素
let before = frame.pixels[100];
let after = trn.apply(before.unwrap());
println!("Pixel 100: {} -> {}", before.unwrap(), after);
```

### 2. 资源加载调试

**添加日志：**
```rust
pub fn load_palette(&mut self, path: &str) -> Result<Arc<Palette>> {
    if self.palette_cache.contains_key(path) {
        println!("[Cache Hit] Palette: {}", path);
    } else {
        println!("[Loading] Palette: {}", path);
    }
    // ...
}
```

### 3. 场景调试

**显示调试信息：**
```rust
// 在屏幕上显示：
// - 玩家位置
// - 当前场景
// - FPS
// - 资源缓存数量
```

---

## 🔜 下一步（Step 6）

完成Step 5.3后，下一步是：

**Step 6: 地图生成系统 Part 1 - 房间和走廊**
- 实现.DUN地图数据加载
- 实现.TIL瓦片集加载
- 实现.CEL瓦片图像加载
- 实现完整的地图渲染系统
- 实现随机地牢生成（教堂地牢）
- **完整的城镇加载**（替换Step 5.3的预览版本）

---

## 📝 文档更新清单

完成Step 5.3后需要更新：

1. ✅ **功能对照表** (`FEATURE_COMPARISON.md`)
   - Step 5全部标记为完成
   - 更新资源系统完成度

2. ✅ **Master Plan** (`master_plan.md`)
   - 更新当前进度
   - Step 5标记为完成

3. ✅ **README** (`README.md`)
   - 更新项目进度
   - 添加城镇场景预览截图

4. ✅ **总结文档** (`step-5.3-summary.md`)
   - 记录实现细节
   - 记录学习要点和踩坑点
   - 性能数据

---

**文档版本：** 1.0  
**创建日期：** 2025-11-25  
**作者：** AI Assistant  
**状态：** 📝 设计阶段

**关联文档：**
- [Step 5总体规划](step-5-resource-formats.md)
- [Step 5.1总结](step-5.1-FINAL-SUMMARY.md)
- [Step 5.2总结](step-5.2-integration-complete-summary.md)
- [Master Plan](master_plan.md)
- [功能对照表](FEATURE_COMPARISON.md)















