# libmpq FFI 使用指南

## 📋 前提条件

你需要先编译 libmpq 库。

### Windows (MSVC)

```bash
cd ../3rdParty/libmpq
mkdir build
cd build
cmake .. -G "Visual Studio 17 2022" -A x64
cmake --build . --config Release
```

### Windows (MinGW)

```bash
cd ../3rdParty/libmpq
mkdir build
cd build
cmake .. -G "MinGW Makefiles" -DCMAKE_BUILD_TYPE=Release
cmake --build .
```

### Linux/macOS

```bash
cd ../3rdParty/libmpq
mkdir build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make
```

## 🚀 使用方法

### 1. 编译 libmpq

确保 `libmpq.lib` (Windows) 或 `libmpq.a` (Linux/macOS) 在 `3rdParty/libmpq/build` 目录中。

### 2. 运行提取工具

```bash
cd rust-diablo
cargo run --example extract_with_libmpq --features use-libmpq
```

### 3. 在代码中使用

```rust
use rust_diablo::resources::LibMpqArchive;

fn main() {
    // 打开 MPQ
    let archive = LibMpqArchive::open("assets/Diabdat.mpq")
        .expect("Failed to open MPQ");
    
    // 检查文件
    if archive.has_file("levels/towndata/town.pal") {
        // 读取文件
        let data = archive.read_file("levels/towndata/town.pal")
            .expect("Failed to read file");
        
        println!("Read {} bytes", data.len());
    }
}
```

## 📦 提取的文件

运行提取工具后，文件会保存在 `assets/extracted/` 目录中：

- `town.pal` - 城镇调色板 (768 字节)
- `l1.pal` - 地牢 1 层调色板
- `l2.pal` - 地牢 2 层调色板
- `l3.pal` - 地牢 3 层调色板
- `l4.pal` - 地牢 4 层调色板
- 等等...

## 🔧 故障排除

### 错误: `libmpq support not enabled`

确保使用 `--features use-libmpq` 标志：

```bash
cargo run --example extract_with_libmpq --features use-libmpq
```

### 错误: `Failed to open MPQ archive`

- 检查 `assets/Diabdat.mpq` 是否存在
- 检查 libmpq 是否正确编译
- 检查 `build.rs` 中的路径设置

### 链接错误

如果遇到链接错误，可能需要：

1. **Windows**: 安装 vcpkg 并安装 zlib 和 bzip2
   ```bash
   vcpkg install zlib:x64-windows bzip2:x64-windows
   ```

2. **Linux**: 安装开发库
   ```bash
   sudo apt-get install zlib1g-dev libbz2-dev
   ```

3. **macOS**: 使用 Homebrew
   ```bash
   brew install zlib bzip2
   ```

## 💡 优点

使用 libmpq FFI 的优点：

1. ✅ **完整支持**: 支持所有 MPQ 格式和压缩类型
2. ✅ **经过验证**: libmpq 已被广泛使用和测试
3. ✅ **快速开发**: 无需重新实现复杂的算法
4. ✅ **可靠性**: Diablo 1/2、StarCraft 等都使用这个库

## 🔄 从 FFI 迁移到纯 Rust

你可以：
1. 先用 libmpq FFI 快速推进项目
2. 后续逐步用纯 Rust 实现替换
3. 保持两种实现并存，通过 feature 切换

```toml
[features]
default = ["use-libmpq"]  # 默认使用 libmpq
pure-rust = []            # 纯 Rust 实现
```


























