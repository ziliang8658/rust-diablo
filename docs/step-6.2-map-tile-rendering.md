# Step 6.2: 地图、瓦片和纹理渲染系统

## 📋 目标

实现完整的地图和瓦片渲染系统，包括：

1. **CEL/PAL解析器** - 原生Rust实现，解析Diablo原版图形文件
2. **6种TileType解码器** - 支持所有瓦片形状（Square, Triangle, Trapezoid等）
3. **纹理管理器** - 统一管理解码后的瓦片纹理
4. **地图渲染** - 正确渲染地板和墙体，实现等距投影

## 🎯 核心原则

**完全复刻原版实现！**

- ✅ 使用与C++相同的解码算法
- ✅ 保持与原版100%兼容
- ✅ 参考原版代码的每一处细节
- ✅ 不自由发挥，严格对照实现

## 📐 架构设计

### 原版C++架构（参考）

```
加载流程（Source/diablo.cpp: LoadLvlGFX）:
  LoadFileInMem("l1.cel") → pDungeonCels (raw bytes)
      ↓
  LoadPalette("town.pal") → 256色调色板
      ↓
  SetDungeonMicros() → 解析MIN，建立frame映射
      ↓
  ReencodeDungeonCels() → 优化内存布局，移除padding
      ↓
  GetDunFrame(frame_idx) → 返回像素数据指针
      ↓
  RenderTileFrame() → 渲染到屏幕

渲染流程（Source/engine/render/scrollrt.cpp）:
  DrawTileContent() → 遍历可见区域
      ↓
  两次渲染Pass:
    Pass 1: DrawFloorTile() → 渲染地板（LeftTriangle + RightTriangle）
    Pass 2: DrawCell() → 渲染墙体（Trapezoid, Square等）
      ↓
  DrawCell() 细节:
    - 遍历4个microtile（从DPieceMicros获取）
    - 对每个microtile调用RenderTileFrame()
    - 应用光照（dLight数组）
      ↓
  RenderTileFrame() → 根据TileType分发到具体渲染器
```

**参考文件：**
- `Source/diablo.cpp` - `LoadLvlGFX()`
- `Source/levels/gendung.cpp` - `SetDungeonMicros()`, `ReencodeDungeonCels()`
- `Source/levels/reencode_dun_cels.cpp` - 6种TileType的重编码函数
- `Source/engine/render/scrollrt.cpp` - `DrawTileContent()`, `DrawCell()`, `DrawFloorTile()`
- `Source/engine/render/dun_render.cpp` - `RenderTileFrame()`, 各种形状渲染器

### Rust实现架构（对应）

```
rust-diablo/src/
├── resources/
│   ├── cel_loader.rs          [新建] - CEL文件解析
│   ├── palette.rs              [新建] - PAL调色板加载
│   ├── dungeon_cel.rs          [扩展] - Dungeon CEL特殊处理
│   └── resource_manager.rs     [已有] - 统一资源管理（含MPQ支持）
│
├── tiles/
│   ├── decoder/                [新建目录]
│   │   ├── mod.rs             - 解码器统一接口
│   │   ├── square.rs          - Square解码器
│   │   ├── triangle.rs        - LeftTriangle/RightTriangle解码器
│   │   ├── trapezoid.rs       - LeftTrapezoid/RightTrapezoid解码器
│   │   └── transparent.rs     - TransparentSquare解码器（RLE）
│   │
│   ├── texture_manager.rs      [新建] - 纹理管理和缓存
│   ├── min.rs                  [已有] - MicroTile定义
│   ├── til.rs                  [已有] - MegaTile定义
│   └── types.rs                [已有] - TileType枚举
│
└── world/
    └── mod.rs                  [修改] - 地图渲染逻辑
```

## 🔧 核心模块详解

### 1. CEL文件解析器（dungeon_cel.rs）⚠️ **基础已有，需扩展**

**参考：** `Source/levels/gendung.cpp: LoadFileInMem()`, `GetDunFrame()`

**状态：** `src/resources/dungeon_cel.rs` 已有基础框架，但只支持Square格式！

