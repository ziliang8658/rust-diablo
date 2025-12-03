# Step 6: 地图生成系统 Part 1 - 瓦片系统和基础房间生成

## 📅 设计日期
2025-11-25

## 🎯 目标概述

Step 6 将实现 Diablo 1 的地图生成系统的第一部分，包括瓦片系统、数据格式加载和基础房间生成算法。这是游戏核心玩法的重要基础，将使玩家能够在随机生成的地牢中探险。

### 为什么这一步如此重要

地图生成系统是 Diablo 1 的核心特色之一，每次游戏都能体验到不同的地牢布局。本步骤将：
- ✅ 实现原版瓦片系统（MIN/TIL/SOL格式）
- ✅ 建立地图数据结构和坐标系统
- ✅ 实现教堂地牢（L1）的房间生成算法
- ✅ 为后续的怪物生成、物品掉落等系统打下基础

## 📦 功能拆分和实施计划

由于地图生成系统较为复杂，**建议分成3个子步骤逐步实现**：

### Step 6.1: 瓦片系统和数据格式（预计 300-400 行）
**目标：** 加载和解析原版瓦片数据

**核心功能：**
- MIN格式加载（微型瓦片数据）
- TIL格式加载（MegaTile定义）
- SOL格式加载（瓦片属性）
- 瓦片坐标系统（MegaTile vs MicroTile）
- 瓦片渲染系统集成

**参考代码：**
- `Source/levels/gendung.cpp::LoadMinData()`
- `Source/levels/gendung.cpp::SetDungeonMicros()`
- `Source/levels/gendung.cpp::LoadLevelSOLData()`
- `Source/levels/dun_tile.hpp` - 瓦片类型定义

**验收标准：**
- ✅ 能够从MPQ加载MIN/TIL/SOL文件
- ✅ 正确解析瓦片数据结构
- ✅ 能够渲染单个瓦片
- ✅ 单元测试覆盖率 ≥ 80%

---

### Step 6.2: 地图数据结构和基础房间生成（预计 350-450 行）
**目标：** 实现核心地图数据结构和简单房间生成

**核心功能：**
- Dungeon 数据结构（dungeon[][]数组）
- DungeonMask（房间区域标记）
- 固定房间生成（FirstRoom算法）
- 碰撞检测集成

**参考代码：**
- `Source/levels/gendung.h` - 地图数据结构
- `Source/levels/drlg_l1.cpp::FirstRoom()` - 房间布局
- `Source/levels/drlg_l1.cpp::CheckRoom()` - 碰撞检测
- `Source/levels/drlg_l1.cpp::MapRoom()` - 房间标记

**验收标准：**
- ✅ 能够生成3个主要房间和走廊
- ✅ 房间之间不重叠
- ✅ 能够在游戏中渲染地图
- ✅ 玩家碰撞检测正确工作

---

### Step 6.3: 递归房间生成和墙壁系统（预计 250-350 行）
**目标：** 实现完整的房间生成和墙壁装饰

**核心功能：**
- 递归房间生成（GenerateRoom）
- 墙壁和门生成
- 阴影和装饰
- Miniset 系统（楼梯等特殊结构）

**参考代码：**
- `Source/levels/drlg_l1.cpp::GenerateRoom()` - 递归生成
- `Source/levels/drlg_l1.cpp::HorizontalWall()` - 墙壁生成
- `Source/levels/drlg_l1.cpp::PlaceMiniSet()` - 特殊结构
- `Source/levels/drlg_l1.cpp::ApplyShadowsPatterns()` - 阴影

**验收标准：**
- ✅ 能够生成多层嵌套房间
- ✅ 墙壁和门正确放置
- ✅ 楼梯等特殊结构正确生成
- ✅ 地图美观，符合原版风格

---

## 📊 代码量估算

| 子步骤 | 预估代码量 | 主要模块 |
|--------|-----------|---------|
| Step 6.1 | 300-400行 | tile/min.rs, tile/til.rs, tile/sol.rs |
| Step 6.2 | 350-450行 | dungeon/mod.rs, dungeon/generator.rs |
| Step 6.3 | 250-350行 | dungeon/walls.rs, dungeon/miniset.rs |
| **总计** | **900-1200行** | **~1000行** |

加上单元测试和集成测试，预计总代码量 **1200-1500行**。

---

## 🗺️ 技术架构设计

### 模块结构

```
src/
├── engine/
│   ├── mod.rs
│   ├── isometric.rs       # 🆕 等距投影系统（Step 6.1）
│   └── direction.rs       # 已有
│
├── tiles/                  # 瓦片系统（Step 6.1）
│   ├── mod.rs             # 模块入口
│   ├── min.rs             # MIN格式加载
│   ├── til.rs             # TIL格式加载
│   ├── sol.rs             # SOL格式加载
│   └── types.rs           # 瓦片类型定义
│
├── dungeon/               # 地牢生成系统（Step 6.2-6.3）
│   ├── mod.rs            # 模块入口
│   ├── types.rs          # 地图数据结构
│   ├── generator_l1.rs   # 教堂地牢生成
│   ├── walls.rs          # 墙壁生成
│   ├── miniset.rs        # 特殊结构
│   └── renderer.rs       # 地图渲染（Isometric）
│
└── game.rs               # 游戏主循环（集成）
```

### 核心数据结构

