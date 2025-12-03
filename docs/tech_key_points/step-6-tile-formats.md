# Step 6 技术要点 - 瓦片格式详解

## 📅 创建日期
2025-11-25

## 🎯 文档目标

本文档详细介绍 Diablo 1 地图系统使用的瓦片格式，包括：
- **MIN 格式** - 微型瓦片数据
- **TIL 格式** - MegaTile 定义
- **SOL 格式** - 瓦片属性
- **等距投影系统** - 菱形瓦片渲染
- **坐标系统** - 多层级坐标转换

---

## 🎨 等距投影基础

### 什么是等距投影？

**等距投影（Isometric Projection）** 是一种将3D物体投影到2D平面的技术，特点是：
- 所有轴以相同比例缩放（Iso-metric = 等度量）
- 平行线保持平行
- 不会产生透视失真
- 创造伪3D效果

### Diablo 1 的菱形瓦片

Diablo 1 使用 **64×32 像素的菱形瓦片**：

```
         32px
    ┌──────────┐
    │    /\    │
    │   /  \   │  
32px│  /    \  │ 32px
    │ /  64  \ │
    │<───────→│
    │ \      / │
    │  \    /  │
    │   \  /   │
    │    \/    │
    └──────────┘
         32px
```

**常量定义：**
```rust
pub const TILE_WIDTH: i32 = 64;   // 菱形宽度
pub const TILE_HEIGHT: i32 = 32;  // 菱形高度
pub const HALF_TILE_WIDTH: i32 = 32;
pub const HALF_TILE_HEIGHT: i32 = 16;
```

**原版代码参考：** `Source/levels/dun_tile.hpp` Line 7-8

---

## 🗺️ 坐标系统详解

Diablo 1 使用**四层坐标系统**：

### 1. 世界坐标（World Coordinates）

**用途：** 游戏逻辑（碰撞检测、AI寻路）

```
World Space (正交网格)
     X →
  Y  ┌─┬─┬─┬─┐
  ↓  ├─┼─┼─┼─┤
     ├─┼─┼─┼─┤
     ├─┼─┼─┼─┤
     └─┴─┴─┴─┘

特点：
- 正方形网格
- 原点在左上角
- X轴向右，Y轴向下
- 单位：1 = 1个瓦片
```

### 2. 屏幕坐标（Screen Coordinates）

**用途：** 渲染显示

```
Screen Space (菱形网格)
       ◇
      ◇ ◇
     ◇ ◇ ◇
    ◇ ◇ ◇ ◇
     ◇ ◇ ◇
      ◇ ◇
       ◇

特点：
- 菱形网格
- 原点在左上角
- 单位：像素
```

### 3. MegaTile 坐标（地图生成层）

**用途：** 地图生成算法

```rust
pub const DMAXX: usize = 40;  // 地图宽度
pub const DMAXY: usize = 40;  // 地图高度

// 范围：[0, 40) × [0, 40)
// 每个 MegaTile = 2×2 个 MicroTile
```

### 4. MicroTile 坐标（渲染和碰撞层）

**用途：** 精确碰撞检测和渲染

```rust
pub const MAXDUNX: usize = 112;  // 渲染网格宽度
pub const MAXDUNY: usize = 112;  // 渲染网格高度

// 范围：[0, 112) × [0, 112)
// 包含 16 个瓦片的边界
```

---

## 🔄 坐标转换公式

### 世界坐标 ↔ 屏幕坐标

**原版代码参考：** `Source/engine/displacement.hpp` Line 151-168

#### 世界 → 屏幕（worldToScreen）

```rust
/// 将世界坐标转换为屏幕坐标
#[inline]
pub fn world_to_screen(world_x: i32, world_y: i32) -> (i32, i32) {
    // 等距投影变换矩阵：
    // [-32,  32] [dx]   [ 32(dy - dx)]
    // [-16, -16] [dy] = [-16(dy + dx)]
    
    let screen_x = (world_y - world_x) * 32;
    let screen_y = (world_y + world_x) * -16;
    (screen_x, screen_y)
}
```