**已有功能：**
```rust
pub struct DungeonCelSprite {
    pub frames: Vec<DungeonCelFrame>,
}

impl DungeonCelSprite {
    /// 从字节流加载CEL文件 ✅ 已实现（基础）
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    
    // ⚠️ parse_frame() 只支持32x32 Square格式！
    // 行159-192: TODO注释明确指出需要支持"reencoded format"
}

pub struct DungeonCelFrame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<Option<u8>>,  // 索引色，None = 透明
}
```

**需要扩展：**
```rust
// ❌ 缺少：与MIN数据集成，根据TileType选择解码器
// ❌ 缺少：6种TileType的解码逻辑（见下一节）
```

**CEL文件格式：**
```
Offset | Size | Description
-------|------|-------------
0x00   | 4    | Frame Count (N)
0x04   | 4    | Offset[0] - 第1帧起始位置
0x08   | 4    | Offset[1] - 第2帧起始位置
...    | ...  | ...
0x04+N*4 | 4  | Offset[N] - 文件末尾
Offset[0] | Variable | Frame 0 数据（已编码）
Offset[1] | Variable | Frame 1 数据（已编码）
...
```

### 2. 调色板加载器（palette.rs）✅ **已完成**

**参考：** `Source/engine/palette.cpp: LoadPalette()`, `Source/engine/load_pcx.cpp`

**状态：** `src/resources/palette.rs` 已完整实现，无需修改！

**已有功能：**
```rust
pub struct Palette {
    pub colors: [Color; 256],
}

impl Palette {
    /// 从PAL文件加载（768字节：256 × 3）✅ 已实现
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    
    /// 从MPQ直接加载 ✅ 已实现
    pub fn from_mpq(mpq: &mut MpqManager, path: &str) -> Result<Self>;
    
    /// 索引到RGB转换 ✅ 已实现
    pub fn to_rgb(&self, index: u8) -> Color;
    
    /// 索引到RGBA转换（支持透明度）✅ 已实现
    pub fn to_rgba(&self, index: u8, transparent: bool) -> [u8; 4];
    
    /// 批量索引转RGBA ✅ 已实现
    pub fn indices_to_rgba(&self, indices: &[u8], transparent: bool) -> Vec<u8>;
}
```

**PAL文件格式：**
```
每个颜色3字节（R, G, B），共768字节
格式：已经是8位RGB（0-255），无需转换！

注意：之前误认为需要6位到8位转换是错误的。
经过验证原版C++代码（Source/engine/load_pcx.cpp:53-57），
PAL文件直接存储8位RGB值，不需要任何转换。
```

**参考代码：** 
- `Source/engine/palette.cpp:175-186` - `LoadPalette()`
- `Source/engine/load_pcx.cpp:53-58` - PAL文件直接读取
- `rust-diablo/src/resources/palette.rs` - ✅ 完整实现

**测试状态：** ✅ 有完善的单元测试（13个测试用例）

### 3. TileType解码器（tiles/decoder/）

**核心原则：** 严格对照原版 `ReencodeDungeonCels*` 函数实现！

#### 3.1 Square（32x32纯矩形）

**参考：** `Source/levels/reencode_dun_cels.cpp: ReencodeDungeonCelsSquare()`

```rust
pub fn decode_square(raw_data: &[u8]) -> Result<Vec<u8>> {
    // 原版逻辑：直接复制32*32=1024字节
    if raw_data.len() < 1024 {
        return Err(anyhow!("Square data too short"));
    }
    Ok(raw_data[..1024].to_vec())
}
```

**布局：**
```
32x32纯像素阵列，无padding
总大小：1024字节
```

#### 3.2 LeftTriangle（32x31变宽行，左对齐）

**参考：** `Source/levels/reencode_dun_cels.cpp: ReencodeDungeonCelsLeftTriangle()`

