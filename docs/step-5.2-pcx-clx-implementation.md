# Step 5.2: PCX图像格式和CLX精灵格式实现

## 📅 开始日期
2025-01-XX

## 📋 步骤概述

Step 5.2 是 Step 5（原版资源格式完整支持）的第二个子步骤，主要实现 PCX 图像格式和 CLX 精灵格式的解析和加载。这是连接 Rust 实现与原版游戏资源的关键步骤，使游戏能够直接使用原版 DIABDAT.MPQ 中的图像和精灵资源。

### 与 Step 5.1 的关系

- **Step 5.1** ✅ 已完成：MPQ 归档系统和调色板系统
- **Step 5.2** 🔄 当前步骤：PCX 图像格式和 CLX 精灵格式
- **Step 5.3** ⏸️ 待开发：TRN 颜色转换和场景加载集成

### 为什么这一步如此重要

1. **直接使用原版资源**：PCX 和 CLX 是 Diablo 1 的核心资源格式，必须实现才能加载原版图像和精灵
2. **视觉一致性**：保持与原版 100% 的视觉一致性
3. **动画支持**：CLX 格式支持多帧动画，是角色和怪物动画的基础
4. **UI 系统基础**：PCX 格式用于 UI 图像，是后续 UI 系统的基础

## 🎯 目标

实现 PCX 图像格式和 CLX 精灵格式的完整解析，能够：
1. 从 MPQ 文件加载 PCX 图像
2. 从 MPQ 文件加载 CLX 精灵
3. 正确解码 RLE 压缩数据
4. 应用调色板转换为 RGBA 纹理
5. 支持多帧动画（CLX）
6. 集成到游戏渲染系统

## 📊 代码量估算

**预估代码量：650-900 行**

| 模块 | 预估代码量 | 说明 |
|------|-----------|------|
| PCX 加载器 | 150-200 行 | PCX 解析、RLE 解码 |
| CLX 加载器 | 300-400 行 | CLX 解析、CL2 RLE 解码（最复杂） |
| 测试代码 | 100-150 行 | 单元测试、集成测试 |
| 游戏集成 | 100-150 行 | PCX/CLX 渲染测试 |

## 📋 功能清单

### 1. PCX 图像格式 (`resources/pcx.rs`)

#### 1.1 PCX 文件结构

PCX 文件由三部分组成：
1. **文件头**（128 字节）：包含图像元数据
2. **像素数据**（RLE 压缩）：256 色索引图像
3. **调色板**（768 字节，可选）：文件末尾的 RGB 调色板

**参考代码：**
- `Source/utils/pcx.hpp` - PCX 文件头定义
- `Source/utils/pcx_to_clx.cpp` - PCX 解析实现

#### 1.2 PCX 文件头结构

```rust
pub struct PcxHeader {
    pub manufacturer: u8,      // 0x0A (PCX 标识)
    pub version: u8,           // 版本号
    pub encoding: u8,          // 编码方式（1 = RLE）
    pub bits_per_pixel: u8,   // 每像素位数（8 = 256色）
    pub xmin: u16,            // 图像左边界
    pub ymin: u16,            // 图像上边界
    pub xmax: u16,            // 图像右边界
    pub ymax: u16,            // 图像下边界
    pub hdpi: u16,            // 水平分辨率
    pub vdpi: u16,            // 垂直分辨率
    pub colormap: [u8; 48],   // 16 色调色板（通常不使用）
    pub reserved: u8,         // 保留字节
    pub n_planes: u8,         // 颜色平面数（1 = 单平面）
    pub bytes_per_line: u16,  // 每行字节数（必须是偶数）
    pub palette_info: u16,    // 调色板信息
    pub hscreen_size: u16,    // 水平屏幕大小
    pub vscreen_size: u16,    // 垂直屏幕大小
    pub filler: [u8; 54],     // 填充字节
}
```

**图像尺寸计算：**
```rust
width = xmax - xmin + 1
height = ymax - ymin + 1
```

#### 1.3 PCX RLE 解码