**数学推导：**
```
旋转 -135° + 缩放
旋转矩阵：
  [cos(-135°), -sin(-135°)]   [-0.707,  0.707]
  [sin(-135°),  cos(-135°)] ≈ [-0.707, -0.707]

缩放 (宽度32, 高度16)：
  [-32,  32]
  [-16, -16]

结果：
  screen_x = (world_y - world_x) * 32
  screen_y = -(world_x + world_y) * 16
```

**示例：**
```rust
// (0, 0) → (0, 0)       // 原点
// (1, 0) → (-32, -16)   // 向右移动
// (0, 1) → (32, -16)    // 向下移动
// (1, 1) → (0, -32)     // 对角移动
```

#### 屏幕 → 世界（screenToWorld）

```rust
/// 将屏幕坐标转换为世界坐标
#[inline]
pub fn screen_to_world(screen_x: i32, screen_y: i32) -> (i32, i32) {
    // 逆矩阵变换
    let world_x = (2 * screen_y + screen_x) / -64;
    let world_y = (2 * screen_y - screen_x) / -64;
    (world_x, world_y)
}
```

**逆矩阵推导：**
```
原矩阵：
  [-32,  32]
  [-16, -16]

逆矩阵：
  [-1/64,  1/64]
  [-1/64, -1/64]

乘以 [-1, -1]：
  [1/64, -1/64]
  [1/64,  1/64]

简化：
  world_x = (screen_y / -32) + (screen_x / -64)
          = (2 * screen_y + screen_x) / -64
  
  world_y = (screen_y / -32) - (screen_x / -64)
          = (2 * screen_y - screen_x) / -64
```

---

### MegaTile ↔ MicroTile

```rust
/// MegaTile → MicroTile
#[inline]
pub fn mega_to_micro(mega_x: i32, mega_y: i32) -> (i32, i32) {
    // 每个 MegaTile = 2×2 个 MicroTile
    // 加上 16 个瓦片的边界
    (mega_x * 2 + 16, mega_y * 2 + 16)
}

/// MicroTile → MegaTile
#[inline]
pub fn micro_to_mega(micro_x: i32, micro_y: i32) -> (i32, i32) {
    // 减去边界，除以2
    ((micro_x - 16) / 2, (micro_y - 16) / 2)
}
```

**为什么有 16 的偏移？**
- 地图边界预留空间
- 避免数组越界
- 方便相机滚动

---

## 📦 MIN 格式 - 微型瓦片数据

### 文件结构

MIN 文件存储**微型瓦片索引**，每个索引指向 CEL 文件中的一帧。

```
文件布局：
┌─────────────────────────────┐
│  File Header (无)            │
├─────────────────────────────┤
│  Tile 0: u16 (2 bytes)      │  ← 第一个微型瓦片
├─────────────────────────────┤
│  Tile 1: u16 (2 bytes)      │
├─────────────────────────────┤
│  ...                         │
├─────────────────────────────┤
│  Tile N: u16 (2 bytes)      │  ← 最后一个微型瓦片
└─────────────────────────────┘

总大小 = N × 2 字节
```

### u16 编码格式

每个 u16 值编码了**瓦片类型**和**CEL帧索引**：

```
Bit Layout:
  15  14  13  12  11  10   9   8   7   6   5   4   3   2   1   0
┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
│ T │ T │ T │ 0 │ I │ I │ I │ I │ I │ I │ I │ I │ I │ I │ I │ I │
└───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘
  ↑───────↑       ↑─────────────────────────────────────────────↑
  Tile Type       CEL Frame Index (1-based)
  (3 bits)        (12 bits, 0-4095)
```