```rust
pub fn decode_left_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; 32 * 31];  // 32x31输出
    let mut src = 0;
    let mut dst = 0;
    
    // 下半部分（0-15行，16行，每行一对）
    for i in 0..8 {
        src += 2;  // 跳过2字节padding
        
        // 第一行（宽度: 2, 6, 10, 14, 18, 22, 26, 30）
        let width1 = 2 + i * 4;
        output[dst..dst+width1].copy_from_slice(&raw_data[src..src+width1]);
        src += width1;
        dst += 32;
        
        // 第二行（宽度: 4, 8, 12, 16, 20, 24, 28, 32）
        let width2 = 4 + i * 4;
        output[dst..dst+width2].copy_from_slice(&raw_data[src..src+width2]);
        src += width2;
        dst += 32;
    }
    
    // 上半部分（16-30行，15行，每行一对）
    let mut width = 32;
    for _ in 0..7 {
        src += 2;  // 跳过padding
        
        width -= 2;
        output[dst..dst+width].copy_from_slice(&raw_data[src..src+width]);
        src += width;
        dst += 32;
        
        width -= 2;
        output[dst..dst+width].copy_from_slice(&raw_data[src..src+width]);
        src += width;
        dst += 32;
    }
    
    // 最后一行（行30，宽度2）
    src += 2;
    width -= 2;
    output[dst..dst+width].copy_from_slice(&raw_data[src..src+width]);
    
    Ok(output)
}
```

**布局说明：**
```
原版数据中，每对行之间有2字节padding（用于对齐）
解码后输出为32宽度阵列，右侧填充0（透明）

行0:  [##]                            padding=2, width=2
行1:  [####]                          width=4
行2:  [######]                        padding=2, width=6
行3:  [########]                      width=8
...
行14: [############################]  width=30
行15: [##############################] width=32
行16: [############################]  padding=2, width=30
行17: [##########################]    width=28
...
行30: [##]                            padding=2, width=2
```

#### 3.3 RightTriangle（32x31变宽行，右对齐）

**参考：** `Source/levels/reencode_dun_cels.cpp: ReencodeDungeonCelsRightTriangle()`

```rust
pub fn decode_right_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; 32 * 31];
    let mut src = 0;
    let mut dst = 0;
    
    // 下半部分（0-15行）
    for i in 0..8 {
        // 第一行（右对齐）
        let width1 = 2 + i * 4;
        let offset1 = 32 - width1;
        output[dst+offset1..dst+offset1+width1]
            .copy_from_slice(&raw_data[src..src+width1]);
        src += width1 + 2;  // +2跳过padding
        dst += 32;
        
        // 第二行（右对齐）
        let width2 = 4 + i * 4;
        let offset2 = 32 - width2;
        output[dst+offset2..dst+offset2+width2]
            .copy_from_slice(&raw_data[src..src+width2]);
        src += width2;
        dst += 32;
    }
    
    // 上半部分（16-30行）
    let mut width = 32;
    for _ in 0..7 {
        width -= 2;
        let offset = 32 - width;
        output[dst+offset..dst+offset+width]
            .copy_from_slice(&raw_data[src..src+width]);
        src += width + 2;  // +2跳过padding
        dst += 32;
        
        width -= 2;
        let offset = 32 - width;
        output[dst+offset..dst+offset+width]
            .copy_from_slice(&raw_data[src..src+width]);
        src += width;
        dst += 32;
    }
    
    // 最后一行
    width -= 2;
    let offset = 32 - width;
    output[dst+offset..dst+offset+width]
        .copy_from_slice(&raw_data[src..src+width]);
    
    Ok(output)
}
```

**布局说明：**
```
与LeftTriangle镜像，右对齐，左侧填充0

行0:                              [##] offset=30, width=2
行1:                          [####]   offset=28, width=4
...
行15: [##############################] offset=0, width=32
...
行30:                              [##] offset=30, width=2
```

#### 3.4 LeftTrapezoid（下三角+上矩形，左对齐）

**参考：** `Source/levels/reencode_dun_cels.cpp: ReencodeDungeonCelsLeftTrapezoid()`

```rust
pub fn decode_left_trapezoid(raw_data: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; 32 * 32];  // 32x32输出
    let mut src = 0;
    let mut dst = 0;
    
    // 下半部分：LeftTriangle的下半部（0-15行）
    for i in 0..8 {
        src += 2;  // 跳过padding
        
        let width1 = 2 + i * 4;
        output[dst..dst+width1].copy_from_slice(&raw_data[src..src+width1]);
        src += width1;
        dst += 32;
        
        let width2 = 4 + i * 4;
        output[dst..dst+width2].copy_from_slice(&raw_data[src..src+width2]);
        src += width2;
        dst += 32;
    }
    
    // 上半部分：32x16纯矩形（无padding）
    let rect_size = 32 * 16;
    output[dst..dst+rect_size].copy_from_slice(&raw_data[src..src+rect_size]);
    
    Ok(output)
}
```

