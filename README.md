# Rust Diablo

A Rust rewrite of Diablo 1 for learning purposes.

## 项目目标

这是一个学习项目，目标是用Rust语言逐步重写Diablo 1游戏引擎。我们采用渐进式开发方法，从最基础的框架开始，逐步完善功能，直至完成整个游戏。

## 当前进度

- ✅ **Step 1: 基础游戏框架** - 完成
  - SDL2窗口和渲染系统
  - 基础游戏循环
  - 输入处理框架

- ✅ **Step 2: 基础渲染系统和坐标系统** - 完成
  - 2D 数学库（Point, Rect）
  - 颜色系统和绘图 API
  - 实体系统
  - 网格地图系统
  - 玩家移动控制（WASD/方向键）

- ✅ **Step 3: 精灵系统和玩家渲染** - 完成
  - 纹理加载和管理系统
  - 精灵渲染系统
  - 动画系统框架
  - 资源管理系统
  - 玩家精灵图像渲染

- ✅ **Step 4.1: 方向系统和动画系统** - 完成
  - 8方向移动系统（Direction枚举）
  - 完整动画状态机（Idle, Walk等）
  - 动画与移动同步
  - 方向向量归一化（对角线速度一致）

- ✅ **Step 4.2: 碰撞检测和相机系统** - 完成
  - 碰撞检测系统（CollisionMap）
  - 瓦片碰撞验证（AABB检测）
  - 相机跟随系统（Camera）
  - 视口剔除优化
  - 多种瓦片类型（Floor, Wall, Grass, Water）
  - 瓦片纹理渲染系统
  - 猴子精灵图系统（静止/移动动画）

## 构建要求

- Rust 1.70 或更高版本
- SDL2 开发库
- SDL2_image 开发库（用于加载图像文件）

### Windows 上安装 SDL2 和 SDL2_image

1. **下载 SDL2 开发库：**
   - 访问：https://github.com/libsdl-org/SDL/releases
   - 下载 `SDL2-devel-2.x.x-VC.zip`（MSVC 版本）
   - 解压到某个目录（例如：`F:\SDL2-2.28.3\`）

2. **下载 SDL2_image 开发库：**
   - 访问：https://github.com/libsdl-org/SDL_image/releases
   - 下载 `SDL2_image-devel-2.x.x-VC.zip`（MSVC 版本）
   - 解压后，将 `lib\x64\SDL2_image.lib` 复制到 SDL2 的 `lib\x64\` 目录
   - 将 `SDL2_image.dll` 复制到 SDL2 的 `lib\x64\` 目录（运行时需要）

3. **验证安装：**
   - 确保 `F:\SDL2-2.28.3\lib\x64\` 目录下有以下文件：
     - `SDL2.lib`
     - `SDL2_image.lib`
     - `SDL2.dll`
     - `SDL2_image.dll`

4. **如果你的 SDL2 在其他位置，修改以下文件中的路径：**
   - `.cargo/config.toml` - 修改 SDL2 库路径（lib 目录）
   - `build-windows.ps1` 和 `run-windows.ps1` - 修改 SDL2.dll 路径
   - 将所有 `F:\SDL2-2.28.3\` 改为你的 SDL2 安装路径

### Linux 上安装 SDL2 和 SDL2_image

```bash
# Ubuntu/Debian
sudo apt-get install libsdl2-dev libsdl2-image-dev

# Fedora
sudo dnf install SDL2-devel SDL2_image-devel

# Arch
sudo pacman -S sdl2 sdl2_image
```

### macOS 上安装 SDL2 和 SDL2_image

```bash
brew install sdl2 sdl2_image
```

## 编译和运行

### Windows

```powershell
# 进入项目目录
cd rust-diablo

# 使用提供的脚本运行（会自动设置环境变量和复制 DLL）
.\run-windows.ps1

# 或者仅编译
.\build-windows.ps1
```

**注意**：如果你的 SDL2 在不同位置，请修改以下文件中的路径：
- `.cargo/config.toml` - 修改 SDL2 库路径（lib 目录）
- `build-windows.ps1` 和 `run-windows.ps1` - 修改 SDL2.dll 路径
- 将所有 `F:\SDL2-2.28.3\` 改为你的 SDL2 安装路径

### Linux/macOS

```bash
# 进入项目目录
cd rust-diablo

# 编译并运行
cargo run

# 编译发布版本
cargo build --release
```

## 项目结构

```
rust-diablo/
├── Cargo.toml              # Rust项目配置
├── src/
│   ├── main.rs            # 程序入口
│   ├── lib.rs             # 库入口
│   ├── game.rs             # 游戏循环和状态管理
│   ├── engine/             # 引擎层
│   │   ├── mod.rs          # SDL2封装
│   │   └── direction.rs   # 方向系统
│   ├── math/               # 数学库
│   │   ├── point.rs        # 点坐标
│   │   └── rect.rs         # 矩形
│   ├── renderer/           # 渲染系统
│   │   ├── color.rs        # 颜色定义
│   │   └── camera.rs       # 相机系统
│   ├── sprite/             # 精灵系统
│   │   ├── animation.rs    # 动画系统
│   │   ├── texture.rs      # 纹理管理
│   │   └── sprite.rs       # 精灵定义
│   ├── entity/             # 实体系统
│   │   └── mod.rs          # 实体定义
│   ├── world/              # 世界系统
│   │   ├── mod.rs          # 世界管理
│   │   └── collision.rs    # 碰撞检测
│   └── assets/             # 资源管理
│       └── mod.rs          # 资源路径
├── assets/                 # 游戏资源
│   └── sprites/            # 精灵图
│       ├── player.png      # 玩家精灵图（猴子动画）
│       ├── tile_floor.png  # 地板纹理
│       ├── tile_wall.png   # 墙壁纹理
│       ├── tile_grass.png  # 草地纹理
│       └── tile_water.png  # 水面纹理
└── docs/                   # 文档
    ├── MASTER_PLAN.md      # 总体规划
    ├── step-*.md           # 步骤文档
    └── tech_key_points/    # 技术要点
