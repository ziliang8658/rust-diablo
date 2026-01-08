# CEL解码诊断代码 - 已添加

## 已完成的诊断代码

### Rust端

#### 1. DungeonCelSprite加载统计 ✅

**文件**: `rust-diablo/src/resources/dungeon_cel.rs`

**位置1**: `from_bytes()` (Line 163)
```rust
// [DIAGNOSIS] 统计is_decoded标志
let decoded_count = frames.iter().filter(|f| f.is_decoded).count();
println!("[CEL LOAD] Main CEL: {} frames, {} decoded, {} need decoding",
         frames.len(), decoded_count, frames.len() - decoded_count);
```

**位置2**: `from_clx_bytes()` (Line 452)
```rust
// [DIAGNOSIS] 统计is_decoded标志
let decoded_count = frames.iter().filter(|f| f.is_decoded).count();
println!("[CEL LOAD] Special CEL (CLX): {} frames, {} decoded, {} need decoding",
         frames.len(), decoded_count, frames.len() - decoded_count);
```

**预期输出**:
```
[CEL LOAD] Main CEL: 1000 frames, 0 decoded, 1000 need decoding
[CEL LOAD] Special CEL (CLX): 200 frames, 200 decoded, 0 need decoding
```

#### 2. TileTextureManager CEL分析 ✅

**文件**: `rust-diablo/src/tiles/texture_manager.rs`

**位置**: `load_for_dungeon()` (Line 295)
```rust
// [DIAGNOSIS] CEL解码状态分析
println!("\n=== TileTextureManager CEL Analysis ===");
println!("Main CEL frames: {}", cel_sprite.frames.len());
let main_decoded = cel_sprite.frames.iter().filter(|f| f.is_decoded).count();
println!("  - Already decoded: {}", main_decoded);
println!("  - Need decoding: {}", cel_sprite.frames.len() - main_decoded);

if let Some(ref special) = special_cel_sprite {
    println!("Special CEL frames: {}", special.frames.len());
    let special_decoded = special.frames.iter().filter(|f| f.is_decoded).count();
    println!("  - Already decoded: {}", special_decoded);
    println!("  - Need decoding: {}", special.frames.len() - special_decoded);
} else {
    println!("Special CEL: None");
}
```

**预期输出**:
```
=== TileTextureManager CEL Analysis ===
Main CEL frames: 1000
  - Already decoded: 0
  - Need decoding: 1000
Special CEL frames: 200
  - Already decoded: 200
  - Need decoding: 0
```

#### 3. get_decoded_tile()详细日志 ✅

**文件**: `rust-diablo/src/tiles/texture_manager.rs`

**位置**: `get_decoded_tile()` (Line 708-745)
```rust
// [DIAGNOSIS] 详细日志
static mut DECODE_LOG_COUNT: usize = 0;
let should_log = unsafe {
    DECODE_LOG_COUNT += 1;
    DECODE_LOG_COUNT <= 50 // 前50个piece详细输出
};

if should_log {
    println!("[CEL DECODE] piece={} block={} frame_idx={} source={} is_decoded={} raw_size={} tile_type={:?}",
             piece_index, block_index, frame_idx, source_name, 
             cel_frame.is_decoded, cel_frame.raw_data.len(), tile_type);
}

let decoded = if cel_frame.is_decoded {
    if should_log {
        println!("[CEL DECODE] → Using pre-decoded data (size={})", raw_data.len());
    }
    raw_data.clone()
} else {
    if should_log {
        println!("[CEL DECODE] → Calling decode_tile() for type={:?}", tile_type);
    }
    let result = decode_tile(tile_type, raw_data).with_context(|| {
        format!("Failed to decode frame {} from {} CEL (type={:?})",
                frame_idx, source_name, tile_type)
    })?;
    if should_log {
        println!("[CEL DECODE] → Decoded size: {} bytes", result.len());
    }
    result
};
```

**预期输出**:
```
[CEL DECODE] piece=0 block=0 frame_idx=1 source=main is_decoded=false raw_size=512 tile_type=LeftTriangle
[CEL DECODE] → Calling decode_tile() for type=LeftTriangle
[CEL DECODE] → Decoded size: 512 bytes

[CEL DECODE] piece=0 block=1 frame_idx=2 source=main is_decoded=false raw_size=512 tile_type=RightTriangle
[CEL DECODE] → Calling decode_tile() for type=RightTriangle
[CEL DECODE] → Decoded size: 512 bytes

[CEL DECODE] piece=100 block=5 frame_idx=1205 source=special is_decoded=true raw_size=1024 tile_type=Square
[CEL DECODE] → Using pre-decoded data (size=1024)
[CEL DECODE] → Decoded size: 1024 bytes
```

#### 4. 解码失败追踪 ✅ (已在之前添加)

**文件**: `rust-diablo/src/world/mod.rs`

**位置**: `render_micro_tile()` (Line 1102-1115)
```rust
Err(e) => {
    // [DIAGNOSIS] Track decode failures
    static mut DECODE_FAILURES: std::collections::HashSet<usize> = std::collections::HashSet::new();
    static mut FAILURE_COUNT: usize = 0;
    unsafe {
        if DECODE_FAILURES.insert(level_piece_id) {
            FAILURE_COUNT += 1;
            if FAILURE_COUNT <= 20 {
                println!("⚠️  [DECODE FAILURE] piece={} block={} error={:?}", 
                    level_piece_id, block_index, e);
            }
        }
    }
    // ...
}
```

### C++端

