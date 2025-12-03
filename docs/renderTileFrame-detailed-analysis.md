# RenderTileFrame 函数详细解析

## 函数签名

```cpp
DVL_ATTRIBUTE_HOT void RenderTileFrame(
    const Surface &out,           // 输出缓冲区（目标渲染表面）
    const Lightmap &lightmap,      // 光照贴图（用于每像素光照）
    const Point &position,         // 渲染位置（屏幕坐标）
    TileType tile,                 // 瓦片类型（LeftTriangle, RightTriangle, Square等）
    const uint8_t *src,            // 源数据（解码后的像素数据）
    int_fast16_t height,           // 瓦片高度（像素数，如31）
    MaskType maskType,             // 遮罩类型（Solid, Transparent, Left, Right）
    const uint8_t *tbl             // 光照查找表（Light Table）
)
```

**位置**: `Source/engine/render/dun_render.cpp:1048-1086`

## 函数执行流程

### 1. 调试偏移（可选）

```cpp
#ifdef DEBUG_RENDER_OFFSET_X
    position.x += DEBUG_RENDER_OFFSET_X;
#endif
#ifdef DEBUG_RENDER_OFFSET_Y
    position.y += DEBUG_RENDER_OFFSET_Y;
#endif
```

**作用**: 在调试模式下，可以添加额外的偏移量来调整渲染位置。这通常用于调试渲染问题。

### 2. 计算裁剪区域（Clipping）

```cpp
const Clip clip = CalculateClip(position.x, position.y, DunFrameWidth, height, out);
```

**CalculateClip 函数** (`Source/engine/render/dun_render.cpp:248-258`):

```cpp
struct Clip {
    int_fast16_t top;      // 顶部裁剪量（从顶部被裁剪的像素数）
    int_fast16_t bottom;   // 底部裁剪量（从底部被裁剪的像素数）
    int_fast16_t left;     // 左侧裁剪量（从左侧被裁剪的像素数）
    int_fast16_t right;    // 右侧裁剪量（从右侧被裁剪的像素数）
    int_fast16_t width;   // 裁剪后的宽度
    int_fast16_t height;  // 裁剪后的高度
};

DVL_ALWAYS_INLINE Clip CalculateClip(
    int_fast16_t x,      // 渲染位置的X坐标
    int_fast16_t y,      // 渲染位置的Y坐标
    int_fast16_t w,      // 瓦片宽度（DunFrameWidth = 32）
    int_fast16_t h,      // 瓦片高度（如31）
    const Surface &out   // 输出表面
)
{
    Clip clip;
    // 顶部裁剪：如果 y+1 < h，说明瓦片顶部超出了屏幕顶部
    clip.top = y + 1 < h ? h - (y + 1) : 0;
    
    // 底部裁剪：如果 y+1 > out.h()，说明瓦片底部超出了屏幕底部
    clip.bottom = y + 1 > out.h() ? (y + 1) - out.h() : 0;
    
    // 左侧裁剪：如果 x < 0，说明瓦片左侧超出了屏幕左侧
    clip.left = x < 0 ? -x : 0;
    
    // 右侧裁剪：如果 x + w > out.w()，说明瓦片右侧超出了屏幕右侧
    clip.right = x + w > out.w() ? x + w - out.w() : 0;
    
    // 计算裁剪后的实际尺寸
    clip.width = w - clip.left - clip.right;
    clip.height = h - clip.top - clip.bottom;
    
    return clip;
}
```

**关键点**:
- **Y坐标系统**: 注意 `y + 1` 的使用。这是因为 isometric 渲染中，Y坐标可能从底部开始计算。
- **裁剪目的**: 确保只渲染屏幕可见区域内的像素，避免越界访问。

**示例**:
```
假设:
- position = (10, 5)
- DunFrameWidth = 32, height = 31
- out.w() = 640, out.h() = 480

如果瓦片完全在屏幕内:
- clip.top = 0
- clip.bottom = 0
- clip.left = 0
- clip.right = 0
- clip.width = 32
- clip.height = 31

如果瓦片部分超出右边界 (x=620, w=32, out.w()=640):
- clip.right = 620 + 32 - 640 = 12
- clip.width = 32 - 0 - 12 = 20 (只渲染前20个像素)
```

