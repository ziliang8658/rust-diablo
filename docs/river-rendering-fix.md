# 河流渲染问题修复方案

## 核心问题发现

经过对 C++ 和 Rust 代码的对比分析,发现以下关键问题：

### 问题 1: Floor Tile 渲染逻辑不一致

**C++ 逻辑** (`scrollrt.cpp DrawFloorTile`):
```cpp
// 只渲染 blocks 0-1
// 强制使用 LeftTriangle/RightTriangle
// 使用 MaskType::Solid（渲染所有像素，不跳过）
```

**Rust 逻辑** (`world/mod.rs draw_floor_at`):
```rust
// Line 1259-1287
self.render_micro_tile(..., 0, ..., true, MaskType::Solid)?;  // is_floor=true
self.render_micro_tile(..., 1, ..., true, MaskType::Solid)?;
```

问题：`is_floor=true` 会在 `render_micro_tile` 中强制使用 LeftTriangle/RightTriangle，这是**正确的**。

但关键问题在于：**河流的 piece_id 对应的 SOL 属性可能错误地返回 is_floor=true**

### 问题 2: Triangle Tile Padding 可能有误

**位置**: `world/mod.rs Line 736-791` (`pad_triangle_to_32x31`)

**row widths 数组**:
```rust
const WIDTHS: [usize; 31] = [
    2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
    30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2
];
```

这个数组是正确的（总和 = 512）。

但 padding 逻辑可能有问题：
- LeftTriangle: 应该 left-aligned（数据在左侧，右侧填充 0）
- RightTriangle: 应该 right-aligned（左侧填充 0，数据在右侧）

### 问题 3: DirectRenderer vs TextureManager 行为差异

用户可能在使用 TextureManager 模式，但 DirectRenderer 的行为可能更接近 C++。

## 修复步骤

### Step 1: 启用详细的诊断日志

Rust 代码已经包含诊断日志（Line 1083-1122），但可能需要增加更多信息。

建议添加：
1. 输出河流和土路的具体 piece_id
2. 输出它们的 SOL 属性
3. 对比 C++ 的相同位置

### Step 2: 使用 DirectRenderer 模式进行测试

DirectRenderer 更接近 C++ 的像素级渲染，可以排除 SDL2 纹理渲染的问题。

在运行时按 `R` 键切换模式：
```rust
// world/mod.rs Line 333
pub fn toggle_direct_renderer(&mut self) {
    self.use_direct_renderer = !self.use_direct_renderer;
}
```

### Step 3: 验证 Triangle Padding

创建一个测试脚本来验证 padding 逻辑：

```rust
// 测试 LeftTriangle padding
let compact_data = vec![0u8; 512]; // 全黑数据
let padded = World::pad_triangle_to_32x31(&compact_data, TileType::LeftTriangle);
assert_eq!(padded.len(), 992); // 32 * 31

// 检查第一行（width=2, left-aligned）
assert_eq!(padded[0], 0);  // 数据
assert_eq!(padded[1], 0);  // 数据
assert_eq!(padded[2], 0);  // 填充
// ...
assert_eq!(padded[31], 0); // 填充
```

### Step 4: 检查 SOL 数据

运行时输出：
```
[RUST SOL DIAGNOSIS] Checking SOL data for TRANSPARENT properties:
[RUST SOL DIAGNOSIS] Total pieces in SOL: 1512
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT: 0
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT_LEFT: 0
[RUST SOL DIAGNOSIS] Pieces with TRANSPARENT_RIGHT: 0
[RUST SOL DIAGNOSIS] ⚠️ NO PIECES WITH TRANSPARENT PROPERTY FOUND!
```

如果 TRANSPARENT 属性为 0，说明 Town SOL 数据可能没有 TRANSPARENT 标记。

这意味着 `check_transparency` 会返回 `false`，所有 tiles 都使用 `MaskType::Solid`。

## 具体修复方案

### 方案 A: 检查并修复 SOL 数据加载

**位置**: `tiles/sol.rs Line 169-234`

检查 Town 是否需要特殊的 SOL 修复（类似 Cathedral 的修复）。

### 方案 B: 添加 Town 特定的渲染逻辑

如果 Town 的河流渲染需要特殊处理，可能需要：

1. 识别河流的 piece_id 范围
2. 对这些 pieces 应用特定的渲染逻辑

### 方案 C: 修复 Triangle Decoding

如果黑色缺口是解码问题，可能需要：

1. 检查 `triangle.rs` 的解码器
2. 确保输出的 512 bytes 是正确的
3. 验证 padding 逻辑

## 紧急修复（临时方案）

如果问题紧急，可以：

1. **切换到 DirectRenderer 模式** - 更接近 C++ 行为
2. **禁用透明度** - 强制所有 tiles 使用 MaskType::Solid
3. **增加诊断输出** - 定位具体的错误 piece_id

## 验证方法

### 1. 运行 C++ 版本并记录日志

```bash
# 在 C++ 版本中启用调试输出
# scrollrt.cpp Line 1160-1175 的日志
./devilutionx > cpp_log.txt 2>&1
```

### 2. 运行 Rust 版本并记录日志

```bash
cargo run --release > rust_log.txt 2>&1
```

### 3. 对比日志

查找以下关键信息：
- `[FLOOR CHECK]` - is_floor 判断
- `[FLOOR SKIPPED]` - 哪些 tiles 被跳过
- piece_id 和 SOL 属性的对应关系

### 4. 导出瓦片图像

使用 export_indexed_tile_as_png 导出可疑的 tiles 进行视觉检查。

## 预期结果

修复后应该看到：
- 河流正确渲染在中心
- 土路在河流两岸
- 没有黑色缺口
- 所有 triangle tiles 正确对齐
