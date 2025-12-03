# Palette 测试完成总结

## 测试概述

使用 libmpq FFI 成功实现了从 MPQ 文件读取和测试 palette 的功能。

## 测试结果

### ✅ 所有测试通过

1. **test_read_town_pal_with_libmpq_ffi** - 成功读取并解析 town.pal
2. **test_read_multiple_palettes** - 成功读取 4 个 palette 文件
3. **test_palette_color_ranges** - 成功分析颜色范围

### 测试详情

#### 1. 读取 town.pal

- ✅ 成功打开 MPQ 存档
- ✅ 找到文件（使用 Windows 路径格式：`levels\towndata\town.pal`）
- ✅ 文件大小：768 字节（正确）
- ✅ 成功读取文件数据
- ✅ 成功解析为 256 色 Palette

**颜色示例**：
- 索引 0: RGB(0, 0, 0) - 黑色（透明色）
- 索引 1: RGB(181, 93, 27) - 棕色
- 索引 128: RGB(159, 159, 255) - 浅蓝色
- 索引 255: RGB(255, 255, 255) - 白色

#### 2. 读取多个 Palette 文件

成功读取了 4 个 palette 文件：
- ✅ `levels\towndata\town.pal` (城镇)
- ✅ `levels\l1data\l1.pal` (地牢1层)
- ✅ `levels\l2data\l2.pal` (地牢2层)
- ✅ `levels\l3data\l3.pal` (地牢3层)
- ❌ `levels\l4data\l4.pal` (地牢4层) - 未找到（可能不在 Diabdat.mpq 中）

#### 3. 颜色范围分析

- ✅ R 通道范围：0 - 255
- ✅ G 通道范围：0 - 255
- ✅ B 通道范围：0 - 255
- ⚠️  发现 1 组重复颜色：RGB(0, 0, 0) 出现在索引 [0, 127]

## 关键发现

### 1. 路径格式

MPQ 文件中的路径使用 Windows 格式（反斜杠 `\`）：
- ✅ 正确：`levels\towndata\town.pal`
- ❌ 错误：`levels/towndata/town.pal`

### 2. Palette 文件格式

- **大小**：768 字节（256 颜色 × 3 字节 RGB）
- **格式**：未压缩的原始 RGB 数据
- **结构**：每个颜色 3 字节，按顺序存储（R, G, B）

### 3. 颜色索引特性

- **索引 0**：黑色，通常用作透明色
- **索引 0-127**：层级特定颜色（不同层级有不同的 palette）
- **索引 128-255**：全局颜色（所有层级共享）

### 4. 透明度处理

- 索引 0 在透明模式下 alpha = 0
- 索引 0 在不透明模式下 alpha = 255
- 其他索引始终 alpha = 255

## 测试代码位置

- **测试文件**：`rust-diablo/tests/test_palette_libmpq_ffi.rs`
- **Palette 实现**：`rust-diablo/src/resources/palette.rs`
- **FFI 绑定**：`rust-diablo/src/resources/libmpq_ffi.rs`

## 测试命令

```powershell
# 运行所有 palette FFI 测试
cargo test --features use-libmpq --test test_palette_libmpq_ffi -- --nocapture

# 运行单个测试
cargo test test_read_town_pal_with_libmpq_ffi --features use-libmpq --test test_palette_libmpq_ffi -- --nocapture
```

## 功能验证

### ✅ 已验证的功能

1. **MPQ 文件读取**
   - 打开 MPQ 存档
   - 查找文件（支持路径格式自动检测）
   - 获取文件大小
   - 读取文件数据

2. **Palette 解析**
   - 从字节数组解析 palette
   - 验证文件大小（768 字节）
   - 提取 256 种颜色

3. **颜色操作**
   - 索引到 RGB 转换
   - 索引到 RGBA 转换（支持透明度）
   - 批量索引到 RGBA 转换

4. **颜色分析**
   - 颜色范围统计
   - 重复颜色检测

## 下一步计划

1. **集成到游戏引擎**
   - 在渲染系统中使用 palette
   - 实现索引图像到 RGBA 的转换
   - 支持不同层级的 palette 切换

2. **性能优化**
   - 缓存解析后的 palette
   - 优化批量转换性能
   - 考虑使用 SIMD 加速

3. **功能扩展**
   - 支持 palette 动画（颜色循环）
   - 支持 palette 混合
   - 支持自定义 palette

## 技术要点

### 1. FFI 使用

```rust
// 打开 MPQ 存档
let result = libmpq__archive_open(
    &mut archive as *mut mpq_archive,
    c_path.as_ptr(),
    0, // archive_offset
);

// 查找文件
let result = libmpq__file_number(
    archive,
    c_filename.as_ptr(),
    &mut file_number,
);

// 读取文件
let result = libmpq__file_read(
    archive,
    file_number,
    buffer.as_mut_ptr(),
    unpacked_size,
    &mut transferred,
);
```

### 2. 路径格式处理

```rust
// 尝试两种路径格式
let filenames = vec![
    "levels\\towndata\\town.pal",  // Windows 格式（反斜杠）
    "levels/towndata/town.pal",    // Unix 格式（正斜杠）
];
```

### 3. Palette 解析

```rust
// 从字节数组解析
let palette = Palette::from_bytes(&buffer)?;

// 颜色转换
let color = palette.to_rgb(index);
let rgba = palette.to_rgba(index, transparent);
let rgba_data = palette.indices_to_rgba(&indices, transparent);
```

## 学习要点

1. **MPQ 文件格式**：
   - 路径使用 Windows 格式（反斜杠）
   - 文件可能压缩，也可能未压缩
   - 需要使用正确的路径格式才能找到文件

2. **Palette 格式**：
   - 固定大小：768 字节
   - 简单格式：RGB 三元组
   - 索引 0 的特殊用途（透明色）

3. **FFI 安全**：
   - 使用 `unsafe` 块调用 C 函数
   - 正确处理字符串转换（`CString`）
   - 确保资源正确释放（`libmpq__archive_close`）

4. **测试策略**：
   - 测试多种路径格式
   - 测试多个文件
   - 测试边界情况
   - 测试功能完整性

## 相关文档

- [Bug 修复：DLL 未找到](./bug-fix-dll-not-found.md)
- [Bug 修复：town.pal 路径格式](./bug-fix-town-pal-path-format.md)
- [libmpq FFI 快速开始](./step-5.1-libmpq-ffi-quickstart.md)

---

**完成日期**: 2025-01-XX  
**状态**: ✅ 完成  
**测试覆盖**: 3 个测试，全部通过


















