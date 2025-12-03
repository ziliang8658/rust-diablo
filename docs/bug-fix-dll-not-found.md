# Bug 修复：测试运行时 DLL 未找到错误

## 问题描述

运行 libmpq FFI 测试时出现错误：

```
error: test failed, to rerun pass `--lib`
Caused by:
  process didn't exit successfully: `...\rust_diablo-d96bd642c8ed7c74.exe test_libmpq_ffi --nocapture` (exit code: 0xc0000135, STATUS_DLL_NOT_FOUND)
```

## 问题原因

1. **动态库依赖**：虽然 `build.rs` 尝试静态链接 zlib 和 bzip2，但 vcpkg 提供的是动态库（DLL）
2. **运行时路径**：测试可执行文件在 `target\debug\deps` 目录中，但 DLL 文件在 `vcpkg_installed\x64-windows\bin` 目录中
3. **Windows DLL 搜索路径**：Windows 在以下位置搜索 DLL：
   - 可执行文件所在目录
   - 系统目录
   - PATH 环境变量中的目录

## 解决方案

### 方案 1：复制 DLL 到测试目录（推荐）

在运行测试前，将 DLL 复制到测试可执行文件目录：

```powershell
Copy-Item "vcpkg_installed\x64-windows\bin\zlib1.dll" -Destination "target\debug\deps\zlib1.dll" -Force
Copy-Item "vcpkg_installed\x64-windows\bin\bz2.dll" -Destination "target\debug\deps\bz2.dll" -Force
```

### 方案 2：使用测试脚本

使用提供的测试脚本 `test-libmpq-ffi.ps1`，它会自动处理 DLL 复制：

```powershell
.\test-libmpq-ffi.ps1
```

### 方案 3：设置 PATH 环境变量

在运行测试时临时设置 PATH：

```powershell
$env:PATH = "G:\DevilutionX\rust-diablo\vcpkg_installed\x64-windows\bin;" + $env:PATH
cargo test --features use-libmpq --test test_libmpq_ffi
```

## 其他修复

在修复 DLL 问题的过程中，还修复了测试代码中的以下问题：

1. **API 不匹配**：测试代码使用了旧的 API（`libmpq__file_open`, `libmpq__file_close`），但 FFI 绑定已更新为使用文件编号直接读取
2. **类型错误**：`LIBMPQ_OPEN_EXISTING` 是 `i32`，但 `libmpq__archive_open` 需要 `i64` 类型的 `archive_offset`

### 修复后的 API 使用

```rust
// 旧 API（已废弃）
let mut file_handle: mpq_file = ptr::null_mut();
libmpq__file_open(archive, file_number, &mut file_handle);
libmpq__file_unpacked_size(file_handle, &mut size);
libmpq__file_read(file_handle, archive, buffer, size, &mut transferred);
libmpq__file_close(file_handle);

// 新 API（当前使用）
let mut file_number: u32 = 0;
libmpq__file_number(archive, filename, &mut file_number);
libmpq__file_size_unpacked(archive, file_number, &mut size);
libmpq__file_read(archive, file_number, buffer, size, &mut transferred);
// 不需要关闭文件句柄
```

## 验证

修复后，所有测试应该能够正常运行：

```powershell
cargo test --features use-libmpq --test test_libmpq_ffi -- --nocapture
```

预期输出：
```
running 3 tests
test tests::test_file_check ... ok
test tests::test_open_close_archive ... ok
test tests::test_read_file ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured
```

## 相关文件

- `rust-diablo/tests/test_libmpq_ffi.rs` - 测试代码（已修复）
- `rust-diablo/src/resources/libmpq_ffi.rs` - FFI 绑定定义
- `rust-diablo/test-libmpq-ffi.ps1` - 测试运行脚本
- `rust-diablo/build.rs` - 构建配置

## 学习要点

1. **Windows DLL 加载机制**：了解 Windows 如何搜索和加载 DLL
2. **静态 vs 动态链接**：理解静态链接和动态链接的区别，以及各自的优缺点
3. **FFI 兼容性**：当 C 库 API 更新时，需要同步更新 Rust FFI 绑定和测试代码
4. **测试环境配置**：测试可能需要额外的运行时环境配置（如 DLL 路径）

## 参考

- [Windows DLL 搜索顺序](https://docs.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order)
- [Rust FFI 指南](https://doc.rust-lang.org/nomicon/ffi.html)
- [vcpkg 文档](https://vcpkg.io/en/index.html)

---

**修复日期**: 2025-01-XX  
**状态**: ✅ 已修复  
**影响范围**: libmpq FFI 测试


