**布局说明：**
```
行0-15:  LeftTriangle下半部（变宽）
行16-31: 32x16纯矩形（用于墙体上部）
```

#### 3.5 RightTrapezoid（下三角+上矩形，右对齐）

**参考：** `Source/levels/reencode_dun_cels.cpp: ReencodeDungeonCelsRightTrapezoid()`

```rust
pub fn decode_right_trapezoid(raw_data: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; 32 * 32];
    let mut src = 0;
    let mut dst = 0;
    
    // 下半部分：RightTriangle的下半部（0-15行）
    for i in 0..8 {
        let width1 = 2 + i * 4;
        let offset1 = 32 - width1;
        output[dst+offset1..dst+offset1+width1]
            .copy_from_slice(&raw_data[src..src+width1]);
        src += width1 + 2;  // +2跳过padding
        dst += 32;
        
        let width2 = 4 + i * 4;
        let offset2 = 32 - width2;
        output[dst+offset2..dst+offset2+width2]
            .copy_from_slice(&raw_data[src..src+width2]);
        src += width2;
        dst += 32;
    }
    
    // 上半部分：32x16纯矩形
    let rect_size = 32 * 16;
    output[dst..dst+rect_size].copy_from_slice(&raw_data[src..src+rect_size]);
    
    Ok(output)
}
```

#### 3.6 TransparentSquare（RLE编码，32x32）

**参考：** `Source/levels/reencode_dun_cels.cpp: ReencodeDungeonCelsTransparentSquare()`

```rust
pub fn decode_transparent_square(raw_data: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; 32 * 32];
    let mut src = 0;
    let mut dst = 0;
    
    // 32行RLE解码
    for _row in 0..32 {
        let mut draw_width = 32;
        
        while draw_width > 0 {
            if src >= raw_data.len() {
                return Err(anyhow!("RLE data incomplete"));
            }
            
            let val = raw_data[src] as i8;  // signed byte
            src += 1;
            
            if val > 0 {
                // 正数：val个实际像素
                let count = val as usize;
                output[dst..dst+count].copy_from_slice(&raw_data[src..src+count]);
                src += count;
                dst += count;
                draw_width -= count;
            } else {
                // 负数或0：-val个透明像素（已初始化为0，直接跳过）
                let count = (-val) as usize;
                dst += count;
                draw_width -= count;
            }
        }
    }
    
    Ok(output)
}
```

**RLE编码说明：**
```
每行独立编码
控制字节：
  > 0: 后面跟val个实际像素数据
  < 0: -val个透明像素（跳过）
  = 0: 特殊情况，表示0个透明像素
```

### 4. 纹理管理器（texture_manager.rs）

```rust
pub struct TileTextureManager {
    cel_data: CelData,
    palette: Palette,
    decoded_cache: HashMap<u16, Vec<u8>>,  // frame_idx -> decoded pixels
    min_data: MinData,  // MicroTile定义（frame到TileType的映射）
}

impl TileTextureManager {
    /// 加载完整tileset
    pub fn load_for_dungeon(
        dungeon_type: DungeonType,
        mpq_manager: &mut MpqManager
    ) -> Result<Self>;
    
    /// 获取解码后的纹理（带缓存）
    pub fn get_decoded_tile(&mut self, frame_idx: u16) -> Result<&[u8]>;
    
    /// 批量预解码所有常用瓦片
    pub fn preload_common_tiles(&mut self) -> Result<()>;
}
```

### 5. 地图渲染更新（world/mod.rs）

**参考：** `Source/engine/render/scrollrt.cpp: DrawTileContent()`

