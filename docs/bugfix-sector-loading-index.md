# Bug修复：Sector加载索引计算问题

## 问题描述

对比C++和Rust的dPiece数组后，发现33个sector内部差异，主要集中在：
- Sector3s的(36-45, 78-79)区域：Rust加载了错误的大值（1236-1255）
- Sector1s的(46, 78-79)：Rust加载了错误的大值（547/548）
- Sector2s的(48-49, 20-21)：Rust值比C++大4

## 根本原因

### C++代码逻辑（FillSector）

```cpp
for (WorldTileCoord j = 0; j < size.height; j++) {
    int xx = xi;  // 每行开始时重置xx
    for (WorldTileCoord i = 0; i < size.width; i++) {
        // ...
        dPiece[xx + 0][yy + 0] = v1;
        dPiece[xx + 1][yy + 0] = v2;
        dPiece[xx + 0][yy + 1] = v3;
        dPiece[xx + 1][yy + 1] = v4;
        xx += 2;  // 每列后递增xx
    }
    yy += 2;
}
```

### Rust代码问题（修复前）

```rust
for j in 0..height {
    let yy = offset_y + j * 2;
    for i in 0..width {
        let xx = offset_x + i * 2;  // 每次都重新计算
        // ...
    }
}
```

**问题**：虽然数学上等价（`offset_x + i * 2` = 累加的`xx`），但C++的方式更清晰，而且在某些边界情况下可能有细微差异。

## 修复方案

修改Rust代码，使其完全匹配C++的逻辑：

```rust
for j in 0..height {
    let yy = offset_y + j * 2;
    let mut xx = offset_x;  // 每行开始时重置xx（匹配C++: int xx = xi;）
    for i in 0..width {
        // ...
        xx += 2;  // 每列后递增xx（匹配C++: xx += 2;）
    }
}
```

## 修复内容

1. **每行开始时重置xx**：`let mut xx = offset_x;`（匹配C++的`int xx = xi;`）
2. **循环内递增xx**：`xx += 2;`（匹配C++的`xx += 2;`）
3. **边界检查**：使用`break`而不是`continue`，避免跳过整行

## 预期效果

修复后，sector内部的33个差异应该消失，dPiece数组应该与C++完全一致。

## 参考

- C++代码：`Source/levels/town.cpp` FillSector() Line 33-57
- Rust代码：`rust-diablo/src/world/dungeon_map.rs` load_sector_to_dpiece() Line 346-373





