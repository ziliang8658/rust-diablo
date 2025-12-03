# libmpq FFI 方案 - 完整设置指南

## 📋 当前状态

✅ FFI 代码已完成  
✅ 配置文件已准备  
⏸️ 需要安装依赖

---

## 🚀 快速开始（3步完成）

### 步骤 1: 安装 vcpkg

```powershell
# 在任意位置克隆 vcpkg（推荐 C:\vcpkg）
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg

# 进入目录
cd C:\vcpkg

# 初始化 vcpkg
.\bootstrap-vcpkg.bat

# 集成到系统（这样 cargo 可以自动找到库）
.\vcpkg integrate install

# 设置环境变量
[System.Environment]::SetEnvironmentVariable('VCPKG_ROOT', 'C:\vcpkg', 'User')
```

**注意**: 设置环境变量后需要**重启终端**。

### 步骤 2: 安装依赖库

```powershell
# 回到 rust-diablo 目录
cd G:\DevilutionX\rust-diablo

# 运行安装脚本
.\setup-dependencies.ps1
```

或手动安装：

```powershell
C:\vcpkg\vcpkg install zlib:x64-windows
C:\vcpkg\vcpkg install bzip2:x64-windows
```

### 步骤 3: 编译测试

```powershell
# 清理并重新编译
cargo clean
cargo build

# 运行测试
cargo test test_libmpq -- --nocapture
```

---

## ✅ 预期结果

编译成功后，你应该看到：

```
warning: rust-diablo@0.1.0: Building libmpq from "../3rdParty/libmpq/libmpq"
warning: rust-diablo@0.1.0: Using vcpkg at: "C:\\vcpkg"
   Compiling rust-diablo v0.1.0
    Finished dev [unoptimized + debuginfo] target(s)
```

运行测试：

```
test test_libmpq_version ... ok
test test_read_town_palette_with_libmpq ... ok
  ✓ Read town.pal: 768 bytes
  ✓ Parsed palette: 256 colors

test result: ok. 7 passed; 0 failed
```

---

## 📝 已创建的文件

### 核心代码
- ✅ `src/resources/mpq_libmpq_sys.rs` - FFI 绑定层
- ✅ `src/resources/mpq_libmpq.rs` - 安全封装层
- ✅ `tests/test_libmpq_palette.rs` - 测试代码

### 配置文件
- ✅ `build.rs` - 编译配置
- ✅ `vcpkg.json` - vcpkg 依赖声明
- ✅ `.cargo/config.toml` - Cargo 配置
- ✅ `Cargo.toml` - 项目依赖

### 脚本
- ✅ `setup-dependencies.ps1` - 依赖安装脚本
- ✅ `build-libmpq-en.ps1` - libmpq 编译脚本

### 文档
- ✅ `docs/step-5.1-ffi-setup-guide.md` - 设置指南
- ✅ `docs/step-5.1-libmpq-ffi-quickstart.md` - 快速开始
- ✅ `SETUP_FFI.md` - 本文档

---

## 🔧 故障排除

### Q1: vcpkg 下载很慢

**A**: 可以使用国内镜像：

```powershell
# 设置镜像（清华大学）
$env:VCPKG_DOWNLOADS="https://mirrors.tuna.tsinghua.edu.cn/vcpkg-downloads"
```

### Q2: 没有安装 Git

**A**: 下载安装 Git for Windows:
https://git-scm.com/download/win

### Q3: 没有安装 Visual Studio

**A**: 需要安装 Visual Studio 2019+ 或 Build Tools:
https://visualstudio.microsoft.com/downloads/

确保选择 "Desktop development with C++" 工作负载。

### Q4: 编译时找不到 zlib

**A**: 
```powershell
# 验证安装
C:\vcpkg\vcpkg list | Select-String "zlib"

# 重新安装
C:\vcpkg\vcpkg install zlib:x64-windows --recurse

# 确认环境变量
echo $env:VCPKG_ROOT
```

---

## 🎯 完成后的功能

启用 FFI 后，你将可以：

- ✅ 读取所有 MPQ 文件（包括 Huffman 压缩的）
- ✅ 加载所有 palette 文件
- ✅ 完整的资源加载支持

---

## 📚 参考链接

- [vcpkg GitHub](https://github.com/Microsoft/vcpkg)
- [vcpkg 中文文档](https://github.com/microsoft/vcpkg/blob/master/README_zh_CN.md)
- [Rust FFI 文档](https://doc.rust-lang.org/nominom/ffi.html)

---

## 💡 如果遇到问题

如果设置过程中遇到任何问题，你也可以：

1. **继续使用方案A**（当前实现）
   - 已经支持大部分功能
   - 纯 Rust，无需外部依赖

2. **完成 Huffman 实现**
   - 作为学习项目
   - 框架已经准备好

选择最适合你的方案即可！

---

**更新时间**: 2025-11-24  
**预计设置时间**: 15-30分钟（取决于网络速度）






