**解码方法：**
```rust
let tile_data: u16 = 0x1ABC;  // 示例数据

// 提取瓦片类型（高3位）
let tile_type = (tile_data >> 12) & 0x7;  // 0x1

// 提取CEL帧索引（低12位）
let cel_index = tile_data & 0xFFF;  // 0xABC (2748)
```

### 瓦片类型枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TileType {
    Square = 0,              // 正方形 (32×32)
    TransparentSquare = 1,   // 透明正方形 (RLE编码)
    LeftTriangle = 2,        // 左三角 (32×31)
    RightTriangle = 3,       // 右三角 (32×31)
    LeftTrapezoid = 4,       // 左梯形 (32×32)
    RightTrapezoid = 5,      // 右梯形 (32×32)
}

impl TileType {
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            0 => Some(TileType::Square),
            1 => Some(TileType::TransparentSquare),
            2 => Some(TileType::LeftTriangle),
            3 => Some(TileType::RightTriangle),
            4 => Some(TileType::LeftTrapezoid),
            5 => Some(TileType::RightTrapezoid),
            _ => None,
        }
    }
}
```

**原版代码参考：** `Source/levels/dun_tile.hpp` Line 20-78

### Rust 实现示例

```rust
pub struct MinData {
    pub tiles: Vec<MicroTile>,
}

pub struct MicroTile {
    pub cel_index: u16,      // CEL 帧索引 (1-based)
    pub tile_type: TileType, // 瓦片类型
}

impl MinData {
    /// 从 MPQ 加载 MIN 文件
    pub fn load_from_mpq(mpq: &Mpq, path: &str) -> Result<Self> {
        let data = mpq.read_file(path)?;
        
        // 验证文件大小
        if data.len() % 2 != 0 {
            return Err(anyhow!("MIN file size must be even"));
        }
        
        // 解析每个 u16
        let tiles: Vec<MicroTile> = data
            .chunks_exact(2)
            .map(|chunk| {
                let value = u16::from_le_bytes([chunk[0], chunk[1]]);
                Self::decode_tile(value)
            })
            .collect();
        
        Ok(MinData { tiles })
    }
    
    /// 解码单个瓦片
    fn decode_tile(value: u16) -> MicroTile {
        let cel_index = value & 0xFFF;  // 低12位
        let type_bits = ((value >> 12) & 0x7) as u8;  // 高3位
        let tile_type = TileType::from_bits(type_bits)
            .unwrap_or(TileType::Square);
        
        MicroTile { cel_index, tile_type }
    }
}
```

**原版代码参考：** `Source/levels/gendung.cpp::LoadMinData()` Line 76-102

---

## 🧱 TIL 格式 - MegaTile 定义

### 文件结构

TIL 文件定义 **MegaTile**，每个 MegaTile 由 4 个 MicroTile 组成（2×2）。

```
文件布局：
┌─────────────────────────────┐
│  File Header (无)            │
├─────────────────────────────┤
│  MegaTile 0: 8 bytes        │
│    ├─ micro1: u16           │  ← 左上
│    ├─ micro2: u16           │  ← 右上
│    ├─ micro3: u16           │  ← 左下
│    └─ micro4: u16           │  ← 右下
├─────────────────────────────┤
│  MegaTile 1: 8 bytes        │
├─────────────────────────────┤
│  ...                         │
└─────────────────────────────┘

每个 MegaTile = 4 × u16 = 8 字节
总大小 = N × 8 字节
```

### MegaTile 布局

```
MegaTile (2×2 MicroTile)
┌────────┬────────┐
│ micro1 │ micro2 │  ← 第一行
├────────┼────────┤
│ micro3 │ micro4 │  ← 第二行
└────────┴────────┘
   ↑        ↑
 左列      右列

每个 microN 是 MIN 数组的索引
```

### Rust 实现示例

```rust
#[derive(Debug, Clone, Copy)]
pub struct MegaTile {
    pub micro1: u16,  // 左上 (MIN索引)
    pub micro2: u16,  // 右上
    pub micro3: u16,  // 左下
    pub micro4: u16,  // 右下
}

