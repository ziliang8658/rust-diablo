# Step 5: 原版资源格式完整支持

## 📅 开始日期
2025年1月20日

## 📋 步骤划分

由于代码量较大（1430-1900行），Step 5分为三个子步骤实现，每个子步骤都包含测试和游戏集成：

- **Step 5.1: MPQ归档系统和调色板系统**（基础资源加载）
- **Step 5.2: PCX图像格式和CLX精灵格式**（核心资源格式）
- **Step 5.3: TRN颜色转换和场景加载集成**（完整功能）

每个子步骤完成后都要：
1. ✅ 编写完整的测试用例
2. ✅ 集成到游戏中验证效果
3. ✅ 确保可以实际运行和查看结果

## 📊 代码量估算

**预估总代码量：1430-1900行**

### 各子步骤代码量分布

| 子步骤 | 模块 | 预估代码量 | 说明 |
|--------|------|-----------|------|
| **Step 5.1** | MPQ管理器 | 200-250行 | MPQ归档读取、优先级系统 |
| | 调色板系统 | 100-150行 | 调色板加载、RGB转换 |
| | 测试代码 | 100-150行 | 单元测试、集成测试 |
| | 游戏集成 | 50-100行 | 基础资源加载测试 |
| | **小计** | **450-650行** | |
| **Step 5.2** | PCX加载器 | 150-200行 | PCX解析、RLE解码 |
| | CLX加载器 | 300-400行 | CLX解析、CL2 RLE解码（最复杂） |
| | 测试代码 | 100-150行 | 单元测试、集成测试 |
| | 游戏集成 | 100-150行 | PCX/CLX渲染测试 |
| | **小计** | **650-900行** | |
| **Step 5.3** | TRN转换 | 80-100行 | 颜色映射表 |
| | 资源管理器 | 250-300行 | 统一接口、缓存管理 |
| | 场景加载集成 | 150-200行 | 资源集成到游戏、渲染系统 |
| | 测试代码 | 100-150行 | 单元测试、集成测试 |
| | **小计** | **580-750行** | |
| **总计** | | **1680-2300行** | |

**注：** 实际代码量可能因实现细节和错误处理而有所变化。每个子步骤完成后都要进行测试和游戏集成验证。

## 🎯 目标

实现原版Diablo资源格式的完整支持，包括MPQ归档、PCX图像、CEL/CL2/CLX精灵、调色板和TRN颜色转换。这是连接Rust实现与原版游戏数据的关键步骤，使游戏能够直接使用原版DIABDAT.MPQ中的资源。

## 📋 子步骤详细说明

### Step 5.1: MPQ归档系统和调色板系统（基础资源加载）

**目标：** 实现资源加载的基础设施，能够从MPQ文件读取数据，并理解调色板系统。

**功能清单：**
1. **MPQ归档系统** (`resources/mpq.rs`)
   - MPQ归档读取（使用`mpq` crate）
   - 文件列表枚举
   - 文件哈希查找
   - 文件解压缩
   - MPQ优先级系统（多MPQ文件加载，优先级覆盖）
   - MPQ缓存系统（已加载文件缓存）

2. **调色板系统** (`resources/palette.rs`)
   - 调色板文件加载（.PAL，768字节）
   - 调色板结构（256色RGB）
   - 调色板应用到索引图像
   - 透明色处理（索引0通常为透明）

**测试要求：**
- 单元测试：MPQ文件加载、文件查找、调色板加载、RGB转换
- 集成测试：从MPQ加载调色板文件
- 游戏集成：在游戏中加载MPQ并读取调色板，验证调色板数据正确

**游戏集成验证：**
- 创建测试场景，从DIABDAT.MPQ加载一个调色板文件（如`levels/towndata/town.pal`）
- 在控制台或日志中输出调色板信息（前10个颜色值）
- 验证MPQ文件路径搜索正常工作

**验收标准：**
- ✅ 能够成功加载DIABDAT.MPQ文件
- ✅ 能够从MPQ中查找和读取文件
- ✅ 能够正确解析调色板文件
- ✅ 调色板RGB值正确
- ✅ 所有测试通过

---

### Step 5.2: PCX图像格式和CLX精灵格式（核心资源格式）

**目标：** 实现PCX和CLX格式的解析，能够加载并渲染原版图像和精灵。

**功能清单：**
1. **PCX图像格式** (`resources/pcx.rs`)
   - PCX文件头解析（128字节）
   - RLE压缩解码
   - 调色板读取（文件末尾768字节）
   - 转换为RGBA纹理

2. **CLX精灵格式** (`resources/clx.rs`)
   - CLX文件头解析（帧数、帧偏移）
   - CLX帧头解析（宽度、高度）
   - CL2 RLE像素解码
   - 透明像素处理
   - 多帧动画支持

**测试要求：**
- 单元测试：PCX解析、RLE解码、CLX解析、CL2 RLE解码
- 集成测试：从MPQ加载PCX和CLX文件
- 游戏集成：在游戏中渲染PCX图像和CLX精灵

**游戏集成验证：**
- **PCX场景**：加载`ui_art/logo.pcx`或`ui_art/title.pcx`，在游戏窗口中显示
- **CLX场景**：加载战士站立动画`plrgfx/warrior/whs/whsas.cl2`，替换当前玩家精灵
- 验证图像清晰、颜色正确、动画流畅

