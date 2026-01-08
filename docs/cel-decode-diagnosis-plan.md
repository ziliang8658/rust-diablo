# CEL解码诊断计划

## 问题假设

您怀疑piece的CEL解析出现问题，存在两种解析路径：
1. **直接解析CEL** - 主CEL文件（town.cel）需要解码
2. **转换再解析** - 特殊CEL文件（towns.cel，CLX格式）已经解码

## 关键代码位置

### Rust端

#### 1. DungeonCelFrame结构
**文件**: `rust-diablo/src/resources/dungeon_cel.rs` (Line 38-44)

```rust
pub struct DungeonCelFrame {
    pub raw_data: Vec<u8>,
    pub is_decoded: bool,  // ⚠️ 关键标志
}
```

#### 2. 解码逻辑分支
**文件**: `rust-diablo/src/tiles/texture_manager.rs` (Line 709-721)

```rust
let decoded = if cel_frame.is_decoded {
    // Special CEL from CLX format - already decoded, just clone
    raw_data.clone()
} else {
    // Main CEL - needs decoding with appropriate TileType decoder
    decode_tile(tile_type, raw_data).with_context(|| {
        format!("Failed to decode frame {} from {} CEL (type={:?})",
                frame_idx, source_name, tile_type)
    })?
};
```

**问题点**:
- `is_decoded`标志是否正确设置？
- `decode_tile()`是否正确实现？
- 两种路径的数据格式是否一致？

### C++端

#### 1. CEL加载
**文件**: `Source/levels/gendung.cpp::LoadLvlGFX()` (Line 1235-1264)

#### 2. 特殊CEL转换
**文件**: `Source/levels/gendung.cpp::ReencodeDungeonCels()` (Line 1266-1326)

#### 3. 渲染时解码
**文件**: `Source/engine/render/dun_render.cpp::RenderTile()`

---

## 诊断代码添加

### Phase 1: 追踪is_decoded标志

#### Rust - DungeonCelSprite加载时

**文件**: `rust-diablo/src/resources/dungeon_cel.rs`

在`from_bytes()`和`from_clx_bytes()`中添加统计：

```rust
// 在from_bytes()末尾添加
let decoded_count = frames.iter().filter(|f| f.is_decoded).count();
println!("[CEL LOAD] Main CEL: {} frames, {} decoded, {} need decoding",
         frames.len(), decoded_count, frames.len() - decoded_count);

// 在from_clx_bytes()末尾添加
let decoded_count = frames.iter().filter(|f| f.is_decoded).count();
println!("[CEL LOAD] Special CEL (CLX): {} frames, {} decoded, {} need decoding",
         frames.len(), decoded_count, frames.len() - decoded_count);
```

#### Rust - TileTextureManager初始化时

**文件**: `rust-diablo/src/tiles/texture_manager.rs`

在`new()`方法中添加：

```rust
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
}
```

### Phase 2: 追踪每个piece的解码路径

#### Rust - get_decoded_tile()详细日志

**文件**: `rust-diablo/src/tiles/texture_manager.rs` (Line 650-726)

```rust
fn get_decoded_tile(&mut self, piece_index: usize, block_index: usize) -> Result<&[u8]> {
    // ... existing code ...
    
    // [DIAGNOSIS] 添加在Line 693之后
    let (cel_frame, source_name) = if array_idx < main_cel_len {
        (&self.cel_sprite.frames[array_idx], "main")
    } else {
        let adjusted_idx = array_idx - main_cel_len;
        (&self.special_cel_sprite.as_ref().unwrap().frames[adjusted_idx], "special")
    };
    
    // [DIAGNOSIS] 详细日志
    static mut DECODE_LOG_COUNT: usize = 0;
    unsafe {
        DECODE_LOG_COUNT += 1;
        if DECODE_LOG_COUNT <= 50 {  // 前50个piece详细输出
            println!("[CEL DECODE] piece={} block={} frame_idx={} source={} is_decoded={} raw_size={} tile_type={:?}",
                     piece_index, block_index, frame_idx, source_name, 
                     cel_frame.is_decoded, cel_frame.raw_data.len(), tile_type);
        }
    }
    
    // Get raw data from CEL frame
    let raw_data = &cel_frame.raw_data;
    
    let decoded = if cel_frame.is_decoded {
        println!("[CEL DECODE] → Using pre-decoded data (size={})", raw_data.len());
        raw_data.clone()
    } else {
        println!("[CEL DECODE] → Calling decode_tile() for type={:?}", tile_type);
        decode_tile(tile_type, raw_data).with_context(|| {
            format!("Failed to decode frame {} from {} CEL (type={:?})",
                    frame_idx, source_name, tile_type)
        })?
    };
    
    println!("[CEL DECODE] → Decoded size: {} bytes", decoded.len());
    
    // ... rest of code ...
}
```