```rust
impl World {
    pub fn render_with_tiles(&self, engine: &mut Engine, camera: &Camera) {
        // 遍历可见区域（等距投影）
        for map_y in visible_y_range {
            for map_x in visible_x_range {
                let mega_tile_id = self.tiles[map_y][map_x];
                let mega_tile = &self.til_data.tiles[mega_tile_id];
                
                // 计算屏幕坐标（等距投影）
                let screen_x = (map_x - map_y) * 32;
                let screen_y = (map_x + map_y) * 16;
                
                // Pass 1: 渲染地板（如果是地板瓦片）
                if is_floor_tile(mega_tile) {
                    self.render_floor(engine, mega_tile, screen_x, screen_y);
                }
                
                // Pass 2: 渲染墙体（如果有墙体）
                if has_walls(mega_tile) {
                    self.render_walls(engine, mega_tile, screen_x, screen_y);
                }
            }
        }
    }
    
    fn render_floor(&self, engine: &mut Engine, mega_tile: &MegaTile, x: i32, y: i32) {
        // 地板由LeftTriangle和RightTriangle组成
        let left_micro = mega_tile.micro1;  // LeftTriangle
        let right_micro = mega_tile.micro2; // RightTriangle
        
        // 渲染左三角
        let left_texture = self.texture_mgr.get_decoded_tile(left_micro.frame());
        engine.draw_texture(left_texture, x - 32, y, 32, 31);
        
        // 渲染右三角
        let right_texture = self.texture_mgr.get_decoded_tile(right_micro.frame());
        engine.draw_texture(right_texture, x, y, 32, 31);
    }
    
    fn render_walls(&self, engine: &mut Engine, mega_tile: &MegaTile, x: i32, y: i32) {
        // 墙体可能由多个microtile组成
        // micro1, micro2: 下层（Trapezoid或Square）
        // micro3, micro4: 上层（如果存在）
        
        for (micro_idx, &micro) in mega_tile.all_micros().iter().enumerate() {
            if micro.has_value() && !micro.is_floor() {
                let texture = self.texture_mgr.get_decoded_tile(micro.frame());
                
                // 根据micro_idx计算垂直偏移
                let y_offset = if micro_idx >= 2 { -32 } else { 0 };
                
                // 根据TileType调整水平位置
                let x_offset = match micro.tile_type() {
                    TileType::LeftTrapezoid | TileType::LeftTriangle => -32,
                    TileType::RightTrapezoid | TileType::RightTriangle => 0,
                    TileType::Square | TileType::TransparentSquare => -16,
                };
                
                engine.draw_texture(texture, x + x_offset, y + y_offset, 32, 32);
            }
        }
    }
}
```

## 📝 实施步骤

### 阶段1：CEL/PAL基础设施（Week 1）

#### 1.1 创建CEL加载器
- [ ] 创建 `src/resources/cel_loader.rs`
- [ ] 实现 `CelData::from_bytes()`
- [ ] 实现 `get_frame_raw()`
- [ ] 编写单元测试（手工构造简单CEL数据）

**测试：**
```rust
#[test]
fn test_cel_loader_basic() {
    let mut data = vec![0u8; 12];  // header for 1 frame
    data[0..4].copy_from_slice(&1u32.to_le_bytes());  // frame_count=1
    data[4..8].copy_from_slice(&12u32.to_le_bytes()); // offset[0]=12
    data[8..12].copy_from_slice(&20u32.to_le_bytes()); // offset[1]=20 (end)
    data.extend_from_slice(&[1,2,3,4,5,6,7,8]);  // frame data
    
    let cel = CelData::from_bytes(&data).unwrap();
    assert_eq!(cel.frame_count, 1);
    assert_eq!(cel.get_frame_raw(0), &[1,2,3,4,5,6,7,8]);
}
```

#### 1.2 创建PAL加载器
- [ ] 创建 `src/resources/palette.rs`
- [ ] 实现 `Palette::from_pal_bytes()`
- [ ] 实现6位到8位RGB转换
- [ ] 编写单元测试

**测试：**
```rust
#[test]
fn test_palette_6bit_to_8bit() {
    let mut pal_data = vec![0u8; 768];
    pal_data[0] = 0;    // R=0 -> 0
    pal_data[1] = 63;   // G=63 -> 255
    pal_data[2] = 31;   // B=31 -> 127
    
    let palette = Palette::from_pal_bytes(&pal_data).unwrap();
    let color = palette.get_color(0);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 255);
    assert_eq!(color.b, 127);
}
```

#### 1.3 集成到ResourceManager
- [ ] 在 `resource_manager.rs` 中添加 `load_palette()` 方法
- [ ] 更新 `load_dungeon_cel()` 使用新的 `CelData`

