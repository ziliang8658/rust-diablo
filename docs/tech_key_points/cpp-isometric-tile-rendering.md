# C++ 等距瓦片（Isometric Tile）渲染算法详解

## 📋 目录

1. [概述](#概述)
2. [等距投影数学原理](#等距投影数学原理)
3. [坐标转换算法](#坐标转换算法)
4. [渲染循环算法](#渲染循环算法)
5. [完整实例分析](#完整实例分析)
6. [关键代码解读](#关键代码解读)
7. [总结](#总结)

---

## 概述

Diablo 1 使用**等距投影（Isometric Projection）**来创建伪3D视觉效果。这是一种2D渲染技术，通过将正交网格（世界坐标）转换为菱形网格（屏幕坐标）来实现。

### 关键概念

- **世界坐标（World Coordinates）**：游戏逻辑使用的正交网格坐标
- **屏幕坐标（Screen Coordinates）**：实际渲染位置的像素坐标
- **瓦片尺寸**：每个等距瓦片为 64×32 像素（菱形）
- **投影角度**：-135° 旋转 + 缩放变换

**参考代码：**
- `Source/engine/displacement.hpp` - 坐标转换核心算法
- `Source/engine/render/scrollrt.cpp` - 渲染循环实现

---

## 等距投影数学原理

### 1. 投影变换矩阵

等距投影本质上是一个**旋转 + 缩放**的线性变换：

```cpp
// 旋转矩阵（-135度）
cos(-135°) = -√2/2 ≈ -0.707
sin(-135°) = -√2/2 ≈ -0.707

// 缩放因子
scale_x = 32  // 水平方向缩放（瓦片宽度的一半）
scale_y = 16  // 垂直方向缩放（瓦片高度的一半）
```

### 2. 变换矩阵推导

**原始变换矩阵：**

```
旋转矩阵 × 缩放矩阵：

[cos(-135°), -sin(-135°)]   [scale_x,    0    ]   [-32,  32]
[sin(-135°),  cos(-135°)] × [   0,    scale_y] = [-16, -16]
```

**简化后的变换公式：**

```cpp
screen_x = (world_y - world_x) * 32
screen_y = (world_y + world_x) * -16
```

**数学推导过程：**

```cpp
// 完整矩阵乘法：
// [-32,  32] [world_x]   = [-32*world_x + 32*world_y]   = [32*(world_y - world_x)]
// [-16, -16] [world_y]     [-16*world_x - 16*world_y]     [-16*(world_x + world_y)]

// 因为 Diablo 使用右上方向为正，需要翻转 Y 轴：
screen_y = -(-16*(world_x + world_y)) = 16*(world_x + world_y)
// 但实际代码中是 -16，这是因为坐标系定义不同
```

**参考：** `Source/engine/displacement.hpp::worldToScreen()` Line 151-154

---

## 坐标转换算法

### 1. 世界坐标 → 屏幕坐标

**C++ 实现：**

```cpp:Source/engine/displacement.hpp
DVL_ALWAYS_INLINE constexpr DisplacementOf<DeltaT> worldToScreen() const
{
    static_assert(std::is_signed<DeltaT>::value, "DeltaT must be signed for transformations involving a rotation");
    return { (deltaY - deltaX) * 32, (deltaY + deltaX) * -16 };
}
```

**参数说明：**
- `deltaX`：世界坐标 X 方向位移
- `deltaY`：世界坐标 Y 方向位移

**返回值：**
- 屏幕坐标的 X 和 Y 像素偏移量

**实例计算：**

| 世界坐标 | 计算过程 | 屏幕坐标 | 说明 |
|---------|---------|---------|------|
| (0, 0)  | (0-0)×32, (0+0)×-16 | (0, 0) | 原点 |
| (1, 0)  | (0-1)×32, (0+1)×-16 | (-32, -16) | 向右移动 |
| (0, 1)  | (1-0)×32, (1+0)×-16 | (32, -16) | 向下移动 |
| (1, 1)  | (1-1)×32, (1+1)×-16 | (0, -32) | 对角线移动 |
| (2, 3)  | (3-2)×32, (3+2)×-16 | (32, -80) | 一般情况 |

### 2. 屏幕坐标 → 世界坐标（逆变换）

**C++ 实现：**

```cpp:Source/engine/displacement.hpp
DVL_ALWAYS_INLINE constexpr DisplacementOf<DeltaT> screenToWorld() const
{
    static_assert(std::is_signed<DeltaT>::value, "DeltaT must be signed for transformations involving a rotation");
    return { (2 * deltaY + deltaX) / -64, (2 * deltaY - deltaX) / -64 };
}
```

**逆矩阵推导：**

原矩阵为：
```
M = [-32,  32]
    [-16, -16]
```

逆矩阵为：
```
M^(-1) = [1/64,  1/64]   × [ 2, -1]
         [1/64, -1/64]     [-1,  2]
```

简化后：
```
world_x = (2 * screen_y + screen_x) / -64
world_y = (2 * screen_y - screen_x) / -64
```

**实例计算：**

| 屏幕坐标 | 计算过程 | 世界坐标 | 说明 |
|---------|---------|---------|------|
| (0, 0) | (0+0)/-64, (0-0)/-64 | (0, 0) | 原点 |
| (-32, -16) | (-32+(-32))/-64, (-32-(-32))/-64 | (1, 0) | 逆向计算 |

---

## 渲染循环算法

### 核心函数：`DrawTileContent()`

这是渲染等距瓦片的**核心函数**，负责遍历可见区域并绘制每个瓦片。

**函数签名：**

```cpp:Source/engine/render/scrollrt.cpp
void DrawTileContent(
    const Surface &out,              // 输出缓冲区
    const Lightmap &lightmap,        // 光照贴图
    Point tilePosition,              // 起始瓦片位置（dPiece坐标）
    Point targetBufferPosition,      // 起始屏幕位置（像素坐标）
    int rows,                        // 渲染行数
    int columns                      // 每行瓦片数
)
```

### 算法步骤

#### Step 1: 初始化

```cpp
// 添加额外的行以确保所有可见的MicroTile都被渲染
rows += MicroTileLen;
```

#### Step 2: 行循环（从上到下）

```cpp
for (int i = 0; i < rows; i++) {
    bool skip = false;
    
    // 列循环（从左到右）
    for (int j = 0; j < columns; j++) {
        // ... 渲染逻辑 ...
    }
    
    // 移动到下一行
    // ... 行更新逻辑 ...
}
```

#### Step 3: 列循环中的移动方向

在等距投影中，**向右移动**（East方向）的坐标变化：

```cpp
// 世界坐标：向东移动
tilePosition += Direction::East;  // world_x += 1, world_y -= 1

// 屏幕坐标：向右移动
targetBufferPosition.x += TILE_WIDTH;  // +64 像素
```

**为什么 world_y -= 1？**

在等距投影中，向东移动在屏幕上的效果是：
- 向右移动（screen_x 增加）
- 稍微向上移动（screen_y 略微减少）

根据变换公式：
```
screen_x = (world_y - world_x) * 32
```

要增加 `screen_x`，需要：
- 增加 `world_y`，或
- 减少 `world_x`

但考虑到等距网格的特性，标准做法是同时改变两者。

#### Step 4: 行切换逻辑（关键！）

这是等距渲染中最复杂的部分：

```cpp:Source/engine/render/scrollrt.cpp
// 移动到下一行的屏幕位置（向下移动半个瓦片高度）
targetBufferPosition.y += TILE_HEIGHT / 2;  // +16 像素

if ((i & 1) != 0) {
    // 奇数行：向右偏移
    tilePosition.x++;           // 世界坐标向右
    columns--;                  // 列数减少
    targetBufferPosition.x += TILE_WIDTH / 2;  // 屏幕位置右移32像素
} else {
    // 偶数行：向左偏移
    tilePosition.y++;           // 世界坐标向下
    columns++;                  // 列数增加
    targetBufferPosition.x -= TILE_WIDTH / 2;  // 屏幕位置左移32像素
}
```

**为什么需要交替偏移？**

等距网格的特点：每一行的起始位置在屏幕上**左右交替**：

```
屏幕上的等距网格布局：

行 0:     ◇  ◇  ◇  ◇
行 1:      ◇  ◇  ◇
行 2:     ◇  ◇  ◇  ◇
行 3:      ◇  ◇  ◇

行 0 从 x=0 开始
行 1 从 x=32 开始（偏移+32）
行 2 从 x=0 开始（偏移-32）
行 3 从 x=32 开始（偏移+32）
```

#### Step 5: 瓦片绘制

```cpp
if (InDungeonBounds(tilePosition)) {
    // 绘制当前瓦片
    DrawDungeon(out, lightmap, tilePosition, targetBufferPosition);
}
```

---

## 完整实例分析

### 实例 1：渲染一个 3×3 的等距网格

**初始条件：**
- 起始瓦片位置：(10, 10)
- 起始屏幕位置：(100, 50)
- 屏幕宽度：640 像素
- 屏幕高度：480 像素
- 瓦片宽度：64 像素
- 瓦片高度：32 像素

**计算参数：**
```cpp
columns = (640 / 64) + 2 = 12  // 每行12个瓦片
rows = (480 / 16) + 4 = 34     // 34行
```

**渲染过程追踪：**

#### 行 0（偶数行，i=0）

```cpp
// 初始状态
tile_x = 10, tile_y = 10
screen_x = 100, screen_y = 50
columns = 12

// 绘制12个瓦片
for (j = 0; j < 12; j++) {
    // 瓦片 (10, 10) 绘制在屏幕 (100, 50)
    DrawDungeon(out, lightmap, (10, 10), (100, 50));
    
    // 移动到下一个瓦片（East方向）
    tile_x = 11, tile_y = 9   // world_x += 1, world_y -= 1
    screen_x = 164            // screen_x += 64
    
    // 瓦片 (11, 9) 绘制在屏幕 (164, 50)
    DrawDungeon(out, lightmap, (11, 9), (164, 50));
    
    // ... 继续到 j=11 ...
}

// 行结束，移动到下一行
screen_y = 66  // 50 + 16

// 偶数行：向左偏移
tile_y = 11      // tile_y++
columns = 13     // columns++
screen_x = 68    // 100 - 32
```

#### 行 1（奇数行，i=1）

```cpp
// 当前状态
tile_x = 10, tile_y = 11
screen_x = 68, screen_y = 66
columns = 13

// 绘制13个瓦片（奇数行多一个）
for (j = 0; j < 13; j++) {
    DrawDungeon(out, lightmap, (10, 11), (68, 66));
    // ... 向东移动 ...
}

// 行结束，移动到下一行
screen_y = 82  // 66 + 16

// 奇数行：向右偏移
tile_x = 11      // tile_x++
columns = 12     // columns--
screen_x = 100   // 68 + 32
```

#### 行 2（偶数行，i=2）

```cpp
// 当前状态
tile_x = 11, tile_y = 11
screen_x = 100, screen_y = 82
columns = 12

// ... 类似行0 ...
```

**可视化展示：**

```
世界坐标网格（部分）：

     10  11  12  13
    ┌───┬───┬───┬───┐
10  │ A │ B │ C │ D │
    ├───┼───┼───┼───┤
11  │ E │ F │ G │ H │
    ├───┼───┼───┼───┤
12  │ I │ J │ K │ L │
    └───┴───┴───┴───┘

屏幕上的等距投影（菱形布局）：

行 0:      A    B    C    D
         (100) (164) (228) (292)
         
行 1:        E    F    G    H    I
           (68) (132) (196) (260) (324)
           
行 2:          J    K    L    M
             (100) (164) (228) (292)
```

### 实例 2：坐标转换验证

**问题：** 将世界坐标 (5, 7) 转换为屏幕坐标，然后验证逆变换。

**步骤 1：世界 → 屏幕**

```cpp
world_x = 5, world_y = 7

screen_x = (world_y - world_x) * 32
         = (7 - 5) * 32
         = 2 * 32
         = 64

screen_y = (world_y + world_x) * -16
         = (7 + 5) * -16
         = 12 * -16
         = -192

结果：屏幕坐标 (64, -192)
```

**步骤 2：屏幕 → 世界（逆变换验证）**

```cpp
screen_x = 64, screen_y = -192

world_x = (2 * screen_y + screen_x) / -64
        = (2 * (-192) + 64) / -64
        = (-384 + 64) / -64
        = -320 / -64
        = 5  ✅

world_y = (2 * screen_y - screen_x) / -64
        = (2 * (-192) - 64) / -64
        = (-384 - 64) / -64
        = -448 / -64
        = 7  ✅
```

验证成功！转换是双向一致的。

---

## 关键代码解读

### 1. 瓦片移动方向（Direction::East）

**定义：**

```cpp:Source/engine/displacement.hpp
case Direction::East:
    return { 1, -1 };  // deltaX = 1, deltaY = -1
```

**在等距投影中的效果：**

```
世界坐标变化：
  world_x += 1   (向右)
  world_y -= 1   (向上，在逻辑空间中)

屏幕坐标变化（通过变换）：
  screen_x = ((world_y - deltaY) - (world_x + deltaX)) * 32
           = ((world_y - (-1)) - (world_x + 1)) * 32
           = ((world_y + 1) - (world_x + 1)) * 32
           = (world_y - world_x) * 32 + 32
            ↑ 原始值                  ↑ +32像素
  
  screen_y = ((world_y - deltaY) + (world_x + deltaX)) * -16
           = ((world_y - (-1)) + (world_x + 1)) * -16
           = (world_y + world_x + 2) * -16
           = 原始值 - 32
  
结果：屏幕向右移动64像素，稍微向上移动
```

### 2. 行切换逻辑的数学原理

**为什么需要交替偏移？**

等距投影中，相邻行的X坐标需要偏移半个瓦片宽度（32像素）。

**数学证明：**

假设我们在世界坐标 (x, y)，屏幕位置为 (sx, sy)。

下一行的世界坐标有两种可能：
- **选项1：** (x, y+1) - 向下移动
- **选项2：** (x+1, y) - 向右移动

计算它们的屏幕X坐标：

```cpp
选项1：screen_x1 = ((y+1) - x) * 32 = (y - x) * 32 + 32
选项2：screen_x2 = (y - (x+1)) * 32 = (y - x) * 32 - 32

差值：screen_x1 - screen_x2 = 64 像素 = TILE_WIDTH
```

所以：
- 使用 (x, y+1) 时，屏幕X需要 **+32**（左移，因为起始点在上方）
- 使用 (x+1, y) 时，屏幕X需要 **-32**（右移）

**交替规则：**
- 偶数行（i % 2 == 0）：使用 (x, y+1)，屏幕X **-32**
- 奇数行（i % 2 == 1）：使用 (x+1, y)，屏幕X **+32**

---

## 总结

### 关键要点

1. **等距投影公式：**
   - 世界→屏幕：`screen_x = (world_y - world_x) * 32`, `screen_y = (world_y + world_x) * -16`
   - 屏幕→世界：`world_x = (2 * screen_y + screen_x) / -64`, `world_y = (2 * screen_y - screen_x) / -64`

2. **渲染循环特点：**
   - 按行从上到下渲染
   - 每行从左到右渲染
   - 行间需要交替偏移（±32像素）
   - 列数在奇数行减少，偶数行增加

3. **移动方向：**
   - 向东（East）：世界坐标 (x+1, y-1)，屏幕坐标 (+64, 略上)
   - 行切换：向下移动半个瓦片高度（+16像素），左右交替偏移

4. **性能优化：**
   - 只渲染可见区域内的瓦片
   - 使用边界检查避免越界访问
   - 可以跳过某些特殊瓦片（如被墙遮挡的部分）

### 参考代码位置

- **坐标转换：** `Source/engine/displacement.hpp` Line 151-168
- **渲染循环：** `Source/engine/render/scrollrt.cpp::DrawTileContent()` Line 966-1019
- **地板渲染：** `Source/engine/render/scrollrt.cpp::DrawFloor()` Line 927-955
- **Rust实现：** `rust-diablo/src/engine/isometric.rs` 和 `rust-diablo/src/world/mod.rs`

### 学习要点

1. **数学基础：** 理解线性变换、旋转矩阵、逆矩阵
2. **坐标系统：** 掌握多坐标系之间的转换
3. **渲染算法：** 理解等距网格的遍历顺序
4. **调试技巧：** 通过可视化追踪坐标变换过程

---

**文档版本：** 1.0  
**最后更新：** 2024  
**参考项目：** DevilutionX





