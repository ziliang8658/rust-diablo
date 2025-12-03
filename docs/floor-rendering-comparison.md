# Floor Block 渲染逻辑对比分析

## 概述

本文档详细对比 Rust 和 C++ 中绘制 floor block 的实现差异，重点关注 `draw_floor_at` (Rust) 和 `DrawFloorTile` (C++) 两个函数。

## 调用流程对比

### C++ 调用流程

```
DrawFloor() [scrollrt.cpp:928]
  └─> for each tile:
      └─> if (IsFloor(tilePosition)):
          └─> DrawFloorTile() [scrollrt.cpp:653]
              ├─> 获取 lightTableIndex = dLight[tilePosition.x][tilePosition.y]
              ├─> 获取 light table: tbl = LightTables[lightTableIndex].data()
              ├─> 获取 levelPieceId = dPiece[tilePosition.x][tilePosition.y]
              ├─> Block 0: 
              │   └─> if (levelCelBlock.hasValue()):
              │       └─> RenderTileFrame(
              │           tileType = TileType::LeftTriangle,
              │           height = DunFrameTriangleHeight (31),
              │           position = targetBufferPosition,
              │           maskType = MaskType::Solid,
              │           lightTable = tbl
              │       )
              └─> Block 1:
                  └─> if (levelCelBlock.hasValue()):
                      └─> RenderTileFrame(
                          tileType = TileType::RightTriangle,
                          height = DunFrameTriangleHeight (31),
                          position = targetBufferPosition + RightFrameDisplacement (x+32, y),
                          maskType = MaskType::Solid,
                          lightTable = tbl
                      )
```

### Rust 调用流程

```
render_with_texture_manager() [world/mod.rs:293]
  └─> for each tile:
      └─> if (is_floor):
          └─> draw_floor_at() [world/mod.rs:636]
              ├─> Block 0:
              │   └─> render_micro_tile(
              │       level_piece_id, 
              │       block_index = 0,
              │       screen_x, 
              │       screen_y
              │   )
              └─> Block 1:
                  └─> render_micro_tile(
                      level_piece_id,
                      block_index = 1,
                      screen_x + 32,
                      screen_y
                  )
```

## 详细差异分析

### 1. Block 位置计算

#### C++ 实现
```cpp
// Block 0: 位置 = targetBufferPosition
RenderTileFrame(out, lightmap, targetBufferPosition, ...);

// Block 1: 位置 = targetBufferPosition + RightFrameDisplacement
// RightFrameDisplacement = { DunFrameWidth, 0 } = { 32, 0 }
RenderTileFrame(out, lightmap, targetBufferPosition + RightFrameDisplacement, ...);
```

#### Rust 实现
```rust
// Block 0: 位置 = screen_x, screen_y
self.render_micro_tile(engine, texture_mgr, level_piece_id, 0, screen_x, screen_y)?;

// Block 1: 位置 = screen_x + 32, screen_y
self.render_micro_tile(engine, texture_mgr, level_piece_id, 1, screen_x + 32, screen_y)?;
```

**差异**: ✅ **一致** - 两者都是 block 0 在原始位置，block 1 在 x+32 的位置。

---

### 2. TileType 处理

#### C++ 实现
```cpp
// C++ 显式指定 TileType
RenderTileFrame(..., TileType::LeftTriangle, ...);   // Block 0
RenderTileFrame(..., TileType::RightTriangle, ...);  // Block 1
```

#### Rust 实现
```rust
// Rust 从 block 数据中读取 TileType
// render_micro_tile 内部会使用 block.tile_type()
// 但注释说："The TileType stored in the block should already be correct (Triangle for floors)"
```

**差异**: ⚠️ **潜在差异** - C++ 强制使用 `LeftTriangle`/`RightTriangle`，而 Rust 依赖 block 中存储的 TileType。如果 block 中的 TileType 不正确，可能导致渲染差异。

---

### 3. 高度处理

#### C++ 实现
```cpp
// C++ 显式指定高度为 31 (DunFrameTriangleHeight)
RenderTileFrame(..., DunFrameTriangleHeight, ...);
// DunFrameTriangleHeight = 31
```

#### Rust 实现
```rust
// Rust 从解码后的像素数据计算高度
let pixel_count = rgba_pixels.len() / 4;
let width = 32u32;
let height = (pixel_count / 32) as u32;  // 从像素数据推断
```

**差异**: ⚠️ **潜在差异** - C++ 固定使用 31 像素高度，而 Rust 从解码数据推断。如果解码数据不正确，高度可能不匹配。

---

### 4. 光照处理 (Lighting)

#### C++ 实现
```cpp
// 1. 获取光照表索引
const int lightTableIndex = dLight[tilePosition.x][tilePosition.y];

// 2. 获取光照表数据
const uint8_t *tbl = LightTables[lightTableIndex].data();

// 3. RenderTileFrame 内部使用 lightmap 和 light table 进行光照混合
RenderTileFrame(out, lightmap, position, tileType, src, height, maskType, tbl);
// 内部会调用 RenderTileDispatch，使用 tbl 进行颜色混合
```

#### Rust 实现
```rust
// Rust 直接绘制解码后的 RGBA 像素，没有光照处理
engine.draw_rgba_texture(&texture_id, rgba_pixels, width, height, rect)?;
// 没有使用 lightmap 或 light table
```

**差异**: ❌ **重大差异** - C++ 使用光照系统（lightmap + light table）进行颜色混合，而 Rust 直接绘制原始颜色。这会导致 Rust 渲染的 floor 看起来更亮或颜色不正确。

---

### 5. MaskType 处理

