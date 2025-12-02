# Rust Diablo

用 Rust 重写的 Diablo 1 游戏引擎 - 学习型项目

## 📋 项目概述

这是一个渐进式的 Rust 学习项目，目标是完整复刻 Diablo 1 游戏的所有特性。项目采用逐步实现的方式，每个步骤包含详细的设计文档、实现代码和学习总结。

## 🚀 快速开始

### 前置要求

- Rust 1.70+
- SDL2 开发库
- Diablo 1 游戏资源文件 (Diabdat.mpq)

### 编译和运行

```powershell
cd rust-diablo

# 编译
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example basic_window
```

### 配置资源文件

将 `Diabdat.mpq` 放到 `assets/` 目录：

```
rust-diablo/
└── assets/
    └── Diabdat.mpq
```

## 📂 项目结构

```
rust-diablo/
├── src/
│   ├── game.rs              # 游戏主循环
│   ├── window.rs            # 窗口管理
│   ├── input.rs             # 输入处理
│   └── resources/           # 资源管理
│       ├── mpq.rs           # MPQ 归档（原生实现）
│       ├── mpq_libmpq.rs    # MPQ 归档（FFI 封装，备用）
│       ├── palette.rs       # 调色板系统
│       ├── pkware.rs        # PKWare 解压
│       ├── huffman.rs       # Huffman 解压（进行中）
│       └── compression.rs   # 多重压缩处理
├── tests/                   # 集成测试
├── examples/                # 示例代码
├── assets/                  # 游戏资源
└── docs/                    # 开发文档
    ├── MASTER_PLAN.md       # 总体规划
    └── step-*.md            # 各步骤文档
```

## 📖 开发文档

### 总体规划
- [总体规划](docs/master_plan.md) - 项目长期规划和路线图
- [功能对照表](docs/feature-comparison.md) - 原版vs Rust实现对照 **NEW**

### Step 4: 动画和碰撞系统
- [实现文档](docs/step4-animation-collision.md) - Step 4 完整实现

### Step 5: 资源格式支持
- [Step 5.1 总结](docs/step-palette-libmpq-ffi-complete-summary.md) - MPQ和Palette系统
- [Step 5.2 实现文档](docs/step-5.2-pcx-clx-implementation.md) - PCX/CLX/CL2格式实现
- [Step 5.2 集成总结](docs/step-5.2-integration-complete-summary.md) - 游戏集成完整总结
- [Step 5.3 设计文档](docs/step-5.3-trn-and-town-scene.md) - TRN和城镇场景设计
- [Step 5.3 游戏集成](docs/step-5.3-game-integration-summary.md) - 游戏集成完成总结 **最新**

## 🎯 当前进度

### Step 6: 地图生成系统（设计阶段）⏳ - 最新进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| 📝 设计文档 | ✅ 100% | 完整设计文档已完成 |
| 🎨 等距投影系统 | 📋 计划中 | 菱形瓦片渲染 |
| 🗂️ 瓦片数据加载 | 📋 计划中 | MIN/TIL/SOL格式 |
| 🏗️ 房间生成算法 | 📋 计划中 | BSP递归生成 |
| 🧱 墙壁和装饰 | 📋 计划中 | Miniset系统 |

**关键技术突破：**
- 🔥 **等距投影（Isometric Projection）** - 64x32菱形瓦片
- 🔄 世界坐标 ↔ 屏幕坐标转换系统
- 📐 MegaTile vs MicroTile 坐标层级

**实施计划：**
- **Step 6.1**: 等距投影 + 瓦片系统（300-400行，2-3天）
- **Step 6.2**: 地图数据结构 + 房间生成（350-450行，3-4天）
- **Step 6.3**: 墙壁生成 + Miniset（250-350行，2-3天）

### 已完成的步骤

- ✅ Step 1-3: 基础框架、渲染系统、精灵系统 (~1200行代码)
- ✅ Step 4: 完整动画系统和碰撞检测 (~900行代码)
- ✅ Step 5: 原版资源格式完整支持 (~3100行代码)
- 📋 Step 6: 地图生成系统（设计中，预计~1200行代码）
  - ✅ Step 5.1: MPQ归档与Palette系统 (~1000行代码)
    - ✅ MPQ 读取（原生实现）
    - ✅ 压缩算法（PKWare, Zlib, Huffman）
  - ✅ Step 5.2: PCX/CLX/CL2精灵格式 (~900行代码)
    - ✅ PCX图像加载（RLE解压缩）
    - ✅ CLX/CL2精灵格式
    - ✅ 战士精灵动画集成
  - ✅ Step 5.3: TRN和城镇场景 (~1200行代码)
    - ✅ TRN颜色转换系统
    - ✅ ResourceManager资源管理器
    - ✅ SimpleTown城镇场景预览
    - ✅ 场景系统和游戏集成
  - ✅ libmpq FFI 封装
  - ✅ Palette 系统
