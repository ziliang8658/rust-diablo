# 三角形像素布局调试经验

## 问题描述

在实现 floor tile 渲染时，发现 Rust 渲染的 LeftTriangle 和 RightTriangle 出现大量黑色色块，像素排列与 C++ 原版不一致。

## 问题现象

1. **LeftTriangle (block 0)**: 像素位置错乱，下半部分右对齐，上半部分有偏移
2. **RightTriangle (block 1)**: 出现黑色三角形，像素排列不正确

## 根本原因

Rust 解码器输出的像素布局与 C++ 渲染器期望的布局不一致：

| 类型 | Rust 解码器输出 | C++ 渲染期望 | 差异 |
|------|----------------|-------------|------|
| **LeftTriangle** | 左对齐（紧凑） | 右对齐/有偏移 | 下半部分需要右对齐，上半部分需要左偏移 |
| **RightTriangle** | 右对齐 | 左对齐 | 所有行都需要从右移到左 |

## 调试过程

### 阶段1: 缩小问题范围

**策略**: 只渲染特定 piece/frame，便于对比

```rust
// 只渲染 piece 311
if level_piece_id == 311 {
    // render...
}

// 只渲染 frame 706 (LeftTriangle)
if frame == 706 {
    // render...
}
```

**C++ 对应代码**:
```cpp
// Source/engine/render/scrollrt.cpp
static bool g_foundPiece311 = false;
if (levelPieceId == 311) {
    g_foundPiece311 = true;
}
if (g_foundPiece311 && levelPieceId != 311) {
    return; // Skip
}
```

**学习要点**:
- 使用静态变量跟踪状态，找到第一个目标后只渲染该目标
- 这样可以快速定位问题，避免大量无关输出干扰

### 阶段2: 输出像素值对比

**策略**: 将 C++ 和 Rust 的像素值输出到文件，逐行对比

**Rust 实现**:
```rust
// rust-diablo/src/world/mod.rs
if current_frame == 706 && block_index == 0 {
    use std::io::Write;
    if let Ok(mut file) = std::fs::File::create("rust_frame706_left.txt") {
        for row in 0..height as usize {
            let start = row * width as usize;
            let end = start + width as usize;
            let row_data: Vec<String> = indexed_pixels[start..end]
                .iter()
                .map(|p| p.to_string())
                .collect();
            let _ = writeln!(file, "row {}: {}", row, row_data.join(","));
        }
    }
}
```

**C++ 实现**:
```cpp
// Source/engine/render/scrollrt.cpp
if (frame == 706) {
    std::ofstream file("cpp_frame706_left.txt");
    for (int row = 0; row < height; ++row) {
        int y = targetBufferPosition.y - row; // C++ 从底部向上渲染
        file << "row " << row << ": ";
        for (int col = 0; col < width; ++col) {
            uint8_t pixel = *out.at(x, y);
            file << static_cast<int>(pixel);
        }
        file << "\n";
    }
}
```

**发现的关键差异**:
1. **Row 0**: Rust 第1-2列是 119,121，C++ 第31-32列是 119,121
2. **Row 1**: Rust 第1-4列是 110,119,124,126，C++ 第29-32列是 110,119,124,126
3. **Row 16**: Rust 第1-2列开始，C++ 第3-4列开始
4. **Row 17**: Rust 第1-4列开始，C++ 第5-8列开始

**学习要点**:
- 注意 C++ 渲染是从底部向上（`y = targetBufferPosition.y - row`），Rust 存储是从顶部向下
- 逐行对比能精确定位像素偏移量

### 阶段3: 推导转换公式

**观察结果**:
- **LeftTriangle 下半部分 (row 0-15)**: Rust 左对齐 → C++ 右对齐
  - Row 0: 宽度2，偏移30 (32-2)
  - Row 1: 宽度4，偏移28 (32-4)
  - Row 15: 宽度32，偏移0 (32-32)
  - 公式: `offset = 32 - 2*(row+1)`

- **LeftTriangle 上半部分 (row 16-30)**: Rust 左对齐 → C++ 左偏移
  - Row 16: 偏移2 (2*(16-15))
  - Row 17: 偏移4 (2*(17-15))
  - Row 30: 偏移30 (2*(30-15))
  - 公式: `offset = 2*(row-15)`

- **RightTriangle**: Rust 右对齐 → C++ 左对齐
  - 所有行都需要从右移到左
  - 公式: `src_offset = 32 - pixel_width`, 复制到 `dst_offset = 0`

## 最终解决方案

### LeftTriangle 像素重排

```rust
// rust-diablo/src/world/mod.rs
if tile_type_num == TileType::LeftTriangle as u8 {
    let mut rearranged = vec![0u8; indexed_pixels.len()];
    for row in 0..31usize {
        let row_start = row * 32;
        if row <= 15 {
            // Lower half: width = 2*(row+1), right-align
            let pixel_width = 2 * (row + 1);
            let offset = 32 - pixel_width;
            for i in 0..pixel_width {
                rearranged[row_start + offset + i] = indexed_pixels[row_start + i];
            }
        } else {
            // Upper half: left-pad with offset = 2*(row-15)
            let offset = 2 * (row - 15);
            let pixel_width = 32 - offset;
            for i in 0..pixel_width {
                rearranged[row_start + offset + i] = indexed_pixels[row_start + i];
            }
        }
    }
    indexed_pixels = rearranged;
}
```

