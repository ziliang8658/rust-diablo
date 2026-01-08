# MIN 文件 Block 索引解码对比分析

## 概述

本文档对比 Rust 和 C++ 对 MIN 文件的 block cell 索引解码实现，以确认是否存在解析问题。

## 关键发现

### 1. LevelCelBlock 位布局

**C++ 实现** (`Source/levels/dun_tile.hpp`):
```cpp
struct LevelCelBlock {
    uint16_t data;
    
    uint16_t frame() const {
        return data & 0xFFF;  // 低 12 位：frame 索引（1-based）
    }
    
    TileType type() const {
        return static_cast<TileType>((data & 0x7000) >> 12);  // 高 3 位（bits 12-14）：TileType
    }
};
```

**Rust 实现** (`rust-diablo/src/tiles/types.rs`):
```rust
pub struct LevelCelBlock {
    pub data: u16,
}

impl LevelCelBlock {
    pub fn frame(&self) -> u16 {
        self.data & 0xFFF  // 低 12 位：frame 索引（1-based）✓
    }
    
    pub fn tile_type(&self) -> TileType {
        let type_value = ((self.data & 0x7000) >> 12) as u8;  // 高 3 位（bits 12-14）：TileType ✓
        TileType::from_u8(type_value).unwrap_or(TileType::Square)
    }
}
```

**结论**：位布局完全一致 ✓

### 2. 字节序处理

**C++ 实现** (`Source/levels/gendung.cpp`):
```cpp
// 1. LoadFileInMem<uint16_t> 按 native endian 读取文件
const std::unique_ptr<uint16_t[]> levelPieces = LoadMinData(tileCount);

// 2. 使用 Swap16LE 转换为小端序
const LevelCelBlock levelCelBlock { Swap16LE(pieces[blocks - 2 + (block & 1) - (block & 0xE)]) };
```

**Rust 实现** (`rust-diablo/src/tiles/min.rs`):
```rust
// 直接从小端序字节读取
let val = u16::from_le_bytes([data[offset], data[offset + 1]]);
mt.push(LevelCelBlock::new(val));
```

**关键点**：
- `Swap16LE` 的作用：如果系统是大端序，则交换字节；如果系统是小端序，则不交换。最终确保数据是小端序格式。
- `from_le_bytes` 的作用：明确从小端序字节转换为 u16。
- **在 Windows（小端序系统）上，两者应该产生相同结果** ✓

**潜在问题**：
- 如果 `LoadFileInMem<uint16_t>` 已经按 native endian 读取了数据，而文件本身是小端序，那么：
  - 在小端序系统上：数据正确，`Swap16LE` 不做任何事
  - 在大端序系统上：数据错误，`Swap16LE` 会交换字节以修正

**验证方法**：
```rust
// 检查系统字节序
#[cfg(target_endian = "little")]
println!("System is Little Endian - from_le_bytes should match Swap16LE");

#[cfg(target_endian = "big")]
println!("System is Big Endian - Swap16LE will swap bytes, from_le_bytes won't");
```

### 3. Block 重排序逻辑

**C++ 实现** (`Source/levels/gendung.cpp` Line 603):
```cpp
const LevelCelBlock levelCelBlock { Swap16LE(pieces[blocks - 2 + (block & 1) - (block & 0xE)]) };
```

**Rust 实现** (`rust-diablo/src/tiles/min.rs` Line 87-89):
```rust
let src_idx = (blocks_per_piece as isize - 2 + (block as isize & 1)
    - (block as isize & 0xE)) as usize;
piece.mt[block] = raw_mt[src_idx];
```

**重排序公式验证**（Town，blocks=16）：
```
Block 0: 16-2+0-0 = 14 → 从原始 block 14 读取
Block 1: 16-2+1-0 = 15 → 从原始 block 15 读取
Block 2: 16-2+0-2 = 12 → 从原始 block 12 读取
Block 3: 16-2+1-2 = 13 → 从原始 block 13 读取
...
```

**结论**：重排序公式完全一致 ✓

### 4. MIN 文件读取流程对比

**C++ 流程**：
1. `LoadMinData()` → `LoadFileInMem<uint16_t>()` → 返回 `uint16_t[]`（native endian）
2. `SetDungeonMicros()` → 对每个 block 应用重排序公式并调用 `Swap16LE()`
3. 创建 `LevelCelBlock` 并存储到 `DPieceMicros[levelPieceId].mt[block]`