**验收标准：**
- ✅ 能够正确解析PCX文件
- ✅ PCX图像在游戏中正确显示
- ✅ 能够正确解析CLX文件
- ✅ CLX精灵在游戏中正确渲染
- ✅ CLX多帧动画正确播放
- ✅ 所有测试通过

---

### Step 5.3: TRN颜色转换和场景加载集成（完整功能）

**目标：** 实现TRN颜色转换，完善资源管理器，并完成场景加载集成。**最终目标：实现城镇场景预览，玩家可以在城镇中行走。**

**功能清单：**
1. **TRN颜色转换** (`resources/trn.rs`)
   - TRN文件加载（256字节映射表）
   - 颜色映射应用
   - 应用到精灵像素

2. **资源管理器** (`resources/mod.rs`)
   - 统一资源加载接口
   - 资源缓存管理
   - MPQ管理器集成
   - 调色板管理器
   - 纹理缓存

3. **场景加载和资源集成** (`game.rs`, `sprite/texture.rs`)
   - PCX图像渲染到游戏场景
   - CLX精灵集成到实体系统
   - 调色板应用到渲染系统
   - 资源加载到SDL纹理系统
   - 场景测试（加载并显示原版资源）

4. **城镇场景预览** ⭐️ 新增功能 (`world/town.rs`, `game.rs`)
   - 加载城镇背景图像（使用PCX或静态场景）
   - 加载城镇调色板（`levels/towndata/town.pal`）
   - 创建简化的城镇场景（静态背景 + 玩家行走）
   - 玩家可以在城镇场景中移动
   - **注意**：这是预览版本，完整的地图系统（.DUN, .TIL, .CEL）将在Step 6实现

**测试要求：**
- 单元测试：TRN加载、颜色映射、资源管理器缓存
- 集成测试：完整资源加载流程、多资源组合场景
- 游戏集成：综合场景测试、城镇场景测试

**游戏集成验证：**
- **调色板应用场景**：加载城镇调色板，应用到PCX和CLX资源
- **TRN转换场景**：加载TRN文件，应用到精灵，验证颜色变化
- **综合场景**：同时加载多个资源（PCX背景 + CLX精灵），验证正确叠加显示
- **城镇场景预览** ⭐️ 重要：
  - 加载城镇调色板（`levels/towndata/town.pal`）
  - 创建城镇场景（可以使用PCX背景或简化的静态场景）
  - 玩家可以在场景中移动（WASD控制）
  - 验证玩家移动流畅，场景正确显示
  - **目标**：玩家可以在城镇场景中自由行走

**验收标准：**
- ✅ 能够正确应用TRN颜色转换
- ✅ 资源管理器正常工作
- ✅ 资源缓存正常工作
- ✅ 场景加载功能完整
- ✅ 所有资源正确集成到游戏
- ✅ **城镇场景预览**：能够加载城镇场景，玩家可以在其中行走
- ✅ 所有测试通过
- ✅ 测试覆盖率 ≥ 85%

**城镇场景实现说明：**

由于完整的地图系统（.DUN, .TIL, .CEL格式）将在Step 6实现，Step 5.3的城镇场景预览可以采用以下方案：

**方案1：PCX背景方案（推荐）**
- 加载城镇背景PCX图像（如果有）
- 作为静态背景渲染
- 玩家在背景上移动
- 使用现有的碰撞检测系统（简化版）

**方案2：简化瓦片方案**
- 使用现有的瓦片系统
- 加载城镇调色板应用到瓦片
- 创建简化的城镇布局（手动或使用简单的生成逻辑）
- 玩家在瓦片地图上移动

**方案3：混合方案**
- 使用PCX作为背景
- 使用简化的碰撞地图（可行走区域）
- 玩家在背景上移动，碰撞检测使用简化地图

**完整城镇加载将在Step 6实现：**
- 加载.DUN地图数据文件
- 加载.TIL瓦片集文件
- 加载.CEL瓦片图像文件
- 完整的地图渲染系统
- 完整的碰撞检测系统

---

## 📋 功能清单（总体）

### 1. MPQ归档系统 (`resources/mpq.rs`)
- [x] MPQ归档读取（使用`mpq` crate）
- [x] 文件列表枚举
- [x] 文件哈希查找
- [x] 文件解压缩
- [x] MPQ优先级系统（多MPQ文件加载，优先级覆盖）
- [x] MPQ缓存系统（已加载文件缓存）

**参考代码：**
- `Source/mpq/mpq_reader.cpp` - MPQ读取实现
- `Source/engine/assets.cpp` - MPQ管理和优先级系统

### 2. PCX图像格式 (`resources/pcx.rs`)
- [x] PCX文件头解析（128字节）
- [x] RLE压缩解码
- [x] 调色板读取（文件末尾768字节）
- [x] 转换为RGBA纹理

**参考代码：**
- `Source/engine/load_pcx.cpp` - PCX加载实现
- `Source/utils/pcx.hpp` - PCX格式定义

### 3. 调色板系统 (`resources/palette.rs`)
- [x] 调色板文件加载（.PAL，768字节）
- [x] 调色板结构（256色RGB）
- [x] 调色板应用到索引图像
- [x] 透明色处理（索引0通常为透明）

**参考代码：**
- `Source/engine/palette.cpp` - 调色板系统
- `Source/utils/palette_kd_tree.hpp` - 调色板KD树（高级功能，可选）