**RLE 规则：**
- 如果字节 `<= 0xBF (191)`：直接是像素值
- 如果字节 `>= 0xC0 (192)`：
  - 重复次数 = `字节 & 0x3F`（取低 6 位）
  - 下一个字节是重复的值

**参考代码：**
- `Source/utils/pcx_to_clx.cpp:119-131` - PCX RLE 解码实现

**实现示例：**
```rust
fn decode_pcx_rle(data: &[u8], width: u16, height: u16) -> Result<Vec<u8>> {
    let mut pixels = Vec::new();
    let mut data_idx = 0;
    
    for _ in 0..height {
        let mut x = 0;
        while x < width {
            if data_idx >= data.len() {
                return Err(anyhow!("RLE decode overflow"));
            }
            
            let byte = data[data_idx];
            data_idx += 1;
            
            if byte <= 0xBF {
                // 直接像素值
                pixels.push(byte);
                x += 1;
            } else {
                // RLE 压缩
                let run_length = (byte & 0x3F) as usize;
                if data_idx >= data.len() {
                    return Err(anyhow!("RLE decode overflow"));
                }
                let value = data[data_idx];
                data_idx += 1;
                
                for _ in 0..run_length {
                    pixels.push(value);
                    x += 1;
                }
            }
        }
        
        // PCX 每行必须是偶数字节，跳过填充字节
        if width % 2 != 0 && data_idx < data.len() {
            data_idx += 1;
        }
    }
    
    Ok(pixels)
}
```

#### 1.4 PCX 调色板读取

**调色板位置：**
- 文件末尾 769 字节
- 第一个字节是分隔符 `0x0C`
- 接下来 768 字节是 256 色 RGB 调色板（每个颜色 3 字节：R, G, B）

**参考代码：**
- `Source/utils/pcx_to_clx.cpp:167-181` - PCX 调色板读取

**注意：**
- 不是所有 PCX 文件都有调色板
- 如果没有调色板，需要使用外部调色板（如关卡调色板）

#### 1.5 PCX 数据结构

```rust
pub struct PcxImage {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,           // 256 色索引图像
    pub palette: Option<Palette>,  // 可选的内置调色板
}

impl PcxImage {
    /// 从字节数组解析 PCX 文件
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    
    /// 转换为 RGBA 纹理（需要调色板）
    pub fn to_rgba(&self, palette: &Palette, transparent: Option<u8>) -> Vec<u8>;
}
```

### 2. CLX 精灵格式 (`resources/clx.rs`)

#### 2.1 CLX 文件结构

CLX 文件由以下部分组成：
1. **文件头**：
   - 帧数（4 字节，小端序）
   - 每帧的偏移量（每帧 4 字节，小端序）
   - 文件大小（4 字节，小端序，最后一个偏移量之后）
2. **帧数据**（每帧）：
   - 帧头（6 字节）：
     - 帧头大小（2 字节，通常是 6）
     - 宽度（2 字节）
     - 高度（2 字节）
   - 像素数据（CL2 RLE 编码）

**参考代码：**
- `Source/engine/clx_sprite.hpp` - CLX 格式定义
- `Source/utils/clx_decode.hpp` - CLX 解码函数

#### 2.2 CLX 文件头解析

```rust
pub struct ClxHeader {
    pub num_frames: u32,
    pub frame_offsets: Vec<u32>,
    pub file_size: u32,
}

fn parse_clx_header(data: &[u8]) -> Result<ClxHeader> {
    if data.len() < 8 {
        return Err(anyhow!("CLX header too small"));
    }
    
    let num_frames = u32::from_le_bytes([
        data[0], data[1], data[2], data[3]
    ]);
    
    let mut frame_offsets = Vec::new();
    for i in 0..=num_frames {
        let offset = 4 + (i as usize * 4);
        if offset + 4 > data.len() {
            return Err(anyhow!("CLX frame offset out of bounds"));
        }
        let offset_value = u32::from_le_bytes([
            data[offset], data[offset + 1],
            data[offset + 2], data[offset + 3]
        ]);
        frame_offsets.push(offset_value);
    }
    
    Ok(ClxHeader {
        num_frames,
        frame_offsets,
        file_size: frame_offsets[num_frames as usize],
    })
}
```

