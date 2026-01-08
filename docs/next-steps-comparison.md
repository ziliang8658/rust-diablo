# 下一步：C++ vs Rust 对比

## ✅ 已完成

### Rust端诊断代码
- ✅ CEL加载统计
- ✅ SOL诊断
- ✅ CEL解码详细日志（前50个）
- ✅ MIN Frame分析
- ✅ 编译成功，可以运行

### C++端诊断代码
- ✅ SOL诊断（`Source/levels/gendung.cpp`）
- ✅ CEL加载部分统计（`Source/diablo.cpp` Line 1379-1387）
- ✅ MIN Frame分析（`Source/levels/gendung.cpp`）
- ⚠️ CEL解码详细日志（未添加，但可以先对比基础数据）

---

## 🚀 立即执行

### 1. 编译并运行C++项目

```bash
# 进入build目录
cd build

# 编译（如果还没编译）
cmake --build . --config Debug

# 运行并保存日志（根据您的可执行文件名调整）
DevilutionX.exe > cpp_diagnosis.log 2>&1

# 或者如果是其他名字
.\DevilutionX.exe *> cpp_diagnosis.log
```

### 2. 提取关键信息

**使用PowerShell提取关键诊断信息**:

```powershell
# 1. SOL诊断
Select-String -Path cpp_diagnosis.log -Pattern "SOL DIAGNOSIS" | Format-Table

# 2. CEL加载
Select-String -Path cpp_diagnosis.log -Pattern "CEL|town.cel" | Select-Object -First 10

# 3. MIN Frame分析
Select-String -Path cpp_diagnosis.log -Pattern "MIN Frame Analysis" -Context 0,5

# 4. Transparency
Select-String -Path cpp_diagnosis.log -Pattern "TRANSPARENCY"
```

### 3. 对比关键数据

创建这个对比表格：

```
┌─────────────────────────────────┬──────────┬──────────┬────────┐
│ 项目                            │ Rust     │ C++      │ 一致？ │
├─────────────────────────────────┼──────────┼──────────┼────────┤
│ SOL Total pieces                │ 1258     │ ?        │ ?      │
│ SOL TRANSPARENT pieces          │ 37       │ ?        │ ?      │
│ Main CEL frames                 │ 3547     │ ?        │ ?      │
│ Special CEL frames              │ 18       │ ?        │ ?      │
│ MIN MAX frame index             │ 3547     │ ?        │ ?      │
│ TRANSPARENT pieces (first 10)   │ 见下方   │ ?        │ ?      │
└─────────────────────────────────┴──────────┴──────────┴────────┘
```

**Rust的TRANSPARENT pieces (first 10)**:
```
[1134, 1136, 1138, 1170, 1171, 1172, 1173, 1174, 1175, 1176]
```

---

## 🔍 关键对比点

### 优先级1: SOL数据一致性

**为什么重要**: SOL定义了每个piece的属性（TRANSPARENT等），如果这个不一致，会导致渲染完全不同。

**对比项**:
1. Total pieces数量（可能不同，因为C++可能加载更多）
2. **TRANSPARENT pieces数量（必须一致）**
3. First 10 TRANSPARENT pieces列表（应该一致）

### 优先级2: CEL文件一致性

**为什么重要**: CEL文件包含所有纹理数据，如果frame数量不一致，说明加载或解析有问题。

**对比项**:
1. **Main CEL frame count（必须一致）**
2. **Special CEL frame count（必须一致）**
3. MAX frame index in MIN（应该一致）

### 优先级3: Transparency数据

**为什么重要**: Transparency数据决定哪些区域需要透明混合。

**对比项**:
1. Non-zero transparency values数量
2. Unique transparency values数量
3. Unique values列表

---

## 📋 Rust端已有数据（供对比）

### CEL加载
```
[CEL LOAD] Main CEL: 3547 frames, 0 decoded, 3547 need decoding
Special CEL frames: 18
  - Already decoded: 18
  - Need decoding: 0
```

### SOL诊断
```
[RUST SOL DIAGNOSIS] Total pieces in SOL: 1258
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT: 37
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT_LEFT: 0
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT_RIGHT: 0
[RUST SOL DIAGNOSIS] First 10 transparent pieces: [1134, 1136, 1138, 1170, 1171, 1172, 1173, 1174, 1175, 1176]
```

### MIN Frame分析
```
=== MIN Frame Index Analysis ===
  Main CEL frames: 3547
  MAX frame index in MIN: 3547
  Unique frame indices used: 3547
```

---

## 🎯 如果数据一致

如果所有基础数据都一致，说明：
- ✅ 文件加载正确
- ✅ 数据解析正确
- ⚠️ **问题可能在渲染逻辑**

**下一步**: 对比渲染日志，找出piece=XXX时，Rust和C++的渲染参数差异。

---

## 🚨 如果数据不一致

如果发现不一致，需要：

1. **记录差异** - 具体哪些数字不一致
2. **检查文件路径** - 是否加载了相同的文件
3. **检查解析逻辑** - 对比Rust和C++的解析代码

---

## 📄 相关文档

- `rust-diablo/docs/cpp-diagnosis-instructions.md` - C++诊断详细说明
- `rust-diablo/docs/cpp-rust-comparison-guide.md` - 完整对比指南
- `rust-diablo/docs/cel-decode-diagnosis-plan.md` - CEL解码诊断计划

---

**创建日期**: 2024-12-04  
**下一步**: 编译运行C++，提取诊断信息，对比基础数据




