# C++ Block 渲染逻辑详细分析

## 概述

本文档详细分析 C++ 中 `DrawFloorTile` 函数如何渲染 floor tiles 的 block 0 和 block 1。

## C++ DrawFloorTile 函数分析

### 函数位置
`Source/engine/render/scrollrt.cpp:653-678`

### 完整代码
```cpp
void DrawFloorTile(const Surface &out, const Lightmap &lightmap, Point tilePosition, Point targetBufferPosition)
{
    const int lightTableIndex = dLight[tilePosition.x][tilePosition.y];
    const uint8_t *tbl = LightTables[lightTableIndex].data();
    
    const uint16_t levelPieceId = dPiece[tilePosition.x][tilePosition.y];
    
    // Block 0 (mt[0])
    {
        const LevelCelBlock levelCelBlock { DPieceMicros[levelPieceId].mt[0] };
        if (levelCelBlock.hasValue()) {
            RenderTileFrame(out, lightmap, targetBufferPosition, TileType::LeftTriangle,
                GetDunFrame(pDungeonCels.get(), levelCelBlock.frame()), 
                DunFrameTriangleHeight, MaskType::Solid, tbl);
        }
    }
    
    // Block 1 (mt[1])
    {
        const LevelCelBlock levelCelBlock { DPieceMicros[levelPieceId].mt[1] };
        if (levelCelBlock.hasValue()) {
            RenderTileFrame(out, lightmap, targetBufferPosition + RightFrameDisplacement, TileType::RightTriangle,
                GetDunFrame(pDungeonCels.get(), levelCelBlock.frame()), 
                DunFrameTriangleHeight, MaskType::Solid, tbl);
        }
    }
}
```

## 关键信息

### 1. Block 0 的渲染
- **数据来源**: `DPieceMicros[levelPieceId].mt[0]`
- **位置**: `targetBufferPosition` (即 `screen_x, screen_y`)
- **强制类型**: `TileType::LeftTriangle`（忽略 block 中存储的实际类型）
- **高度**: `DunFrameTriangleHeight = 31` 像素
- **Frame**: 使用 `levelCelBlock.frame()`（从 block 数据中读取）

### 2. Block 1 的渲染
- **数据来源**: `DPieceMicros[levelPieceId].mt[1]`
- **位置**: `targetBufferPosition + RightFrameDisplacement`
  - `RightFrameDisplacement = { DunFrameWidth, 0 } = { 32, 0 }`
  - 即 `screen_x + 32, screen_y`
- **强制类型**: `TileType::RightTriangle`（忽略 block 中存储的实际类型）
- **高度**: `DunFrameTriangleHeight = 31` 像素
- **Frame**: 使用 `levelCelBlock.frame()`（从 block 数据中读取）

## LeftTriangle vs RightTriangle 的区别

### LeftTriangle（左对齐三角形）
- **定义**: `Source/levels/dun_tile.hpp:37-48`
- **描述**: "Left-pointing 32x31 triangle"
- **像素布局**: 左对齐，右边填充透明
  ```
  Row 0:  [##]                            (左边2像素，右边30透明)
  Row 1:  [####]                          (左边4像素，右边28透明)
  ...
  Row 15: [##############################] (32像素填满)
  Row 16: [############################]  (左边30像素，右边2透明)
  ...
  Row 30: [##]                            (左边2像素，右边30透明)
  ```
- **渲染特点**: 
  - `RenderLeftTriangleLower`: `dst += XStep * (LowerHeight - 1);` 然后使用 `dstPitch + XStep`
  - 从右边开始绘制（有偏移）

### RightTriangle（右对齐三角形）
- **定义**: `Source/levels/dun_tile.hpp:50-61`
- **描述**: "Right-pointing 32x31 triangle"
- **像素布局**: 右对齐，左边填充透明
  ```
  Row 0:                              [##] (左边30透明，右边2像素)
  Row 1:                          [####]   (左边28透明，右边4像素)
  ...
  Row 15: [##############################] (32像素填满)
  Row 16: [############################]  (左边2透明，右边30像素)
  ...
  Row 30:                              [##] (左边30透明，右边2像素)
  ```
