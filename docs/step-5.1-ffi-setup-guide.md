# libmpq FFI 方案设置指南

## 📋 前置要求

使用 FFI 方案需要：
1. C 编译器（MSVC/GCC/Clang）
2. zlib 库
3. bzip2 库

---

## 🚀 Windows 设置（使用 vcpkg）

### 1. 安装 vcpkg

```powershell
# 克隆 vcpkg
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg

# 初始化
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# 集成到系统
.\vcpkg integrate install

# 设置环境变量
[System.Environment]::SetEnvironmentVariable('VCPKG_ROOT', 'C:\vcpkg', 'User')
```

### 2. 安装依赖

#### 方法 A: 使用脚本（推荐）

```powershell
cd rust-diablo
.\setup-dependencies.ps1
```

#### 方法 B: 手动安装

```powershell
vcpkg install zlib:x64-windows
vcpkg install bzip2:x64-windows
```

### 3. 编译项目

```powershell
cargo build
```

---

## 🐧 Linux 设置

### Ubuntu/Debian

```bash
# 安装依赖
sudo apt-get install zlib1g-dev libbz2-dev

# 编译
cd rust-diablo
cargo build
```

### Fedora/RHEL

```bash
# 安装依赖
sudo dnf install zlib-devel bzip2-devel

# 编译
cd rust-diablo
cargo build
```

---

## 🍎 macOS 设置

```bash
# 安装 Homebrew（如果还没有）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装依赖
brew install zlib bzip2

# 编译
cd rust-diablo
cargo build
```

---

## ✅ 验证安装

编译成功后，运行测试：

```powershell
cargo test test_libmpq -- --nocapture
```

预期输出：
```
test test_libmpq_version ... ok
test test_read_town_palette_with_libmpq ... ok
✓ Read town.pal: 768 bytes
✓ Parsed palette: 256 colors
```

---

## 🐛 故障排除

### 问题 1: vcpkg 未找到

**错误**: `vcpkg not found`

**解决**:
1. 确认 vcpkg 已安装
2. 设置环境变量: `$env:VCPKG_ROOT = "C:\vcpkg"`
3. 重启终端

### 问题 2: zlib 链接错误

**错误**: `cannot find -lz`

**解决**:
```powershell
# 确认已安装
vcpkg list | Select-String "zlib"

# 重新安装
vcpkg install zlib:x64-windows --recurse
```

### 问题 3: 编译器未找到

**错误**: `cl.exe not found`

**解决**:
1. 安装 Visual Studio 2019+ with C++ workload
2. 使用 Developer Command Prompt
3. 或运行: `vcvarsall.bat x64`

---

## 📚 参考

- [vcpkg 文档](https://github.com/Microsoft/vcpkg)
- [Rust cc crate](https://docs.rs/cc/)
- [libmpq 源码](https://github.com/diasurgical/libmpq)

---

**更新日期**: 2025-11-24






