```rust
// Step 6.1: 瓦片系统
pub struct MinData {
    tiles: Vec<MicroTile>,  // 微型瓦片数据
}

pub struct TilData {
    mega_tiles: Vec<MegaTile>,  // 2x2微型瓦片组合
}

pub struct SolData {
    properties: Vec<TileProperties>,  // 瓦片属性
}

pub struct MegaTile {
    micro1: u16,  // 左上
    micro2: u16,  // 右上
    micro3: u16,  // 左下
    micro4: u16,  // 右下
}

pub struct MicroTile {
    cel_index: u16,    // CEL文件中的索引
    tile_type: TileType,  // 瓦片类型
}

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Square,              // 正方形
    TransparentSquare,   // 透明正方形
    LeftTriangle,        // 左三角
    RightTriangle,       // 右三角
    LeftTrapezoid,       // 左梯形
    RightTrapezoid,      // 右梯形
}

bitflags! {
    pub struct TileProperties: u8 {
        const NONE = 0;
        const SOLID = 1 << 0;           // 不可行走
        const BLOCK_LIGHT = 1 << 1;     // 阻挡光线
        const BLOCK_MISSILE = 1 << 2;   // 阻挡飞行物
        const TRANSPARENT = 1 << 3;     // 透明
        const TRAP = 1 << 7;            // 陷阱
    }
}

// Step 6.2: 地图数据结构
pub struct Dungeon {
    pub width: usize,     // DMAXX = 40
    pub height: usize,    // DMAXY = 40
    pub tiles: [[u8; DMAXY]; DMAXX],  // 瓦片ID
    pub mask: Bitset2d,   // 房间区域标记
    pub protected: Bitset2d,  // 保护区域（不能被覆盖）
}

pub struct DungeonGenerator {
    dungeon: Dungeon,
    rng: StdRng,  // 随机数生成器
    vertical_layout: bool,  // 垂直/水平布局
}

// Step 6.3: 房间和墙壁
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct Miniset {
    pub width: usize,
    pub height: usize,
    pub search: [[u8; 6]; 6],   // 搜索模式
    pub replace: [[u8; 6]; 6],  // 替换模式
}
```

---

## 🎨 等距投影系统（Isometric Projection）

### 为什么使用 Isometric 投影？

Diablo 1 使用**菱形瓦片（Diamond Tiles）**和**等距投影（Isometric Projection）**来实现 2.5D 视觉效果。这种投影方式：
- ✅ 提供伪3D视觉效果
- ✅ 更好的深度感
- ✅ 更美观的视觉呈现
- ✅ 90年代经典游戏的标志性风格

### 瓦片尺寸

```rust
pub const TILE_WIDTH: i32 = 64;   // 菱形瓦片宽度（像素）
pub const TILE_HEIGHT: i32 = 32;  // 菱形瓦片高度（像素）
```

**菱形瓦片可视化：**
```
       32px
    ┌────────┐
    │   /\   │
32px│  /  \  │ 32px
    │ /    \ │
    │<──────>│ 64px
    │ \    / │
    │  \  /  │
    │   \/   │
    └────────┘
       32px
```

每个 MegaTile（地图瓦片）渲染为一个 64x32 像素的菱形。

---

### 坐标系统转换

Diablo 1 使用两套坐标系统：

#### 1. 世界坐标（World Coordinates）
- 用于逻辑计算（碰撞检测、寻路等）
- 正方形网格，正交坐标
- 原点在左上角

```
World Coordinates (逻辑网格)
  0   1   2   3   4  (x)
0 □ - □ - □ - □ - □
  |   |   |   |   |
1 □ - □ - □ - □ - □
  |   |   |   |   |
2 □ - □ - □ - □ - □
(y)
```

#### 2. 屏幕坐标（Screen Coordinates）
- 用于渲染显示
- 菱形网格，等距投影
- 原点在左上角

```
Screen Coordinates (渲染网格)
       0,0
        ◇
      ◇   ◇
    ◇   ◇   ◇
  ◇   ◇   ◇   ◇
    ◇   ◇   ◇
      ◇   ◇
        ◇
```

---

### 坐标转换公式

**原版代码参考：** `Source/engine/displacement.hpp` Line 151-168

#### 世界坐标 → 屏幕坐标（worldToScreen）

```rust
pub fn world_to_screen(world_x: i32, world_y: i32) -> (i32, i32) {
    let screen_x = (world_y - world_x) * 32;
    let screen_y = (world_y + world_x) * -16;
    (screen_x, screen_y)
}
```

**推导过程：**
```
等距投影是一个 -135° 旋转 + 缩放变换
旋转矩阵（-135°）：
  [cos(-135°), -sin(-135°)]   [~-0.7,  ~0.7]
  [sin(-135°),  cos(-135°)] = [~-0.7, ~-0.7]

缩放因子：x轴 32，y轴 16（瓦片尺寸的一半）

变换矩阵：
  [-32,  32] [dx]   [ 32(dy - dx)]
  [-16, -16] [dy] = [-16(dy + dx)]
```

**示例：**
```rust
// 世界坐标 (3, 5) → 屏幕坐标
let (sx, sy) = world_to_screen(3, 5);
// sx = (5 - 3) * 32 = 64
// sy = (5 + 3) * -16 = -128
```

#### 屏幕坐标 → 世界坐标（screenToWorld）

```rust
pub fn screen_to_world(screen_x: i32, screen_y: i32) -> (i32, i32) {
    let world_x = (2 * screen_y + screen_x) / -64;
    let world_y = (2 * screen_y - screen_x) / -64;
    (world_x, world_y)
}
```

**这是 worldToScreen 的逆矩阵。**

---

### 渲染顺序（Painter's Algorithm）

为了正确处理遮挡关系，需要**从后到前、从上到下**渲染：

```rust
// 渲染顺序示例
for y in 0..map_height {
    for x in 0..map_width {
        let world_pos = (x, y);
        let (screen_x, screen_y) = world_to_screen(x, y);
        render_tile(screen_x, screen_y);
    }
}
```