```

## 文档

### 规划文档
- 📋 **[完整实现的中长期规划](docs/MASTER_PLAN.md)** - 50个Steps的详细规划（总体纲要）
- 📊 **[功能对照表](docs/FEATURE_COMPARISON.md)** - 与原版DevilutionX的功能对照
- 🧪 **[测试指南](docs/TESTING_GUIDE.md)** - TDD开发指南和测试要求

### 技术要点文档
- 🎨 **[精灵表系统技术详解](docs/tech_key_points/sprite-sheet-system.md)** - 精灵表系统完整实现（动画、渲染、坐标系统）
- ⭐ **[Step 5: 资源格式技术详解](docs/tech_key_points/step5-resource-formats.md)** - MPQ/PCX/CEL/CLX等格式的完整实现细节

### Step文档（已完成）
- [Step 1: 基础游戏框架](docs/step-1-foundation.md)
- [Step 2: 基础渲染系统和坐标系统](docs/step-2-rendering.md)
- [Step 3: 精灵系统和玩家渲染](docs/step-3-sprites.md)
- [Step 4: 动画系统和碰撞检测](docs/step4-animation-collision.md) - 完整设计文档
- [Step 4.2: 碰撞检测与相机系统总结](docs/step-4.2-summary.md) - 实现总结和练习任务

### Bug修复文档
- [子像素移动累积丢失问题](docs/bugfix-subpixel-movement-accumulation.md) - 高帧率下移动卡顿问题修复

## 控制

- `ESC` - 退出游戏
- `W` / `↑` - 向上移动
- `S` / `↓` - 向下移动
- `A` / `←` - 向左移动
- `D` / `→` - 向右移动
- 支持8方向移动（W+A, W+D, S+A, S+D 等）
- 关闭窗口 - 退出游戏

## 游戏特性

### 当前实现的功能

- ✅ **8方向移动**：流畅的8方向移动，对角线移动速度已归一化
- ✅ **动画系统**：完整的动画状态机，支持Idle和Walk状态切换
- ✅ **猴子精灵图**：静止时显示猴子，移动时显示猴子乘坐云朵的动画
- ✅ **碰撞检测**：玩家无法穿过墙壁和水域
- ✅ **相机跟随**：相机自动跟随玩家，保持玩家在屏幕中心
- ✅ **多种地形**：支持地板、墙壁、草地、水域等不同瓦片类型
- ✅ **瓦片纹理**：每种瓦片类型都有对应的纹理图片
- ✅ **视口优化**：只渲染相机范围内的内容，提升性能

### 视觉效果

- 🐵 **玩家角色**：可爱的猴子角色，静止和移动都有动画
- 🎨 **多样化地图**：使用噪声函数生成包含不同地形的随机地图
- 🖼️ **纹理渲染**：所有瓦片都使用纹理图片而非纯色方块

## 技术栈

- **Rust** - 系统编程语言（2021 Edition）
- **SDL2** - 跨平台多媒体库（窗口、输入、渲染）
- **SDL2_image** - 图像加载支持（PNG格式）
- **anyhow** - 错误处理
- **PIL/Pillow** - 精灵图生成工具（Python脚本）

## 资源生成

### 生成玩家精灵图

玩家精灵图（猴子动画）可以通过Python脚本自动生成：

```bash
cd rust-diablo/assets
pip install pillow
python create_monkey_sprite.py
```

这将生成 `sprites/player.png`，包含：
- 帧1：静止的猴子
- 帧2-4：猴子乘坐云朵的移动动画

更多信息请查看：[精灵图资源说明](assets/sprites/README.md)

## 开发计划

### 当前阶段
1. ✅ Step 1: 基础游戏框架
2. ✅ Step 2: 基础渲染系统和坐标系统
3. ✅ Step 3: 精灵系统和玩家渲染
4. ✅ Step 4.1: 方向系统和动画系统
5. ✅ Step 4.2: 碰撞检测和相机系统
6. ⏳ Step 5: 原版资源格式完整支持（下一步）

### 代码统计
- **已完成代码量**：约 2,300 行
- **当前功能**：基础框架、渲染、精灵、动画、碰撞、相机

### 完整规划
详细的中长期开发规划请查看：**[完整实现的中长期规划](docs/MASTER_PLAN.md)**

该规划包含：
- 📍 **第一阶段（Step 4-12）**：可玩原型 - 3-4个月
  - ✅ Step 4.1-4.2: 动画、碰撞、相机（已完成）
  - ⏳ Step 5: 原版资源格式支持（下一步）
  - ⏳ Step 6-12: 地图生成、怪物、战斗、物品等
- 📍 **第二阶段（Step 13-25）**：核心系统完善 - 4-6个月
- 📍 **第三阶段（Step 26-35）**：网络与UI - 3-4个月
- 📍 **第四阶段（Step 36-50）**：高级功能（Lua、Mod、跨平台） - 4-5个月
- 📍 **第五阶段（Step 51+）**：持续优化和维护

**预计总开发时间：14-19个月**  
**预计总代码量：70,000-75,000行**

## 参考

本项目参考了 [DevilutionX](https://github.com/diasurgical/devilutionX) 的原始代码实现。

## 许可

本项目仅用于学习目的。原始 Diablo 游戏版权归 Blizzard Entertainment 所有。