pub struct TilData {
    pub mega_tiles: Vec<MegaTile>,
}

impl TilData {
    /// 从 MPQ 加载 TIL 文件
    pub fn load_from_mpq(mpq: &Mpq, path: &str) -> Result<Self> {
        let data = mpq.read_file(path)?;
        
        // 验证文件大小
        if data.len() % 8 != 0 {
            return Err(anyhow!("TIL file size must be multiple of 8"));
        }
        
        // 解析每个 MegaTile (8字节)
        let mega_tiles: Vec<MegaTile> = data
            .chunks_exact(8)
            .map(|chunk| MegaTile {
                micro1: u16::from_le_bytes([chunk[0], chunk[1]]),
                micro2: u16::from_le_bytes([chunk[2], chunk[3]]),
                micro3: u16::from_le_bytes([chunk[4], chunk[5]]),
                micro4: u16::from_le_bytes([chunk[6], chunk[7]]),
            })
            .collect();
        
        Ok(TilData { mega_tiles })
    }
    
    /// 获取指定 MegaTile
    pub fn get_tile(&self, index: usize) -> Option<&MegaTile> {
        self.mega_tiles.get(index)
    }
}
```

**原版代码参考：** `Source/levels/gendung.cpp::DRLG_LPass3()` Line 768-801

---

## 🛡️ SOL 格式 - 瓦片属性

### 文件结构

SOL 文件定义每个**瓦片的物理属性**（碰撞、光线等）。

```
文件布局：
┌─────────────────────────────┐
│  File Header (无)            │
├─────────────────────────────┤
│  Tile 0: u8 (1 byte)        │  ← 第一个瓦片的属性
├─────────────────────────────┤
│  Tile 1: u8 (1 byte)        │
├─────────────────────────────┤
│  ...                         │
├─────────────────────────────┤
│  Tile N: u8 (1 byte)        │  ← 最后一个瓦片的属性
└─────────────────────────────┘

总大小 = N 字节
```

### 属性标志位

每个 u8 字节是**位标志**：

```
Bit Layout:
  7   6   5   4   3   2   1   0
┌───┬───┬───┬───┬───┬───┬───┬───┐
│ T │ 0 │ R │ L │ P │ M │ B │ S │
└───┴───┴───┴───┴───┴───┴───┴───┘
  ↑       ↑   ↑   ↑   ↑   ↑   ↑
  │       │   │   │   │   │   └─ Solid (不可行走)
  │       │   │   │   │   └───── Block Light (阻挡光线)
  │       │   │   │   └─────── Block Missile (阻挡飞行物)
  │       │   │   └───────── Transparent (透明)
  │       │   └─────────── Transparent Left (左侧透明)
  │       └───────────── Transparent Right (右侧透明)
  └─────────────────── Trap (陷阱)
```

### 属性定义

```rust
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TileProperties: u8 {
        const NONE              = 0;
        const SOLID             = 1 << 0;  // 0x01 - 不可行走
        const BLOCK_LIGHT       = 1 << 1;  // 0x02 - 阻挡光线
        const BLOCK_MISSILE     = 1 << 2;  // 0x04 - 阻挡飞行物
        const TRANSPARENT       = 1 << 3;  // 0x08 - 透明
        const TRANSPARENT_LEFT  = 1 << 4;  // 0x10 - 左侧透明
        const TRANSPARENT_RIGHT = 1 << 5;  // 0x20 - 右侧透明
        const TRAP              = 1 << 7;  // 0x80 - 陷阱
    }
}
```

**原版代码参考：** `Source/levels/dun_tile.hpp` Line 105-117

### Rust 实现示例

```rust
pub struct SolData {
    pub properties: Vec<TileProperties>,
}