**渲染顺序可视化：**
```
渲染顺序（数字越小越先渲染）：
  1
 2 3
4 5 6
 7 8
  9
```

这确保了：
- 后面的物体先画
- 前面的物体后画（覆盖后面的）
- 正确的深度感

---

### MegaTile vs MicroTile 坐标

除了世界坐标和屏幕坐标，还有两个层级的地图坐标：

1. **MegaTile 坐标（地图生成）**
   - 用于地图生成逻辑
   - 范围：[0, 40) x [0, 40)
   - 每个 MegaTile 包含 2x2 个 MicroTile

2. **MicroTile 坐标（渲染和碰撞）**
   - 用于渲染和精确碰撞检测
   - 范围：[0, 112) x [0, 112)
   - 转换：`MicroTile = MegaTile * 2 + 16`（16瓦片边界）

```rust
// MegaTile → MicroTile
pub fn mega_to_micro(mega_x: i32, mega_y: i32) -> (i32, i32) {
    (mega_x * 2 + 16, mega_y * 2 + 16)
}

// MicroTile → MegaTile
pub fn micro_to_mega(micro_x: i32, micro_y: i32) -> (i32, i32) {
    ((micro_x - 16) / 2, (micro_y - 16) / 2)
}
```

---

### 实现建议

在 Step 6.1 中，需要实现：

```rust
// src/engine/isometric.rs

pub const TILE_WIDTH: i32 = 64;
pub const TILE_HEIGHT: i32 = 32;

/// 世界坐标转屏幕坐标
#[inline]
pub fn world_to_screen(world_x: i32, world_y: i32) -> (i32, i32) {
    let screen_x = (world_y - world_x) * 32;
    let screen_y = (world_y + world_x) * -16;
    (screen_x, screen_y)
}

/// 屏幕坐标转世界坐标
#[inline]
pub fn screen_to_world(screen_x: i32, screen_y: i32) -> (i32, i32) {
    let world_x = (2 * screen_y + screen_x) / -64;
    let world_y = (2 * screen_y - screen_x) / -64;
    (world_x, world_y)
}

/// MegaTile转MicroTile
#[inline]
pub fn mega_to_micro(mega_x: i32, mega_y: i32) -> (i32, i32) {
    (mega_x * 2 + 16, mega_y * 2 + 16)
}

/// MicroTile转MegaTile
#[inline]
pub fn micro_to_mega(micro_x: i32, micro_y: i32) -> (i32, i32) {
    ((micro_x - 16) / 2, (micro_y - 16) / 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_to_screen() {
        let (sx, sy) = world_to_screen(0, 0);
        assert_eq!(sx, 0);
        assert_eq!(sy, 0);
        
        let (sx, sy) = world_to_screen(1, 0);
        assert_eq!(sx, -32);
        assert_eq!(sy, -16);
        
        let (sx, sy) = world_to_screen(0, 1);
        assert_eq!(sx, 32);
        assert_eq!(sy, -16);
    }
    
    #[test]
    fn test_screen_to_world() {
        // 往返转换应该一致
        let (wx, wy) = (5, 7);
        let (sx, sy) = world_to_screen(wx, wy);
        let (wx2, wy2) = screen_to_world(sx, sy);
        assert_eq!(wx, wx2);
        assert_eq!(wy, wy2);
    }
    
    #[test]
    fn test_mega_micro_conversion() {
        let (mx, my) = mega_to_micro(10, 10);
        assert_eq!(mx, 36);  // 10 * 2 + 16
        assert_eq!(my, 36);
        
        let (mx2, my2) = micro_to_mega(mx, my);
        assert_eq!(mx2, 10);
        assert_eq!(my2, 10);
    }
}
```

---

## 🎨 瓦片系统详解（Step 6.1）

> **📖 详细技术文档：** 瓦片格式的完整技术细节请参考 [Step 6 技术要点 - 瓦片格式详解](./tech_key_points/step-6-tile-formats.md)
> 
> 该文档包含：
> - 等距投影数学推导
> - MIN/TIL/SOL 格式详细说明
> - 完整的 Rust 实现代码
> - 单元测试示例
> - 常见陷阱和解决方案

### MIN 文件格式

MIN文件包含微型瓦片（MicroTile）的索引数据。

**文件结构：**
```
MIN文件 = [u16; n]  // n个u16值，每个值是瓦片索引

每个u16值编码：
- 低12位 (0-11)：CEL文件中的帧索引 (1-based)
- 高3位 (12-14)：瓦片类型 (TileType)
```

**示例代码：**
```rust
pub fn load_min_file(mpq: &Mpq, path: &str) -> Result<MinData> {
    let data = mpq.read_file(path)?;
    let tiles: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    
    let micro_tiles: Vec<MicroTile> = tiles
        .iter()
        .map(|&val| MicroTile {
            cel_index: val & 0xFFF,  // 低12位
            tile_type: TileType::from((val >> 12) & 0x7),  // 高3位
        })
        .collect();
    
    Ok(MinData { tiles: micro_tiles })
}
```

**参考代码：** `Source/levels/gendung.cpp::LoadMinData()`

---

### TIL 文件格式

TIL文件定义MegaTile，每个MegaTile由4个MicroTile组成（2x2）。

**文件结构：**
```
TIL文件 = [MegaTile; n]

struct MegaTile {
    micro1: u16,  // 左上微型瓦片索引（指向MIN数据）
    micro2: u16,  // 右上
    micro3: u16,  // 左下
    micro4: u16,  // 右下
}
```