### 阶段2：TileType解码器（Week 2）

每个解码器的实施流程：
1. 创建模块文件
2. 对照C++代码逐行翻译
3. 编写手工测试用例
4. 运行测试验证

#### 2.1 Square解码器
- [ ] 创建 `src/tiles/decoder/square.rs`
- [ ] 实现 `decode_square()`
- [ ] 测试：手工构造32x32测试数据

#### 2.2 Triangle解码器
- [ ] 创建 `src/tiles/decoder/triangle.rs`
- [ ] 实现 `decode_left_triangle()`
- [ ] 实现 `decode_right_triangle()`
- [ ] 测试：验证行宽和padding跳过

**重点测试：**
```rust
#[test]
fn test_left_triangle_widths() {
    let output = decode_left_triangle(&test_data).unwrap();
    
    // 验证行0宽度=2
    assert_eq!(count_non_zero(&output[0..32]), 2);
    
    // 验证行1宽度=4
    assert_eq!(count_non_zero(&output[32..64]), 4);
    
    // 验证行15宽度=32
    assert_eq!(count_non_zero(&output[15*32..16*32]), 32);
}
```

#### 2.3 Trapezoid解码器
- [ ] 创建 `src/tiles/decoder/trapezoid.rs`
- [ ] 实现 `decode_left_trapezoid()`
- [ ] 实现 `decode_right_trapezoid()`
- [ ] 测试：验证下三角+上矩形结构

#### 2.4 TransparentSquare解码器
- [ ] 创建 `src/tiles/decoder/transparent.rs`
- [ ] 实现 `decode_transparent_square()` (RLE)
- [ ] 测试：验证RLE解码正确性

**测试RLE：**
```rust
#[test]
fn test_rle_decoding() {
    // 构造RLE数据：5个实际像素，10个透明，5个实际像素
    let mut rle_data = vec![];
    rle_data.push(5i8 as u8);  // 5个实际像素
    rle_data.extend_from_slice(&[1,2,3,4,5]);
    rle_data.push((-10i8) as u8);  // 10个透明
    rle_data.push(5i8 as u8);  // 5个实际像素
    rle_data.extend_from_slice(&[6,7,8,9,10]);
    // ... 填充到32像素
    
    let output = decode_transparent_square(&rle_data).unwrap();
    assert_eq!(output[0..5], [1,2,3,4,5]);
    assert_eq!(output[5..15], [0; 10]);  // 透明
    assert_eq!(output[15..20], [6,7,8,9,10]);
}
```

#### 2.5 统一解码器接口
- [ ] 创建 `src/tiles/decoder/mod.rs`
- [ ] 实现 `decode_tile(tile_type, raw_data)`

```rust
pub fn decode_tile(tile_type: TileType, raw_data: &[u8]) -> Result<Vec<u8>> {
    match tile_type {
        TileType::Square => square::decode_square(raw_data),
        TileType::LeftTriangle => triangle::decode_left_triangle(raw_data),
        TileType::RightTriangle => triangle::decode_right_triangle(raw_data),
        TileType::LeftTrapezoid => trapezoid::decode_left_trapezoid(raw_data),
        TileType::RightTrapezoid => trapezoid::decode_right_trapezoid(raw_data),
        TileType::TransparentSquare => transparent::decode_transparent_square(raw_data),
    }
}
```

### 阶段3：纹理管理器（Week 3 Day 1-3）

#### 3.1 实现TileTextureManager
- [ ] 创建 `src/tiles/texture_manager.rs`
- [ ] 实现加载和缓存逻辑
- [ ] 集成所有解码器

#### 3.2 集成测试
- [ ] 编写 `tests/real_data_tests.rs`
- [ ] 使用 `#[ignore]` 标记MPQ相关测试
- [ ] 运行测试验证

```rust
#[test]
#[ignore]
fn test_load_and_decode_cathedral() {
    let mut rm = ResourceManager::new(vec![("DIABDAT.MPQ", 1000)]).unwrap();
    let mut tex_mgr = TileTextureManager::load_for_dungeon(
        DungeonType::Cathedral,
        rm.mpq_manager_mut()
    ).unwrap();
    
    // 测试解码几个关键帧
    let floor_tile = tex_mgr.get_decoded_tile(1).unwrap();  // LeftTriangle
    assert_eq!(floor_tile.len(), 32 * 31);
    
    let wall_tile = tex_mgr.get_decoded_tile(10).unwrap();  // LeftTrapezoid
    assert_eq!(wall_tile.len(), 32 * 32);
}
```