impl SolData {
    /// 从 MPQ 加载 SOL 文件
    pub fn load_from_mpq(mpq: &Mpq, path: &str) -> Result<Self> {
        let data = mpq.read_file(path)?;
        
        // 每个字节是一个瓦片的属性
        let properties: Vec<TileProperties> = data
            .iter()
            .map(|&byte| TileProperties::from_bits_truncate(byte))
            .collect();
        
        Ok(SolData { properties })
    }
    
    /// 检查瓦片是否可行走
    pub fn is_walkable(&self, tile_index: usize) -> bool {
        if let Some(&props) = self.properties.get(tile_index) {
            !props.contains(TileProperties::SOLID)
        } else {
            false
        }
    }
    
    /// 检查瓦片是否阻挡光线
    pub fn blocks_light(&self, tile_index: usize) -> bool {
        if let Some(&props) = self.properties.get(tile_index) {
            props.contains(TileProperties::BLOCK_LIGHT)
        } else {
            false
        }
    }
}
```

**原版代码参考：** `Source/levels/gendung.cpp::LoadLevelSOLData()`

---

## 🔗 数据关联

### 完整的瓦片渲染流程

```
1. 地图生成层 (MegaTile)
   dungeon[x][y] = tile_id (1-based)
   ↓

2. TIL 查询
   mega_tile = TIL_data[tile_id - 1]
   ↓ 得到 4 个 MicroTile 索引
   
3. MIN 查询
   micro1 = MIN_data[mega_tile.micro1]
   micro2 = MIN_data[mega_tile.micro2]
   micro3 = MIN_data[mega_tile.micro3]
   micro4 = MIN_data[mega_tile.micro4]
   ↓ 得到 4 个 CEL 帧索引和瓦片类型
   
4. SOL 查询（碰撞检测）
   properties = SOL_data[tile_id - 1]
   ↓ 检查是否可行走、是否阻挡光线等
   
5. CEL 渲染
   for each micro_tile:
       render_cel_frame(micro_tile.cel_index, micro_tile.tile_type)
```

### 示例代码

```rust
pub struct TileSystem {
    min_data: MinData,
    til_data: TilData,
    sol_data: SolData,
    cel_sprite: ClxSprite,
}

impl TileSystem {
    /// 渲染单个 MegaTile
    pub fn render_mega_tile(
        &self,
        tile_id: u8,
        screen_x: i32,
        screen_y: i32,
        canvas: &mut Canvas,
    ) -> Result<()> {
        if tile_id == 0 {
            return Ok(()); // 空瓦片
        }
        
        // 1. 从 TIL 获取 MegaTile
        let mega_tile = self.til_data.get_tile((tile_id - 1) as usize)
            .ok_or_else(|| anyhow!("Invalid tile_id: {}", tile_id))?;
        
        // 2. 渲染 4 个 MicroTile
        self.render_micro_tile(mega_tile.micro1, screen_x, screen_y, canvas)?;
        self.render_micro_tile(mega_tile.micro2, screen_x + 32, screen_y, canvas)?;
        self.render_micro_tile(mega_tile.micro3, screen_x, screen_y + 16, canvas)?;
        self.render_micro_tile(mega_tile.micro4, screen_x + 32, screen_y + 16, canvas)?;
        
        Ok(())
    }
    
    /// 渲染单个 MicroTile
    fn render_micro_tile(
        &self,
        min_index: u16,
        screen_x: i32,
        screen_y: i32,
        canvas: &mut Canvas,
    ) -> Result<()> {
        // 从 MIN 获取 MicroTile
        let micro_tile = self.min_data.tiles.get(min_index as usize)
            .ok_or_else(|| anyhow!("Invalid min_index: {}", min_index))?;
        
        // 渲染对应的 CEL 帧
        self.cel_sprite.render_frame(
            micro_tile.cel_index as usize - 1,  // 转为0-based
            screen_x,
            screen_y,
            canvas,
        )?;
        
        Ok(())
    }
    