**坐标关系：**
```
MegaTile (1个) = MicroTile (2x2)
+-------+-------+
| micro1| micro2|  ← 2x MicroTile 宽度
+-------+-------+
| micro3| micro4|
+-------+-------+
```

**示例代码：**
```rust
pub fn load_til_file(mpq: &Mpq, path: &str) -> Result<TilData> {
    let data = mpq.read_file(path)?;
    let mega_tiles: Vec<MegaTile> = data
        .chunks_exact(8)  // 每个MegaTile 8字节
        .map(|chunk| {
            let micro1 = u16::from_le_bytes([chunk[0], chunk[1]]);
            let micro2 = u16::from_le_bytes([chunk[2], chunk[3]]);
            let micro3 = u16::from_le_bytes([chunk[4], chunk[5]]);
            let micro4 = u16::from_le_bytes([chunk[6], chunk[7]]);
            MegaTile { micro1, micro2, micro3, micro4 }
        })
        .collect();
    
    Ok(TilData { mega_tiles })
}
```

**参考代码：** `Source/levels/gendung.cpp::DRLG_LPass3()`

---

### SOL 文件格式

SOL文件定义每个瓦片的属性（是否可行走、是否阻挡光线等）。

**文件结构：**
```
SOL文件 = [u8; n]  // n个字节，每个字节是TileProperties标志位
```

**属性定义：**
```rust
bitflags! {
    pub struct TileProperties: u8 {
        const NONE = 0;
        const SOLID = 1 << 0;           // 不可行走
        const BLOCK_LIGHT = 1 << 1;     // 阻挡光线
        const BLOCK_MISSILE = 1 << 2;   // 阻挡飞行物
        const TRANSPARENT = 1 << 3;     // 透明
        const TRANSPARENT_LEFT = 1 << 4;   // 左侧透明
        const TRANSPARENT_RIGHT = 1 << 5;  // 右侧透明
        const TRAP = 1 << 7;            // 陷阱
    }
}
```

**示例代码：**
```rust
pub fn load_sol_file(mpq: &Mpq, path: &str) -> Result<SolData> {
    let data = mpq.read_file(path)?;
    let properties: Vec<TileProperties> = data
        .iter()
        .map(|&byte| TileProperties::from_bits_truncate(byte))
        .collect();
    
    Ok(SolData { properties })
}
```

**参考代码：** `Source/levels/gendung.cpp::LoadLevelSOLData()`

---

## 🏗️ 地图数据结构（Step 6.2）

### 坐标系统

Diablo 1 使用两套坐标系统：

1. **MegaTile 坐标（地图坐标）**
   - 用于地图生成逻辑
   - 范围：[0, DMAXX) x [0, DMAXY)
   - DMAXX = 40, DMAXY = 40

2. **MicroTile 坐标（渲染坐标）**
   - 用于渲染和碰撞检测
   - 范围：[0, MAXDUNX) x [0, MAXDUNY)
   - MAXDUNX = 112, MAXDUNY = 112
   - 转换：MicroTile = MegaTile * 2 + 16 （有16瓦片边界）

**坐标转换：**
```rust
// MegaTile → MicroTile
pub fn mega_to_micro(mega_x: i32, mega_y: i32) -> (i32, i32) {
    (mega_x * 2 + 16, mega_y * 2 + 16)
}

// MicroTile → MegaTile
pub fn micro_to_mega(micro_x: i32, micro_y: i32) -> (i32, i32) {
    ((micro_x - 16) / 2, (micro_y - 16) / 2)
}
```

---

### Dungeon 数据结构

```rust
pub const DMAXX: usize = 40;  // MegaTile地图宽度
pub const DMAXY: usize = 40;  // MegaTile地图高度
pub const MAXDUNX: usize = 112;  // MicroTile地图宽度
pub const MAXDUNY: usize = 112;  // MicroTile地图高度

pub struct Dungeon {
    // MegaTile层（地图生成）
    pub tiles: [[u8; DMAXY]; DMAXX],  // 瓦片ID (1-based, 0=未使用)
    pub mask: Bitset2d<DMAXX, DMAXY>,  // 房间区域标记
    pub protected: Bitset2d<DMAXX, DMAXY>,  // 保护区域
    
    // MicroTile层（渲染和碰撞）
    pub pieces: [[u16; MAXDUNY]; MAXDUNX],  // 微型瓦片ID
    pub solid: [[bool; MAXDUNY]; MAXDUNX],  // 是否可行走
    
    // 元数据
    pub level_type: DungeonType,
    pub seed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonType {
    Town,       // 城镇
    Cathedral,  // 教堂（L1, 1-4层）
    Catacombs,  // 地下墓穴（L2, 5-8层）
    Caves,      // 洞穴（L3, 9-12层）
    Hell,       // 地狱（L4, 13-16层）
}

impl Dungeon {
    pub fn new(level_type: DungeonType, seed: u32) -> Self {
        Self {
            tiles: [[0; DMAXY]; DMAXX],
            mask: Bitset2d::new(),
            protected: Bitset2d::new(),
            pieces: [[0; MAXDUNY]; MAXDUNX],
            solid: [[false; MAXDUNY]; MAXDUNX],
            level_type,
            seed,
        }
    }
    
    // 检查MegaTile坐标是否在边界内
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < DMAXX as i32 && y >= 0 && y < DMAXY as i32
    }
    
    // 设置瓦片
    pub fn set_tile(&mut self, x: usize, y: usize, tile_id: u8) {
        if x < DMAXX && y < DMAXY {
            self.tiles[x][y] = tile_id;
        }
    }
    
    // 获取瓦片
    pub fn get_tile(&self, x: usize, y: usize) -> u8 {
        if x < DMAXX && y < DMAXY {
            self.tiles[x][y]
        } else {
            0
        }
    }
}
```

