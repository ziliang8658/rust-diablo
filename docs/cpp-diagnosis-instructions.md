# C++ 诊断输出指南

## 当前状态

### ✅ 已添加的诊断

1. **SOL诊断** - `Source/levels/gendung.cpp::LoadLevelSOLData()` (Line 538-577)
   - 已经添加，会在加载SOL后输出统计信息

### ⚠️ 需要添加的诊断

1. **CEL加载统计** - 需要找到LoadLvlGFX()函数并添加统计
2. **CEL解码日志** - 需要在渲染时添加解码日志
3. **Transparency统计** - 已有部分，需要确保输出

---

## 建议的C++诊断添加位置

### 1. CEL加载统计（如果有LoadLvlGFX函数）

**位置**: 应该在`diablo.cpp`或`levels/gendung.cpp`中

**需要输出**:
```cpp
Log("[C++ CEL LOAD] Main CEL: {} frames loaded", frameCount);
Log("[C++ CEL LOAD] Special CEL: {} frames loaded", specialFrameCount);
```

### 2. Transparency加载统计

**位置**: `Source/levels/gendung.cpp::LoadTransparency()` (Line 671-712)

**已有输出**:
```cpp
LogVerbose("[C++ TRANSPARENCY] Loaded transparency data: {} tiles, {} non-zero values, {} unique values",
           size.width * size.height, nonZeroCount, uniqueValues.size());
```

**注意**: 使用`LogVerbose`，可能需要设置为verbose模式才能看到

---

## 当前可以做的对比

即使不添加更多C++诊断，也可以对比以下内容：

### Rust输出（从日志中提取）

#### 1. CEL加载统计
```bash
Get-Content rust_cel_diagnosis.log | Select-String -Pattern "CEL LOAD|TileTextureManager CEL Analysis" -Context 0,5
```

**关键输出**:
```
[CEL LOAD] Main CEL: 3547 frames, 0 decoded, 3547 need decoding
Special CEL frames: 18
  - Already decoded: 18
  - Need decoding: 0
```

#### 2. SOL诊断
```bash
Get-Content rust_cel_diagnosis.log | Select-String -Pattern "SOL DIAGNOSIS"
```

**关键输出**:
```
[RUST SOL DIAGNOSIS] Total pieces in SOL: 1258
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT: 37
```

#### 3. CEL解码日志（前50个）
```bash
Get-Content rust_cel_diagnosis.log | Select-String -Pattern "CEL DECODE" | Select-Object -First 50
```

---

## C++需要查找和添加的诊断

### 优先级1: CEL文件加载统计

**需要找到的位置**:
- `LoadLvlGFX()` 函数
- 加载`pDungeonCels`的地方
- 加载特殊CEL文件的地方

**需要输出**:
```cpp
// 主CEL
Log("[C++ CEL LOAD] Main CEL: {} frames", mainCelFrameCount);

// 特殊CEL
if (pSpecialCels) {
    Log("[C++ CEL LOAD] Special CEL: {} frames", specialCelFrameCount);
} else {
    Log("[C++ CEL LOAD] No special CEL loaded");
}
```

### 优先级2: MIN Frame分析

**位置**: `Source/levels/gendung.cpp::LoadMinData()` (Line 77-116)

**已有部分输出**:
```cpp
Log("=== DEBUG: MIN Frame Analysis:");
Log("  Total pieces: {}", numPieces);
Log("  MAX frame index in MIN: {}", maxFrameIdx);
```

**可以对比**: Rust的MIN Frame分析输出

### 优先级3: Transparency统计

**位置**: `Source/levels/gendung.cpp::LoadTransparency()` (Line 671-712)

**已有输出** (使用LogVerbose):
```cpp
LogVerbose("[C++ TRANSPARENCY] Loaded transparency data: {} tiles, {} non-zero values, {} unique values",
           size.width * size.height, nonZeroCount, uniqueValues.size());
```

**问题**: 使用`LogVerbose`，可能不会显示在正常日志中

**建议**: 改为`Log`确保输出

---

## 对比计划

### Step 1: 对比数据加载统计

| 项目 | Rust输出位置 | C++需要查找的位置 |
|------|-------------|------------------|
| Main CEL frames | `[CEL LOAD] Main CEL: 3547 frames` | 需要找到CEL加载函数 |
| Special CEL frames | `Special CEL frames: 18` | 需要找到特殊CEL加载 |
| SOL pieces | `Total pieces in SOL: 1258` | `LoadLevelSOLData()` |
| TRANSPARENT pieces | `Pieces with TRANSPARENT: 37` | `LoadLevelSOLData()` |
| MIN pieces | `Total pieces: X` | `LoadMinData()` |

### Step 2: 对比CEL解码

| 项目 | Rust输出 | C++需要添加 |
|------|---------|------------|
| frame_idx | `frame_idx=2415` | 需要添加 |
| source | `source=main` | 需要添加 |
| is_decoded | `is_decoded=false` | 需要添加 |
| raw_size | `raw_size=544` | 需要添加 |
| tile_type | `tile_type=LeftTriangle` | 需要添加 |
| decoded_size | `Decoded size: 992 bytes` | 需要添加 |

### Step 3: 对比渲染日志

**Rust已有**:
```
[RUST] Tile piece=168 block=5 at screen=(544,512) micro=(88,54) type=TransparentSquare is_floor=false mask=Solid transparency=false
```

**C++已有**:
```
[C++] Tile piece=168 block=5 at screen=(...) micro=(...) type=TransparentSquare is_floor=false mask=Solid transparency=false
```

**可以对比**: piece_id, block, tile_type, mask, transparency是否一致

---

## 立即可以做的事情

### 1. 对比SOL诊断

**Rust输出**:
```
[RUST SOL DIAGNOSIS] Total pieces in SOL: 1258
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT: 37
```

**C++需要查找**:
- 编译并运行C++项目
- 查找日志中的`[C++ SOL DIAGNOSIS]`输出
- 对比数字是否一致

### 2. 对比MIN Frame分析

**Rust输出** (从日志中提取):
```
=== MIN Frame Index Analysis ===
  Main CEL frames: 3547
  MAX frame index in MIN: 3547
```

**C++输出** (已有):
```
=== DEBUG: MIN Frame Analysis:
  Total pieces: X
  MAX frame index in MIN: Y
```

### 3. 对比Transparency加载

**Rust输出**:
```
TransList initialized: X active indices
```

**C++输出** (需要改为Log而不是LogVerbose):
```
[C++ TRANSPARENCY] Loaded transparency data: X tiles, Y non-zero values, Z unique values
```

---

## 下一步行动

### 立即执行

1. **编译并运行C++项目**
   - 保存所有日志输出到文件
   - 建议: `cpp_diagnosis.log`

2. **提取C++的关键诊断信息**
   ```bash
   # 查找SOL诊断
   findstr /C:"SOL DIAGNOSIS" cpp_diagnosis.log
   
   # 查找MIN分析
   findstr /C:"MIN Frame Analysis" cpp_diagnosis.log
   
   # 查找Transparency
   findstr /C:"TRANSPARENCY" cpp_diagnosis.log
   ```

3. **对比Rust和C++的输出**
   - SOL pieces数量是否一致？
   - TRANSPARENT pieces数量是否一致？
   - MIN frame index最大值是否一致？

### 后续添加（如果需要）

如果发现不一致，再添加C++端的CEL解码详细日志。

---

**创建日期**: 2024-12-04  
**目的**: 指导如何对比Rust和C++的诊断输出