### RightTriangle 像素重排

```rust
else if tile_type_num == TileType::RightTriangle as u8 {
    // RightTriangle: Rust decoder outputs RIGHT-aligned, C++ renders LEFT-aligned
    let mut rearranged = vec![0u8; indexed_pixels.len()];
    for row in 0..31usize {
        let row_start = row * 32;
        let pixel_width = if row <= 15 {
            2 * (row + 1)  // 2, 4, 6, ..., 32
        } else {
            32 - 2 * (row - 15)  // 30, 28, 26, ..., 2
        };
        // Rust decoder has pixels at the END of row (right-aligned)
        // C++ expects pixels at the START of row (left-aligned)
        let src_offset = 32 - pixel_width;
        for i in 0..pixel_width {
            rearranged[row_start + i] = indexed_pixels[row_start + src_offset + i];
        }
    }
    indexed_pixels = rearranged;
}
```

## 参考代码位置

### C++ 原始实现

**解码器**:
- `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsLeftTriangle()` (Line 39-58)
- `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsRightTriangle()` (Line 75-92)

**渲染器**:
- `Source/engine/render/dun_render.cpp::RenderTileFrame()` (Line 1052+)
- `Source/engine/render/scrollrt.cpp::DrawFloorTile()` (Line 687-726)

### Rust 实现

**解码器**:
- `rust-diablo/src/tiles/decoder/triangle.rs::decode_left_triangle()`
- `rust-diablo/src/tiles/decoder/triangle.rs::decode_right_triangle()`

**渲染器（修复位置）**:
- `rust-diablo/src/world/mod.rs::render_micro_tile()` (Line 934-975)

## 关键学习点

### 1. 像素布局差异的来源

**为什么会有差异？**

- **C++ 解码器**: `ReencodeDungeonCels` 在加载时预处理，移除 padding，输出特定布局
- **Rust 解码器**: 按原始 CEL 格式解码，输出紧凑的左对齐或右对齐数据
- **C++ 渲染器**: 期望特定布局（LeftTriangle 右对齐/有偏移，RightTriangle 左对齐）

**解决方案**: 在渲染前进行像素重排，将 Rust 解码器的输出转换为 C++ 渲染器期望的格式。

### 2. 调试技巧

**分层调试**:
1. 先缩小范围（只渲染特定 piece/frame）
2. 再输出数据对比（文件输出）
3. 最后推导公式（数学规律）

**对比方法**:
- 逐行对比像素值
- 注意行顺序（C++ 从底部向上，Rust 从顶部向下）
- 记录偏移量和宽度规律

### 3. 三角形像素布局规律

**LeftTriangle (32×31)**:
```
Row 0-15:  宽度递增 (2, 4, 6, ..., 32)，C++ 期望右对齐
Row 16-30: 宽度递减 (30, 28, 26, ..., 2)，C++ 期望左偏移
```

**RightTriangle (32×31)**:
```
Row 0-15:  宽度递增 (2, 4, 6, ..., 32)，C++ 期望左对齐
Row 16-30: 宽度递减 (30, 28, 26, ..., 2)，C++ 期望左对齐
```

### 4. 性能考虑

**当前实现**: 每次渲染都进行像素重排（O(992) 操作）

**优化方向**:
- 可以在解码时直接输出 C++ 兼容格式，避免运行时重排
- 或者缓存重排后的结果（但需要管理缓存大小）

**权衡**: 
- 当前方案：简单直接，易于调试
- 优化方案：需要修改解码器，可能影响其他使用场景

## 踩坑点总结

### 1. 行顺序混淆

**问题**: C++ 渲染从底部向上，Rust 存储从顶部向下

**解决**: 对比时注意行索引的含义，C++ 的 `row 0` 是底部，Rust 的 `row 0` 是顶部

### 2. 对齐方式理解错误

**问题**: 最初以为 RightTriangle 不需要重排

**原因**: 没有仔细检查 `decode_right_triangle` 的实现，它输出的是右对齐数据

**解决**: 仔细阅读解码器代码，确认输出格式

### 3. 调试代码未清理

**问题**: 调试时添加了大量过滤和输出代码

**解决**: 修复后及时清理，保留核心修复逻辑

## 相关文档

- [三角形解码器实现](../step-6-tile-formats.md#三角形解码)
- [Floor Tile 渲染流程](../step-6.2-map-tile-rendering.md)
- [C++ 渲染架构分析](../cpp_rendering_architecture.md)

## 总结

这次调试的核心是理解**数据格式转换**：Rust 解码器输出的是"紧凑格式"，而 C++ 渲染器期望的是"渲染格式"。通过在渲染前进行像素重排，我们成功解决了像素布局不一致的问题。

**关键经验**:
1. 缩小问题范围，快速定位
2. 输出数据对比，精确分析
3. 推导数学规律，实现转换
4. 及时清理调试代码，保持代码整洁

---

**创建日期**: 2025-01-XX  
**最后更新**: 2025-01-XX  
**相关 Issue/Bug**: Triangle pixel layout mismatch