### 3. 早期退出检查

```cpp
if (clip.width <= 0 || clip.height <= 0) return;
```

**作用**: 如果裁剪后的宽度或高度为0或负数，说明瓦片完全在屏幕外，直接返回，不进行渲染。

### 4. 计算目标缓冲区指针

```cpp
uint8_t *dst = out.at(static_cast<int>(position.x + clip.left), 
                      static_cast<int>(position.y - clip.bottom));
const uint16_t dstPitch = out.pitch();
```

**关键点**:
- **`out.at(x, y)`**: 返回指向表面坐标 `(x, y)` 的像素指针。
- **`position.x + clip.left`**: 调整X坐标，跳过左侧被裁剪的部分。
- **`position.y - clip.bottom`**: 调整Y坐标。注意是**减法**，因为Y坐标可能从底部开始。
- **`out.pitch()`**: 返回每行的字节数（stride）。这通常等于 `width * bytes_per_pixel`，但可能包含额外的填充字节。

**内存布局示例**:
```
假设 dstPitch = 640 (每行640字节)
屏幕布局:
Row 0: [pixel0][pixel1]...[pixel639]
Row 1: [pixel0][pixel1]...[pixel639]
...

访问像素 (x, y):
dst = out.at(x, y)
// 实际内存地址 = base_address + y * dstPitch + x * bytes_per_pixel
```

### 5. 渲染统计（可选）

```cpp
#ifdef DUN_RENDER_STATS
    ++DunRenderStats[DunRenderType { tile, maskType }];
#endif
```

**作用**: 在启用统计时，记录每种瓦片类型和遮罩类型的渲染次数。

### 6. 根据 MaskType 分发渲染

```cpp
switch (maskType) {
case MaskType::Solid:
    RenderTileDispatch</*Transparent=*/false>(tile, dst, dstPitch, src, tbl, lightmap, clip);
    break;
case MaskType::Transparent:
    RenderTileDispatch</*Transparent=*/true>(tile, dst, dstPitch, src, tbl, lightmap, clip);
    break;
case MaskType::Left:
    RenderLeftTrapezoidOrTransparentSquareDispatch<MaskType::Left>(tile, dst, dstPitch, src, tbl, lightmap, clip);
    break;
case MaskType::Right:
    RenderRightTrapezoidOrTransparentSquareDispatch<MaskType::Right>(tile, dst, dstPitch, src, tbl, lightmap, clip);
    break;
}
```

#### MaskType 说明

**MaskType::Solid** (`Source/engine/render/dun_render.hpp:28-30`):
- 整个瓦片完全不透明
- 用于普通地板、墙壁等

**MaskType::Transparent** (`Source/engine/render/dun_render.hpp:32-33`):
- 整个瓦片使用透明度混合
- 用于半透明物体

**MaskType::Left** (`Source/engine/render/dun_render.hpp:35-61`):
- 上半部分的**右上角三角形**使用透明度混合
- 只能用于 `LeftTrapezoid` 和 `TransparentSquare`
- 下半部分16行完全不透明，上半部分16行右上角逐渐透明

**MaskType::Right** (`Source/engine/render/dun_render.hpp:63-89`):
- 上半部分的**左上角三角形**使用透明度混合
- 只能用于 `RightTrapezoid` 和 `TransparentSquare`
- 下半部分16行完全不透明，上半部分16行左上角逐渐透明

#### RenderTileDispatch 函数

**位置**: `Source/engine/render/dun_render.cpp:925-937`

```cpp
template <LightType Light, bool Transparent>
void RenderTileType(..., TileType tile, ...)
{
    switch (tile) {
    case TileType::Square:
        RenderSquare<Light, Transparent>(dst, dstPitch, src, tbl, lightmap, clip);
        break;
    case TileType::TransparentSquare:
        RenderTransparentSquare<Light, Transparent>(dst, dstPitch, src, tbl, lightmap, clip);
        break;
    case TileType::LeftTriangle:
        RenderLeftTriangle<Light, Transparent>(dst, dstPitch, src, tbl, &lightmap, clip);
        break;
    case TileType::RightTriangle:
        RenderRightTriangle<Light, Transparent>(dst, dstPitch, src, tbl, &lightmap, clip);
        break;
    case TileType::LeftTrapezoid:
        RenderLeftTrapezoid<Light, Transparent ? MaskType::Transparent : MaskType::Solid>
            (dst, dstPitch, src, tbl, lightmap, clip);
        break;
    case TileType::RightTrapezoid:
        RenderRightTrapezoid<Light, Transparent ? MaskType::Transparent : MaskType::Solid>
            (dst, dstPitch, src, tbl, lightmap, clip);
        break;
    }
}
```

