# Tile调试对比指南

## 概述

本文档说明如何使用添加的调试代码来对比C++和Rust实现的tile渲染结果。

## 调试代码位置

### C++调试代码
- **文件**: `Source/engine/render/scrollrt.cpp`
- **函数**: `DrawFloor()`
- **调试位置**: 行 943-978

### Rust调试代码
- **文件**: `rust-diablo/src/world/mod.rs`
- **函数**: `render_with_texture_manager()`
- **调试位置**: 行 382-450

## 如何修改调试位置

### C++代码
修改 `DrawFloor()` 函数中的 `debug_tile` 变量：

```cpp
const Point debug_tile = { 25, 25 }; // 修改为你想要调试的dPiece坐标
```

### Rust代码
修改 `render_with_texture_manager()` 函数中的常量：

```rust
const DEBUG_TILE_X: i32 = 25;  // 修改为你想要调试的dPiece X坐标
const DEBUG_TILE_Y: i32 = 25;  // 修改为你想要调试的dPiece Y坐标
```

**注意**: 确保C++和Rust使用相同的dPiece坐标，以便对比结果。

## 调试输出内容

### 1. 中心Tile信息
- **dPiece坐标**: tile在dPiece数组中的位置
- **levelPieceId**: tile的piece ID（从dPiece数组获取）
- **IsFloor**: 是否为地板tile（基于SOL数据判断）

### 2. Block信息
输出前4个block的详细信息：
- **hasValue**: block是否有值（非0）
- **frame**: block的frame索引
- **type**: block的类型（LeftTriangle, RightTriangle, Square等）

### 3. 周边Tile分布
输出8个方向的相邻tile信息：
- **方向**: S, SW, W, NW, N, NE, E, SE
- **dPiece坐标**: 相邻tile的坐标
- **pieceId**: 相邻tile的piece ID
- **IsFloor**: 相邻tile是否为地板

## 等轴测坐标方向映射

在等轴测坐标（dPiece）中，8个方向的坐标变化：

| 方向 | dPiece坐标变化 | 说明 |
|------|---------------|------|
| S (South) | (0, +1) | 向下 |
| SW (SouthWest) | (-1, +1) | 左下 |
| W (West) | (-1, 0) | 向左 |
| NW (NorthWest) | (-1, -1) | 左上 |
| N (North) | (0, -1) | 向上 |
| NE (NorthEast) | (+1, -1) | 右上 |
| E (East) | (+1, 0) | 向右 |
| SE (SouthEast) | (+1, +1) | 右下 |

## 示例输出对比

### C++输出示例
```
=== C++ Tile Debug at dPiece(25,25) ===
  levelPieceId: 218
  IsFloor: true
  block[0]: hasValue=1, frame=1234, type=0
  block[1]: hasValue=1, frame=1235, type=1
  block[2]: hasValue=0, frame=0, type=0
  block[3]: hasValue=0, frame=0, type=0
  Surrounding tiles:
    S: dPiece(25,26) -> pieceId=219, IsFloor=true
    SW: dPiece(24,26) -> pieceId=220, IsFloor=true
    W: dPiece(24,25) -> pieceId=221, IsFloor=false
    NW: dPiece(24,24) -> pieceId=222, IsFloor=false
    N: dPiece(25,24) -> pieceId=223, IsFloor=true
    NE: dPiece(26,24) -> pieceId=224, IsFloor=true
    E: dPiece(26,25) -> pieceId=225, IsFloor=true
    SE: dPiece(26,26) -> pieceId=226, IsFloor=true
```

### Rust输出示例
```
=== Rust Tile Debug at dPiece(25,25) ===
  levelPieceId: 218
  IsFloor: true
  block[0]: hasValue=true, frame=1234, type=LeftTriangle
  block[1]: hasValue=true, frame=1235, type=RightTriangle
  block[2]: hasValue=false, frame=0, type=LeftTriangle
  block[3]: hasValue=false, frame=0, type=LeftTriangle
  Surrounding tiles:
    S: dPiece(25,26) -> pieceId=219, IsFloor=true
    SW: dPiece(24,26) -> pieceId=220, IsFloor=true
    W: dPiece(24,25) -> pieceId=221, IsFloor=false
    NW: dPiece(24,24) -> pieceId=222, IsFloor=false
    N: dPiece(25,24) -> pieceId=223, IsFloor=true
    NE: dPiece(26,24) -> pieceId=224, IsFloor=true
    E: dPiece(26,25) -> pieceId=225, IsFloor=true
    SE: dPiece(26,26) -> pieceId=226, IsFloor=true
```

## 对比检查清单

运行两个版本后，检查以下项目是否一致：

### ✅ 必须一致的项目
1. **levelPieceId**: 中心tile的piece ID必须相同
2. **IsFloor**: 中心tile和所有周边tile的IsFloor值必须相同
3. **周边tile的pieceId**: 8个方向的piece ID必须相同
4. **block[0]和block[1]的frame**: 地板tile的前两个block的frame必须相同

### ⚠️ 可能不同的项目（但需要验证）
1. **block的type显示**: C++显示数字，Rust显示枚举名称（但值应该相同）
2. **block[2]和block[3]**: 对于地板tile，这些通常是0，但如果有值也需要一致

## 常见问题排查

### 问题1: levelPieceId不一致
- **可能原因**: dPiece数组数据不同
- **检查**: 确认地图加载逻辑是否一致

### 问题2: 周边tile的pieceId不一致
- **可能原因**: 
  - 坐标计算错误
  - 方向映射错误
- **检查**: 
  - 确认等轴测坐标的方向映射是否正确
  - 确认dPiece坐标的计算是否正确

### 问题3: IsFloor判断不一致
- **可能原因**: 
  - SOL数据加载不同
  - TileProperties判断逻辑不同
- **检查**: 
  - 确认SOL数据是否正确加载
  - 确认TileProperties的判断逻辑是否一致

### 问题4: block信息不一致
- **可能原因**: 
  - MIN数据加载不同
  - block重排序逻辑不同
- **检查**: 
  - 确认MIN数据是否正确加载
  - 确认SetDungeonMicros的重排序逻辑是否一致

## 调试技巧

1. **选择有代表性的tile**: 选择一个包含多种tile类型的区域进行调试
2. **对比多个位置**: 如果发现不一致，尝试多个不同的tile位置
3. **检查边界情况**: 特别关注地图边界附近的tile
4. **逐步缩小范围**: 如果发现不一致，逐步缩小问题范围

## 注意事项

1. **调试代码只执行一次**: 使用`static bool debug_printed`（C++）和`let mut debug_printed`（Rust）确保只输出一次
2. **坐标系统**: 确保使用dPiece坐标，不是world坐标或屏幕坐标
3. **地图加载**: 确保两个版本加载的是相同的地图数据
4. **数据一致性**: 确保MIN、TIL、SOL数据在两个版本中是一致的

## 下一步

如果发现不一致：
1. 记录具体的差异
2. 检查相关的数据加载和转换逻辑
3. 对比相关的C++和Rust实现
4. 修复差异并重新测试