    /// 检查瓦片是否可行走
    pub fn is_tile_walkable(&self, tile_id: u8) -> bool {
        if tile_id == 0 {
            return false;
        }
        self.sol_data.is_walkable((tile_id - 1) as usize)
    }
}
```

---

## 📂 文件路径

教堂地牢（L1）的瓦片文件：

```
levels/l1data/l1.min    - 微型瓦片索引（~2600个瓦片）
levels/l1data/l1.til    - MegaTile定义（~200个MegaTile）
levels/l1data/l1.sol    - 瓦片属性（~200个属性）
levels/l1data/l1.cel    - 瓦片图形（CEL格式）
```

**完整路径结构：**
```
DIABDAT.MPQ
└── levels/
    ├── l1data/          # 教堂 (Cathedral)
    │   ├── l1.min
    │   ├── l1.til
    │   ├── l1.sol
    │   └── l1.cel
    ├── l2data/          # 地下墓穴 (Catacombs)
    │   ├── l2.min
    │   ├── l2.til
    │   ├── l2.sol
    │   └── l2.cel
    ├── l3data/          # 洞穴 (Caves)
    │   └── ...
    ├── l4data/          # 地狱 (Hell)
    │   └── ...
    └── towndata/        # 城镇 (Town)
        └── ...
```

---

## 🧪 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tile_type_decoding() {
        let tile_data: u16 = 0x1ABC;
        
        let tile_type = (tile_data >> 12) & 0x7;
        let cel_index = tile_data & 0xFFF;
        
        assert_eq!(tile_type, 1);  // TransparentSquare
        assert_eq!(cel_index, 0xABC);  // 2748
    }
    
    #[test]
    fn test_load_min_file() {
        let mpq = Mpq::open("assets/DIABDAT.MPQ").unwrap();
        let min_data = MinData::load_from_mpq(&mpq, "levels/l1data/l1.min").unwrap();
        
        assert!(!min_data.tiles.is_empty());
        assert!(min_data.tiles.len() > 2000);
    }
    
    #[test]
    fn test_load_til_file() {
        let mpq = Mpq::open("assets/DIABDAT.MPQ").unwrap();
        let til_data = TilData::load_from_mpq(&mpq, "levels/l1data/l1.til").unwrap();
        
        assert!(!til_data.mega_tiles.is_empty());
        
        // 检查第一个 MegaTile
        let mega = til_data.get_tile(0).unwrap();
        assert!(mega.micro1 < 3000);
        assert!(mega.micro2 < 3000);
    }
    
    #[test]
    fn test_tile_properties() {
        let props = TileProperties::SOLID | TileProperties::BLOCK_LIGHT;
        
        assert!(props.contains(TileProperties::SOLID));
        assert!(props.contains(TileProperties::BLOCK_LIGHT));
        assert!(!props.contains(TileProperties::TRANSPARENT));
    }
    
    #[test]
    fn test_coordinate_conversion() {
        // 世界 → 屏幕 → 世界
        let (wx, wy) = (5, 7);
        let (sx, sy) = world_to_screen(wx, wy);
        let (wx2, wy2) = screen_to_world(sx, sy);
        
        assert_eq!(wx, wx2);
        assert_eq!(wy, wy2);
    }
    
    #[test]
    fn test_mega_micro_conversion() {
        let (mega_x, mega_y) = (10, 10);
        let (micro_x, micro_y) = mega_to_micro(mega_x, mega_y);
        
        assert_eq!(micro_x, 36);  // 10 * 2 + 16
        assert_eq!(micro_y, 36);
        
        let (mega_x2, mega_y2) = micro_to_mega(micro_x, micro_y);
        assert_eq!(mega_x, mega_x2);
        assert_eq!(mega_y, mega_y2);
    }
}
```

---

## ⚠️ 常见陷阱

### 1. 索引是 1-based

**问题：** MIN/TIL 索引是 1-based（0 表示空）