### Phase 3: 对比C++的CEL处理

#### C++ - LoadLvlGFX()添加统计

**文件**: `Source/levels/gendung.cpp`

在`LoadLvlGFX()`末尾添加：

```cpp
void LoadLvlGFX(const char *lvlName)
{
    // ... existing code ...
    
    // [DIAGNOSIS] 统计CEL信息
    {
        // 计算main CEL的frame数量
        const uint32_t *mainFrameTable = reinterpret_cast<const uint32_t *>(pDungeonCels.get());
        uint32_t mainFrameCount = Swap32LE(mainFrameTable[0]);
        
        Log("[C++ CEL LOAD] Main CEL: {} frames loaded", mainFrameCount);
        
        if (pSpecialCels) {
            const uint32_t *specialFrameTable = reinterpret_cast<const uint32_t *>(pSpecialCels->Data());
            uint32_t specialFrameCount = Swap32LE(specialFrameTable[0]);
            Log("[C++ CEL LOAD] Special CEL: {} frames loaded", specialFrameCount);
        } else {
            Log("[C++ CEL LOAD] No special CEL");
        }
    }
}
```

#### C++ - RenderTile()添加解码日志

**文件**: `Source/engine/render/dun_render.cpp`

找到`RenderTile()`函数，添加：

```cpp
void RenderTile(/* ... */)
{
    // ... existing code ...
    
    // [DIAGNOSIS] 前50个tile详细输出
    static int renderCount = 0;
    if (renderCount < 50) {
        renderCount++;
        const uint8_t *frameData = GetDunFrame(dungeonCels, frameIndex);
        Log("[C++ RENDER] frame_idx={} tile_type={} frame_data_ptr={:p} first_byte={:02x}",
            frameIndex, static_cast<int>(tileType), 
            static_cast<const void*>(frameData), frameData[0]);
    }
    
    // ... rest of code ...
}
```

### Phase 4: 对比decode_tile()实现

#### Rust - decode_tile()详细日志

**文件**: `rust-diablo/src/tiles/cl2_decoder.rs` (或相应文件)

在`decode_tile()`函数开始添加：

```rust
pub fn decode_tile(tile_type: TileType, data: &[u8]) -> Result<Vec<u8>> {
    static mut DECODE_COUNT: usize = 0;
    unsafe {
        DECODE_COUNT += 1;
        if DECODE_COUNT <= 20 {
            println!("[DECODE_TILE] type={:?} input_size={} first_bytes=[{:02x} {:02x} {:02x} {:02x}]",
                     tile_type, data.len(),
                     data.get(0).unwrap_or(&0), data.get(1).unwrap_or(&0),
                     data.get(2).unwrap_or(&0), data.get(3).unwrap_or(&0));
        }
    }
    
    let result = match tile_type {
        TileType::Square => decode_square(data)?,
        TileType::TransparentSquare => decode_transparent_square(data)?,
        TileType::LeftTriangle => decode_left_triangle(data)?,
        TileType::RightTriangle => decode_right_triangle(data)?,
        // ... other types ...
    };
    
    unsafe {
        if DECODE_COUNT <= 20 {
            println!("[DECODE_TILE] → output_size={} first_pixels=[{:02x} {:02x} {:02x} {:02x}]",
                     result.len(),
                     result.get(0).unwrap_or(&0), result.get(1).unwrap_or(&0),
                     result.get(2).unwrap_or(&0), result.get(3).unwrap_or(&0));
        }
    }
    
    Ok(result)
}
```

#### C++ - 对应的解码函数

**文件**: `Source/engine/render/dun_render.cpp`

找到对应的解码函数（如`DecodeTileSquare`, `DecodeTileLeftTriangle`等），添加类似日志。

---

## 预期输出分析

### 正常情况

#### Rust输出示例：
```
[CEL LOAD] Main CEL: 1000 frames, 0 decoded, 1000 need decoding
[CEL LOAD] Special CEL (CLX): 200 frames, 200 decoded, 0 need decoding

=== TileTextureManager CEL Analysis ===
Main CEL frames: 1000
  - Already decoded: 0
  - Need decoding: 1000
Special CEL frames: 200
  - Already decoded: 200
  - Need decoding: 0

[CEL DECODE] piece=0 block=0 frame_idx=1 source=main is_decoded=false raw_size=512 tile_type=LeftTriangle
[CEL DECODE] → Calling decode_tile() for type=LeftTriangle
[DECODE_TILE] type=LeftTriangle input_size=512 first_bytes=[00 01 02 03]
[DECODE_TILE] → output_size=512 first_pixels=[00 00 00 01]
[CEL DECODE] → Decoded size: 512 bytes

[CEL DECODE] piece=100 block=5 frame_idx=1205 source=special is_decoded=true raw_size=1024 tile_type=Square
[CEL DECODE] → Using pre-decoded data (size=1024)
[CEL DECODE] → Decoded size: 1024 bytes
```

