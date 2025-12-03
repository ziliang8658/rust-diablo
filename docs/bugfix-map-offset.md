# Bug修复：地图加载偏移量不一致

## 问题描述

在对比C++和Rust的调试输出时，发现虽然两个版本都找到了相同的 `levelPieceId=290`，但它们出现在不同的 dPiece 坐标位置：

- **C++**: `dPiece(13,40)` 
- **Rust**: `dPiece(35,18)`

而且周边 tile 的坐标和 pieceId 也不一致，导致无法正确对比渲染结果。

## 根本原因

C++ 和 Rust 加载 `sector1s.dun` 时使用了不同的偏移量：

### C++代码（正确）

参考：`Source/levels/town.cpp:202`

```cpp
FillSector("levels\\towndata\\sector1s.dun", 46, 46);
```

C++ 将 sector1s 加载到 dPiece 数组的 `(46, 46)` 位置开始。

### Rust代码（错误，已修复）

**修复前：**
```rust
// sector1s is at offset (46, 46) in the full town map
// But for testing, we load at (0, 0) to see it properly
if let Err(e) = world.load_town_sector(
    // ...
    0,   // offset_x (normally 46 for sector1s) ❌
    0,   // offset_y (normally 46 for sector1s) ❌
    // ...
)
```

Rust 将 sector1s 加载到 dPiece 数组的 `(0, 0)` 位置开始。

**修复后：**
```rust
// sector1s is at offset (46, 46) in the full town map
// Reference: Source/levels/town.cpp:202
// FillSector("levels\\towndata\\sector1s.dun", 46, 46);
if let Err(e) = world.load_town_sector(
    // ...
    46,  // offset_x (must match C++ FillSector) ✅
    46,  // offset_y (must match C++ FillSector) ✅
    // ...
)
```

## 影响

这个错误导致：
1. **坐标不一致**：同样的 tile 在 C++ 和 Rust 中出现在不同的 dPiece 坐标
2. **无法正确对比**：调试输出中的坐标和周边 tile 信息无法直接对比
3. **可能影响其他功能**：如果代码依赖特定的坐标位置，可能会出现问题

## 修复位置

- **文件**: `rust-diablo/src/game.rs`
- **函数**: 地图加载代码
- **行数**: 626-627

## Town Sector 布局

根据 C++ 代码，Town 由4个 sector 组成：

```cpp
FillSector("levels\\towndata\\sector1s.dun", 46, 46);  // 右下
FillSector("levels\\towndata\\sector2s.dun", 46, 0);   // 右上
FillSector("levels\\towndata\\sector3s.dun", 0, 46);   // 左下
FillSector("levels\\towndata\\sector4s.dun", 0, 0);    // 左上
```

```
sector4s (0,0)    |  sector2s (46,0)
------------------|------------------
sector3s (0,46)   |  sector1s (46,46)
```

## 验证

修复后，C++ 和 Rust 的调试输出应该显示相同的坐标：
- 中心 tile 的 dPiece 坐标应该相同
- 周边 8 个方向的 tile 坐标应该相同
- pieceId 和 IsFloor 值应该相同

## 注意事项

1. **玩家位置搜索**：玩家位置搜索范围 `(30..80)` 仍然有效，因为：
   - sector1s 加载到 `(46, 46)` 开始
   - sector1s 通常是 25x25 MegaTiles = 50x50 dPiece tiles
   - 所以 sector1s 覆盖 `(46..96, 46..96)` 范围
   - 搜索范围 `(30..80)` 覆盖 `(46..80, 46..80)`，可以找到合适的 tile

2. **其他 Sector**：如果以后需要加载其他 sector，也要使用正确的偏移量：
   - sector2s: `(46, 0)`
   - sector3s: `(0, 46)`
   - sector4s: `(0, 0)`

## 参考

- C++原版：`Source/levels/town.cpp:202`
- C++ FillSector实现：`Source/levels/town.cpp:26-58`
- Rust load_dun_to_dpiece实现：`rust-diablo/src/world/dungeon_map.rs:238-307`