```rust
// ❌ 错误
let micro_tile = min_data.tiles[mega_tile.micro1];

// ✅ 正确
if mega_tile.micro1 > 0 {
    let micro_tile = min_data.tiles[(mega_tile.micro1 - 1) as usize];
}
```

### 2. 字节序问题

**问题：** 所有数据都是小端序（Little Endian）

```rust
// ✅ 正确
let value = u16::from_le_bytes([chunk[0], chunk[1]]);

// ❌ 错误（大端序）
let value = u16::from_be_bytes([chunk[0], chunk[1]]);
```

### 3. MegaTile 布局

**问题：** MicroTile 的顺序是特定的

```
正确：                错误：
┌────┬────┐          ┌────┬────┐
│ m1 │ m2 │          │ m1 │ m3 │
├────┼────┤          ├────┼────┤
│ m3 │ m4 │          │ m2 │ m4 │
└────┴────┘          └────┴────┘
```

### 4. 屏幕坐标偏移

**问题：** 等距投影的 Y 坐标是负数

```rust
// ✅ 正确
let screen_y = (world_y + world_x) * -16;  // 负号！

// ❌ 错误
let screen_y = (world_y + world_x) * 16;
```

---

## 📚 参考资源

### 原版代码

| 功能 | 文件 | 行号 |
|------|------|------|
| MIN 加载 | `gendung.cpp` | 76-102 |
| TIL 加载 | `gendung.cpp` | 768-801 |
| SOL 加载 | `gendung.cpp` | `LoadLevelSOLData()` |
| 瓦片类型 | `dun_tile.hpp` | 20-78 |
| 等距投影 | `displacement.hpp` | 151-168 |
| 常量定义 | `dun_tile.hpp` | 7-8 |

### 学习资源

- [Isometric Game Programming](https://www.gamasutra.com/view/feature/131801/designing_a_2d_tile_map_engine.php)
- [Red Blob Games - Grids](https://www.redblobgames.com/grids/hexagons/)
- [Diablo 1 File Formats](http://www.zezula.net/en/mpq/mpqformat.html)

---

## 📊 数据规模统计

### 教堂地牢（L1）

| 文件 | 大小 | 数量 | 说明 |
|------|------|------|------|
| l1.min | ~5 KB | ~2600 项 | 微型瓦片索引 |
| l1.til | ~1.6 KB | ~200 项 | MegaTile定义 |
| l1.sol | ~200 B | ~200 项 | 瓦片属性 |
| l1.cel | ~200 KB | ~2600 帧 | 瓦片图形 |

**内存占用估算：**
```rust
// MIN: 2600 × 4 bytes (MicroTile结构) = 10 KB
// TIL: 200 × 8 bytes (MegaTile结构) = 1.6 KB
// SOL: 200 × 1 byte (TileProperties) = 200 B
// CEL: ~200 KB (解码后可能更大)
// 总计: ~212 KB
```

---

## 🎯 总结

### 关键要点

1. **等距投影** - 64×32 菱形瓦片，创造 2.5D 效果
2. **四层坐标** - 世界、屏幕、MegaTile、MicroTile
3. **MIN 格式** - 微型瓦片索引（u16编码）
4. **TIL 格式** - MegaTile定义（2×2组合）
5. **SOL 格式** - 瓦片属性（位标志）
6. **索引关联** - dungeon → TIL → MIN → CEL

### 实现建议

1. **先实现等距投影** - 坐标转换是基础
2. **加载顺序** - MIN → TIL → SOL → CEL
3. **缓存优化** - 预加载所有瓦片数据
4. **边界检查** - 所有索引访问都要验证
5. **测试驱动** - 每个格式都写单元测试

---

**文档版本：** 1.0  
**创建日期：** 2025-11-25  
**相关文档：**
- [Step 6 设计文档](../step-6-dungeon-generation-design.md)
- [Step 6 实施计划](../step-6-implementation-plan.md)
- [master_plan.md](../master_plan.md)