#### C++输出示例：
```
[C++ CEL LOAD] Main CEL: 1000 frames loaded
[C++ CEL LOAD] Special CEL: 200 frames loaded

[C++ RENDER] frame_idx=1 tile_type=2 frame_data_ptr=0x12345678 first_byte=00
[C++ RENDER] frame_idx=1205 tile_type=0 frame_data_ptr=0x12345abc first_byte=ff
```

### 异常情况检测

#### 问题1: is_decoded标志错误
**症状**:
```
[CEL LOAD] Main CEL: 1000 frames, 500 decoded, 500 need decoding  ← 主CEL不应该有decoded
```

**原因**: Main CEL被错误标记为decoded

**解决**: 检查`DungeonCelSprite::from_bytes()`，确保`is_decoded=false`

#### 问题2: 解码失败
**症状**:
```
[CEL DECODE] → Calling decode_tile() for type=LeftTriangle
⚠️  [DECODE FAILURE] piece=123 block=5 error=InvalidData
```

**原因**: decode_tile()无法正确解码数据

**解决**: 对比C++的解码实现，修复Rust解码器

#### 问题3: frame_idx超出范围
**症状**:
```
[CEL DECODE] piece=456 block=2 frame_idx=9999 source=main is_decoded=false
Error: frame index 9999 out of bounds (max 1000)
```

**原因**: MIN文件中的frame index错误

**解决**: 检查MIN加载逻辑

#### 问题4: 数据大小不匹配
**症状**:
```
[DECODE_TILE] type=LeftTriangle input_size=512 first_bytes=[00 01 02 03]
[DECODE_TILE] → output_size=256 first_pixels=[00 00 00 01]  ← 应该是512
```

**原因**: 解码器输出大小错误

**解决**: 修复解码器逻辑

---

## 实施步骤

### Step 1: 添加基础诊断 (立即)
- [ ] Rust: DungeonCelSprite加载统计
- [ ] Rust: TileTextureManager CEL分析
- [ ] C++: LoadLvlGFX统计

### Step 2: 添加详细解码日志 (高优先级)
- [ ] Rust: get_decoded_tile()详细日志
- [ ] Rust: decode_tile()详细日志
- [ ] C++: RenderTile()日志

### Step 3: 运行并收集日志 (立即)
- [ ] 编译Rust和C++
- [ ] 运行并保存日志
- [ ] 对比输出

### Step 4: 分析差异 (根据日志)
- [ ] 检查frame数量是否一致
- [ ] 检查is_decoded标志是否正确
- [ ] 检查解码路径是否正确
- [ ] 检查解码输出大小是否一致

### Step 5: 修复问题 (根据分析)
- [ ] 修复is_decoded标志设置
- [ ] 修复decode_tile()实现
- [ ] 修复frame index计算
- [ ] 修复数据格式处理

---

## 关键检查点

### 检查点1: Main CEL vs Special CEL
**问题**: 是否正确区分主CEL和特殊CEL？

**验证**:
- Main CEL应该全部`is_decoded=false`
- Special CEL应该全部`is_decoded=true`

### 检查点2: Frame Index计算
**问题**: MIN中的frame index是否正确映射到CEL frame？

**验证**:
- 对比C++和Rust的frame index计算
- 检查是否有off-by-one错误

### 检查点3: TileType匹配
**问题**: 每个frame的TileType是否正确？

**验证**:
- 对比C++和Rust对同一piece的TileType判断
- 检查MIN数据解析是否一致

### 检查点4: 解码器实现
**问题**: decode_tile()是否正确实现了所有TileType？

**验证**:
- 对比C++的解码函数
- 检查输出大小和格式

---

## 参考代码位置

### Rust
- `rust-diablo/src/resources/dungeon_cel.rs` - CEL加载
- `rust-diablo/src/tiles/texture_manager.rs` - 纹理管理和解码调用
- `rust-diablo/src/tiles/cl2_decoder.rs` - CL2/CEL解码器

### C++
- `Source/levels/gendung.cpp` - CEL加载和转换
- `Source/engine/render/dun_render.cpp` - 瓦片渲染和解码
- `Source/engine/render/dun_render.hpp` - GetDunFrame()

---

**创建日期**: 2024-12-04  
**目的**: 深度分析CEL解码逻辑，找出Rust和C++的差异




