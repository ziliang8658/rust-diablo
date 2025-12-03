# Bug修复：等轴测坐标方向映射错误

## 问题描述

在对比C++和Rust的调试输出时，发现周边tile的坐标计算不一致。虽然pieceId和IsFloor值相同，但坐标位置不同。

## 根本原因

Rust代码中的等轴测坐标方向映射与C++不一致。

### C++的方向映射（正确）

参考：`Source/engine/displacement.hpp:247-272`

```cpp
case Direction::South:      return { 1, 1 };
case Direction::SouthWest:  return { 0, 1 };
case Direction::West:       return { -1, 1 };
case Direction::NorthWest:  return { -1, 0 };
case Direction::North:      return { -1, -1 };
case Direction::NorthEast:  return { 0, -1 };
case Direction::East:       return { 1, -1 };
case Direction::SouthEast:  return { 1, 0 };
```

### Rust的方向映射（错误，已修复）

**修复前：**
```rust
let directions = [
    ((0, 1), "S"),      // ❌ 错误：应该是 (1, 1)
    ((-1, 1), "SW"),   // ❌ 错误：应该是 (0, 1)
    ((-1, 0), "W"),    // ❌ 错误：应该是 (-1, 1)
    ((-1, -1), "NW"),  // ❌ 错误：应该是 (-1, 0)
    ((0, -1), "N"),    // ❌ 错误：应该是 (-1, -1)
    ((1, -1), "NE"),   // ❌ 错误：应该是 (0, -1)
    ((1, 0), "E"),     // ❌ 错误：应该是 (1, -1)
    ((1, 1), "SE"),    // ❌ 错误：应该是 (1, 0)
];
```

**修复后：**
```rust
let directions = [
    ((1, 1), "S"),      // ✅ 正确：South = (1, 1)
    ((0, 1), "SW"),     // ✅ 正确：SouthWest = (0, 1)
    ((-1, 1), "W"),     // ✅ 正确：West = (-1, 1)
    ((-1, 0), "NW"),    // ✅ 正确：NorthWest = (-1, 0)
    ((-1, -1), "N"),    // ✅ 正确：North = (-1, -1)
    ((0, -1), "NE"),    // ✅ 正确：NorthEast = (0, -1)
    ((1, -1), "E"),     // ✅ 正确：East = (1, -1)
    ((1, 0), "SE"),     // ✅ 正确：SouthEast = (1, 0)
];
```

## 影响

这个错误导致：
1. **调试输出不一致**：虽然pieceId和IsFloor值正确，但坐标位置显示错误
2. **可能影响其他功能**：如果代码中其他地方使用了相同的方向映射，也会受到影响

## 修复位置

- **文件**: `rust-diablo/src/world/mod.rs`
- **函数**: `render_with_texture_manager()`
- **行数**: 424-433

## 验证

修复后，C++和Rust的调试输出应该完全一致：
- 中心tile的坐标应该相同
- 周边8个方向的tile坐标应该相同
- pieceId和IsFloor值应该相同

## 等轴测坐标系统说明

在等轴测投影中，方向映射与标准笛卡尔坐标系不同：

| 方向 | 等轴测坐标变化 | 说明 |
|------|---------------|------|
| South | (1, 1) | 向右下 |
| SouthWest | (0, 1) | 向下 |
| West | (-1, 1) | 向左下 |
| NorthWest | (-1, 0) | 向左 |
| North | (-1, -1) | 向左上 |
| NorthEast | (0, -1) | 向上 |
| East | (1, -1) | 向右上 |
| SouthEast | (1, 0) | 向右 |

## 参考

- C++原版：`Source/engine/displacement.hpp:247-272`
- C++使用：`Source/engine/render/scrollrt.cpp:959-962`