- **渲染特点**:
  - `RenderRightTriangleLower`: 直接使用 `dstPitch`（没有偏移）
  - 从左边开始绘制

## 渲染流程

### RenderTileFrame 函数
位置: `Source/engine/render/dun_render.cpp:1048-1086`

```cpp
void RenderTileFrame(..., TileType tile, const uint8_t *src, int_fast16_t height, MaskType maskType, ...)
{
    // 计算裁剪区域
    const Clip clip = CalculateClip(position.x, position.y, DunFrameWidth, height, out);
    
    // 根据 MaskType 调用不同的渲染函数
    switch (maskType) {
    case MaskType::Solid:
        RenderTileDispatch</*Transparent=*/false>(tile, dst, dstPitch, src, tbl, lightmap, clip);
        break;
    // ...
    }
}
```

### RenderTileDispatch 函数
位置: `Source/engine/render/dun_render.cpp:925-937`

```cpp
template <LightType Light, bool Transparent>
void RenderTileType(..., TileType tile, ...)
{
    switch (tile) {
    case TileType::LeftTriangle:
        RenderLeftTriangle<Light, Transparent>(dst, dstPitch, src, tbl, lightmap, clip);
        break;
    case TileType::RightTriangle:
        RenderRightTriangle<Light, Transparent>(dst, dstPitch, src, tbl, lightmap, clip);
        break;
    // ...
    }
}
```

## 关键发现

### 1. 位置映射
- **Block 0**: 屏幕位置 `(x, y)`，使用 `LeftTriangle`
- **Block 1**: 屏幕位置 `(x+32, y)`，使用 `RightTriangle`

### 2. 类型强制
- C++ **强制**使用 `LeftTriangle` 和 `RightTriangle`，**完全忽略** block 中存储的实际 `TileType`
- 这意味着即使 block 中存储的是 `Square` 或其他类型，也会被当作 Triangle 来解码和渲染

### 3. Frame 使用
- C++ 使用 block 中存储的 `frame()` 值来获取 CEL 数据
- 但解码时使用强制的 Triangle 类型

### 4. 像素布局差异
- **LeftTriangle**: 像素在左边，右边透明（左对齐）
- **RightTriangle**: 像素在右边，左边透明（右对齐）
- 两个三角形组合在一起应该形成一个完整的菱形 tile

## 可能的顺序问题

如果渲染结果看起来顺序颠倒，可能的原因：

1. **Block 位置交换**: Block 0 和 Block 1 的屏幕位置应该交换
2. **Triangle 类型交换**: Block 0 应该用 `RightTriangle`，Block 1 应该用 `LeftTriangle`
3. **解码问题**: Rust 的解码器可能对 Left/Right Triangle 的理解与 C++ 不同

## 验证方法

1. 检查 C++ 调试输出中 block 0 和 block 1 的 frame 值
2. 检查 Rust 调试输出中相同位置的 frame 值
3. 对比两个 frame 值对应的实际图像内容
4. 确认 LeftTriangle 和 RightTriangle 在 Rust 中的解码是否正确

## 参考代码位置

- `DrawFloorTile`: `Source/engine/render/scrollrt.cpp:653-678`
- `RenderTileFrame`: `Source/engine/render/dun_render.cpp:1048-1086`
- `RenderLeftTriangle`: `Source/engine/render/dun_render.cpp:617-631`
- `RenderRightTriangle`: `Source/engine/render/dun_render.cpp:678-692`
- `RightFrameDisplacement`: `Source/engine/render/scrollrt.cpp:100` = `{32, 0}`
- `DunFrameTriangleHeight`: `Source/levels/dun_tile.hpp:127` = `31`