#### C++ 实现
```cpp
// C++ 使用 MaskType::Solid 进行渲染
RenderTileFrame(..., MaskType::Solid, ...);
// MaskType::Solid 表示完全不透明，没有透明度混合
```

#### Rust 实现
```rust
// Rust 没有显式的 MaskType 概念
// 透明度处理在解码阶段完成（RGBA 格式）
// 如果所有像素都透明，会跳过渲染
let has_visible_pixels = rgba_pixels.chunks(4).any(|p| p[3] > 0);
if !has_visible_pixels {
    return Ok(());
}
```

**差异**: ⚠️ **概念差异** - C++ 使用 MaskType 控制透明度混合方式，Rust 使用 RGBA alpha 通道。对于 floor tiles（MaskType::Solid），两者应该等效，但实现方式不同。

---

### 6. 数据获取方式

#### C++ 实现
```cpp
// 直接从 DPieceMicros 数组获取原始 CEL 数据
const LevelCelBlock levelCelBlock { DPieceMicros[levelPieceId].mt[0] };
const uint8_t *src = GetDunFrame(pDungeonCels.get(), levelCelBlock.frame());
// src 是压缩的 CEL 格式数据，需要在渲染时解码
```

#### Rust 实现
```rust
// 从 TileTextureManager 获取预解码的 RGBA 数据
match texture_mgr.borrow_mut().get_decoded_tile(level_piece_id, block_index) {
    Ok(rgba_pixels) => {
        // rgba_pixels 是已经解码的 RGBA 格式
        engine.draw_rgba_texture(..., rgba_pixels, ...)?;
    }
}
```

**差异**: ⚠️ **实现差异** - C++ 在渲染时实时解码 CEL 数据，Rust 使用预解码的 RGBA 数据。这可能导致性能差异，但理论上结果应该一致（如果解码正确）。

---

### 7. 错误处理

#### C++ 实现
```cpp
// C++ 使用简单的 if 检查
if (levelCelBlock.hasValue()) {
    RenderTileFrame(...);
}
// 如果 hasValue() 为 false，直接跳过，不渲染
```

#### Rust 实现
```rust
// Rust 有更详细的错误处理
if !block.has_value() {
    return Ok(());  // 跳过渲染
}
// 如果解码失败，也会跳过
match texture_mgr.borrow_mut().get_decoded_tile(...) {
    Ok(rgba_pixels) => { ... },
    Err(_e) => { /* 静默失败 */ }
}
```

**差异**: ✅ **一致** - 两者都会在 block 没有值时跳过渲染。

---

## 关键差异总结

### 1. ❌ 光照处理缺失（最重要）

**C++**: 使用 `lightmap` + `LightTables[lightTableIndex]` 进行光照混合  
**Rust**: 直接绘制原始颜色，没有光照处理

**影响**: Rust 渲染的 floor 会看起来更亮，颜色可能不正确。

**修复建议**: 需要在 Rust 中添加光照处理：
```rust
// 需要获取 lightTableIndex
let light_table_index = self.get_light_table_index(tx, ty)?;
let light_table = self.get_light_table(light_table_index)?;

// 在绘制时应用光照
engine.draw_rgba_texture_with_lighting(
    &texture_id, 
    rgba_pixels, 
    light_table,
    rect
)?;
```

### 2. ⚠️ TileType 强制指定 vs 从数据读取

**C++**: 强制使用 `TileType::LeftTriangle` / `RightTriangle`  
**Rust**: 从 block 数据中读取 TileType

**影响**: 如果 block 数据中的 TileType 不正确，可能导致渲染错误。

**修复建议**: Rust 应该也强制指定 TileType：
```rust
// 在 render_micro_tile 中，对于 floor，强制使用 Triangle 类型
// 或者验证 block.tile_type() 是否为 Triangle
```

### 3. ⚠️ 高度计算方式

**C++**: 固定使用 `DunFrameTriangleHeight = 31`  
**Rust**: 从像素数据推断 `height = pixel_count / 32`

**影响**: 如果解码数据不正确，高度可能不匹配。

**修复建议**: 对于 floor triangles，应该固定使用 31 像素高度：
```rust
// 对于 floor block，使用固定高度
let height = if is_floor_triangle { 31 } else { calculated_height };
```

### 4. ✅ 其他方面基本一致

- Block 位置计算：一致
- hasValue 检查：一致
- 错误处理：一致（Rust 更详细）

---

## 代码位置参考

### C++ 关键代码
- `DrawFloorTile()`: `Source/engine/render/scrollrt.cpp:653-678`
- `RenderTileFrame()`: `Source/engine/render/dun_render.cpp:1048-1086`
- `RightFrameDisplacement`: `Source/engine/render/scrollrt.cpp:100` = `{32, 0}`
- `DunFrameTriangleHeight`: `Source/levels/dun_tile.hpp:127` = `31`

### Rust 关键代码
- `draw_floor_at()`: `rust-diablo/src/world/mod.rs:636-653`
- `render_micro_tile()`: `rust-diablo/src/world/mod.rs:719-794`
- Block 位置: `screen_x + 32` (对应 C++ 的 `RightFrameDisplacement`)

---

## 结论

**主要问题**: Rust 缺少光照处理，这是导致渲染差异的最可能原因。其他差异（TileType、高度）虽然存在，但影响较小。

**优先级修复顺序**:
1. 🔴 **高优先级**: 添加光照处理（lightmap + light table）
2. 🟡 **中优先级**: 验证/强制 TileType 为 Triangle
3. 🟢 **低优先级**: 验证高度计算是否正确