### 4. CLX精灵格式 (`resources/clx.rs`)
- [x] CLX文件头解析（帧数、帧偏移）
- [x] CLX帧头解析（宽度、高度）
- [x] CL2 RLE像素解码
- [x] 透明像素处理
- [x] 多帧动画支持

**参考代码：**
- `Source/engine/clx_sprite.hpp` - CLX格式定义
- `Source/utils/clx_decode.cpp` - CLX解码实现
- `Source/engine/load_clx.cpp` - CLX加载

### 5. TRN颜色转换 (`resources/trn.rs`)
- [x] TRN文件加载（256字节映射表）
- [x] 颜色映射应用
- [x] 应用到精灵像素

**参考代码：**
- `Source/engine/load_file.hpp` - 文件加载（TRN作为通用文件）

### 6. 资源管理器 (`resources/mod.rs`)
- [x] 统一资源加载接口
- [x] 资源缓存管理
- [x] MPQ管理器集成
- [x] 调色板管理器
- [x] 纹理缓存

**参考代码：**
- `Source/engine/assets.cpp` - 资源管理
- `Source/engine/load_file.hpp` - 文件加载接口

### 7. 场景加载和资源集成 (`game.rs`, `sprite/texture.rs`)
- [x] PCX图像渲染到游戏场景
- [x] CLX精灵集成到实体系统
- [x] 调色板应用到渲染系统
- [x] 资源加载到SDL纹理系统
- [x] 场景测试（加载并显示原版资源）

**功能要求：**
1. **PCX图像场景**
   - 从MPQ加载PCX图像（如`ui_art/logo.pcx`）
   - 转换为SDL纹理
   - 在游戏窗口中渲染显示

2. **CLX精灵场景**
   - 从MPQ加载CLX精灵（如`plrgfx/warrior/whs/whsas.cl2`）
   - 应用调色板转换为RGBA纹理
   - 集成到实体系统，替换当前猴子精灵
   - 支持多帧动画播放

3. **调色板应用场景**
   - 加载关卡调色板（如`levels/towndata/town.pal`）
   - 应用到所有使用该调色板的资源
   - 验证颜色正确性

4. **资源集成测试场景**
   - 创建测试场景，加载多个原版资源
   - 验证资源正确加载和渲染
   - 验证性能（加载时间、内存使用）

**参考代码：**
- `Source/engine/render/scrollrt.cpp` - 渲染系统
- `Source/player.cpp::NewPlrAnim()` - 玩家精灵加载
- `Source/engine/load_pcx.cpp` - PCX加载和渲染

## 🏗️ 架构设计

### 模块结构

```
src/resources/
├── mod.rs          # 资源管理器主模块
├── mpq.rs          # MPQ归档读取
├── pcx.rs          # PCX图像加载
├── palette.rs      # 调色板系统
├── clx.rs          # CLX精灵加载
└── trn.rs          # 颜色转换
```

### 数据流

```
MPQ归档
  ↓
文件数据（压缩）
  ↓
解压缩
  ↓
格式解析（PCX/CLX/PAL/TRN）
  ↓
转换为游戏内部格式
  ↓
缓存到ResourceManager
  ↓
渲染系统使用
```

## 📝 实现细节

### 1. MPQ管理器

```rust
pub struct MpqManager {
    archives: Vec<(mpq::Archive, i32)>, // (归档, 优先级)
}

impl MpqManager {
    pub fn load_mpq(&mut self, path: &str, priority: i32) -> Result<()>;
    pub fn find_file(&self, path: &str) -> Option<Vec<u8>>;
    pub fn has_file(&self, path: &str) -> bool;
}
```

**优先级规则：**
- 数字越大，优先级越高
- 查找文件时，从高优先级到低优先级
- 高优先级文件覆盖低优先级文件（Mod系统基础）

### 2. PCX加载器

```rust
pub struct PcxImage {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,      // 256色索引
    pub palette: Palette,
}

pub fn load_pcx(data: &[u8]) -> Result<PcxImage>;
```

**RLE解码规则：**
- 如果字节 >= 0xC0: `(字节 & 0x3F)` 表示重复次数，下一个字节是值
- 否则：直接是像素值

### 3. 调色板系统

```rust
#[derive(Clone)]
pub struct Palette {
    pub colors: [Color; 256],
}

impl Palette {
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    pub fn to_rgba(&self, index: u8, transparent: bool) -> [u8; 4];
}
```

**调色板布局：**
- 0-127: 关卡特定颜色
- 128-255: 全局颜色（蓝色、红色、黄色等）

### 4. CLX精灵加载

```rust
pub struct ClxFrame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<Option<u8>>,  // None表示透明
}

pub fn load_clx(data: &[u8]) -> Result<Vec<ClxFrame>>;
```

**CL2 RLE编码规则：**
- 负数（i8 < 0）: 重复下一个像素 `|value|` 次
- 正数（i8 > 0）: 接下来 `value` 个字节是原始像素
- 0x80: 透明像素（跳过）

### 5. TRN颜色转换

```rust
pub struct ColorTransform {
    pub map: [u8; 256],
}

impl ColorTransform {
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    pub fn apply(&self, color_index: u8) -> u8;
    pub fn apply_to_pixels(&self, pixels: &mut [u8]);
}
```

### 6. 资源管理器

