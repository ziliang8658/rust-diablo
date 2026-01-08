# C++ vs Rust 诊断对比指南

## 📋 当前诊断状态

### ✅ Rust端（已完成）

1. **CEL加载统计** ✅
   - Main CEL: frame数量、decoded状态
   - Special CEL: frame数量、decoded状态

2. **SOL诊断** ✅
   - Total pieces数量
   - TRANSPARENT pieces数量

3. **CEL解码日志** ✅
   - 前50个frame的详细解码信息

4. **MIN Frame分析** ✅
   - MAX frame index
   - Unique frame indices

### ✅ C++端（部分完成）

1. **SOL诊断** ✅
   - 已添加（`Source/levels/gendung.cpp::LoadLevelSOLData()` Line 538-577）

2. **CEL加载统计** ⚠️ 部分
   - 已有部分输出（`Source/diablo.cpp::LoadLvlGFX()` Line 1379-1387）
   - 但输出格式与Rust不一致

3. **MIN Frame分析** ✅
   - 已有输出（`Source/levels/gendung.cpp::LoadMinData()` Line 110-114）

4. **CEL解码日志** ❌ 未添加
   - 需要添加渲染时的解码日志

---

## 🎯 立即可以对比的内容

### 1. SOL诊断对比

#### Rust输出
```
[RUST SOL DIAGNOSIS] Total pieces in SOL: 1258
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT: 37
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT_LEFT: 0
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT_RIGHT: 0
[RUST SOL DIAGNOSIS] First 10 transparent pieces: [1134, 1136, 1138, 1170, 1171, 1172, 1173, 1174, 1175, 1176]
```

#### C++输出（应该类似）
```
[C++ SOL DIAGNOSIS] Total pieces in SOL: 2048
[C++ SOL DIAGNOSIS] Pieces with TRANSPARENT: 37
[C++ SOL DIAGNOSIS] Pieces with TRANSPARENT_LEFT: 0
[C++ SOL DIAGNOSIS] Pieces with TRANSPARENT_RIGHT: 0
[C++ SOL DIAGNOSIS] First 10 transparent pieces: [...]
```

**对比点**:
- TRANSPARENT pieces数量应该一致（37）
- First 10 transparent pieces应该一致

### 2. CEL加载统计对比

#### Rust输出
```
[CEL LOAD] Main CEL: 3547 frames, 0 decoded, 3547 need decoding
Special CEL frames: 18
  - Already decoded: 18
  - Need decoding: 0
```

#### C++输出（已有部分）
```
=== DEBUG: Loaded Town CEL: levels\towndata\town.cel
=== DEBUG: Main CEL frame count: 3547
=== DEBUG: Loaded Town Special CEL: levels\towndata\towns (width=64)
=== DEBUG: Special CEL has 18 sprites
```

**对比点**:
- Main CEL frame count应该一致（3547）
- Special CEL frame count应该一致（18）

### 3. MIN Frame分析对比

#### Rust输出
```
=== MIN Frame Index Analysis ===
  Main CEL frames: 3547
  MAX frame index in MIN: 3547
  Unique frame indices used: 3547
```

#### C++输出（已有）
```
=== DEBUG: MIN Frame Analysis:
  Total pieces: X
  Total uint16 values: Y
  MAX frame index in MIN: Z
  Blocks with frames: W
```

**对比点**:
- MAX frame index应该一致（3547）

---

## 📝 建议的C++改进

### 改进1: 统一CEL加载输出格式

**位置**: `Source/diablo.cpp::LoadLvlGFX()` (Line 1379-1387)

**当前代码**:
```cpp
// DEBUG: Count frames in pDungeonCels and pSpecialCels
const uint32_t mainCelFrameCount = *reinterpret_cast<const uint32_t*>(pDungeonCels.get());
Log("=== DEBUG: Main CEL frame count: {}", mainCelFrameCount);

if (pSpecialCels.has_value()) {
    ClxSpriteList specialList { *pSpecialCels };
    Log("=== DEBUG: Special CEL has {} sprites", specialList.numSprites());
}
```