### 阶段4：地图渲染集成（Week 3 Day 4-5 + Week 4）

#### 4.1 更新World渲染
- [ ] 修改 `src/world/mod.rs`
- [ ] 实现 `render_floor()` 和 `render_walls()`
- [ ] 实现等距投影坐标计算

#### 4.2 测试渲染
- [ ] 创建测试地图（floor + wall）
- [ ] 运行游戏，验证渲染效果
- [ ] 调试坐标偏移和图层叠加

#### 4.3 性能优化
- [ ] 添加纹理预加载
- [ ] 优化解码缓存
- [ ] 添加可见性裁剪

## 🧪 测试策略

> 📖 **详细验证指南：** 参见 [`decoder-verification-guide.md`](./decoder-verification-guide.md)

### 日常开发：手工测试用例

**优点：**
- ✅ 无需MPQ文件
- ✅ 快速迭代（秒级）
- ✅ 数据可控，易于调试

**示例：**
```rust
// tests/cel_decoder_tests.rs

#[test]
fn test_decode_square_checkerboard() {
    // 手工构造32x32棋盘格
    let mut raw = vec![0u8; 1024];
    for y in 0..32 {
        for x in 0..32 {
            raw[y * 32 + x] = if (x + y) % 2 == 0 { 1 } else { 2 };
        }
    }
    
    let decoded = decode_square(&raw).unwrap();
    assert_eq!(decoded.len(), 1024);
    assert_eq!(decoded[0], 1);  // (0,0) -> 1
    assert_eq!(decoded[1], 2);  // (0,1) -> 2
}
```

### 最终验证：真实MPQ数据测试

**优点：**
- ✅ 100%兼容性验证
- ✅ 发现边界情况
- ✅ 真实数据覆盖

**运行方式：**
```bash
# 设置MPQ路径
export DIABDAT_MPQ_PATH=/path/to/DIABDAT.MPQ

# 运行MPQ测试
cargo test -- --ignored

# 日常开发（跳过MPQ测试）
cargo test
```

**示例：**
```rust
// tests/real_data_tests.rs

#[test]
#[ignore]
fn test_decode_all_cathedral_tiles() {
    let mut rm = create_rm_with_mpq().expect("MPQ not found");
    
    let cel_data = CelData::from_mpq(
        rm.mpq_manager_mut(),
        "levels/l1data/l1.cel"
    ).unwrap();
    
    let min_data = MinData::load_for_dungeon(
        rm.mpq_manager_mut(),
        DungeonType::Cathedral
    ).unwrap();
    
    // 遍历所有有效的frame
    for i in 1..min_data.len() {
        let block = min_data.get(i).unwrap();
        if block.has_value() {
            let raw = cel_data.get_frame_raw(block.frame());
            let decoded = decode_tile(block.tile_type(), raw);
            
            assert!(decoded.is_ok(), 
                "Failed to decode frame {} (type={:?})", 
                block.frame(), block.tile_type());
        }
    }
    
    println!("✓ Successfully decoded all Cathedral tiles!");
}
```

## 📚 参考代码对照表

| Rust模块 | 对应C++文件 | 关键函数 |
|---------|------------|---------|
| `cel_loader.rs` | `levels/gendung.cpp` | `LoadFileInMem()` |
| `palette.rs` | `palette.cpp` | `LoadPalette()`, 6-bit转换 |
| `decoder/square.rs` | `levels/reencode_dun_cels.cpp` | `ReencodeDungeonCelsSquare()` |
| `decoder/triangle.rs` | `levels/reencode_dun_cels.cpp` | `ReencodeDungeonCelsLeftTriangle()`, `ReencodeDungeonCelsRightTriangle()` |
| `decoder/trapezoid.rs` | `levels/reencode_dun_cels.cpp` | `ReencodeDungeonCelsLeftTrapezoid()`, `ReencodeDungeonCelsRightTrapezoid()` |
| `decoder/transparent.rs` | `levels/reencode_dun_cels.cpp` | `ReencodeDungeonCelsTransparentSquare()` |
| `texture_manager.rs` | `levels/gendung.cpp` | `SetDungeonMicros()`, `GetDunFrame()` |
| `world/mod.rs` (渲染) | `engine/render/scrollrt.cpp` | `DrawTileContent()`, `DrawCell()`, `DrawFloorTile()` |