```rust
pub struct ResourceManager {
    mpq_manager: MpqManager,
    palette_cache: HashMap<String, Palette>,
    texture_cache: HashMap<String, TextureId>,
    sprite_cache: HashMap<String, Vec<ClxFrame>>,
    trn_cache: HashMap<String, ColorTransform>,
}

impl ResourceManager {
    pub fn new() -> Result<Self>;
    pub fn load_pcx(&mut self, path: &str) -> Result<TextureId>;
    pub fn load_clx(&mut self, path: &str) -> Result<&Vec<ClxFrame>>;
    pub fn load_palette(&mut self, path: &str) -> Result<&Palette>;
    pub fn load_trn(&mut self, path: &str) -> Result<&ColorTransform>;
}
```

## 🔧 依赖更新

需要在 `Cargo.toml` 中添加：

```toml
[dependencies]
mpq = "0.8"  # MPQ归档支持
```

## 🧪 测试要求

### 单元测试

1. **MPQ测试**
   - MPQ文件加载
   - 文件查找
   - 优先级覆盖

2. **PCX测试**
   - PCX文件解析
   - RLE解码
   - 调色板读取

3. **调色板测试**
   - 调色板加载
   - RGB转换
   - 透明色处理

4. **CLX测试**
   - CLX文件解析
   - 帧头解析
   - RLE像素解码
   - 透明像素处理

5. **TRN测试**
   - TRN文件加载
   - 颜色映射应用

### 集成测试

1. **资源加载流程**
   - 从MPQ加载PCX
   - 从MPQ加载CLX
   - 调色板应用到图像

2. **缓存系统**
   - 重复加载使用缓存
   - 缓存失效机制

3. **场景加载测试**
   - PCX图像场景渲染
   - CLX精灵场景渲染
   - 调色板应用场景
   - 多资源组合场景

### 性能测试

1. MPQ文件查找性能
2. PCX解码性能
3. CLX解码性能
4. 资源缓存命中率

**测试覆盖率目标：≥ 85%**

## 📚 参考资源

### 原版代码参考

- `Source/mpq/mpq_reader.cpp` - MPQ读取
- `Source/engine/assets.cpp` - 资源管理
- `Source/engine/load_pcx.cpp` - PCX加载
- `Source/engine/clx_sprite.hpp` - CLX格式
- `Source/utils/clx_decode.cpp` - CLX解码
- `Source/engine/palette.cpp` - 调色板系统

### 技术文档