**Rust 流程**：
1. `MinData::from_bytes()` → 读取原始字节数据
2. 对每个 block 使用 `u16::from_le_bytes()` 读取（小端序）
3. 应用重排序公式
4. 创建 `LevelCelBlock` 并存储到 `pieces[piece_idx].mt[block]`

**关键差异**：
- C++ 在读取时应用重排序，Rust 在读取后应用重排序
- C++ 使用 `Swap16LE`，Rust 使用 `from_le_bytes`
- **在 Windows（小端序）上，两者应该产生相同结果** ✓

## 潜在问题分析

### 问题 1：字节序处理不一致

**假设**：`LoadFileInMem<uint16_t>` 可能已经处理了字节序，或者文件本身就是 native endian 格式。

**验证方法**：
1. 检查 `LoadFileInMem` 的实现
2. 对比原始 MIN 文件的字节序
3. 在运行时打印前几个 block 的原始值

### 问题 2：重排序时机不同

**C++**：在读取时直接应用重排序
```cpp
pieces[blocks - 2 + (block & 1) - (block & 0xE)]  // 直接索引重排序后的位置
```

**Rust**：先读取所有 blocks，再应用重排序
```rust
// 先读取所有 blocks
for _ in 0..blocks_per_piece {
    let val = u16::from_le_bytes([data[offset], data[offset + 1]]);
    mt.push(LevelCelBlock::new(val));
}
// 然后应用重排序
let src_idx = (blocks_per_piece as isize - 2 + (block as isize & 1) - (block as isize & 0xE)) as usize;
piece.mt[block] = raw_mt[src_idx];
```

**结论**：虽然时机不同，但逻辑等价 ✓

### 问题 3：Frame 索引提取

**C++**：
```cpp
levelCelBlock.frame()  // 返回 data & 0xFFF
```

**Rust**：
```rust
block.frame()  // 返回 self.data & 0xFFF
```

**结论**：完全一致 ✓

## 验证建议

### 1. 运行时对比

在 Rust 和 C++ 版本中，对同一个 piece 的 block 0 和 block 1 打印：
- 原始 `data` 值（十六进制）
- `frame()` 值
- `type()` 值

**Rust 调试代码**：
```rust
// 在 render_micro_tile 中添加
if is_floor && (block_index == 0 || block_index == 1) {
    let block = piece.mt.get(block_index).unwrap();
    println!("[MIN DEBUG] piece={} block={} raw_data=0x{:04X} frame={} type={:?}", 
        level_piece_id, block_index, block.data, block.frame(), block.tile_type());
}
```

**C++ 调试代码**：
```cpp
// 在 DrawFloorTile 中添加
const LevelCelBlock levelCelBlock { DPieceMicros[levelPieceId].mt[0] };
LogVerbose("[MIN DEBUG] piece={} block=0 raw_data=0x{:04X} frame={} type={}", 
    levelPieceId, levelCelBlock.data, levelCelBlock.frame(), 
    static_cast<int>(levelCelBlock.type()));
```

### 2. 文件级对比

创建一个工具，直接读取 MIN 文件并对比：
- 原始字节值
- 解析后的 frame 索引
- 解析后的 tile type

### 3. 特定 Piece 验证

选择一个已知的 floor tile piece（例如 Town 的 piece 0），对比：
- C++ 解析的 block 0 和 block 1 的 frame 索引
- Rust 解析的 block 0 和 block 1 的 frame 索引

## 结论

基于代码分析：

1. **位布局**：完全一致 ✓
2. **重排序公式**：完全一致 ✓
3. **Frame 索引提取**：完全一致 ✓
4. **字节序处理**：在 Windows（小端序）上应该一致，但需要运行时验证

**建议**：
1. 添加运行时调试日志，对比相同 piece 的解析结果
2. 如果发现不一致，检查 `LoadFileInMem` 的实现细节
3. 验证 MIN 文件本身的字节序格式

## 参考文件

- **C++ LevelCelBlock**：`Source/levels/dun_tile.hpp`
- **C++ MIN 加载**：`Source/levels/gendung.cpp::SetDungeonMicros()`
- **Rust LevelCelBlock**：`rust-diablo/src/tiles/types.rs`
- **Rust MIN 加载**：`rust-diablo/src/tiles/min.rs::MinData::from_bytes()`



