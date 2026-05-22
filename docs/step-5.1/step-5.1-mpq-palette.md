# Step 5.1: MPQ归档系统和调色板系统 - 设计文档

## 📅 日期
2025年1月20日

## 🎯 目标

实现资源加载的基础设施，能够从MPQ文件读取数据，并理解调色板系统。这是Step 5的第一个子步骤。

## 📋 功能需求

### 1. MPQ归档系统 (`resources/mpq.rs`)
- MPQ归档读取
- 文件查找和读取
- 文件解压缩（PKWare, zlib, bzip2）
- MPQ优先级系统（多MPQ文件加载，优先级覆盖）
- 路径搜索系统

### 2. 调色板系统 (`resources/palette.rs`)
- 调色板文件加载（.PAL，768字节）
- 调色板结构（256色RGB）
- 调色板应用到索引图像
- 透明色处理（索引0通常为透明）

## 🔧 技术选型

### MPQ库选择：mpq-rust

**原计划 vs 最终方案：**

| 方案 | 描述 | 优势 | 劣势 | 决策 |
|------|------|------|------|------|
| **原计划**：手动实现 | 自己实现MPQ格式解析、表解密、PKWare解压缩 | 完全控制、学习价值高 | 开发成本高、维护复杂、容易出错 | ❌ 放弃 |
| **最终方案**：mpq-rust | 使用成熟的第三方库 | 稳定可靠、开发效率高、已验证 | 依赖外部库 | ✅ 采用 |

**mpq-rust 库信息：**
- **库名**: mpq-rust
- **版本**: 0.8
- **仓库**: https://github.com/msierks/mpq-rust
- **许可证**: MIT/Apache-2.0（双许可证）
- **作者**: Michael Sierks

**选择理由：**
1. ✅ **完整的MPQ v1格式支持** - 适用于Diablo 1的MPQ格式
2. ✅ **内置解密和解压缩** - 支持表解密和PKWare/zlib/bzip2解压缩
3. ✅ **简单易用的API** - `Archive::open()`, `open_file()`, `read()`
4. ✅ **经过测试验证** - 已在实际项目中使用
5. ✅ **减少维护成本** - 专注于游戏逻辑而非底层格式

**放弃手动实现的原因：**

手动实现需要：
1. 实现MPQ表解密算法（需要crypt table）
2. 实现PKWare解压缩（复杂的算法）
3. 处理各种边界情况和错误
4. 编写大量测试代码

预计工作量：~1000行代码 + ~300行测试

使用mpq-rust：~200行代码 + 复用库的测试

**节省工作量：** ~1100行代码（约80%）

## 🏗️ 架构设计

### 模块结构

```
src/resources/
├── mod.rs          # 资源管理器主模块
├── mpq.rs          # MPQ归档管理（使用mpq-rust）
└── palette.rs      # 调色板系统
```

**注意：** `mpq_format.rs` 已删除（不再需要手动实现格式定义）

### MPQ管理器设计

```rust
pub struct MpqManager {
    /// MPQ archives, sorted by priority (low priority first, high priority last)
    archives: Vec<MpqArchiveWrapper>,
}

struct MpqArchiveWrapper {
    /// The underlying MPQ archive from mpq-rust
    archive: Archive,  // 来自 mpq crate
    /// Archive priority (higher number = higher priority)
    priority: i32,
    /// Archive path (for display/debugging)
    path: PathBuf,
}
```

**关键API：**

```rust
impl MpqManager {
    pub fn new() -> Self;
    pub fn load_mpq(&mut self, path: &str, priority: i32) -> Result<()>;
    pub fn find_file(&mut self, path: &str) -> Option<Vec<u8>>;
    pub fn has_file(&mut self, path: &str) -> bool;
    pub fn get_loaded_archives(&self) -> Vec<String>;
}
```

### 调色板系统设计

```rust
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone)]
pub struct Palette {
    pub colors: [Color; 256],
}

impl Palette {
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    pub fn from_mpq(mpq: &mut MpqManager, path: &str) -> Result<Self>;
    pub fn to_rgb(&self, index: u8) -> Color;
    pub fn to_rgba(&self, index: u8, transparent: bool) -> [u8; 4];
    pub fn indices_to_rgba(&self, indices: &[u8], transparent: bool) -> Vec<u8>;
}
```