**建议改为**（匹配Rust格式）:
```cpp
// [DIAGNOSIS] CEL loading statistics
const uint32_t mainCelFrameCount = Swap32LE(*reinterpret_cast<const uint32_t*>(pDungeonCels.get()));
Log("[C++ CEL LOAD] Main CEL: {} frames, need decoding", mainCelFrameCount);

if (pSpecialCels.has_value()) {
    ClxSpriteList specialList { *pSpecialCels };
    Log("[C++ CEL LOAD] Special CEL: {} frames, already decoded (CLX format)", specialList.numSprites());
} else {
    Log("[C++ CEL LOAD] No special CEL loaded");
}
```

### 改进2: Transparency输出级别

**位置**: `Source/levels/gendung.cpp::LoadTransparency()` (Line 702)

**当前代码**:
```cpp
LogVerbose("[C++ TRANSPARENCY] Loaded transparency data: {} tiles, {} non-zero values, {} unique values",
           size.width * size.height, nonZeroCount, uniqueValues.size());
```

**建议改为**（使用Log确保输出）:
```cpp
Log("[C++ TRANSPARENCY] Loaded transparency data: {} tiles, {} non-zero values, {} unique values",
    size.width * size.height, nonZeroCount, uniqueValues.size());
```

---

## 🚀 执行步骤

### Step 1: 编译并运行C++项目

1. **编译C++项目**
   ```bash
   cd build
   cmake --build . --config Debug
   ```

2. **运行并保存日志**
   ```bash
   # Windows
   DevX.exe > cpp_diagnosis.log 2>&1
   
   # 或使用PowerShell
   .\DevX.exe *> cpp_diagnosis.log
   ```

### Step 2: 提取关键诊断信息

**使用PowerShell**:
```powershell
# SOL诊断
Select-String -Path cpp_diagnosis.log -Pattern "SOL DIAGNOSIS" -Context 0,5

# CEL加载
Select-String -Path cpp_diagnosis.log -Pattern "CEL|town.cel|towns" -Context 0,2

# MIN Frame分析
Select-String -Path cpp_diagnosis.log -Pattern "MIN Frame Analysis" -Context 0,5

# Transparency
Select-String -Path cpp_diagnosis.log -Pattern "TRANSPARENCY" -Context 0,2
```

**使用cmd**:
```cmd
findstr /C:"SOL DIAGNOSIS" cpp_diagnosis.log
findstr /C:"CEL" cpp_diagnosis.log
findstr /C:"MIN Frame Analysis" cpp_diagnosis.log
findstr /C:"TRANSPARENCY" cpp_diagnosis.log
```

### Step 3: 对比结果

创建对比表格：

| 项目 | Rust | C++ | 一致？ |
|------|------|-----|--------|
| SOL Total pieces | 1258 | ? | ? |
| TRANSPARENT pieces | 37 | ? | ? |
| Main CEL frames | 3547 | ? | ? |
| Special CEL frames | 18 | ? | ? |
| MAX frame index | 3547 | ? | ? |

---

## 🔍 如果发现不一致

### 场景1: SOL pieces数量不同

**可能原因**:
- SOL文件加载路径不同（nlevels vs levels）
- SOL文件版本不同

**检查**:
- 对比两边的SOL文件路径
- 检查是否有文件大小差异

### 场景2: CEL frame数量不同

**可能原因**:
- CEL文件加载路径不同
- CEL文件解析方式不同

**检查**:
- 对比CEL文件路径
- 检查frame count计算方法是否一致

### 场景3: TRANSPARENT pieces不同

**可能原因**:
- SOL数据解析差异
- TileProperties标志位理解不同

**检查**:
- 对比First 10 transparent pieces列表
- 检查TileProperties::Transparent的定义

---

## 📄 相关文件

### Rust端
- `rust-diablo/src/resources/dungeon_cel.rs` - CEL加载和诊断
- `rust-diablo/src/tiles/texture_manager.rs` - 纹理管理器诊断
- `rust-diablo/src/tiles/sol.rs` - SOL诊断
- `rust-diablo/src/world/mod.rs` - 渲染日志

### C++端
- `Source/levels/gendung.cpp` - SOL和Transparency诊断
- `Source/diablo.cpp` - CEL加载
- `Source/engine/render/scrollrt.cpp` - 渲染日志

---

**创建日期**: 2024-12-04  
**目的**: 指导如何对比Rust和C++的诊断输出，找出差异