**关键点**:
- 根据 `TileType` 调用相应的渲染函数
- `Light` 模板参数控制光照类型（PerPixel, FullyLit, FullyDark, PartiallyLit）
- `Transparent` 模板参数控制是否使用透明度混合

### 7. 调试字符串显示（可选）

```cpp
#ifdef DEBUG_STR
    const auto [debugStr, flags] = GetTileDebugStr(tile);
    DrawString(out, debugStr, 
               Rectangle { Point { position.x + 2, position.y - 29 }, Size { 28, 28 } }, 
               { .flags = flags });
#endif
```

**作用**: 在调试模式下，在瓦片上绘制一个字符标识：
- `LeftTriangle`: "<" (右对齐)
- `RightTriangle`: ">" (左对齐)
- `Square`: "S" (居中)
- `TransparentSquare`: "T" (居中)
- `LeftTrapezoid`: "\" (居中)
- `RightTrapezoid`: "/" (居中)

## 关键数据结构

### Surface 接口

```cpp
class Surface {
    uint8_t *at(int x, int y);  // 获取 (x, y) 位置的像素指针
    uint16_t pitch() const;      // 返回每行的字节数（stride）
    int w() const;               // 返回宽度
    int h() const;              // 返回高度
};
```

### Lightmap

用于每像素光照计算。包含光照强度信息，用于动态调整像素亮度。

### Light Table (tbl)

预计算的光照查找表，用于快速应用光照效果。每个像素值通过查找表映射到新的颜色值。

## 渲染流程总结

1. **输入验证**: 检查位置和尺寸
2. **裁剪计算**: 计算需要渲染的可见区域
3. **早期退出**: 如果完全不可见，直接返回
4. **目标指针**: 计算目标缓冲区的起始位置
5. **类型分发**: 根据 `MaskType` 和 `TileType` 调用相应的渲染函数
6. **像素绘制**: 实际的像素复制和光照应用在具体的渲染函数中完成

## 在 DrawFloorTile 中的使用

```cpp
void DrawFloorTile(..., Point targetBufferPosition)
{
    // Block 0: 使用 LeftTriangle，位置在 targetBufferPosition
    RenderTileFrame(out, lightmap, targetBufferPosition, 
                    TileType::LeftTriangle, src, 
                    DunFrameTriangleHeight, MaskType::Solid, tbl);
    
    // Block 1: 使用 RightTriangle，位置在 targetBufferPosition + RightFrameDisplacement
    RenderTileFrame(out, lightmap, targetBufferPosition + RightFrameDisplacement, 
                    TileType::RightTriangle, src, 
                    DunFrameTriangleHeight, MaskType::Solid, tbl);
}
```

**关键点**:
- 两个 block 使用相同的 `MaskType::Solid`（完全不透明）
- 位置相差 `RightFrameDisplacement = {32, 0}`（X方向偏移32像素）
- 高度都是 `DunFrameTriangleHeight = 31`

## 与 Rust 实现的对应关系

在 Rust 中，`RenderTileFrame` 的功能应该对应：

1. **裁剪计算**: 需要实现类似的 `CalculateClip` 逻辑
2. **目标指针**: 需要计算正确的缓冲区偏移
3. **类型分发**: 根据 `TileType` 和 `MaskType` 调用相应的解码和渲染函数
4. **像素绘制**: 使用引擎的 `draw_rgba_texture` 或类似函数

## 注意事项

1. **Y坐标系统**: C++ 使用 `position.y - clip.bottom`，说明Y坐标可能从底部开始或使用特殊的坐标系
2. **裁剪逻辑**: 必须正确处理边界情况，避免越界访问
3. **Pitch**: 必须使用 `pitch()` 而不是 `width * bytes_per_pixel`，因为可能有填充字节
4. **模板参数**: C++ 使用模板来优化不同光照和透明度组合的性能