**参考代码：** `Source/levels/gendung.h`

---

## 🎲 房间生成算法（Step 6.2）

### 教堂地牢（L1）算法概述

原版教堂地牢生成分为3个阶段：

1. **FirstRoom()** - 生成主要布局
   - 随机选择垂直或水平布局
   - 生成3个主要房间（Chamber1, Chamber2, Chamber3）
   - 生成连接走廊（Hallway）

2. **GenerateRoom()** - 递归生成子房间
   - 在每个主要房间内递归生成小房间
   - 使用BSP（二叉空间分割）算法
   - 随机房间大小（2-6瓦片）

3. **MakeDmt()** - 生成墙壁和地板
   - 根据DungeonMask生成瓦片类型
   - 判断墙壁方向（垂直/水平/角落）

### FirstRoom() 算法

```rust
impl DungeonGenerator {
    pub fn first_room(&mut self) {
        // 1. 随机布局方向
        self.vertical_layout = self.rng.gen_bool(0.5);
        
        // 2. 随机决定是否生成各个房间
        let has_chamber1 = !self.rng.gen_bool(0.5);
        let has_chamber2 = !self.rng.gen_bool(0.5);
        let has_chamber3 = !self.rng.gen_bool(0.5);
        
        // 至少保留两个房间
        let (has_chamber1, has_chamber2, has_chamber3) = 
            if !has_chamber1 || !has_chamber3 {
                (has_chamber1, true, has_chamber3)
            } else {
                (has_chamber1, has_chamber2, has_chamber3)
            };
        
        // 3. 定义房间区域（水平布局）
        let mut chamber1 = Room { x: 1, y: 15, width: 10, height: 10 };
        let chamber2 = Room { x: 15, y: 15, width: 10, height: 10 };
        let mut chamber3 = Room { x: 29, y: 15, width: 10, height: 10 };
        let mut hallway = Room { x: 1, y: 17, width: 38, height: 6 };
        
        // 4. 如果是垂直布局，交换x和y坐标
        if self.vertical_layout {
            std::mem::swap(&mut chamber1.x, &mut chamber1.y);
            std::mem::swap(&mut chamber3.x, &mut chamber3.y);
            std::mem::swap(&mut hallway.x, &mut hallway.y);
            std::mem::swap(&mut hallway.width, &mut hallway.height);
        }
        
        // 5. 标记房间区域
        if has_chamber1 {
            self.map_room(&chamber1);
            self.generate_room(&chamber1, self.vertical_layout);
        }
        if has_chamber2 {
            self.map_room(&chamber2);
            self.generate_room(&chamber2, self.vertical_layout);
        }
        if has_chamber3 {
            self.map_room(&chamber3);
            self.generate_room(&chamber3, self.vertical_layout);
        }
        
        // 6. 标记走廊
        if !has_chamber1 {
            hallway.x += 17;
            hallway.width -= 17;
        }
        if !has_chamber3 {
            hallway.width -= 16;
        }
        self.map_room(&hallway);
    }
    
    // 标记房间区域到DungeonMask
    fn map_room(&mut self, room: &Room) {
        for y in 0..room.height {
            for x in 0..room.width {
                let mx = (room.x + x) as usize;
                let my = (room.y + y) as usize;
                if mx < DMAXX && my < DMAXY {
                    self.dungeon.mask.set(mx, my);
                }
            }
        }
    }
}
```

**参考代码：** `Source/levels/drlg_l1.cpp::FirstRoom()` (Line 508-552)

---

### GenerateRoom() 递归算法

```rust
impl DungeonGenerator {
    pub fn generate_room(&mut self, area: &Room, vertical_layout: bool) {
        // 1. 随机决定是否旋转布局
        let rotate = self.rng.gen_range(0..4) == 0;
        let vertical_layout = (!vertical_layout && rotate) || (vertical_layout && !rotate);
        
        // 2. 尝试20次生成房间1
        let mut room1 = None;
        for _ in 0..20 {
            let width = (self.rng.gen_range(0..5) + 2) & !1;  // 2-6, 偶数
            let height = (self.rng.gen_range(0..5) + 2) & !1;
            
            let (x, y) = if vertical_layout {
                (area.x - width, area.y + area.height / 2 - height / 2)
            } else {
                (area.x + area.width / 2 - width / 2, area.y - height)
            };
            
            let test_room = Room { x, y, width, height };
            if self.check_room(&test_room) {
                room1 = Some(test_room);
                break;
            }
        }
        
        // 3. 如果成功，标记房间1并继续生成
        if let Some(r1) = room1 {
            self.map_room(&r1);
            
            // 4. 生成房间2（相对位置）
            let room2 = if vertical_layout {
                Room { 
                    x: area.x + area.width, 
                    y: r1.y, 
                    width: r1.width, 
                    height: r1.height 
                }
            } else {
                Room { 
                    x: r1.x, 
                    y: area.y + area.height, 
                    width: r1.width, 
                    height: r1.height 
                }
            };
            
            if self.check_room(&room2) {
                self.map_room(&room2);
                // 递归生成子房间
                self.generate_room(&room2, !vertical_layout);
            }
            
            // 递归生成房间1的子房间
            self.generate_room(&r1, !vertical_layout);
        }
    }
    
    // 检查房间是否可以放置（不重叠）
    fn check_room(&self, room: &Room) -> bool {
        for y in 0..room.height {
            for x in 0..room.width {
                let mx = room.x + x;
                let my = room.y + y;
                
                // 边界检查
                if mx < 0 || mx >= DMAXX as i32 || my < 0 || my >= DMAXY as i32 {
                    return false;
                }
                
                // 重叠检查
                if self.dungeon.mask.test(mx as usize, my as usize) {
                    return false;
                }
            }
        }
        true
    }
}
```

