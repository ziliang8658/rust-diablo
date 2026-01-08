# 桥渲染透明度问题诊断

## 问题描述

**症状**: 桥下方有个缺口小三角形，应该渲染成桥的颜色（木板），但却渲染成了底下水的颜色（深蓝色）

**截图观察**:
- C++原版：桥纹理清晰，缺口小三角形显示为桥的颜色
- Rust版本：桥的缺口小三角形显示为水的颜色（深蓝色）

## 可能的原因

### 原因1: Alpha通道为0

**现象**: 桥的某个block的alpha值全是0，导致完全透明，水透了过来

**机制**:
```rust
// indices_to_rgba() with preserve_zero=true
if index == 0 {
    [r, g, b, 0]  // alpha=0 → 透明
} else {
    [r, g, b, 255]  // alpha=255 → 不透明
}
```

**如果解码后的indexed_pixels全是0**:
- → rgba alpha全是0
- → 渲染时完全透明
- → 显示底层的水

### 原因2: 渲染顺序错误

**正确顺序**:
1. Phase 1: 渲染floor（包括水）
2. Phase 2: 渲染wall（包括桥）

**如果桥被判断为floor**:
- → 桥在Phase 1渲染
- → 水在Phase 2渲染
- → 水盖住了桥

### 原因3: MaskType应用错误

**可能**: 桥的某个三角形block应该用`MaskType::Left`或`Right`，但实际用了`Transparent`

**结果**: mask把不该透明的像素也设为透明了

### 原因4: 解码数据错误

**可能**: 桥的block解码后，数据全是0（表示透明）

**原因**:
- CEL数据本身有问题
- 解码算法错误
- Frame index错误

---

## 已添加的诊断代码

### Rust端

#### 桥piece的alpha诊断

**文件**: `rust-diablo/src/world/mod.rs` (Line 1081-1100)

**功能**:
- 检测桥piece (38, 39, 195, 198, 1125, 1126)
- 统计透明/不透明/半透明像素数量
- 输出前10个palette索引和RGBA值

**输出示例**:
```
🌉 [BRIDGE piece=38 block=0] Alpha analysis:
   Total: 992, Transparent(a=0): 500, Opaque(a=255): 492, Semi: 0
   First 10 palette indices: [0, 0, 0, 0, 0, 45, 46, 47, 48, 49]
   First 10 RGBA: [0, 0, 0, 0, 0, 0, 0, 0, ...]
```

**如果看到**:
- `Transparent: 992, Opaque: 0` → 全透明！这就是问题！
- `First 10 palette indices: [0, 0, 0, ...]` → 解码数据全是0！

---

## 测试步骤

### 1. 关闭Rust程序并重新编译

```bash
# 关闭rust-diablo.exe窗口
cd rust-diablo
cargo build
```

### 2. 运行并查看桥piece诊断

```bash
cargo run > bridge_diagnosis.log 2>&1
```

### 3. 查找桥的诊断输出

```powershell
Select-String -Path bridge_diagnosis.log -Pattern "BRIDGE piece=" -Context 0,5
```

### 4. 分析输出

#### 场景A: 全透明（alpha=0）
```
🌉 [BRIDGE piece=38 block=0] Alpha analysis:
   Total: 992, Transparent(a=0): 992, Opaque(a=255): 0, Semi: 0
   ⚠️  WARNING: NO OPAQUE PIXELS! All transparent or semi-transparent!
   First 10 palette indices: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

**问题**: 解码后的数据全是0  
**原因**: 可能是frame index错误，或解码算法错误

#### 场景B: 部分透明
```
🌉 [BRIDGE piece=38 block=0] Alpha analysis:
   Total: 992, Transparent(a=0): 500, Opaque(a=255): 492, Semi: 0
   First 10 palette indices: [0, 0, 0, 0, 0, 45, 46, 47, 48, 49]
```

**问题**: 前半部分是0（透明），后半部分有颜色  
**可能**: 
- 解码时padding不对
- Triangle的透明区域计算错误

#### 场景C: 全部不透明
```
🌉 [BRIDGE piece=38 block=0] Alpha analysis:
   Total: 992, Transparent(a=0): 0, Opaque(a=255): 992, Semi: 0
   First 10 palette indices: [45, 46, 47, 48, 49, 50, 51, 52, 53, 54]
```

**状态**: 解码正常！  
**那问题在别处**: 渲染顺序或mask应用

---

## 渲染顺序检查

### Phase分类

**检查piece=38是floor还是wall**:

```rust
// 在render_with_texture_manager中
if piece_id == 38 {
    let is_floor = self.is_floor_tile((tx, ty));
    println!("🌉 [BRIDGE] piece=38 at ({},{}) is_floor={}", tx, ty, is_floor);
}
```

**正确应该是**:
- 桥应该是 `is_floor=false` → 在Phase 2渲染（wall阶段）
- 水应该是 `is_floor=true` → 在Phase 1渲染（floor阶段）

**如果桥是is_floor=true**:
- → 桥在Phase 1渲染
- → 后续的wall会盖住桥
- → 导致桥被遮挡

---

## C++端对比

### 添加相同诊断

**文件**: `Source/engine/render/scrollrt.cpp::DrawCell()`

在渲染桥piece时添加：

```cpp
// After getting levelPieceId
if (levelPieceId == 38 || levelPieceId == 39 || 
    levelPieceId == 195 || levelPieceId == 198 ||
    levelPieceId == 1125 || levelPieceId == 1126) {
    
    bool isFloor = IsFloor(tilePosition);
    Log("[C++ BRIDGE] piece={} at ({},{}) is_floor={} transparency={}",
        levelPieceId, tilePosition.x, tilePosition.y, isFloor, transparency);
}
```

**对比C++和Rust**:
- is_floor应该一致
- transparency应该一致

---

## 下一步行动

### 立即执行

1. **关闭Rust程序**
2. **重新编译**: `cargo build`
3. **运行**: `cargo run > bridge_diagnosis.log 2>&1`
4. **查看桥诊断**: `Select-String -Pattern "BRIDGE" bridge_diagnosis.log`

### 根据结果

#### 如果看到"NO OPAQUE PIXELS"
→ 问题在解码，indexed_pixels全是0  
→ 需要检查decode_tile()的实现

#### 如果alpha正常
→ 问题在渲染顺序或mask  
→ 需要检查is_floor判断和Phase分类

#### 如果is_floor=true（桥被判断为floor）
→ 这就是问题！  
→ 需要修复is_floor_tile()的判断逻辑

---

## 参考代码

### Rust
- `rust-diablo/src/world/mod.rs` - 渲染和透明度检查
- `rust-diablo/src/tiles/decoder/triangle.rs` - 三角形解码

### C++  
- `Source/engine/render/scrollrt.cpp` - DrawCell和IsFloor
- `Source/levels/gendung.cpp` - SOLData定义

---

**创建日期**: 2024-12-04  
**目的**: 诊断桥渲染透明度问题，定位缺口小三角形显示水颜色的原因




