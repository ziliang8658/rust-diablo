# Bug 修复：town.pal 文件路径格式问题

## 问题描述

测试代码无法找到 `town.pal` 文件，即使文件确实存在于 MPQ 存档中。

## 问题原因

**MPQ 文件中的路径使用反斜杠（Windows 格式），而不是正斜杠（Unix 格式）**

### 详细分析

1. **C++ 代码中的路径格式**：
   ```cpp
   // Source/engine/palette.cpp:208
   LoadPaletteAndInitBlending("levels\\towndata\\town.pal");
   ```
   C++ 代码使用反斜杠 `\\`（Windows 格式）

2. **MPQ 文件内部存储格式**：
   - MPQ 文件中的路径使用反斜杠 `\`（Windows 格式）
   - 这是 Diablo 1 原始游戏在 Windows 上创建的，所以路径格式是 Windows 风格

3. **测试代码使用的路径**：
   ```rust
   // 错误的路径格式（正斜杠）
   let filename = "levels/towndata/town.pal";
   ```
   使用正斜杠 `/`（Unix 格式）导致哈希不匹配，无法找到文件

4. **libmpq 的哈希计算**：
   - `libmpq__file_number` 使用 `libmpq__file_hash_s` 计算文件路径的哈希值
   - 哈希值对路径格式敏感，`levels\towndata\town.pal` 和 `levels/towndata/town.pal` 会产生不同的哈希值
   - 只有与 MPQ 文件中存储的路径格式完全匹配，才能找到文件

## 解决方案

### 方案 1：使用 Windows 格式路径（推荐）

直接使用反斜杠路径：

```rust
let filename = "levels\\towndata\\town.pal";
```

### 方案 2：尝试两种格式（兼容性更好）

在测试代码中尝试两种路径格式：

```rust
let filenames = vec![
    "levels\\towndata\\town.pal",  // Windows 格式（反斜杠）
    "levels/towndata/town.pal",     // Unix 格式（正斜杠）
];

let mut filename = None;
for test_filename in &filenames {
    let c_test_filename = CString::new(*test_filename).unwrap();
    let mut test_file_number: u32 = 0;
    
    let result = libmpq__file_number(
        archive,
        c_test_filename.as_ptr(),
        &mut test_file_number,
    );
    
    if is_success(result) {
        filename = Some(*test_filename);
        println!("✔ 找到文件，使用路径: {}", test_filename);
        break;
    }
}
```

## 验证结果

修复后，测试输出：

```
✔ 找到文件，使用路径: levels\towndata\town.pal
✔ 文件大小: 768 字节
✔ 成功读取文件: 768 字节
✔ Palette 文件大小正确: 768 字节
test tests::test_read_file ... ok
```

## 关键发现

1. **MPQ 文件路径格式**：
   - Diablo 1 的 MPQ 文件使用 Windows 路径格式（反斜杠 `\`）
   - 这是历史原因，因为游戏最初在 Windows 上开发

2. **路径格式对哈希的影响**：
   - `libmpq__file_hash_s` 函数对路径字符串进行哈希计算
   - 不同的路径分隔符会产生不同的哈希值
   - 必须与 MPQ 文件中存储的路径格式完全匹配

3. **C++ 代码的处理**：
   - 在 Windows 上，`FindAsset` 函数保持路径原样（不替换反斜杠）
   - 在非 Windows 系统上，会将反斜杠替换为正斜杠
   - 但 MPQ 文件内部存储的路径仍然是反斜杠格式

## 相关代码位置

### C++ 代码
- `Source/engine/palette.cpp:208` - 加载 town.pal
- `Source/engine/assets.cpp:127-179` - FindAsset 函数
- `Source/engine/assets.cpp:68-80` - FindMpqFile 函数
- `Source/mpq/mpq_common.cpp:10-15` - CalculateMpqFileHash 函数

### Rust 代码
- `rust-diablo/tests/test_libmpq_ffi.rs` - 测试代码（已修复）
- `rust-diablo/src/resources/libmpq_ffi.rs` - FFI 绑定

## 学习要点

1. **路径格式的重要性**：
   - 在处理文件系统路径时，需要注意不同操作系统的路径格式
   - 在处理历史遗留格式时，需要了解原始格式

2. **哈希函数的特性**：
   - 哈希函数对输入字符串敏感
   - 即使是微小的差异（如路径分隔符）也会产生完全不同的哈希值

3. **跨平台兼容性**：
   - 在 Rust 代码中，可以使用 `std::path::Path` 来处理路径
   - 但在与 C 库交互时，需要确保路径格式匹配

4. **调试技巧**：
   - 当文件查找失败时，尝试不同的路径格式
   - 检查原始代码使用的路径格式
   - 使用测试代码验证不同的路径格式

## 建议

1. **统一路径处理**：
   - 在 Rust 代码中，可以创建一个辅助函数来规范化路径格式
   - 或者提供一个函数来尝试多种路径格式

2. **文档化**：
   - 在 FFI 绑定文档中说明路径格式要求
   - 在测试代码中添加注释说明路径格式

3. **错误处理**：
   - 当文件查找失败时，提供更详细的错误信息
   - 建议尝试的路径格式

---

**修复日期**: 2025-01-XX  
**状态**: ✅ 已修复  
**影响范围**: libmpq FFI 测试，town.pal 文件读取


