**参考代码：** `Source/levels/drlg_l1.cpp::GenerateRoom()` (Line 460-503)

---

### MakeDmt() 墙壁生成

```rust
impl DungeonGenerator {
    pub fn make_dmt(&mut self) {
        for y in 0..(DMAXY - 1) {
            for x in 0..(DMAXX - 1) {
                let curr = self.dungeon.mask.test(x, y);
                let right = self.dungeon.mask.test(x + 1, y);
                let down = self.dungeon.mask.test(x, y + 1);
                let diag = self.dungeon.mask.test(x + 1, y + 1);
                
                let tile = if curr {
                    Tile::Floor  // 13
                } else if !diag && down && right {
                    Tile::Floor  // 移除对角角落
                } else if diag && down && right {
                    Tile::VCorner  // 16 - 垂直角落
                } else if down {
                    Tile::HWall  // 2 - 水平墙
                } else if right {
                    Tile::VWall  // 1 - 垂直墙
                } else if diag {
                    Tile::DWall  // 4 - 对角墙
                } else {
                    Tile::Dirt  // 22 - 土地
                };
                
                self.dungeon.set_tile(x, y, tile as u8);
            }
        }
    }
}

#[repr(u8)]
pub enum Tile {
    VWall = 1,      // 垂直墙
    HWall = 2,      // 水平墙
    Corner = 3,     // 角落
    DWall = 4,      // 对角墙
    Floor = 13,     // 地板
    VCorner = 16,   // 垂直角落
    HCorner = 17,   // 水平角落
    Dirt = 22,      // 土地
    // ... 更多瓦片类型
}
```

**参考代码：** `Source/levels/drlg_l1.cpp::MakeDmt()` (Line 562-582)

---

## 🧪 测试要求

### Step 6.1: 瓦片系统测试

**单元测试：**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_load_min_data() {
        let mpq = Mpq::open("assets/DIABDAT.MPQ").unwrap();
        let min_data = load_min_file(&mpq, "levels/l1data/l1.min").unwrap();
        assert!(!min_data.tiles.is_empty());
    }
    
    #[test]
    fn test_load_til_data() {
        let mpq = Mpq::open("assets/DIABDAT.MPQ").unwrap();
        let til_data = load_til_file(&mpq, "levels/l1data/l1.til").unwrap();
        assert!(!til_data.mega_tiles.is_empty());
    }
    
    #[test]
    fn test_tile_type_decoding() {
        let tile_data: u16 = 0x1ABC;  // 类型1, 索引ABC
        let tile_type = (tile_data >> 12) & 0x7;
        let cel_index = tile_data & 0xFFF;
        assert_eq!(tile_type, 1);
        assert_eq!(cel_index, 0xABC);
    }
    
    #[test]
    fn test_tile_properties() {
        let props = TileProperties::SOLID | TileProperties::BLOCK_LIGHT;
        assert!(props.contains(TileProperties::SOLID));
        assert!(!props.contains(TileProperties::TRANSPARENT));
    }
}
```

---

### Step 6.2: 地图生成测试

**单元测试：**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_first_room_layout() {
        let mut gen = DungeonGenerator::new(12345);
        gen.first_room();
        
        // 至少有一些房间被标记
        let count = gen.dungeon.mask.count();
        assert!(count > 100);  // 至少100个瓦片
        assert!(count < 1600);  // 最多全地图
    }
    
    #[test]
    fn test_check_room_overlap() {
        let mut gen = DungeonGenerator::new(12345);
        let room1 = Room { x: 5, y: 5, width: 10, height: 10 };
        assert!(gen.check_room(&room1));
        
        gen.map_room(&room1);
        
        // 重叠房间应该被拒绝
        let room2 = Room { x: 10, y: 10, width: 10, height: 10 };
        assert!(!gen.check_room(&room2));
    }
    
    #[test]
    fn test_make_dmt_walls() {
        let mut gen = DungeonGenerator::new(12345);
        gen.dungeon.mask.set(5, 5);
        gen.dungeon.mask.set(6, 5);
        gen.make_dmt();
        
        // 应该生成地板和墙壁
        assert_eq!(gen.dungeon.get_tile(5, 5), Tile::Floor as u8);
        assert_ne!(gen.dungeon.get_tile(5, 4), Tile::Floor as u8);
    }
    
    #[test]
    fn test_coordinate_conversion() {
        let (micro_x, micro_y) = mega_to_micro(10, 10);
        assert_eq!(micro_x, 36);  // 10 * 2 + 16
        assert_eq!(micro_y, 36);
        
        let (mega_x, mega_y) = micro_to_mega(36, 36);
        assert_eq!(mega_x, 10);
        assert_eq!(mega_y, 10);
    }
}
```

---

### Step 6.3: 墙壁和特殊结构测试

