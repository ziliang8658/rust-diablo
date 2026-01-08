# Floor Tile Block 解码流程详解

## 概述

本文档详细说明 floor tile 的 block 文件是如何解码的，包括从 MIN 数据到最终像素数据的完整流程。

## 解码流程

### 1. MIN 文件加载

**文件位置**：
- `rust-diablo/src/tiles/min.rs::MinData::from_bytes()`
- `Source/levels/gendung.cpp::LoadMinData()`

**数据结构**：
- MIN 文件包含多个 `LevelCelBlock`（每个 2 字节）
- 每个 piece（levelPieceId）包含 10-16 个 blocks（取决于关卡类型）
- `LevelCelBlock` 编码格式：
  - 低 12 位：frame 索引（1-based）
  - 高 4 位：TileType（Square, LeftTriangle, RightTriangle, 等）

**参考代码**：
```rust
// rust-diablo/src/tiles/min.rs
let val = u16::from_le_bytes([data[offset], data[offset + 1]]);
mt.push(LevelCelBlock::new(val));
```

```cpp
// Source/levels/gendung.cpp Line 603
const LevelCelBlock levelCelBlock { Swap16LE(pieces[blocks - 2 + (block & 1) - (block & 0xE)]) };
```

### 2. Floor Tile 识别

**判断逻辑**：
- 使用 SOL 数据判断是否为 floor tile
- `IsFloor(tilePosition) = !TileHasAny(tilePosition, TileProperties::Solid | TileProperties::BlockMissile)`

**参考代码**：
```cpp
// Source/engine/render/scrollrt.cpp Line 105-108
[[nodiscard]] DVL_ALWAYS_INLINE bool IsFloor(Point tilePosition)
{
	return !TileHasAny(tilePosition, TileProperties::Solid | TileProperties::BlockMissile);
}
```

### 3. Floor Tile 特殊处理

**关键点**：C++ 的 `DrawFloorTile()` **强制使用** `TileType::LeftTriangle` 和 `TileType::RightTriangle`，**忽略** block 中存储的实际 TileType。

**参考代码**：
```cpp
// Source/engine/render/scrollrt.cpp Line 789-801
const LevelCelBlock levelCelBlock { DPieceMicros[levelPieceId].mt[0] };
if (levelCelBlock.hasValue()) {
    RenderTileFrame(out, lightmap, targetBufferPosition, TileType::LeftTriangle,  // ← 强制使用 LeftTriangle
        GetDunFrame(pDungeonCels.get(), levelCelBlock.frame()), ...);
}
const LevelCelBlock levelCelBlock { DPieceMicros[levelPieceId].mt[1] };
if (levelCelBlock.hasValue()) {
    RenderTileFrame(out, lightmap, targetBufferPosition + RightFrameDisplacement, TileType::RightTriangle,  // ← 强制使用 RightTriangle
        GetDunFrame(pDungeonCels.get(), levelCelBlock.frame()), ...);
}
```

**Rust 实现**：
```rust
// rust-diablo/src/world/mod.rs Line 1613-1620
let tile_type = if is_floor {
    match block_index {
        0 => TileType::LeftTriangle,   // Block 0: 强制使用 LeftTriangle
        1 => TileType::RightTriangle,  // Block 1: 强制使用 RightTriangle
        _ => actual_tile_type,  // Blocks 2-15: 使用实际 tile type
    }
} else {
    actual_tile_type  // 非 floor tiles: 使用实际 tile type
};
```

### 4. Frame 索引提取

**从 LevelCelBlock 提取 frame 索引**：
```rust
// rust-diablo/src/world/mod.rs Line 1645-1656
let frame_idx = {
    let mgr = texture_mgr.borrow();
    if let Some(piece) = mgr.get_piece(level_piece_id) {
        if let Some(block) = piece.mt.get(block_index) {
            block.frame() as usize  // frame 索引是 1-based
        }
    }
};
```

**参考代码**：
```cpp
// Source/levels/dun_tile.hpp
// LevelCelBlock 结构：
// - frame(): 提取低 12 位作为 frame 索引
// - type(): 提取高 4 位作为 TileType
```

### 5. CEL 文件数据获取

**流程**：
1. 将 1-based frame 索引转换为 0-based 数组索引
2. 根据索引范围选择 main CEL 或 special CEL
3. 从 CEL frame 中获取原始编码数据（raw_data）