## 🎓 关键学习要点

### 1. CEL文件格式
- **偏移表机制** - 如何通过offset table快速定位frame
- **变长编码** - 不同TileType的数据长度不同
- **性能优化** - 原版为何要ReencodeDungeonCels（移除padding，提升缓存效率）

### 2. 解码算法细节
- **Padding处理** - Triangle/Trapezoid为何需要跳过2字节padding
- **行对齐** - Left/Right的镜像对齐逻辑
- **RLE编码** - TransparentSquare的行扫描RLE实现

### 3. 渲染架构
- **两次渲染Pass** - 为何要分开渲染地板和墙体
- **等距投影** - (x, y) → (screen_x, screen_y) 的坐标转换
- **图层叠加** - micro1-4的垂直堆叠关系

### 4. Rust实践
- **所有权管理** - 如何高效缓存解码后的纹理
- **错误处理** - Result<T> 在文件解析中的应用
- **零拷贝优化** - 使用切片引用避免数据复制

## ⚠️ 常见坑点

### 1. Padding字节
**问题：** Triangle/Trapezoid解码时忘记跳过padding
**现象：** 解码后图像错位、花屏
**解决：** 严格对照C++代码，每对行之间 `src += 2`

### 2. Signed/Unsigned混淆
**问题：** RLE控制字节应该是 `i8`（signed），写成 `u8`
**现象：** 负数被解析为大正数，RLE解码错误
**解决：** `let val = raw_data[src] as i8;`

### 3. 坐标系混淆
**问题：** CEL像素坐标 vs 屏幕坐标 vs 地图坐标
**现象：** 瓦片位置不对，悬浮或下沉
**解决：** 画图理清三种坐标系的转换关系

### 4. 缓存失效
**问题：** 修改解码器后忘记清空缓存
**现象：** 看到的仍是旧的错误图像
**解决：** 测试时清空 `decoded_cache`

## 📊 进度跟踪

创建一个checklist来跟踪进度：

```markdown
## CEL/PAL基础设施
- [ ] CelData加载器
- [ ] Palette加载器
- [ ] 集成到ResourceManager

## 解码器实现
- [ ] Square解码器
- [ ] LeftTriangle解码器
- [ ] RightTriangle解码器
- [ ] LeftTrapezoid解码器
- [ ] RightTrapezoid解码器
- [ ] TransparentSquare解码器
- [ ] 统一接口

## 纹理管理
- [ ] TileTextureManager实现
- [ ] 缓存机制
- [ ] 预加载优化

## 渲染集成
- [ ] World渲染更新
- [ ] 地板渲染
- [ ] 墙体渲染
- [ ] 坐标投影

## 测试
- [ ] 手工测试用例（每个解码器）
- [ ] MPQ真实数据测试
- [ ] 渲染效果验证
```

## 🎯 验收标准

完成本步骤后，应达到：

1. ✅ **所有6种TileType解码器通过测试**
   - 手工测试通过
   - MPQ真实数据测试通过（如有MPQ）

2. ✅ **成功渲染Cathedral地牢**
   - 地板正确显示（三角形瓦片）
   - 墙体正确显示（梯形瓦片）
   - 无花屏、错位、黑块

3. ✅ **代码质量**
   - 每个解码器都有详细注释
   - 对照C++代码位置已标注
   - 测试覆盖率 > 80%

4. ✅ **文档完善**
   - 实现总结文档
   - 踩坑记录
   - 下一步计划

## 🚀 下一步计划（Step 6.3）

完成本步骤后，下一步将实现：

1. **光照系统** - dLight数组，光照贴图
2. **透明度混合** - 半透明物体、阴影
3. **动画系统** - 动态瓦片、火把动画
4. **性能优化** - 批量渲染、SIMD加速

---

**记住：严格对照原版C++代码，不自由发挥！** 🎯