**单元测试：**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_horizontal_wall_placement() {
        let mut gen = DungeonGenerator::new(12345);
        // 生成水平墙壁
        gen.place_horizontal_wall(10, 10, 5);
        
        // 检查墙壁瓦片
        for x in 10..15 {
            let tile = gen.dungeon.get_tile(x, 10);
            assert!(matches!(tile, Tile::HWall | Tile::HDoor));
        }
    }
    
    #[test]
    fn test_miniset_matching() {
        let miniset = Miniset {
            width: 3,
            height: 3,
            search: [[13, 13, 13], [13, 13, 13], [13, 13, 13]],
            replace: [[1, 2, 1], [2, 13, 2], [1, 2, 1]],
        };
        
        let mut gen = DungeonGenerator::new(12345);
        // 设置匹配区域
        for y in 5..8 {
            for x in 5..8 {
                gen.dungeon.set_tile(x, y, 13);
            }
        }
        
        assert!(miniset.matches(&gen.dungeon, 5, 5));
    }
}
```

---

### 集成测试

```rust
// tests/test_dungeon_generation.rs
#[test]
fn test_generate_cathedral_level() {
    let mut gen = DungeonGenerator::new(12345);
    gen.generate_cathedral_level();
    
    // 验证地图已生成
    let tile_count = gen.dungeon.mask.count();
    assert!(tile_count > 0);
    
    // 验证可行走区域
    let walkable_count = count_walkable_tiles(&gen.dungeon);
    assert!(walkable_count > 100);
    
    // 验证连通性（所有房间应该相连）
    assert!(is_fully_connected(&gen.dungeon));
}

#[test]
fn test_render_generated_dungeon() {
    // 集成到游戏引擎
    let mut game = Game::new().unwrap();
    game.load_dungeon_level(1);
    
    // 验证渲染正常
    for _ in 0..60 {
        game.update(16);  // 模拟60帧
    }
}
```

---

## 📝 练习任务

### 基础练习

1. **瓦片编辑器**（入门）
   - 创建一个小工具，可以查看MIN/TIL文件内容
   - 显示每个MegaTile的4个MicroTile组成
   - 显示瓦片属性（SOLID, BLOCK_LIGHT等）

2. **地图查看器**（进阶）
   - 创建一个可以显示生成地图的工具
   - 支持缩放和平移
   - 显示房间边界和走廊

3. **种子比较器**（进阶）
   - 给定相同种子，生成的地图应该完全相同
   - 编写测试验证确定性

### 创意练习

4. **自定义房间模板**（创意）
   - 设计自己的房间模板（Miniset）
   - 添加特殊装饰（宝箱房、图书馆等）

5. **地图生成可视化**（高挑战）
   - 可视化地图生成过程
   - 逐帧显示房间生成顺序
   - 显示墙壁和门的生成过程

6. **性能优化**（高挑战）
   - 地图生成时间应该 < 100ms
   - 使用Criterion进行基准测试
   - 优化房间重叠检测算法

---

## 🎓 学习要点

### 1. 等距投影（Isometric Projection）

等距投影是一种伪3D投影技术，广泛用于2D游戏中创造立体感。

**核心原理：**
- 45° 旋转的正交投影
- 所有平行线保持平行（不像透视投影）
- 距离和角度不会失真

**数学变换：**
```
屏幕坐标 = 旋转矩阵 × 世界坐标
[screen_x]   [-32  32] [world_x]
[screen_y] = [-16 -16] [world_y]
```

**经典使用案例：**
- Diablo 系列（1、2）
- 模拟城市系列
- 文明系列
- 星际争霸

**学习资源：**
- [Isometric Rendering Tutorial](https://gamedevelopment.tutsplus.com/tutorials/creating-isometric-worlds-a-primer-for-game-developers--gamedev-6511)
- [Red Blob Games - Hexagonal Grids](https://www.redblobgames.com/grids/hexagons/)

---

### 2. 二叉空间分割（BSP）算法

教堂地牢使用BSP算法递归生成房间：
- 将空间分成两部分
- 每部分递归生成子房间
- 控制递归深度避免过小房间

**学习资源：**
- [Roguelike Dungeon Generation](http://www.roguebasin.com/index.php?title=Basic_BSP_Dungeon_generation)
- [BSP树可视化](https://www.cs.ubc.ca/~rbridson/docs/bridson-siggraph07-poissondisk.pdf)

---

### 2. 位图（Bitset）优化

DungeonMask使用位图表示房间区域，节省内存：
```rust
// 1600个bool: 1600字节
let mask: [[bool; 40]; 40];

// Bitset: 200字节（1600位 / 8 = 200字节）
let mask: Bitset2d<40, 40>;
```

---

### 3. 瓦片渲染优化

使用MegaTile坐标进行生成，MicroTile坐标进行渲染：
- 生成阶段：只需处理40x40 = 1600个瓦片
- 渲染阶段：扩展到112x112 = 12544个微型瓦片
- 渲染时使用视口裁剪，只渲染可见区域

---

### 4. 随机数生成的确定性

使用固定种子生成相同地图：
```rust
let mut rng = StdRng::seed_from_u64(12345);
```

这对于：
- 多人游戏同步
- 地图重现（bug调试）
- 竞速比赛

---

## ⚠️ 踩坑点

### 1. 坐标系统混淆

**问题：** MegaTile和MicroTile坐标容易混淆

**解决方案：**
- 使用不同的类型表示坐标
```rust
#[derive(Debug, Clone, Copy)]
pub struct MegaTilePos { pub x: i32, pub y: i32 }

#[derive(Debug, Clone, Copy)]
pub struct MicroTilePos { pub x: i32, pub y: i32 }
```

---

### 2. 房间重叠检测bug

**问题：** 原版代码在CheckRoom中有bug，导致某些情况下房间可能重叠

**参考代码：** `Source/levels/drlg_l1.cpp::GenerateRoom()` Line 475

**解决方案：**
- 严格检查所有边界
- 添加额外的边界距离（padding）

---

### 3. 墙壁和门的边界问题

**问题：** 墙壁生成时需要检查相邻瓦片，可能越界

**解决方案：**
- 始终检查数组边界
- 使用safe accessor函数

```rust
fn get_tile_safe(&self, x: i32, y: i32) -> Option<u8> {
    if x >= 0 && x < DMAXX as i32 && y >= 0 && y < DMAXY as i32 {
        Some(self.tiles[x as usize][y as usize])
    } else {
        None
    }
}
```

---

### 4. 瓦片类型枚举和数值转换

**问题：** 原版使用数值表示瓦片类型，容易出错

**解决方案：**
- 使用Rust枚举
- 实现From/Into trait

```rust
#[repr(u8)]
pub enum Tile {
    VWall = 1,
    HWall = 2,
    // ...
}

