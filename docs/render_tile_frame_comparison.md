# RenderTileFrame (C++) vs render_micro_tile_with_height (Rust) 对比分析

## 自顶向下对比

### 1. 调用入口

#### C++: `DrawFloorTile()` → `RenderTileFrame()`
```cpp
// Source/engine/render/scrollrt.cpp:667-668
RenderTileFrame(out, lightmap, targetBufferPosition, TileType::LeftTriangle,
    GetDunFrame(pDungeonCels.get(), levelCelBlock.frame()), DunFrameTriangleHeight, MaskType::Solid, tbl);
```

#### Rust: `draw_floor_tile_at()` → `render_micro_tile_with_height()`
```rust
// rust-diablo/src/world/mod.rs:675
self.render_micro_tile_with_height(engine, texture_mgr, level_piece_id, 0, screen_x, screen_y, 31)?;
```

---

### 2. 函数签名对比

#### C++ RenderTileFrame
```cpp
// Source/engine/render/dun_render.cpp:1048
void RenderTileFrame(
    const Surface &out,              // 目标Surface（像素缓冲区）
    const Lightmap &lightmap,         // 每像素光照映射（本文档不考虑）
    const Point &position,            // 屏幕位置（左下角）
    TileType tile,                    // Tile类型（LeftTriangle/RightTriangle等）
    const uint8_t *src,               // 原始编码数据指针
    int_fast16_t height,             // Tile高度（31 for triangles）
    MaskType maskType,                // 遮罩类型（Solid/Transparent等）
    const uint8_t *tbl                // 光照表或TRN表（本文档不考虑）
)
```

#### Rust render_micro_tile_with_height
```rust
// rust-diablo/src/world/mod.rs:891
fn render_micro_tile_with_height(
    &self,
    engine: &mut Engine,              // 渲染引擎
    texture_mgr: &RefCell<TileTextureManager>,  // 纹理管理器
    level_piece_id: usize,            // Piece ID
    block_index: usize,                // Block索引（0或1）
    screen_x: i32,                    // 屏幕X坐标
    screen_y: i32,                    // 屏幕Y坐标
    fixed_height: u32,                 // 固定高度（31）
) -> Result<()>
```

**关键差异**：
- C++接收**原始编码数据指针**，需要实时解码
- Rust接收**piece_id和block_index**，从缓存获取已解码的RGBA数据

---

### 3. 执行流程对比

#### C++ RenderTileFrame 流程

```
RenderTileFrame()
├─ 1. 调试偏移（可选）
│   ├─ DEBUG_RENDER_OFFSET_X
│   └─ DEBUG_RENDER_OFFSET_Y
│
├─ 2. 计算裁剪区域
│   └─ CalculateClip(position, width, height, out)
│       ├─ clip.top: 顶部裁剪
│       ├─ clip.bottom: 底部裁剪
│       ├─ clip.left: 左侧裁剪
│       ├─ clip.right: 右侧裁剪
│       └─ clip.width/height: 有效渲染尺寸
│
├─ 3. 获取目标缓冲区指针
│   └─ dst = out.at(position.x + clip.left, position.y - clip.bottom)
│   └─ dstPitch = out.pitch()  // 行间距
│
├─ 4. 根据maskType分发渲染
│   ├─ MaskType::Solid → RenderTileDispatch<Transparent=false>
│   ├─ MaskType::Transparent → RenderTileDispatch<Transparent=true>
│   ├─ MaskType::Left → RenderLeftTrapezoidOrTransparentSquareDispatch
│   └─ MaskType::Right → RenderRightTrapezoidOrTransparentSquareDispatch
│
└─ 5. RenderTileDispatch → RenderTileType
    └─ 根据TileType调用具体渲染函数：
        ├─ TileType::Square → RenderSquare()
        ├─ TileType::LeftTriangle → RenderLeftTriangle()
        ├─ TileType::RightTriangle → RenderRightTriangle()
        └─ ... (其他类型)
    
    注：光照处理在RenderTileDispatch内部根据LightType选择不同的模板实例化
```

#### Rust render_micro_tile_with_height 流程

```
render_micro_tile_with_height()
├─ 1. 检查block是否有值
│   └─ texture_mgr.get_piece(level_piece_id).mt[block_index].has_value()
│       └─ 如果为空，直接返回Ok(())
│
├─ 2. 获取已解码的RGBA数据
│   └─ texture_mgr.get_decoded_tile(level_piece_id, block_index)
│       └─ 返回: &[u8] (RGBA像素数据，已应用调色板)
│
├─ 3. 验证数据大小
│   └─ rgba_pixels.len() == (width * height * 4)
│
├─ 4. 检查是否有可见像素
│   └─ rgba_pixels.chunks(4).any(|p| p[3] > 0)  // Alpha > 0
│
└─ 5. 直接绘制整个texture
    └─ engine.draw_rgba_texture(texture_id, rgba_pixels, width, height, rect)
        ├─ 创建SDL texture (ABGR8888格式)
        ├─ 更新texture数据
        └─ 复制到canvas
```

---

### 4. 核心差异分析

#### 4.1 数据获取方式

**C++**：
- 接收原始编码数据指针 `const uint8_t *src`
- 数据来自 `GetDunFrame(pDungeonCels.get(), levelCelBlock.frame())`
- 需要**实时解码**（在渲染循环中）

