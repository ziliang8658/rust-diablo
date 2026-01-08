# CEL 解码逻辑对比分析

## 概述

本文档对比 Rust 和 C++ 对 CEL 文件的解码逻辑，重点关注 frame 索引映射和数据解码的差异。

## 关键发现

### 1. C++ CEL 数据流程

**C++ 流程**：
1. **加载原始 CEL 文件** → `LoadLvlGFX()` 加载 `l1.cel`, `l1s.cel` 等
2. **收集 Frame 索引** → `SetDungeonMicros()` 从 MIN 数据收集所有唯一的 frame 索引
3. **重新编码 CEL** → `ReencodeDungeonCels()` 重新编码所有 frames，创建 `pDungeonCels`
4. **调整 Frame 索引** → `ComputeCelBlockAdjustments()` 计算 frame 索引调整值
5. **应用调整** → 更新 `DPieceMicros` 中的 frame 索引
6. **渲染时查找** → `GetDunFrame(pDungeonCels.get(), frame)` 使用调整后的 frame 索引

**关键代码**：
```cpp
// Source/levels/gendung.cpp Line 617
ReencodeDungeonCels(dungeonCels, frameToTypeList);

// Source/levels/gendung.cpp Line 619-632
std::vector<std::pair<uint16_t, uint16_t>> celBlockAdjustments = ComputeCelBlockAdjustments(frameToTypeList);
for (size_t levelPieceId = 0; levelPieceId < tileCount / blocks; levelPieceId++) {
    for (uint32_t block = 0; block < blocks; block++) {
        LevelCelBlock &levelCelBlock = DPieceMicros[levelPieceId].mt[block];
        const uint16_t frame = levelCelBlock.frame();
        // ... 查找调整值 ...
        levelCelBlock.data -= it->second;  // 调整 frame 索引
    }
}

// Source/engine/render/dun_render.hpp Line 123-127
DVL_ALWAYS_INLINE const uint8_t *GetDunFrame(const std::byte *dungeonCelData, uint32_t frame)
{
    const auto *frameTable = reinterpret_cast<const uint32_t *>(dungeonCelData);
    return reinterpret_cast<const uint8_t *>(&dungeonCelData[Swap32LE(frameTable[frame])]);
}
```

**关键点**：
- `ReencodeDungeonCels` 会**移除未使用的 frames**，只保留 MIN 数据中引用的 frames
- `ComputeCelBlockAdjustments` 计算 frame 索引的调整值（因为移除了未使用的 frames）
- `DPieceMicros` 中的 frame 索引会被**调整**，以匹配重新编码后的数据
- `GetDunFrame` 使用**调整后的 frame 索引**（1-based）直接查找重新编码后的数据

### 2. Rust CEL 数据流程

**Rust 流程**：
1. **加载原始 CEL 文件** → `TileTextureManager::load_for_dungeon()` 加载 `l1.cel`, `l1s.cel` 等
2. **加载 MIN 数据** → `MinData::from_bytes()` 解析 MIN 文件
3. **渲染时查找** → `get_indexed_tile(frame_idx, tile_type)` 使用原始 frame 索引（1-based）转换为 0-based，从原始 CEL 文件获取 frame

**关键代码**：
```rust
// rust-diablo/src/tiles/texture_manager.rs Line 662-765
pub fn get_indexed_tile(&mut self, frame_idx: usize, tile_type: TileType) -> Result<&[u8]> {
    // frame_idx is 1-based (from LevelCelBlock.frame())
    let array_idx = frame_idx - 1; // Convert 1-based to 0-based
    
    // Determine which CEL to use and adjust index
    let (cel_frame, source_name) = if array_idx < main_cel_len {
        (&self.cel_sprite.frames[array_idx], "main")
    } else {
        // Use special CEL
        let adjusted_idx = array_idx - main_cel_len;
        (&self.special_cel_sprite.as_ref().unwrap().frames[adjusted_idx], "special")
    };
    
    // Decode using appropriate TileType decoder
    let decoded = if cel_frame.is_decoded {
        raw_data.clone()
    } else {
        decode_tile(tile_type, raw_data)?
    };
}
```

**关键点**：
- Rust **不进行重新编码**，直接使用原始 CEL 文件
- Frame 索引（1-based）直接转换为 0-based 数组索引
- 从原始 CEL 文件获取 frame 数据
- 使用 `decode_tile()` 解码原始数据

### 3. 关键差异

#### 差异 1：Frame 索引调整

**C++**：
- 重新编码后，未使用的 frames 被移除
- Frame 索引被调整以匹配重新编码后的数据
- `DPieceMicros` 中的 frame 索引是**调整后的值**

**Rust**：
- 不进行重新编码
- Frame 索引是**原始值**（从 MIN 文件读取的原始值）
- 直接使用原始 frame 索引访问原始 CEL 文件

**潜在问题**：
- 如果 C++ 的 frame 索引调整逻辑改变了 frame 索引，Rust 可能会访问错误的 frame

#### 差异 2：数据格式

