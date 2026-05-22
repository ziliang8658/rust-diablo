# libmpq FFI 封装 - 快速开始指南

## 📋 目标

使用 Rust FFI 封装 libmpq C 库，快速解决 Huffman 压缩的 palette 文件解析问题。

---

## 🚀 快速开始

### 1. 编译 libmpq (Windows)

```powershell
# 进入 libmpq 目录
cd 3rdParty/libmpq/build

# 使用 CMake 生成项目
cmake .. -G "Visual Studio 16 2019" -A x64

# 编译 Release 版本
cmake --build . --config Release
```

生成文件位置：`3rdParty/libmpq/build/Release/mpq.lib`

### 2. 配置 Rust 链接

编辑 `rust-diablo/.cargo/config.toml`：

```toml
[target.x86_64-pc-windows-msvc]
rustflags = [
    "-L", "G:/DevilutionX/3rdParty/libmpq/build/Release",
]
```

**注意**：根据你的实际路径调整。

### 3. 编译和测试

```powershell
cd rust-diablo

# 编译
cargo build

# 运行测试
cargo test test_libmpq
```

---

## 📝 代码实现

### 核心文件结构

```
rust-diablo/src/resources/
├── mpq_libmpq_sys.rs   # FFI 原始绑定 (~280 行)
├── mpq_libmpq.rs       # 安全封装层 (~330 行)
└── mod.rs              # 模块导出

rust-diablo/tests/
└── test_libmpq_palette.rs  # 集成测试 (~330 行)
```

### 使用示例

```rust
use rust_diablo::resources::mpq_libmpq::MpqArchiveFFI;
use rust_diablo::resources::palette::Palette;

// 打开 MPQ
let mut mpq = MpqArchiveFFI::open("assets/Diabdat.mpq")?;

// 读取 palette（包括 Huffman 压缩的）
let data = mpq.read_file("levels/towndata/town.pal")?;

// 解析
let palette = Palette::from_bytes(&data)?;
```

---

## ✅ 验证

运行测试验证是否成功：

```powershell
cargo test test_read_town_palette_with_libmpq -- --nocapture
```

预期输出：
```
✓ Read town.pal: 768 bytes
✓ Parsed palette: 256 colors
✓ Test passed
```

---

## 🎓 技术要点

### FFI 安全封装模式

```rust
pub struct MpqArchiveFFI {
    handle: *mut ffi::MpqArchive,  // C 指针
}

impl Drop for MpqArchiveFFI {
    fn drop(&mut self) {
        // 自动清理 C 资源
        unsafe {
            ffi::libmpq__archive_close(self.handle);
        }
    }
}
```

### 字符串转换

```rust
// Rust → C
let cstr = CString::new(filename)?;
unsafe { ffi::func(cstr.as_ptr()) }

// C → Rust
let rust_str = unsafe {
    CStr::from_ptr(c_ptr).to_string_lossy().into_owned()
};
```

---

## 📚 相关文档

- [step-5.1-CURRENT-STATUS.md](step-5.1-CURRENT-STATUS.md) - 整体进度
- [MASTER_PLAN.md](./MASTER_PLAN.md) - 项目规划

---

**最后更新**: 2025-11-24  
**状态**: ✅ 可用  
**难度**: ⭐⭐⭐