#### 2.3 CLX 帧头解析

```rust
pub struct ClxFrameHeader {
    pub header_size: u16,  // 通常是 6
    pub width: u16,
    pub height: u16,
}

fn parse_clx_frame_header(data: &[u8], offset: usize) -> Result<ClxFrameHeader> {
    if offset + 6 > data.len() {
        return Err(anyhow!("CLX frame header out of bounds"));
    }
    
    Ok(ClxFrameHeader {
        header_size: u16::from_le_bytes([data[offset], data[offset + 1]]),
        width: u16::from_le_bytes([data[offset + 2], data[offset + 3]]),
        height: u16::from_le_bytes([data[offset + 4], data[offset + 5]]),
    })
}
```

#### 2.4 CL2 RLE 像素解码

CL2 RLE 编码比 PCX RLE 更复杂，支持透明像素和填充运行。

**CL2 RLE 规则：**

1. **透明运行（Transparent Run）**：
   - 控制字节范围：`0x00 - 0x7F`
   - 值 = 透明像素数量
   - 跳过这些像素（不写入）

2. **不透明填充（Opaque Fill）**：
   - 控制字节范围：`0x80 - 0xBF`
   - 宽度 = `0xBF - 控制字节`
   - 下一个字节是填充颜色值
   - 重复该颜色值 `宽度` 次

3. **不透明像素（Opaque Pixels）**：
   - 控制字节范围：`0xC0 - 0xFF`
   - 宽度 = `控制字节 - 0xBF`
   - 接下来 `宽度` 个字节是原始像素值

**参考代码：**
- `Source/utils/clx_decode.hpp` - CL2 RLE 解码函数定义
- `Source/utils/cl2_to_clx.cpp:72-104` - CL2 RLE 解码实现

**实现示例：**
```rust
fn decode_cl2_rle(
    data: &[u8],
    data_offset: usize,
    width: u16,
    height: u16,
) -> Result<Vec<Option<u8>>> {
    let mut pixels = vec![None; (width as usize * height as usize)];
    let mut pixel_idx = 0;
    let mut data_idx = data_offset;
    
    for _ in 0..height {
        let mut x = 0;
        while x < width {
            if data_idx >= data.len() {
                return Err(anyhow!("CL2 RLE decode overflow"));
            }
            
            let control = data[data_idx];
            data_idx += 1;
            
            if control < 0x80 {
                // 透明运行
                let transparent_count = control as usize;
                for _ in 0..transparent_count {
                    if pixel_idx < pixels.len() {
                        pixels[pixel_idx] = None; // 透明
                        pixel_idx += 1;
                        x += 1;
                    }
                }
            } else if control <= 0xBF {
                // 不透明填充
                let fill_width = (0xBF - control) as usize;
                if data_idx >= data.len() {
                    return Err(anyhow!("CL2 RLE decode overflow"));
                }
                let fill_color = data[data_idx];
                data_idx += 1;
                
                for _ in 0..fill_width {
                    if pixel_idx < pixels.len() {
                        pixels[pixel_idx] = Some(fill_color);
                        pixel_idx += 1;
                        x += 1;
                    }
                }
            } else {
                // 不透明像素
                let pixel_width = (control - 0xBF) as usize;
                for _ in 0..pixel_width {
                    if data_idx >= data.len() {
                        return Err(anyhow!("CL2 RLE decode overflow"));
                    }
                    if pixel_idx < pixels.len() {
                        pixels[pixel_idx] = Some(data[data_idx]);
                        pixel_idx += 1;
                        x += 1;
                    }
                    data_idx += 1;
                }
            }
        }
    }
    
    Ok(pixels)
}
```

#### 2.5 CLX 数据结构

```rust
pub struct ClxFrame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<Option<u8>>,  // None 表示透明像素
}

pub struct ClxSprite {
    pub frames: Vec<ClxFrame>,
}

impl ClxSprite {
    /// 从字节数组解析 CLX 文件
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    
    /// 获取帧数
    pub fn num_frames(&self) -> usize;
    
    /// 获取指定帧
    pub fn get_frame(&self, index: usize) -> Option<&ClxFrame>;
    
    /// 将帧转换为 RGBA 纹理（需要调色板）
    pub fn frame_to_rgba(
        &self,
        frame_index: usize,
        palette: &Palette,
    ) -> Result<Vec<u8>>;
}
```