**参考代码**：
```rust
// rust-diablo/src/tiles/texture_manager.rs Line 662-722
pub fn get_indexed_tile(&mut self, frame_idx: usize, tile_type: TileType) -> Result<&[u8]> {
    let array_idx = frame_idx - 1; // 转换为 0-based
    
    let (cel_frame, source_name) = if array_idx < main_cel_len {
        (&self.cel_sprite.frames[array_idx], "main")
    } else {
        // Use special CEL
        let adjusted_idx = array_idx - main_cel_len;
        (&self.special_cel_sprite.as_ref().unwrap().frames[adjusted_idx], "special")
    };
    
    let raw_data = &cel_frame.raw_data;  // 原始编码数据（544 字节对于三角形）
}
```

**参考代码（C++）**：
```cpp
// Source/engine/render/dun_render.hpp Line 123-127
DVL_ALWAYS_INLINE const uint8_t *GetDunFrame(const std::byte *dungeonCelData, uint32_t frame)
{
	const auto *frameTable = reinterpret_cast<const uint32_t *>(dungeonCelData);
	return reinterpret_cast<const uint8_t *>(&dungeonCelData[Swap32LE(frameTable[frame])]);
}
```

### 6. 解码原始数据

**解码器选择**：
- 根据 **强制使用的** TileType（LeftTriangle/RightTriangle）选择解码器
- **不是**根据 block 中存储的实际 TileType

**参考代码**：
```rust
// rust-diablo/src/tiles/texture_manager.rs Line 737-760
let decoded = if cel_frame.is_decoded {
    // CLX 格式的 special CEL - 已经解码
    raw_data.clone()
} else {
    // Main CEL - 需要使用 TileType 解码器
    decode_tile(tile_type, raw_data)?  // tile_type 是强制使用的 LeftTriangle/RightTriangle
};
```

**三角形解码器**：
```rust
// rust-diablo/src/tiles/decoder/triangle.rs
pub fn decode_left_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    // 输入：544 字节原始编码数据
    // 输出：512 字节紧凑格式（variable-width rows, 无 padding）
    // 行宽度：[2,4,6,8,10,12,14,16,18,20,22,24,26,28,30,32,30,28,26,24,22,20,18,16,14,12,10,8,6,4,2]
}
```

**参考代码（C++）**：
```cpp
// Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsLeftTriangle()
// 解码 544 字节原始数据为 512 字节紧凑格式
```

### 7. Padding 转换（TextureManager 模式）

**为什么需要 Padding**：
- 解码器输出：512 字节紧凑格式（variable-width rows）
- TextureManager 需要：992 字节填充格式（32×31，每行 32 字节）

**Padding 逻辑**：
- **LeftTriangle**：左对齐（padding 在右边）
- **RightTriangle**：右对齐（padding 在左边）

**参考代码**：
```rust
// rust-diablo/src/world/mod.rs Line 757-812
fn pad_triangle_to_32x31(compact_data: &[u8], tile_type: TileType) -> Vec<u8> {
    match tile_type {
        TileType::LeftTriangle => {
            // 左对齐：先复制数据，再添加 padding
            for x in 0..width {
                padded.push(compact_data[src_pos + x]);
            }
            for _ in width..32 {
                padded.push(0); // Transparent padding
            }
        }
        TileType::RightTriangle => {
            // 右对齐：先添加 padding，再复制数据
            for _ in 0..(32 - width) {
                padded.push(0); // Transparent padding
            }
            for x in 0..width {
                padded.push(compact_data[src_pos + x]);
            }
        }
    }
}
```

### 8. 光照应用

**流程**：
1. 获取 micro tile 位置的光照级别
2. 对每个非透明像素（索引 != 0）应用光照

**参考代码**：
```rust
// rust-diablo/src/world/mod.rs Line 1757-1763
let light_level = if micro_x >= 0 && micro_y >= 0 {
    self.lighting.get_light_level(mx, my)
} else {
    self.lighting.ambient_light
};

for pixel in &mut indexed_pixels {
    if *pixel != 0 {
        *pixel = self.lighting.apply_lighting(*pixel, light_level);
    }
}
```

### 9. RGBA 转换

**流程**：
1. 使用 palette 将索引像素转换为 RGBA
2. 透明像素（索引 0）转换为 alpha=0

**参考代码**：
```rust
// rust-diablo/src/world/mod.rs Line 1803
let rgba_pixels = mgr.palette().indices_to_rgba(&indexed_pixels, true);
// true = 索引 0 转换为透明（alpha=0）
```