- ✅ Step 5.2: PCX/CLX/CL2资源格式 (~1560行代码) **最新完成**
  - ✅ PCX 图像加载和显示
  - ✅ CLX 精灵解析（修复RLE bug）
  - ✅ CL2 精灵解析（新增支持）
  - ✅ 战士精灵完整动画
- **总计**: ~4660行核心代码

## 🧪 测试

```powershell
# 运行所有测试
cargo test

# 运行 MPQ 相关测试
cargo test mpq

# 运行 palette 测试
cargo test palette

# 运行压缩算法测试
cargo test compression
```

## 💡 技术亮点

### 1. 原生 MPQ 实现

完全用 Rust 实现的 MPQ 归档读取，包括：
- Hash 表解密和查找
- Block 表读取
- 文件数据解密
- 多重压缩处理

### 2. 压缩算法

实现了多种压缩算法：
- **PKWare**: 完整移植 libmpq 的 explode.c
- **Zlib**: 使用 flate2 crate
- **多重压缩**: 支持 Huffman + Zlib + PKWare 组合

### 3. FFI 备用方案

提供了 libmpq C 库的 FFI 封装作为备用方案：
- 类型安全的绑定层
- RAII 资源管理
- 完整的测试覆盖

## 🎓 学习要点

本项目重点学习：

### Rust 语言特性
- 所有权和借用
- 生命周期管理
- Trait 系统
- 错误处理（Result/anyhow）

### 系统编程
- FFI (Foreign Function Interface)
- Unsafe Rust
- 内存管理
- 二进制数据处理

### 游戏开发
- 游戏循环
- 资源管理
- 文件格式解析
- 渲染系统

### 算法实现
- 压缩算法（PKWare, Zlib, Huffman）
- 加密算法（MPQ Hash）
- 数据结构（Hash Table）

## 📊 代码统计

| 模块 | 代码行数 | 状态 |
|------|---------|------|
| **基础框架 (Step 1-3)** | | |
| 渲染系统 | ~400 | ✅ |
| 精灵系统 | ~300 | ✅ |
| 基础框架 | ~500 | ✅ |
| **动画碰撞 (Step 4)** | | |
| 动画系统 | ~450 | ✅ |
| 碰撞检测 | ~450 | ✅ |
| **资源系统 (Step 5)** | | |
| MPQ 读取 | ~500 | ✅ |
| PKWare 解压 | ~505 | ✅ |
| Huffman 解压 | ~325 | ✅ |
| Palette 系统 | ~490 | ✅ |
| PCX 加载器 | ~350 | ✅ |
| CLX 加载器 | ~520 | ✅ |
| CL2 加载器 | ~280 | ✅ |
| Engine 增强 | ~60 | ✅ |
| **总计** | **~4660** | **✅** |

## 📚 参考资料

- [DevilutionX](https://github.com/diasurgical/devilutionX) - 原始 C++ 实现
- [MPQ 格式规范](http://www.zezula.net/en/mpq/mpqformat.html)
- [libmpq 文档](https://github.com/diasurgical/libmpq)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust FFI Omnibus](http://jakegoulding.com/rust-ffi-omnibus/)

## 🤝 贡献

这是一个学习型项目，欢迎：
- 提出改进建议
- 报告 bug
- 分享学习心得
- 讨论技术实现

## 📝 许可证

本项目仅用于学习目的。游戏资源版权归 Blizzard Entertainment 所有。

## 🔗 相关链接

- [Diablo 1 官方网站](https://diablo.blizzard.com/)
- [DevilutionX 项目](https://github.com/diasurgical/devilutionX)
- [Rust 官方文档](https://www.rust-lang.org/)

---

**最后更新**: 2025-11-25  
**当前版本**: Step 5 完成，Step 6 设计中  
**下一步**: Step 6.1 - 等距投影系统和瓦片加载

**相关文档：**
- 📖 [Step 6 设计文档](docs/step-6-dungeon-generation-design.md)
- 📋 [Step 6 实施计划](docs/step-6-implementation-plan.md)