## 🏗️ 架构设计

### 模块结构

```
src/resources/
├── mod.rs          # 资源管理器（Step 5.3 实现）
├── mpq.rs          # MPQ 归档（Step 5.1 已完成）
├── palette.rs      # 调色板系统（Step 5.1 已完成）
├── pcx.rs          # PCX 图像加载（Step 5.2 新增）
└── clx.rs          # CLX 精灵加载（Step 5.2 新增）
```

### 数据流

```
MPQ 文件
  ↓
文件数据（压缩）
  ↓
解压缩（MPQ 系统）
  ↓
格式解析（PCX/CLX）
  ↓
RLE 解码
  ↓
索引图像数据
  ↓
应用调色板
  ↓
RGBA 纹理
  ↓
SDL 纹理
  ↓
渲染系统
```

## 📝 实现细节

### 1. PCX 加载器实现

**文件：** `src/resources/pcx.rs`

**主要函数：**
```rust
pub struct PcxImage {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,
    pub palette: Option<Palette>,
}

impl PcxImage {
    /// 从字节数组解析 PCX 文件
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // 1. 解析文件头
        // 2. 计算图像尺寸
        // 3. RLE 解码像素数据
        // 4. 读取调色板（如果存在）
    }
    
    /// 转换为 RGBA 纹理
    pub fn to_rgba(
        &self,
        palette: &Palette,
        transparent: Option<u8>,
    ) -> Vec<u8> {
        // 1. 遍历像素索引
        // 2. 使用调色板转换为 RGB
        // 3. 处理透明色
        // 4. 返回 RGBA 数据
    }
}
```

### 2. CLX 加载器实现

**文件：** `src/resources/clx.rs`

**主要函数：**
```rust
pub struct ClxFrame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<Option<u8>>,
}

pub struct ClxSprite {
    pub frames: Vec<ClxFrame>,
}

impl ClxSprite {
    /// 从字节数组解析 CLX 文件
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // 1. 解析文件头（帧数、偏移量）
        // 2. 对每一帧：
        //    a. 解析帧头（宽度、高度）
        //    b. CL2 RLE 解码像素数据
        // 3. 返回 ClxSprite
    }
    
    /// 将帧转换为 RGBA 纹理
    pub fn frame_to_rgba(
        &self,
        frame_index: usize,
        palette: &Palette,
    ) -> Result<Vec<u8>> {
        // 1. 获取指定帧
        // 2. 遍历像素（None = 透明）
        // 3. 使用调色板转换为 RGB
        // 4. 返回 RGBA 数据
    }
}
```

## 🧪 测试要求

### 单元测试

#### PCX 测试

1. **PCX 文件头解析测试**
   - 测试正确的文件头解析
   - 测试图像尺寸计算
   - 测试错误的文件头（太小、格式错误）

2. **PCX RLE 解码测试**
   - 测试单像素解码
   - 测试 RLE 压缩解码
   - 测试边界情况（最后一行、最后一个像素）
   - 测试损坏的数据（越界、格式错误）

3. **PCX 调色板读取测试**
   - 测试有调色板的文件
   - 测试无调色板的文件
   - 测试调色板格式验证

4. **PCX 转 RGBA 测试**
   - 测试索引到 RGB 转换
   - 测试透明色处理
   - 测试完整图像转换

#### CLX 测试

1. **CLX 文件头解析测试**
   - 测试帧数解析
   - 测试帧偏移量解析
   - 测试文件大小验证
   - 测试错误的文件头

2. **CLX 帧头解析测试**
   - 测试帧头解析
   - 测试宽度和高度读取
   - 测试边界检查

3. **CL2 RLE 解码测试**
   - 测试透明运行解码
   - 测试不透明填充解码
   - 测试不透明像素解码
   - 测试混合编码
   - 测试边界情况（越界、格式错误）