**C++**：
- 重新编码后的数据是**紧凑格式**（移除 padding）
- 三角形数据是 512 字节（紧凑格式）
- 数据已经按照 TileType 重新编码

**Rust**：
- 原始 CEL 数据是**原始格式**（包含 padding）
- 三角形数据是 544 字节（原始格式，包含 padding）
- 需要运行时解码

**潜在问题**：
- 如果 C++ 的重新编码改变了数据格式，Rust 的解码可能不匹配

#### 差异 3：Frame 索引映射

**C++**：
```cpp
// GetDunFrame 使用 frame 索引（1-based）直接查找
GetDunFrame(pDungeonCels.get(), levelCelBlock.frame())
// frame 是调整后的值，对应重新编码后的数据
```

**Rust**：
```rust
// get_indexed_tile 使用 frame 索引（1-based）转换为 0-based
let array_idx = frame_idx - 1;
let cel_frame = &self.cel_sprite.frames[array_idx];
// frame 是原始值，对应原始 CEL 文件
```

**潜在问题**：
- 如果 frame 索引调整改变了 frame 值，Rust 会访问错误的 frame

### 4. 验证方法

#### 方法 1：对比 Frame 索引

在 Rust 和 C++ 版本中，对同一个 piece 的 block 0 和 block 1 打印：
- MIN 文件中的原始 frame 索引
- C++ 调整后的 frame 索引
- Rust 使用的 frame 索引

**C++ 调试代码**：
```cpp
// 在 DrawFloorTile 中添加
const LevelCelBlock levelCelBlock { DPieceMicros[levelPieceId].mt[0] };
LogVerbose("[CEL DEBUG] piece={} block=0 min_frame={} adjusted_frame={}", 
    levelPieceId, 
    /* 需要保存原始 MIN frame */,
    levelCelBlock.frame());
```

**Rust 调试代码**：
```rust
// 在 render_micro_tile 中添加
println!("[CEL DEBUG] piece={} block={} min_frame={} rust_frame={}", 
    level_piece_id, block_index, 
    /* 原始 MIN frame */,
    frame_idx);
```

#### 方法 2：对比 CEL 数据

对比同一个 frame 的原始 CEL 数据和 C++ 重新编码后的数据：
- 原始 CEL frame 的字节数据
- C++ 重新编码后的 frame 数据
- Rust 解码后的数据

#### 方法 3：检查 Frame 索引调整

检查 `ComputeCelBlockAdjustments` 的逻辑：
- 哪些 frames 被移除了？
- Frame 索引如何调整？
- 调整后的 frame 索引是否与原始 CEL 文件匹配？

### 5. 可能的问题

#### 问题 1：Frame 索引不匹配

**症状**：
- Rust 和 C++ 渲染不同的 tile
- Frame 索引值不同

**原因**：
- C++ 的 frame 索引被调整，Rust 使用原始值
- 导致访问不同的 frame

**解决方案**：
- 检查 `ComputeCelBlockAdjustments` 的逻辑
- 确认是否需要实现 frame 索引调整

#### 问题 2：数据格式不匹配

**症状**：
- Rust 解码后的数据与 C++ 不匹配
- 出现蓝色残留或黑色缺口

**原因**：
- C++ 使用重新编码后的数据（紧凑格式）
- Rust 使用原始数据（包含 padding）
- 解码逻辑可能不匹配

**解决方案**：
- 检查 `ReencodeDungeonCels` 的逻辑
- 确认解码器是否正确处理原始格式

#### 问题 3：Special CEL 处理

**症状**：
- 某些 tiles 无法正确渲染
- Frame 索引超出范围

**原因**：
- Special CEL（l1s.cel, l2s.cel）的处理可能不同
- Frame 索引映射可能不正确

**解决方案**：
- 检查 special CEL 的加载逻辑
- 确认 frame 索引是否正确映射到 special CEL

## 结论

基于代码分析：

1. **Frame 索引调整**：C++ 会调整 frame 索引以匹配重新编码后的数据，Rust 使用原始值 ⚠️
2. **数据格式**：C++ 使用重新编码后的紧凑格式，Rust 使用原始格式 ⚠️
3. **解码时机**：C++ 在加载时重新编码，Rust 在运行时解码 ⚠️

**建议**：
1. 添加调试日志，对比相同 piece 的 frame 索引
2. 检查 `ComputeCelBlockAdjustments` 的逻辑，确认是否需要实现 frame 索引调整
3. 验证 frame 索引调整是否影响 floor tile 的渲染

## 参考文件

- **C++ CEL 重新编码**：`Source/levels/reencode_dun_cels.cpp`
- **C++ Frame 索引调整**：`Source/levels/gendung.cpp::ComputeCelBlockAdjustments()`
- **C++ Frame 查找**：`Source/engine/render/dun_render.hpp::GetDunFrame()`
- **Rust CEL 加载**：`rust-diablo/src/tiles/texture_manager.rs`
- **Rust Frame 查找**：`rust-diablo/src/tiles/texture_manager.rs::get_indexed_tile()`