## 📝 实现细节

### 1. MPQ文件搜索

**搜索路径优先级：**
1. 当前工作目录
2. 可执行文件目录
3. `assets/` 目录
4. `rust-diablo/assets/` 目录
5. `diablo-asset/` 目录

**路径规范化：**
- 支持绝对路径
- 支持大小写不敏感搜索（Windows）
- 自动尝试大写/小写变体

### 2. MPQ优先级系统

**优先级规则：**
- 数字越大，优先级越高
- 查找文件时，从高优先级到低优先级
- 高优先级文件覆盖低优先级文件

**应用场景：**
```
基础游戏：  DIABDAT.MPQ   (priority: 1000)
扩展包：    HELLFIRE.MPQ  (priority: 2000)
Mod文件：   custom.mpq    (priority: 3000)
```

### 3. 调色板格式

**Diablo调色板文件格式（.PAL）：**
```
文件大小: 768字节
格式: 256个颜色 × 3字节（RGB）

偏移    大小    描述
0       3       Color 0 (通常为透明色)
3       3       Color 1
6       3       Color 2
...
765     3       Color 255
```

**调色板布局：**
```
索引 0-127:   关卡特定颜色（不同关卡使用不同调色板）
索引 128-255: 全局颜色（蓝色、红色、黄色等，所有关卡共享）
```

### 4. 透明色处理

**索引0的特殊处理：**
- **精灵图像**：索引0通常是透明（alpha=0）
- **UI图像**：索引0可能不是透明（如logo.pcx）
- **地图瓦片**：索引0可能是黑色，不是透明

**API设计：**
```rust
// 支持可配置的透明色行为
pub fn to_rgba(&self, index: u8, transparent: bool) -> [u8; 4] {
    let color = self.colors[index as usize];
    let alpha = if transparent && index == 0 { 0 } else { 255 };
    [color.r, color.g, color.b, alpha]
}
```

## 🔄 数据流

```
MPQ文件 (DIABDAT.MPQ)
    ↓
MpqManager::load_mpq()
    ↓
mpq::Archive::open()  ← mpq-rust库
    ↓
MpqManager::find_file("levels/towndata/town.pal")
    ↓
mpq::Archive::open_file()  ← mpq-rust库
    ↓
mpq::File::read()  ← 自动解密和解压缩
    ↓
Vec<u8> (768字节)
    ↓
Palette::from_bytes()
    ↓
Palette (256色RGB)
    ↓
indices_to_rgba()
    ↓
RGBA像素数据
```

## 📚 参考代码

### 原版代码参考

**MPQ系统：**
- `Source/mpq/mpq_reader.cpp` - MPQ读取实现
- `Source/mpq/mpq_reader.hpp` - MPQ接口定义
- `Source/engine/assets.cpp` - MPQ管理和优先级系统

**调色板系统：**
- `Source/engine/palette.cpp` - 调色板系统
- `Source/engine/palette.h` - 调色板接口定义

### 改造思路

**MPQ系统改造：**

| 原版 C++ | Rust 实现 | 说明 |
|---------|----------|------|
| `libmpq` 库 | `mpq-rust` 库 | 使用Rust生态系统的库 |
| 错误码（`int32_t error`） | `Result<T, E>` | Rust惯用错误处理 |
| 指针和手动内存管理 | 所有权和借用 | 内存安全 |
| C字符串（`char*`） | `String`/`&str` | Rust字符串 |

**调色板系统改造：**

| 原版 C++ | Rust 实现 | 说明 |
|---------|----------|------|
| `SDL_Color` | `Color` 结构体 | 自定义类型 |
| `SDL_Color*` 数组 | `[Color; 256]` | 固定大小数组 |
| 指针传递 | 值传递/借用 | Rust所有权 |

## 🧪 测试策略

### 单元测试

**MPQ测试：**
- ✅ `test_mpq_manager_new` - MPQ管理器创建
- ✅ `test_mpq_search_paths` - 路径搜索
- ✅ `test_mpq_find_file_nonexistent` - 文件不存在
- ✅ `test_mpq_priority_ordering` - 优先级排序
- ✅ `test_mpq_load_and_find_file` - 加载和读取文件
- ✅ `test_mpq_error_handling` - 错误处理