impl From<u8> for Tile {
    fn from(val: u8) -> Self {
        match val {
            1 => Tile::VWall,
            2 => Tile::HWall,
            _ => Tile::Dirt,  // 默认值
        }
    }
}
```

---

## 📚 参考资源

### 原版代码参考

| 功能模块 | 原版文件 | 关键函数 | Rust模块 |
|---------|---------|---------|----------|
| 瓦片加载 | `gendung.cpp` | `LoadMinData()` | `tiles/min.rs` |
| 瓦片属性 | `gendung.cpp` | `LoadLevelSOLData()` | `tiles/sol.rs` |
| 地图结构 | `gendung.h` | - | `dungeon/types.rs` |
| 房间生成 | `drlg_l1.cpp` | `FirstRoom()` | `dungeon/generator_l1.rs` |
| 递归生成 | `drlg_l1.cpp` | `GenerateRoom()` | `dungeon/generator_l1.rs` |
| 墙壁生成 | `drlg_l1.cpp` | `MakeDmt()` | `dungeon/walls.rs` |
| 特殊结构 | `gendung.cpp` | `PlaceMiniSet()` | `dungeon/miniset.rs` |

---

### 学习资源

1. **Roguelike开发教程**
   - [RogueBasin](http://www.roguebasin.com/)
   - [Procedural Generation Wiki](https://pcg.fandom.com/)

2. **BSP算法**
   - [BSP Dungeon Generation](http://www.roguebasin.com/index.php?title=Basic_BSP_Dungeon_generation)
   - [Cellular Automata](http://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels)

3. **瓦片地图渲染**
   - [Isometric Rendering](https://gamedevelopment.tutsplus.com/tutorials/creating-isometric-worlds-a-primer-for-game-developers--gamedev-6511)
   - [Tile Map Engine](https://www.gamasutra.com/view/feature/131801/designing_a_2d_tile_map_engine.php)

---

## 🎯 验收标准总结

### Step 6.1: 瓦片系统和等距投影
- [x] **等距投影系统实现**
  - [x] worldToScreen 坐标转换正确
  - [x] screenToWorld 坐标转换正确
  - [x] MegaTile/MicroTile 坐标转换正确
  - [x] 往返转换一致性测试通过
- [x] **瓦片数据加载**
  - [x] 能够从MPQ加载MIN/TIL/SOL文件
  - [x] 正确解析瓦片数据结构
  - [x] 瓦片类型枚举完整
- [x] **渲染系统**
  - [x] 能够渲染单个MegaTile（菱形）
  - [x] 正确的渲染顺序（从后到前）
  - [x] 屏幕裁剪正确工作
- [x] **测试和文档**
  - [x] 单元测试覆盖率 ≥ 80%
  - [x] 坐标转换测试完整
  - [x] 文档完整（格式说明、示例代码）

### Step 6.2: 地图数据结构和房间生成
- [x] 实现Dungeon数据结构
- [x] 实现FirstRoom算法（3个主要房间 + 走廊）
- [x] 房间不重叠，位置合理
- [x] 能够在游戏中显示生成的地图
- [x] 玩家碰撞检测工作正常
- [x] 单元测试覆盖率 ≥ 75%

### Step 6.3: 墙壁和特殊结构
- [x] 实现GenerateRoom递归算法
- [x] 实现MakeDmt墙壁生成
- [x] 实现墙壁装饰（门、拱门等）
- [x] 实现Miniset系统（楼梯等）
- [x] 地图美观，符合原版风格
- [x] 单元测试覆盖率 ≥ 75%

### 最终集成测试
- [x] 能够生成完整的教堂地牢地图
- [x] 地图连通性测试通过
- [x] 相同种子生成相同地图
- [x] 地图生成时间 < 100ms
- [x] 无内存泄漏和崩溃
- [x] 游戏中可以正常探索地牢

---

## 🔜 下一步计划（Step 7）

完成 Step 6 后，下一步是 **Step 7: 怪物系统 Part 1 - 基础怪物**

**核心功能：**
- 怪物数据结构
- 怪物生成系统
- 怪物精灵加载
- 简单AI（站立、巡逻）

**参考代码：**
- `Source/monster.h` - 怪物结构
- `Source/monster.cpp::InitMonster()` - 怪物初始化
- `Source/monstdat.cpp` - 怪物数据

---

## 📊 总结

Step 6 是一个重要的里程碑，将实现地图生成系统的核心功能。通过分成3个子步骤逐步实现，确保每一步都有清晰的目标和验收标准。完成后，游戏将拥有随机生成的地牢，为后续的怪物、物品等系统打下坚实的基础。

**预计开发时间：**
- Step 6.1: 2-3天
- Step 6.2: 3-4天
- Step 6.3: 2-3天
- **总计：7-10天**

**代码量估算：** 900-1200行核心代码 + 300-400行测试代码

---

**文档版本：** 1.0  
**创建日期：** 2025-11-25  
**关联文档：**
- [master_plan.md](./master_plan.md)
- [FEATURE_COMPARISON.md](./FEATURE_COMPARISON.md)
- [Step 5.3 总结](./step-5.3-game-integration-summary.md)