4. **CLX 多帧测试**
   - 测试多帧解析
   - 测试帧索引访问
   - 测试帧转 RGBA

### 集成测试

1. **从 MPQ 加载 PCX**
   - 测试从 MPQ 加载 PCX 文件（如 `ui_art/logo.pcx`）
   - 测试文件查找和读取
   - 测试完整流程（MPQ → PCX → RGBA）

2. **从 MPQ 加载 CLX**
   - 测试从 MPQ 加载 CLX 文件（如 `plrgfx/warrior/whs/whsas.cl2`）
   - 测试文件查找和读取
   - 测试完整流程（MPQ → CLX → RGBA）

3. **调色板应用测试**
   - 测试 PCX 使用内置调色板
   - 测试 PCX 使用外部调色板
   - 测试 CLX 使用调色板
   - 测试透明色处理

### 游戏集成测试

1. **PCX 图像场景**
   - 加载 `ui_art/logo.pcx`
   - 在游戏窗口中显示
   - 验证图像清晰、颜色正确

2. **CLX 精灵场景**
   - 加载战士站立动画 `plrgfx/warrior/whs/whsas.cl2`
   - 替换当前玩家精灵
   - 验证动画流畅播放

3. **多帧动画测试**
   - 加载多帧 CLX 精灵
   - 测试动画播放
   - 验证帧切换正确

**测试覆盖率目标：≥ 85%**

## 🔍 参考代码出处和改造细节

### 1. PCX 加载器

**原版代码：**
- `Source/utils/pcx.hpp` - PCX 文件头定义
- `Source/utils/pcx_to_clx.cpp` - PCX 解析和 RLE 解码
- `Source/engine/load_pcx.cpp` - PCX 加载入口

**改造思路：**
- 原版将 PCX 转换为 CLX 格式，我们直接解析 PCX 并转换为 RGBA 纹理
- 保持相同的 RLE 解码逻辑
- 使用 Rust 的 `Result` 类型处理错误，而非 C++ 的 `Optional`
- 使用 `Vec<u8>` 存储像素数据，而非 C++ 的 `std::unique_ptr<uint8_t[]>`

**关键改造：**
```rust
// 原版 C++ 代码
std::unique_ptr<uint8_t[]> fileBuffer(new uint8_t[pixelDataSize]);
handle.read(fileBuffer.get(), pixelDataSize);

// Rust 版本
let mut pixels = Vec::new();
// RLE 解码逻辑相同
```

### 2. CLX 加载器

**原版代码：**
- `Source/engine/clx_sprite.hpp` - CLX 格式定义
- `Source/utils/clx_decode.hpp` - CL2 RLE 解码函数
- `Source/utils/cl2_to_clx.cpp` - CL2 到 CLX 转换（包含 RLE 解码）
- `Source/engine/load_clx.cpp` - CLX 加载入口

**改造思路：**
- 保持 CLX 格式定义不变
- 使用 Rust 的 `Option<u8>` 表示透明像素（原版使用特殊值或跳过）
- 使用 `Result` 处理解码错误
- 使用 `Vec<ClxFrame>` 存储多帧，而非 C++ 的复杂迭代器系统

**关键改造：**
```rust
// 原版 C++ 代码使用指针和大小
const uint8_t *pixelData() const { return &data_[LoadLE16(data_)]; }

// Rust 版本使用结构体和 Option
pub struct ClxFrame {
    pub pixels: Vec<Option<u8>>, // None 表示透明
}
```

### 3. RLE 解码

**原版代码：**
- `Source/utils/pcx_to_clx.cpp:119-131` - PCX RLE 解码
- `Source/utils/cl2_to_clx.cpp:72-104` - CL2 RLE 解码

**改造思路：**
- 保持相同的 RLE 解码逻辑
- 使用 Rust 的边界检查（`checked_add`、切片边界）
- 使用 `Result` 返回错误，而非 panic

**关键改造：**
```rust
// 原版 C++ 代码可能使用指针算术
const uint8_t byte = *dataPtr++;

// Rust 版本使用索引和边界检查
if data_idx >= data.len() {
    return Err(anyhow!("RLE decode overflow"));
}
let byte = data[data_idx];
data_idx += 1;
```