- [Step 5技术要点 - 资源格式详解](tech_key_points/step5-resource-formats.md)
- [Diablo 1 File Formats](https://github.com/savagesteel/d1-file-formats)
- [MPQ Format Specification](http://www.zezula.net/en/mpq/mpqformat.html)

## 🎯 验收标准（按子步骤）

### Step 5.1 验收标准

1. ✅ 能够成功加载DIABDAT.MPQ文件
2. ✅ 能够从MPQ中查找和读取文件
3. ✅ 能够正确解析调色板文件
4. ✅ 调色板RGB值正确
5. ✅ 所有单元测试通过
6. ✅ **游戏集成验证**：在游戏中加载MPQ并读取调色板，在日志中输出调色板信息

### Step 5.2 验收标准

1. ✅ 能够正确解析PCX文件
2. ✅ PCX图像在游戏中正确显示（测试场景：显示logo.pcx）
3. ✅ 能够正确解析CLX文件
4. ✅ CLX精灵在游戏中正确渲染（测试场景：战士站立动画）
5. ✅ CLX多帧动画正确播放
6. ✅ 所有单元测试通过
7. ✅ **游戏集成验证**：PCX和CLX资源在游戏中正确渲染

### Step 5.3 验收标准

1. ✅ 能够正确应用TRN颜色转换
2. ✅ 资源管理器正常工作
3. ✅ 资源缓存正常工作
4. ✅ 场景加载功能完整
5. ✅ 所有资源正确集成到游戏
6. ✅ **城镇场景预览**：能够加载城镇场景，玩家可以在其中行走 ⭐️ 重要
7. ✅ 所有单元测试通过
8. ✅ 测试覆盖率 ≥ 85%
9. ✅ **游戏集成验证**：综合场景测试，多个资源正确叠加显示
10. ✅ **城镇场景验收**：玩家可以在城镇场景中自由移动，场景正确显示

### 总体验收标准

完成所有子步骤后，应该能够：

1. ✅ 从DIABDAT.MPQ加载资源文件
2. ✅ 解析并显示PCX图像（如logo.pcx）
3. ✅ 加载并渲染CLX精灵（如战士站立动画）
4. ✅ 正确应用调色板到索引图像
5. ✅ 支持TRN颜色转换（怪物变色）
6. ✅ 资源缓存正常工作
7. ✅ **场景加载功能**：能够在游戏中加载并显示原版资源
   - PCX图像正确显示在游戏窗口中
   - CLX精灵正确渲染并支持动画
   - 调色板正确应用到所有资源
8. ✅ **城镇场景预览**：玩家可以在城镇场景中自由行走 ⭐️ 重要目标
   - 城镇场景正确加载和显示
   - 玩家可以在场景中移动（WASD控制）
   - 场景视觉效果正确（调色板、精灵等）
   - **注意**：这是预览版本，完整地图系统在Step 6实现
9. ✅ 所有单元测试通过
10. ✅ 测试覆盖率 ≥ 85%

## 📝 实现步骤（按子步骤）

### Step 5.1: MPQ归档系统和调色板系统

1. **添加依赖**
   - 更新 `Cargo.toml`，添加 `mpq` crate

2. **创建资源模块结构**
   - 创建 `src/resources/` 目录
   - 创建 `resources/mpq.rs` 和 `resources/palette.rs`

3. **实现MPQ管理器** (`resources/mpq.rs`)
   - MPQ归档加载
   - 文件查找
   - 优先级系统
   - MPQ缓存系统

4. **实现调色板系统** (`resources/palette.rs`)
   - 调色板加载
   - RGB转换
   - 透明色处理

5. **编写测试**
   - MPQ单元测试（文件加载、查找、优先级）
   - 调色板单元测试（加载、RGB转换）
   - 集成测试（从MPQ加载调色板）

6. **游戏集成验证** ⭐️ 重要
   - 在 `Game::new()` 中初始化MPQ管理器
   - 加载DIABDAT.MPQ文件
   - 从MPQ加载一个调色板文件（如`levels/towndata/town.pal`）
   - 在日志中输出调色板信息（前10个颜色值）
   - 验证MPQ文件路径搜索正常工作
   - **运行游戏，验证能够成功加载MPQ和调色板**

---

### Step 5.2: PCX图像格式和CLX精灵格式

1. **创建PCX和CLX模块**
   - 创建 `resources/pcx.rs` 和 `resources/clx.rs`

2. **实现PCX加载器** (`resources/pcx.rs`)
   - PCX文件头解析
   - RLE解码
   - 调色板读取（文件末尾768字节）
   - 转换为RGBA纹理

3. **实现CLX加载器** (`resources/clx.rs`)
   - CLX文件头解析
   - 帧头解析
   - CL2 RLE像素解码
   - 透明像素处理
   - 多帧动画支持

4. **编写测试**
   - PCX单元测试（文件解析、RLE解码）
   - CLX单元测试（文件解析、帧头解析、RLE解码）
   - 集成测试（从MPQ加载PCX和CLX）

5. **游戏集成验证** ⭐️ 重要
   - **PCX图像场景**
     - 扩展 `sprite/texture.rs`，支持从PCX创建纹理
     - 在 `Game::new()` 中加载测试PCX图像（`ui_art/logo.pcx`）
     - 在 `Game::render()` 中渲染PCX图像到场景中央
     - **运行游戏，验证PCX图像正确显示**
   
   - **CLX精灵场景**
     - 扩展 `sprite/texture.rs`，支持从CLX帧创建纹理
     - 扩展 `sprite/animation.rs`，支持CLX多帧动画
     - 在 `Entity::create_player()` 中使用CLX精灵替换猴子精灵
     - 加载战士站立动画（`plrgfx/warrior/whs/whsas.cl2`）
     - **运行游戏，验证CLX精灵正确渲染，动画流畅播放**

---

### Step 5.3: TRN颜色转换和场景加载集成

1. **创建TRN模块和资源管理器**
   - 创建 `resources/trn.rs`
   - 创建 `resources/mod.rs`（资源管理器）

2. **实现TRN颜色转换** (`resources/trn.rs`)
   - TRN文件加载
   - 颜色映射应用
   - 应用到精灵像素

3. **实现资源管理器** (`resources/mod.rs`)
   - 统一资源加载接口
   - 资源缓存管理
   - MPQ管理器集成
   - 调色板管理器
   - 纹理缓存

4. **场景加载集成** ⭐️ 重要
   - **调色板应用**
     - 在资源加载时应用调色板
     - 将索引图像转换为RGBA纹理
     - 验证颜色正确性
   
   - **TRN转换场景**
     - 加载TRN文件
     - 应用到精灵
     - 验证颜色变化
   
   - **综合场景**
     - 同时加载多个资源（PCX背景 + CLX精灵）
     - 验证资源正确叠加显示
     - 验证性能可接受（60 FPS）

5. **编写测试**
   - TRN单元测试
   - 资源管理器测试（缓存、统一接口）
   - 集成测试（完整资源加载流程）
   - 场景测试（多资源组合）

6. **实现城镇场景预览** ⭐️ 重要目标
   - **加载城镇资源**
     - 加载城镇调色板（`levels/towndata/town.pal`）
     - 尝试加载城镇背景PCX（如果有）
     - 加载玩家CLX精灵（使用原版战士精灵）
   
   - **创建城镇场景**（`world/town.rs` 或 `game.rs`）
     - **方案1：PCX背景方案**（推荐）
       - 使用PCX作为静态背景（如果有城镇背景图）
       - 创建简化的碰撞地图（可行走区域）
       - 玩家在背景上移动
     - **方案2：简化瓦片方案**
       - 使用现有的瓦片系统
       - 加载城镇调色板应用到瓦片
       - 创建简化的城镇布局（手动定义或简单生成）
       - 玩家在瓦片地图上移动
     - **方案3：混合方案**
       - PCX作为背景
       - 简化的碰撞地图用于碰撞检测
       - 玩家在背景上移动
   
   - **集成到游戏**
     - 在 `Game::new()` 中加载城镇场景
     - 在 `Game::render()` 中渲染城镇场景
     - 玩家可以在场景中移动（WASD控制）
     - 验证玩家移动流畅，场景正确显示
   
   - **验收测试**
     - 运行游戏，进入城镇场景
     - 使用WASD移动玩家
     - 验证玩家可以在场景中自由行走
     - 验证场景视觉效果正确（调色板、精灵等）
     - **目标**：玩家可以在城镇场景中自由行走

7. **游戏集成验证** ⭐️ 重要
   - 创建 `examples/test_resource_scene.rs` 测试场景
   - **调色板应用场景**：加载城镇调色板，应用到PCX和CLX资源
   - **TRN转换场景**：加载TRN文件，应用到精灵，验证颜色变化
   - **综合场景**：同时加载多个资源，验证正确叠加显示
   - **城镇场景预览**：玩家可以在城镇场景中行走
   - **运行游戏，验证所有场景正确显示，性能可接受**

8. **完善和优化**
   - 错误处理完善
   - 性能优化（缓存命中率）
   - 代码清理和文档
   - 测试覆盖率检查（目标 ≥ 85%）

**注意：** Step 5.3的城镇场景是"预览版本"，使用简化的实现。完整的地图系统（.DUN, .TIL, .CEL格式）将在Step 6实现，届时将支持完整的地图加载、瓦片渲染和碰撞检测。

## 🐛 潜在问题和解决方案

### 问题1: MPQ文件路径
**问题：** 如何找到DIABDAT.MPQ文件？
**解决：** 
- 支持多个搜索路径（当前目录、资源目录等）
- 提供配置选项
- 参考原版代码的路径搜索逻辑

### 问题2: CLX RLE解码边界
**问题：** RLE解码时可能越界
**解决：**
- 严格检查数据长度
- 使用 `checked_add` 等安全操作
- 添加边界检查测试

### 问题3: 调色板索引0透明
**问题：** 索引0是否总是透明？
**解决：**
- 提供选项控制透明色
- 某些情况下索引0不是透明（如UI图像）

### 问题4: 内存使用
**问题：** 大量资源可能导致内存占用过高
**解决：**
- 实现LRU缓存
- 支持资源卸载
- 延迟加载

### 问题5: 场景加载集成
**问题：** 如何将解析的资源集成到现有渲染系统？

**解决方案：**
1. **纹理转换**
   - PCX/CLX解析后转换为RGBA像素数据
   - 使用SDL2的`create_texture_from_surface`创建纹理
   - 将纹理ID存储到现有纹理管理系统

2. **动画系统集成**
   - CLX多帧需要映射到现有动画系统
   - 保持动画状态机不变，只替换纹理数据
   - 确保帧率同步

3. **调色板应用时机**
   - 在资源加载时应用调色板（转换为RGBA）
   - 或运行时应用（需要特殊渲染支持）
   - 建议：加载时转换，简化渲染逻辑

4. **错误处理**
   - 资源加载失败时使用fallback（如当前猴子精灵）
   - 提供清晰的错误日志
   - 支持资源重试加载

**参考代码：**
- `Source/engine/render/scrollrt.cpp` - 渲染系统如何集成资源
- `Source/player.cpp::NewPlrAnim()` - 玩家精灵如何加载和渲染

## 📚 学习要点

### 1. 为什么需要实现这些格式？

**原版Diablo的资源格式特点：**
- **MPQ归档**：所有资源打包在一个文件中，便于分发和保护
- **256色调色板**：90年代游戏的标准做法，节省内存和存储
- **RLE压缩**：减少文件大小，但需要运行时解码
- **索引图像**：使用调色板索引而非直接RGB，节省空间

**现代游戏通常使用：**
- PNG/JPG等标准格式
- 直接RGB/RGBA颜色
- 无需运行时解码

**为什么必须实现原版格式？**
1. **兼容性**：直接使用原版DIABDAT.MPQ，无需资源转换
2. **完整性**：保持与原版100%的视觉一致性
3. **学习价值**：理解90年代游戏资源管理技术
4. **扩展支持**：支持Hellfire扩展包和Mod系统

### 2. RLE压缩算法原理

**RLE (Run-Length Encoding)** 是一种简单的压缩算法：

**PCX RLE规则：**
```
如果字节 >= 0xC0 (192):
  重复次数 = 字节 & 0x3F (取低6位)
  下一个字节是重复的值
否则:
  直接是像素值
```

**示例：**
```
输入: 0xC5 0x10 0x20 0x30
解码: 重复5次0x10，然后是0x20，然后是0x30
输出: 0x10 0x10 0x10 0x10 0x10 0x20 0x30
```

**CL2 RLE规则（更复杂）：**
```
负数 (i8 < 0): 重复下一个像素 |value| 次
正数 (i8 > 0): 接下来value个字节是原始像素
0x80: 透明像素（跳过）
```

**为什么使用RLE？**
- 简单高效，适合90年代硬件
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
- **全局颜色**：玩家、怪物、UI等在所有关卡中颜色一致
- **内存优化**：只需加载一个调色板，而非每个关卡一个

**透明色处理：**
- 索引0通常作为透明色
- 但某些图像（如UI）索引0不是透明
- 需要提供选项控制透明色行为

### 4. CLX格式的优势

**格式演进：**
```
CEL (原始) → CL2 (压缩优化) → CLX (运行时格式)
```

**CLX相比CL2的改进：**
1. **简化头部**：CLX帧头只存储宽度和高度，不存储32像素块偏移
2. **运行时优化**：CLX是DevilutionX的运行时格式，更适合现代硬件
3. **向后兼容**：可以从CL2/CEL转换到CLX

**为什么使用CLX？**
- 原版使用CL2，但CLX更适合我们的实现
- 可以从CL2转换，也可以直接加载CLX
- 简化了渲染逻辑

### 5. 资源缓存策略

**缓存类型：**
1. **MPQ文件缓存**：已解压的文件数据
2. **纹理缓存**：已转换为SDL纹理的图像
3. **精灵缓存**：已解析的CLX帧
4. **调色板缓存**：已加载的调色板

**缓存策略：**
- **LRU (Least Recently Used)**：最近最少使用的资源优先淘汰
- **按需加载**：只在需要时加载资源
- **内存限制**：设置最大缓存大小，防止内存溢出

## ⚠️ 踩坑点和注意事项

### 1. MPQ文件路径问题

**问题：** 如何找到DIABDAT.MPQ文件？

**原版代码参考：** `Source/engine/assets.cpp::GetMPQSearchPaths()`

**解决方案：**
```rust
// 搜索路径优先级：
// 1. 当前工作目录
// 2. 可执行文件目录
// 3. 资源目录（assets/）
// 4. 用户配置目录
```

**注意事项：**
- Windows上路径大小写不敏感，但MPQ文件名可能大小写敏感
- 需要支持多个搜索路径
- 提供清晰的错误信息，帮助用户找到MPQ文件

### 2. CLX RLE解码边界检查

**问题：** RLE解码时可能读取越界，导致panic

**原版代码参考：** `Source/utils/clx_decode.cpp`

**解决方案：**
```rust
// 使用 checked_add 防止溢出
let next_idx = pixel_idx.checked_add(count)?;

// 检查数据长度
if data_idx + count > data.len() {
    return Err(anyhow!("RLE decode overflow"));
}

// 使用 Result 而非 panic
```

**测试要点：**
- 测试边界情况（最后一个像素）
- 测试损坏的文件
- 测试空文件

### 3. 调色板索引0透明问题

**问题：** 索引0是否总是透明？

**实际情况：**
- **精灵图像**：索引0通常是透明
- **UI图像**：索引0可能不是透明（如logo.pcx）
- **地图瓦片**：索引0可能是黑色，不是透明

**解决方案：**
```rust
pub fn to_rgba(&self, index: u8, transparent: bool) -> [u8; 4] {
    let alpha = if transparent && index == 0 { 0 } else { 255 };
    // ...
}
```

**设计决策：**
- 提供 `transparent` 参数，让调用者决定
- 默认情况下，索引0作为透明
- UI图像加载时，明确指定 `transparent: false`

### 4. 字节序（Endianness）问题

**问题：** PCX和CLX文件使用小端序（Little Endian）

**原版代码参考：** `Source/utils/endian_read.hpp`

**解决方案：**
```rust
// 使用标准库函数
let width = u16::from_le_bytes([data[8], data[9]]);
let height = u16::from_le_bytes([data[10], data[11]]);

// 或使用 byteorder crate（如果需要）
use byteorder::{LittleEndian, ReadBytesExt};
```

**注意事项：**
- Rust默认是小端序（x86/x64），但显式指定更安全
- 跨平台兼容性考虑

### 5. 内存使用优化

**问题：** 大量资源可能导致内存占用过高

**优化策略：**
1. **延迟加载**：只在需要时加载资源
2. **LRU缓存**：淘汰最近最少使用的资源
3. **资源卸载**：提供手动卸载接口
4. **压缩存储**：保持MPQ中的压缩格式，只在需要时解压

**实现建议：**
```rust
pub struct ResourceManager {
    cache: LruCache<String, Resource>,
    max_cache_size: usize,
}

impl ResourceManager {
    pub fn load_with_limit(&mut self, path: &str) -> Result<Resource> {
        // 检查缓存大小，必要时淘汰
        while self.cache.len() >= self.max_cache_size {
            self.cache.pop_lru();
        }
        // 加载资源
    }
}
```

### 6. MPQ优先级系统

**问题：** 多个MPQ文件可能包含同名文件，如何选择？

**原版代码参考：** `Source/engine/assets.cpp::FindMpqFile()`

**解决方案：**
```rust
// 按优先级从高到低查找
for (archive, priority) in self.archives.iter().rev() {
    if let Ok(data) = archive.read_file(path) {
        return Some(data); // 找到就返回，高优先级覆盖低优先级
    }
}
```

**优先级规则：**
- 数字越大，优先级越高
- Mod文件优先级 > 扩展包 > 基础游戏
- 这为后续Mod系统打下基础

## 🔍 参考代码出处和改造细节

### 1. MPQ管理器

**原版代码：**
- `Source/mpq/mpq_reader.cpp` - MPQ读取实现
- `Source/engine/assets.cpp::LoadMPQ()` - MPQ加载和优先级管理

**改造思路：**
- 原版使用C的libmpq库，我们使用Rust的`mpq` crate
- 保持相同的优先级系统逻辑
- 使用Rust的`Result`类型处理错误，而非C的错误码

**关键改造：**
```rust
// 原版C++代码使用错误码
int32_t error = 0;
auto archive = MpqArchive::Open(path, error);

// Rust版本使用Result
pub fn load_mpq(&mut self, path: &str, priority: i32) -> Result<()> {
    let archive = mpq::Archive::open(path)?;
    // ...
}
```

### 2. PCX加载器

**原版代码：**
- `Source/engine/load_pcx.cpp` - PCX加载入口
- `Source/utils/pcx.hpp` - PCX格式定义
- `Source/utils/pcx_to_clx.cpp` - PCX转CLX（我们不需要，直接渲染）

**改造思路：**
- 原版将PCX转换为CLX格式，我们直接解析PCX并转换为RGBA纹理
- 保持相同的RLE解码逻辑
- 使用Rust的`Vec<u8>`存储像素数据

**关键改造：**
```rust
// 原版C++代码
std::unique_ptr<uint8_t[]> fileBuffer(new uint8_t[pixelDataSize]);
handle.read(fileBuffer.get(), pixelDataSize);

// Rust版本
let mut pixels = Vec::new();
// RLE解码逻辑相同
```

### 3. CLX加载器

**原版代码：**
- `Source/engine/clx_sprite.hpp` - CLX格式定义
- `Source/utils/clx_decode.cpp` - CLX解码实现
- `Source/engine/load_clx.cpp` - CLX加载

**改造思路：**
- 保持CLX格式定义不变
- 使用Rust的`Option<u8>`表示透明像素（原版使用特殊值）
- 使用`Result`处理解码错误

**关键改造：**
```rust
// 原版C++代码使用指针和大小
const uint8_t *pixelData() const { return &data_[LoadLE16(data_)]; }

// Rust版本使用切片和Option
pub struct ClxFrame {
    pub pixels: Vec<Option<u8>>, // None表示透明
}
```

### 4. 调色板系统

**原版代码：**
- `Source/engine/palette.cpp` - 调色板系统
- `Source/utils/palette_kd_tree.hpp` - 调色板KD树（高级功能，可选）

**改造思路：**
- 简化实现，先实现基础调色板功能
- KD树优化可以后续添加
- 使用Rust的数组类型`[Color; 256]`存储调色板

**关键改造：**
```rust
// 原版C++代码使用指针
SDL_Color *outPalette;

// Rust版本使用结构体
pub struct Palette {
    pub colors: [Color; 256],
}
```

### 5. 资源管理器

**原版代码：**
- `Source/engine/assets.cpp` - 资源管理
- `Source/engine/load_file.hpp` - 文件加载接口

**改造思路：**
- 统一所有资源加载接口
- 使用HashMap实现缓存（原版可能使用其他数据结构）
- 提供类型安全的API

**关键改造：**
```rust
// 原版C++代码使用函数重载
OptionalOwnedClxSpriteList LoadPcxSpriteList(...);
OptionalOwnedClxSpriteList LoadClx(...);

// Rust版本使用统一接口和泛型
impl ResourceManager {
    pub fn load_pcx(&mut self, path: &str) -> Result<TextureId>;
    pub fn load_clx(&mut self, path: &str) -> Result<&Vec<ClxFrame>>;
}
```

### 6. 场景加载集成

**原版代码：**
- `Source/engine/render/scrollrt.cpp` - 渲染系统
- `Source/player.cpp::NewPlrAnim()` - 玩家精灵加载和渲染
- `Source/engine/load_pcx.cpp` - PCX加载后如何渲染

**改造思路：**
- 原版资源加载后直接转换为SDL表面（Surface），我们转换为SDL纹理（Texture）
- 保持现有渲染流程不变，只替换资源来源
- 使用现有的纹理管理系统，无需大幅修改渲染代码

**关键改造：**
```rust
// 原版C++代码：PCX加载后转换为CLX，然后渲染
OptionalOwnedClxSpriteList LoadPcxSpriteList(...);
// 渲染时使用CLX数据

// Rust版本：PCX加载后转换为RGBA纹理
let pcx = load_pcx(data)?;
let rgba = pcx.to_rgba(&palette)?;
let texture = engine.create_texture_from_rgba(rgba)?;
// 使用现有纹理系统渲染
```

**场景加载流程：**
```rust
// 1. 初始化资源管理器
let mut res_mgr = ResourceManager::new()?;
res_mgr.load_mpq("DIABDAT.MPQ", 1000)?;

// 2. 加载资源
let pcx_texture = res_mgr.load_pcx("ui_art/logo.pcx")?;
let clx_frames = res_mgr.load_clx("plrgfx/warrior/whs/whsas.cl2")?;

// 3. 转换为游戏纹理
let logo_texture = engine.create_texture_from_pcx(pcx_texture)?;
let player_textures: Vec<TextureId> = clx_frames.iter()
    .map(|frame| engine.create_texture_from_clx(frame, &palette))
    .collect::<Result<_>>()?;

// 4. 在场景中使用
// PCX作为背景
engine.draw_texture(logo_texture, ...)?;
// CLX作为玩家精灵
entity.set_sprites(player_textures);
```

## 🔜 下一步计划

完成Step 5后，下一步是：
- **Step 6: 地图生成系统 Part 1** - 使用原版瓦片数据（.MIN, .TIL, .SOL）
- 可以开始加载原版地图资源
- 可以开始使用原版精灵资源

---

**文档版本：** 1.1  
**最后更新：** 2025-01-20  
**关联文档：** [MASTER_PLAN.md](master_plan.md), [技术要点文档](tech_key_points/step5-resource-formats.md)