**调色板测试：**
- ✅ `test_palette_from_bytes` - 调色板加载
- ✅ `test_palette_invalid_size` - 错误处理
- ✅ `test_palette_to_rgb` - RGB转换
- ✅ `test_palette_to_rgba_transparent` - RGBA透明转换
- ✅ `test_palette_indices_to_rgba` - 批量转换
- ✅ `test_palette_indices_to_rgba_transparent` - 批量透明转换

### 集成测试

**真实MPQ测试：**
```rust
#[test]
fn test_mpq_load_and_find_file() {
    let mut mgr = MpqManager::new();
    
    // 尝试加载真实的 Diabdat.mpq
    if mgr.load_mpq("assets/Diabdat.mpq", 1000).is_ok() {
        // 测试读取调色板文件
        if let Some(data) = mgr.find_file("levels/towndata/town.pal") {
            assert_eq!(data.len(), 768);  // 验证文件大小
            // 验证调色板数据
            let palette = Palette::from_bytes(&data).unwrap();
            // ...
        }
    }
}
```

## ⚠️ 潜在问题和解决方案

### 问题1: MPQ文件路径

**问题：** 如何找到DIABDAT.MPQ文件？

**解决：**
- 支持多个搜索路径
- 大小写不敏感搜索（Windows）
- 清晰的错误消息

### 问题2: 依赖外部库

**问题：** 依赖mpq-rust可能有风险？

**解决：**
- mpq-rust已经过验证（15 stars, 5 forks）
- MIT/Apache-2.0许可证（商业友好）
- 如果未来有问题，可以fork并维护

### 问题3: API差异

**问题：** mpq-rust的API可能与原版不同？

**解决：**
- 封装在`MpqManager`中，对外提供统一接口
- 保持与原版相似的API设计
- 隐藏底层实现细节

## 📊 代码统计

### 代码行数对比

| 方案 | 代码行数 | 说明 |
|------|---------|------|
| **原计划：手动实现** |
| `mpq_format.rs` | ~310行 | 格式定义、crypt table、hash算法 |
| `mpq.rs` | ~830行 | 归档读取、解密、解压缩 |
| **小计** | **~1140行** |
| **最终方案：mpq-rust** |
| `mpq.rs` | ~350行 | MpqManager封装 |
| **节省** | **~790行（69%）** |

### 依赖对比

| 方案 | 依赖 |
|------|------|
| **原计划** | `once_cell`（crypt table）、`pklib`（解压缩） |
| **最终方案** | `mpq`（一个依赖解决所有问题） |

## 🎓 学习要点

### 1. 为什么使用第三方库？

**工程实践原则：**
- ✅ **不要重复造轮子** - 除非有特殊需求
- ✅ **优先使用成熟方案** - 减少bug和维护成本
- ✅ **专注核心价值** - 游戏逻辑 > 底层格式

**何时手动实现？**
- 没有可用的库
- 现有库不满足需求
- 学习和理解为主要目标

### 2. MPQ格式的历史意义

**MPQ (Mike O'Brien Pack) 格式：**
- 1997年由暴雪娱乐开发
- 用于Diablo、StarCraft、Warcraft II等游戏
- 支持压缩、加密、补丁系统

**技术特点：**
- 哈希表快速查找
- 块压缩（PKWare/zlib/bzip2）
- 加密保护
- 支持文件覆盖（Mod系统）

### 3. Rust生态系统的力量

**Rust crates优势：**
- 大量高质量库
- cargo生态系统
- 类型安全和内存安全
- 良好的文档

**本项目使用的crates：**
- `sdl2` - 渲染和输入
- `anyhow` - 错误处理
- `mpq` - MPQ读取

## 🔜 下一步

完成Step 5.1后：
- **Step 5.2**: PCX图像格式和CLX精灵格式
- 可以开始使用MPQ和调色板加载原版资源

## 📖 相关文档

- [Step 5总体规划](../step-5/step-5-resource-formats.md)
- [Step 5.1实现总结](step-5.1-implementation-summary.md)
- [技术要点：资源格式](../tech_key_points/step5-resource-formats.md)

---

**文档版本：** 1.0  
**最后更新：** 2025年1月20日  
**作者：** Claude (AI Assistant)