```rust
// rust-diablo/src/resources/palette.rs Line 112-115
pub fn to_rgba(&self, index: u8, transparent: bool) -> [u8; 4] {
    let color = self.colors[index as usize];
    let alpha = if transparent && index == 0 { 0 } else { 255 };
    color.to_rgba(alpha)
}
```

### 10. 渲染

**流程**：
1. 创建 SDL2 纹理（32×31 RGBA）
2. 设置混合模式（BlendMode::Blend）
3. 使用 `copy_ex` 渲染，启用垂直翻转（`flip_v=true`）

**参考代码**：
```rust
// rust-diablo/src/engine/mod.rs Line 427-437
self.canvas.copy_ex(
    &sdl_texture,
    None,
    sdl_rect,
    0.0,
    None,
    false,  // flip_h - floor tiles 不需要水平翻转
    true,   // flip_v - CLX 格式需要垂直翻转（bottom-to-top → top-to-bottom）
)
```

## 关键点总结

### 1. 强制使用 LeftTriangle/RightTriangle

**为什么**：
- C++ 的 `DrawFloorTile()` 强制使用 `TileType::LeftTriangle`（block 0）和 `TileType::RightTriangle`（block 1）
- 忽略 block 中存储的实际 TileType
- 这确保了 floor tiles 始终使用三角形解码器，即使 block 中存储的是其他类型（如 Square）

**影响**：
- 如果 block 中存储的是 `Square` 类型，但 frame 数据实际上是三角形格式
- 使用错误的解码器会导致解码错误，产生蓝色残留或黑色缺口

### 2. Frame 索引是 1-based

**注意**：
- MIN 数据中的 frame 索引是 1-based（1, 2, 3, ...）
- CEL 数组索引是 0-based（0, 1, 2, ...）
- 需要转换：`array_idx = frame_idx - 1`

### 3. 紧凑格式 vs 填充格式

**解码器输出**：
- 512 字节紧凑格式（variable-width rows，无 padding）
- 行宽度：[2,4,6,...,32,...,6,4,2]

**TextureManager 需要**：
- 992 字节填充格式（32×31，每行 32 字节）
- 需要 padding 转换

**DirectRenderer 使用**：
- 512 字节紧凑格式（直接使用，无需 padding）

### 4. 数据顺序

**存储顺序**：
- CEL 文件：从下到上（row 0 = 底部，row 30 = 顶部）
- Padding 后：从下到上（row 0 = 底部，row 30 = 顶部）

**渲染顺序**：
- SDL2 `flip_v=true`：垂直翻转，row 0 显示在顶部，row 30 显示在底部
- 这与 C++ 的从下到上渲染一致

### 5. 对齐方式

**LeftTriangle**：
- 左对齐（padding 在右边）
- 屏幕位置：`screen_x`

**RightTriangle**：
- 右对齐（padding 在左边）
- 屏幕位置：`screen_x + 32`（RightFrameDisplacement.deltaX = 32）

**边界连接**：
- LeftTriangle 的右边缘（x=31）应该与 RightTriangle 的左边缘（x=32）完美对齐
- 形成完整的菱形 tile

## 调试建议

如果遇到黑色三角形缺口，检查：

1. **Frame 索引是否正确**：
   - 检查 `[FLOOR DECODE]` 日志中的 frame 索引
   - 确认 frame 索引在有效范围内

2. **TileType 是否强制使用**：
   - 检查 `forced_type` 是否为 `LeftTriangle`/`RightTriangle`
   - 检查 `actual_type` 是否被忽略

3. **Padding 是否正确**：
   - 检查 `[PADDING]` 日志
   - 确认第一行和最后一行有 2 个非零像素
   - 确认中间行（row 15）有 32 个非零像素

4. **对齐是否正确**：
   - 检查 LeftTriangle 和 RightTriangle 的屏幕位置
   - 确认它们相差 32 像素（RightFrameDisplacement）

5. **透明像素处理**：
   - 检查 RGBA 转换后的 alpha 通道
   - 确认透明像素（索引 0）的 alpha=0

## 参考文件

- **MIN 文件加载**：`rust-diablo/src/tiles/min.rs`
- **解码器**：`rust-diablo/src/tiles/decoder/triangle.rs`
- **纹理管理**：`rust-diablo/src/tiles/texture_manager.rs`
- **渲染逻辑**：`rust-diablo/src/world/mod.rs::render_micro_tile()`
- **C++ 参考**：`Source/engine/render/scrollrt.cpp::DrawFloorTile()`
- **C++ 解码**：`Source/levels/reencode_dun_cels.cpp`