**Rust**：
- 通过 `piece_id` 和 `block_index` 查找
- 从缓存获取**预解码的RGBA数据**
- 解码在纹理管理器中完成，渲染时直接使用

#### 4.2 渲染方式

**C++**：
- **逐行渲染**，从下往上写入Surface
- 使用 `dst -= dstPitch` 向上移动指针
- 每行调用 `RenderLineTransparentOrOpaque()` 处理像素
- 支持**裁剪**、**透明度混合**

**Rust**：
- **整体绘制**，一次性绘制整个32×31的texture
- 使用SDL的texture系统
- 没有逐行处理，依赖SDL的硬件加速

#### 4.3 裁剪处理

**C++**：
- 使用 `CalculateClip()` 计算裁剪区域
- 支持顶部、底部、左侧、右侧裁剪
- 只渲染可见部分，提高性能

**Rust**：
- **依赖SDL的裁剪**
- 通过 `Rect` 指定绘制区域
- 如果tile超出屏幕，SDL会自动裁剪

#### 4.4 透明度处理

**C++**：
- 根据 `MaskType` 选择不同的混合模式：
  - `MaskType::Solid` → 不透明渲染（BlitPixelsDirect）
  - `MaskType::Transparent` → 透明混合（BlitPixelsBlended）
- 使用 `paletteTransparencyLookup` 进行透明度查找

**Rust**：
- 使用SDL的 `BlendMode::Blend`
- 所有tile都使用相同的混合模式
- 透明度信息在RGBA数据的Alpha通道中

---

### 5. 关键代码对比

#### 5.1 目标缓冲区获取

**C++**：
```cpp
// Source/engine/render/dun_render.cpp:1060
uint8_t *dst = out.at(static_cast<int>(position.x + clip.left), 
                       static_cast<int>(position.y - clip.bottom));
const uint16_t dstPitch = out.pitch();
```

**Rust**：
```rust
// rust-diablo/src/world/mod.rs:940
let rect = Rect::new(screen_x, screen_y, width, height);
engine.draw_rgba_texture(&texture_id, rgba_pixels, width, height, rect)?;
```

#### 5.2 行渲染（C++特有）

**C++**：
```cpp
// Source/engine/render/dun_render.cpp:263
for (auto i = 0; i < Height; ++i, dst -= dstPitch) {
    RenderLineTransparentOrOpaque<Light, Transparent>(dst, src, Width, tbl, &lightmap);
    src += Width;
}
// 注：lightmap参数用于光照处理，本文档不考虑
```

**注意**：`dst -= dstPitch` 表示**从下往上**写入，因为等距投影的Y轴是倒置的。

#### 5.3 数据解码（Rust特有）

**Rust**：
```rust
// rust-diablo/src/world/mod.rs:919
match texture_mgr.borrow_mut().get_decoded_tile(level_piece_id, block_index) {
    Ok(rgba_pixels) => {
        // rgba_pixels 已经是完整的32×31×4字节RGBA数据
        engine.draw_rgba_texture(&texture_id, rgba_pixels, width, height, rect)?;
    }
}
```

---

### 6. 性能影响

#### C++ 优势：
- ✅ 只渲染可见部分（裁剪）
- ✅ 精细的透明度控制
- ✅ 直接写入Surface，无额外拷贝

#### C++ 劣势：
- ❌ 每次渲染都需要解码
- ❌ 逐行处理，CPU密集
- ❌ 代码复杂，难以优化

#### Rust 优势：
- ✅ 预解码，渲染时无解码开销
- ✅ 使用SDL硬件加速
- ✅ 代码简洁，易于维护
- ✅ Texture缓存，重复tile无需重新创建

#### Rust 劣势：
- ❌ 缺少精细的裁剪控制
- ❌ 所有tile使用相同的混合模式
- ❌ 需要额外的内存存储RGBA数据

---

### 7. 潜在问题

#### 7.1 Y轴方向

**C++** 使用 `dst -= dstPitch` 从下往上写入，这是等距投影的特性。

**Rust** 直接绘制texture，需要确保：
- 解码器输出的行顺序正确（top-to-bottom）
- SDL texture的行顺序匹配

#### 7.2 坐标系统

**C++** `position.y` 是**左下角**坐标。

**Rust** `screen_y` 需要确认是**左上角**还是**左下角**。

---

### 8. 改进建议

1. **优化裁剪**：
   - 在调用 `draw_rgba_texture` 前检查tile是否在屏幕内
   - 只解码可见的tile

2. **支持不同的混合模式**：
   - 根据tile类型选择不同的BlendMode
   - Solid tiles使用不透明模式

3. **坐标系统统一**：
   - 确认并统一Y轴方向
   - 确保与C++的坐标系统一致

---

## 总结

C++的 `RenderTileFrame` 是一个**低级别的逐像素渲染函数**，提供了精细的控制（裁剪、透明度），但需要实时解码。

Rust的 `render_micro_tile_with_height` 是一个**高级别的texture绘制函数**，使用预解码数据和SDL硬件加速，代码简洁但功能较少。

两者在架构上有根本差异：C++是**立即模式渲染**（immediate mode），Rust是**保留模式渲染**（retained mode）。

**注**：本文档不考虑光照处理部分，专注于核心渲染逻辑的差异。

