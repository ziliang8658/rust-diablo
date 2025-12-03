# Bug Fix: CLX特殊CEL二次解码问题

**日期**: 2025-12-02  
**严重程度**: 🔴 严重 - 导致特殊瓦片渲染成三角形碎片  
**状态**: ✅ 已修复

---

## 🐛 问题描述

### 症状

从截图观察到的问题：
1. ✅ 角色渲染正常
2. ✅ 树木方向正确（设置 `flip_v=true` 后）
3. ❌ 大量黑色三角形遮挡（棋盘格图案）
4. ❌ 树木和特殊元素显示为碎片/噪点

### 日志信息

```
No value: piece=865 block=2
No value: piece=865 block=4
No value: piece=865 block=6
```

---

## 🔍 根本原因

### 问题1: 二次解码导致乱码

**核心问题**: 特殊CEL文件（l1s.cel等）被解码了两次！

#### 数据流程分析

```
特殊CEL文件 (l1s.cel)
    ↓ CLX格式
from_clx_bytes()
    ↓ ClxSprite::from_bytes()  ← 第1次解码（CL2 RLE解码）
    ↓ decode_cl2_rle()
    ↓ pixels: Vec<Option<u8>>  ← 已经是索引色了！
    ↓ 转换为 raw_data: Vec<u8>
DungeonCelFrame { raw_data, is_decoded: false }  ← ❌ 错误标记！
    ↓
get_indexed_tile()
    ↓ decode_tile(tile_type, raw_data)  ← 第2次解码！
    ↓ 尝试用 LeftTriangle/Square 等解码器解码
    ↓ 💥 已解码的数据被当作编码数据再次解码
    ↓ 产生乱码/噪点
```

#### 为什么会二次解码？

1. **CLX解析**: `ClxSprite::from_bytes()` 已经将CL2 RLE数据解码为索引色像素
2. **错误标记**: `DungeonCelFrame` 的 `is_decoded` 被设为 `false`
3. **重复解码**: `get_indexed_tile()` 看到 `is_decoded=false`，再次调用 `decode_tile()`
4. **乱码产生**: 已解码的索引数据被当作编码的三角形/方块数据解码，产生随机噪点

### 问题2: Y轴方向

**CLX格式**: 像素从底部到顶部存储（bottom-to-top）  
**SDL2格式**: 像素从顶部到底部存储（top-to-bottom）

需要在渲染时设置 `flip_v=true` 来翻转Y轴。

---

## ✅ 修复方案

### 修复1: 添加 `is_decoded` 标志

**文件**: `src/resources/dungeon_cel.rs`

```rust
pub struct DungeonCelFrame {
    pub raw_data: Vec<u8>,
    pub is_decoded: bool,  // ✅ 标记数据是否已解码
}
```

**主CEL**（l1.cel）:
```rust
Ok(DungeonCelFrame {
    raw_data: data.to_vec(),
    is_decoded: false,  // 需要用tile decoder解码
})
```

**特殊CEL**（l1s.cel，CLX格式）:
```rust
frames.push(DungeonCelFrame {
    raw_data,  // CLX已解码的索引数据
    is_decoded: true,  // ✅ 已解码，不要再decode_tile()
});
```

### 修复2: 在 `get_indexed_tile()` 中检查标志

**文件**: `src/tiles/texture_manager.rs`

```rust
// Get raw data from CEL frame
let raw_data = &cel_frame.raw_data;

// ✅ KEY FIX: Check if data is already decoded (CLX special CEL files)
let decoded = if cel_frame.is_decoded {
    // Special CEL from CLX format - already decoded, just clone
    raw_data.clone()
} else {
    // Main CEL - needs decoding with appropriate TileType decoder
    decode_tile(tile_type, raw_data)?
};
```

### 修复3: 渲染时的翻转设置

**文件**: `src/engine/mod.rs`

```rust
self.canvas.copy_ex(
    &sdl_texture,
    None,
    sdl_rect,
    0.0,
    None,
    false,  // flip_h - no horizontal flip
    true,   // flip_v - vertical flip for CLX bottom-to-top format
)
```

---

## 📊 修复验证

### 预期效果

- ✅ 树木方向正确
- ✅ 树木显示清晰（不再是碎片/噪点）
- ✅ 黑色三角形减少或消失
- ✅ 地板和墙体正确混合

### 测试命令

```bash
cargo build --bin rust-diablo
cargo run --bin rust-diablo
```

---

## 🎓 经验教训

### 1. 多种数据格式要区分清楚

**Diablo 1 有多种CEL格式**:
- **主CEL** (l1.cel): 原始编码格式，需要 TileType 解码器
- **特殊CEL** (l1s.cel): CLX格式（CL2 RLE），已经解码

**错误做法**:
```rust
// ❌ 把所有CEL都当作编码数据处理
let decoded = decode_tile(tile_type, raw_data)?;  // 对CLX数据也解码 → 乱码
```

**正确做法**:
```rust
// ✅ 区分已解码和未解码数据
let decoded = if cel_frame.is_decoded {
    raw_data.clone()  // 已解码，直接使用
} else {
    decode_tile(tile_type, raw_data)?  // 未解码，需要解码
};
```

### 2. 坐标系和方向要统一

**不同系统的Y轴方向**:
- **CLX格式**: Bottom-to-top（从下到上）
- **SDL2**: Top-to-bottom（从上到下）
- **瓦片解码器输出**: Top-to-bottom

**解决方案**:
- CLX数据保持原始顺序
- 渲染时设置 `flip_v=true`

### 3. 调试技巧

**如何发现二次解码？**

1. **观察症状**: 碎片/噪点而不是完整图案
2. **检查数据流**: 跟踪数据从加载到渲染的全过程
3. **识别解码点**: 找到所有调用解码器的地方
4. **发现重复**: CLX已解码但又被decode_tile()处理

---

## 📝 相关文件修改

| 文件 | 修改内容 | 行数 |
|------|---------|------|
| `dungeon_cel.rs` | 添加 `is_decoded` 字段 | +2 |
| `dungeon_cel.rs` | `from_bytes()` 设置 `is_decoded=false` | +1 |
| `dungeon_cel.rs` | `from_clx_bytes()` 设置 `is_decoded=true` | +1 |
| `texture_manager.rs` | `get_indexed_tile()` 检查 `is_decoded` | +7 |
| `engine/mod.rs` | 渲染设置 `flip_v=true` | +1 |

---

**文档版本**: 1.0  
**修复日期**: 2025-12-02  
**作者**: AI Assistant (Claude Sonnet 4.5)  
**状态**: ✅ 已修复

---

**核心原则**: 不同格式的数据要区分清楚，避免对已解码数据重复解码！ 🎯