## ⚠️ 踩坑点和注意事项

### 1. PCX RLE 解码边界检查

**问题：** RLE 解码时可能读取越界，导致 panic

**解决方案：**
```rust
// 使用 checked_add 防止溢出
let next_idx = pixel_idx.checked_add(run_length)?;

// 检查数据长度
if data_idx + run_length > data.len() {
    return Err(anyhow!("RLE decode overflow"));
}
```

**测试要点：**
- 测试边界情况（最后一个像素）
- 测试损坏的文件
- 测试空文件

### 2. PCX 每行字节数必须是偶数

**问题：** PCX 格式要求每行字节数必须是偶数，奇数行需要填充字节

**解决方案：**
```rust
// 解码完一行后，如果是奇数宽度，跳过填充字节
if width % 2 != 0 && data_idx < data.len() {
    data_idx += 1;
}
```

**参考代码：** `Source/utils/pcx_to_clx.cpp:132`

### 3. CL2 RLE 解码复杂性

**问题：** CL2 RLE 编码比 PCX RLE 更复杂，有三种不同的编码类型

**解决方案：**
- 仔细实现三种编码类型的解码逻辑
- 使用常量定义编码范围（`0x80`, `0xBF`, `0xC0`）
- 添加详细的单元测试

**参考代码：** `Source/utils/clx_decode.hpp`

### 4. CLX 透明像素处理

**问题：** CLX 使用透明运行（transparent run）表示透明像素，需要正确处理

**解决方案：**
```rust
// 使用 Option<u8> 表示像素
pub struct ClxFrame {
    pub pixels: Vec<Option<u8>>, // None = 透明
}

// 转换为 RGBA 时处理透明
if let Some(color_idx) = pixel {
    let rgb = palette.to_rgb(color_idx);
    rgba.push(rgb[0]);
    rgba.push(rgb[1]);
    rgba.push(rgb[2]);
    rgba.push(255); // 不透明
} else {
    rgba.push(0);
    rgba.push(0);
    rgba.push(0);
    rgba.push(0); // 透明
}
```

### 5. 字节序（Endianness）问题

**问题：** PCX 和 CLX 文件使用小端序（Little Endian）

**解决方案：**
```rust
// 使用标准库函数
let width = u16::from_le_bytes([data[8], data[9]]);
let height = u16::from_le_bytes([data[10], data[11]]);
```

**注意事项：**
- Rust 默认是小端序（x86/x64），但显式指定更安全
- 跨平台兼容性考虑

### 6. PCX 调色板可选性

**问题：** 不是所有 PCX 文件都有内置调色板

**解决方案：**
```rust
pub struct PcxImage {
    pub palette: Option<Palette>,  // 可选的内置调色板
}

// 使用时优先使用内置调色板，否则使用外部调色板
let palette = pcx.palette.as_ref().unwrap_or(external_palette);
```

### 7. CLX 多帧动画支持

**问题：** CLX 文件可能包含多帧，需要正确解析所有帧

**解决方案：**
```rust
// 解析所有帧
for frame_idx in 0..num_frames {
    let frame_offset = frame_offsets[frame_idx as usize];
    let next_offset = frame_offsets[(frame_idx + 1) as usize];
    let frame_data = &data[frame_offset..next_offset];
    // 解析帧...
}
```

## 📚 学习要点

### 1. 为什么需要实现这些格式？

**原版 Diablo 的资源格式特点：**
- **PCX 格式**：90 年代常见的图像格式，支持 RLE 压缩
- **CLX 格式**：Diablo 专用的精灵格式，优化了透明像素和压缩
- **256 色调色板**：节省内存和存储空间
- **RLE 压缩**：减少文件大小，但需要运行时解码

**现代游戏通常使用：**
- PNG/JPG 等标准格式
- 直接 RGB/RGBA 颜色
- 无需运行时解码