#### SOL诊断 ✅ (已在之前添加)

**文件**: `Source/levels/gendung.cpp`

**位置**: `LoadLevelSOLData()` (Line 539-575)
```cpp
// [DIAGNOSIS] Analyze SOL data for transparency properties
{
    int transparentCount = 0;
    int transparentLeftCount = 0;
    int transparentRightCount = 0;
    std::vector<int> transparentPieces;
    
    for (int i = 0; i < MAXTILES; i++) {
        if (HasAnyOf(SOLData[i], TileProperties::Transparent)) {
            transparentCount++;
            transparentPieces.push_back(i);
            if (transparentCount <= 20) {
                Log("[C++ SOL DIAGNOSIS] Piece {} has TRANSPARENT property (flags={:08x})", 
                    i, static_cast<uint32_t>(SOLData[i]));
            }
        }
        // ... 其他属性检查 ...
    }
    
    Log("[C++ SOL DIAGNOSIS] Total pieces in SOL: {}", MAXTILES);
    Log("[C++ SOL DIAGNOSIS] Pieces with TRANSPARENT: {}", transparentCount);
    // ...
}
```

---

## 下一步行动

### 立即执行

1. **编译Rust项目**
```bash
cd rust-diablo
cargo build
```

2. **运行Rust项目并保存日志**
```bash
cargo run > rust_cel_diagnosis.log 2>&1
```

3. **编译并运行C++项目** (您自己操作)
```bash
# 编译C++
# 运行并保存日志到 cpp_cel_diagnosis.log
```

4. **查看关键日志**

#### Rust关键日志搜索：
```bash
# CEL加载统计
grep "CEL LOAD" rust_cel_diagnosis.log

# TileTextureManager分析
grep "TileTextureManager CEL Analysis" rust_cel_diagnosis.log -A 10

# CEL解码详细日志
grep "CEL DECODE" rust_cel_diagnosis.log | head -50

# 解码失败
grep "DECODE FAILURE" rust_cel_diagnosis.log
```

#### C++关键日志搜索：
```bash
# SOL诊断
grep "SOL DIAGNOSIS" cpp_cel_diagnosis.log
```

---

## 预期发现

### 正常情况

#### Rust应该显示：
```
[CEL LOAD] Main CEL: ~1000 frames, 0 decoded, ~1000 need decoding
[CEL LOAD] Special CEL (CLX): ~200 frames, ~200 decoded, 0 need decoding

=== TileTextureManager CEL Analysis ===
Main CEL frames: ~1000
  - Already decoded: 0
  - Need decoding: ~1000
Special CEL frames: ~200
  - Already decoded: ~200
  - Need decoding: 0

[CEL DECODE] piece=0 block=0 frame_idx=1 source=main is_decoded=false raw_size=512 tile_type=LeftTriangle
[CEL DECODE] → Calling decode_tile() for type=LeftTriangle
[CEL DECODE] → Decoded size: 512 bytes
```

### 异常情况

#### 问题1: Main CEL被错误标记为decoded
```
[CEL LOAD] Main CEL: 1000 frames, 500 decoded, 500 need decoding  ← 错误！
```
**原因**: `DungeonCelSprite::from_bytes()`设置了错误的`is_decoded`标志

#### 问题2: 解码失败
```
[CEL DECODE] → Calling decode_tile() for type=LeftTriangle
⚠️  [DECODE FAILURE] piece=123 block=5 error=InvalidData
```
**原因**: `decode_tile()`实现有问题

#### 问题3: 解码大小不对
```
[CEL DECODE] → Decoded size: 256 bytes  ← 应该是512
```
**原因**: 解码器输出大小错误

---

## 关键检查点

### ✅ 检查点1: is_decoded标志
- [ ] Main CEL的`is_decoded`全部为`false`
- [ ] Special CEL的`is_decoded`全部为`true`

### ✅ 检查点2: 解码路径
- [ ] Main CEL的frame走`decode_tile()`路径
- [ ] Special CEL的frame走预解码路径

### ✅ 检查点3: 解码成功率
- [ ] 没有或很少`DECODE FAILURE`
- [ ] 解码大小合理（512或1024字节）

### ✅ 检查点4: Frame数量
- [ ] Rust和C++的frame数量一致

---

## 如果发现问题

### 场景1: Main CEL有decoded=true的frame

**修复位置**: `rust-diablo/src/resources/dungeon_cel.rs::parse_frame()`

确保返回：
```rust
Ok(DungeonCelFrame {
    raw_data: data.to_vec(),
    is_decoded: false,  // 必须是false
})
```

### 场景2: 大量解码失败

**需要对比**: 
1. Rust的`decode_tile()`实现
2. C++的解码函数（`DecodeTileSquare`, `DecodeTileLeftTriangle`等）

**可能原因**:
- 解码算法不一致
- 数据格式理解错误
- 边界条件处理不同

### 场景3: 解码大小不对

**检查**:
- LeftTriangle/RightTriangle应该是512字节（16×32）
- Square应该是1024字节（32×32）
- TransparentSquare应该是1024字节

---

## 总结

已添加的诊断代码覆盖了CEL解码的整个流程：

1. ✅ CEL文件加载时的统计
2. ✅ TileTextureManager初始化时的分析
3. ✅ 每个piece解码时的详细日志
4. ✅ 解码失败的追踪

**现在请编译运行，然后把日志发给我分析！**

---

**创建日期**: 2024-12-04  
**状态**: 诊断代码已添加完成，等待运行结果