**为什么必须实现原版格式？**
1. **兼容性**：直接使用原版 DIABDAT.MPQ，无需资源转换
2. **完整性**：保持与原版 100% 的视觉一致性
3. **学习价值**：理解 90 年代游戏资源管理技术
4. **扩展支持**：支持 Hellfire 扩展包和 Mod 系统

### 2. RLE 压缩算法原理

**RLE (Run-Length Encoding)** 是一种简单的压缩算法：

**PCX RLE 规则：**
```
如果字节 <= 0xBF (191):
  直接是像素值
如果字节 >= 0xC0 (192):
  重复次数 = 字节 & 0x3F (取低 6 位)
  下一个字节是重复的值
```

**CL2 RLE 规则（更复杂）：**
```
透明运行 (0x00-0x7F):
  值 = 透明像素数量

不透明填充 (0x80-0xBF):
  宽度 = 0xBF - 控制字节
  下一个字节是填充颜色值

不透明像素 (0xC0-0xFF):
  宽度 = 控制字节 - 0xBF
  接下来宽度个字节是原始像素值
```

**为什么使用 RLE？**
- 简单高效，适合 90 年代硬件
- 对于重复像素多的图像（如精灵），压缩效果好
- 解码速度快，适合实时渲染

### 3. 调色板系统设计

**调色板布局：**
```
索引 0-127:   关卡特定颜色（不同关卡使用不同调色板）
索引 128-255: 全局颜色（蓝色、红色、黄色等，所有关卡共享）
```

**为什么这样设计？**
- **关卡特定颜色**：不同地牢（教堂、地下墓穴）有不同的色调
- **全局颜色**：玩家、怪物、UI 等在所有关卡中颜色一致
- **内存优化**：只需加载一个调色板，而非每个关卡一个

**透明色处理：**
- 索引 0 通常作为透明色
- 但某些图像（如 UI）索引 0 不是透明
- 需要提供选项控制透明色行为

### 4. CLX 格式的优势

**格式演进：**
```
CEL (原始) → CL2 (压缩优化) → CLX (运行时格式)
```

**CLX 相比 CL2 的改进：**
1. **简化头部**：CLX 帧头只存储宽度和高度，不存储 32 像素块偏移
2. **运行时优化**：CLX 是 DevilutionX 的运行时格式，更适合现代硬件
3. **向后兼容**：可以从 CL2/CEL 转换到 CLX

**为什么使用 CLX？**
- 原版使用 CL2，但 CLX 更适合我们的实现
- 可以从 CL2 转换，也可以直接加载 CLX
- 简化了渲染逻辑

## 🎯 验收标准

### Step 5.2 验收标准

1. ✅ 能够正确解析 PCX 文件
2. ✅ PCX 图像在游戏中正确显示（测试场景：显示 `ui_art/logo.pcx`）
3. ✅ 能够正确解析 CLX 文件
4. ✅ CLX 精灵在游戏中正确渲染（测试场景：战士站立动画）
5. ✅ CLX 多帧动画正确播放
6. ✅ 所有单元测试通过
7. ✅ **游戏集成验证**：PCX 和 CLX 资源在游戏中正确渲染
8. ✅ 测试覆盖率 ≥ 85%

### 游戏集成验证场景

#### PCX 图像场景
- 加载 `ui_art/logo.pcx` 或 `ui_art/title.pcx`
- 在游戏窗口中显示
- 验证图像清晰、颜色正确

#### CLX 精灵场景
- 加载战士站立动画 `plrgfx/warrior/whs/whsas.cl2`
- 替换当前玩家精灵
- 验证动画流畅播放
- 验证所有帧正确渲染

## 🔜 下一步计划

完成 Step 5.2 后，下一步是：
- **Step 5.3: TRN 颜色转换和场景加载集成**
  - 实现 TRN 颜色转换
  - 完善资源管理器
  - 完成场景加载集成
  - **最终目标：实现城镇场景预览，玩家可以在城镇中行走**

---

**文档版本：** 1.0  
**创建日期：** 2025-01-XX  
**关联文档：** 
- [Step 5 总体规划](./step-5-resource-formats.md)
- [Step 5.1 总结](./step-5.1-summary.md)
- [MASTER_PLAN.md](./MASTER_PLAN.md)


















