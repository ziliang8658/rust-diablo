# Triangle解码器修复记录

## 发现的问题

### 症状
**桥下方缺口小三角形显示为水的颜色（深蓝色）而不是桥的颜色**

### 诊断结果

**piece=38 (桥) block=0 (LeftTriangle)**:
```
First 10 palette indices: [57, 57, 0, 0, 0, 0, 0, 0, 0, 0]
                          ^^^^^^^ 只有前2个像素有数据
```

**问题**: 解码后的数据格式错误，Row 1的数据（4个像素）丢失，全是0（透明）

---

## 根本原因

### Rust的错误实现（修复前）

**文件**: `rust-diablo/src/tiles/decoder/triangle.rs`

```rust
// ❌ 错误：每行填充到32宽度
output[dst..dst + width1].copy_from_slice(&raw_data[src..src + width1]);
src += width1;
dst += 32;  // ❌ 每行移动32，导致padding
```

**数据布局（错误）**:
```
[pixel0, pixel1, 0, 0, ... (30个0), pixel2, pixel3, pixel4, pixel5, 0, 0, ... (28个0)]
 ^^^^^^^^^^^^^^ Row0填充到32 ^^^   ^^^^^^^^^^^^^^^^^^^^^ Row1填充到32 ^^^^^^
```

### C++的正确实现

**文件**: `Source/levels/reencode_dun_cels.cpp::ReencodeDungeonCelsLeftTriangle()`

```cpp
// ✅ 正确：直接拷贝，不填充
std::memcpy(dst, src, width);
src += width;
dst += width;  // ✅ 只移动实际宽度，数据紧凑
```

**数据布局（正确）**:
```
[pixel0, pixel1, pixel2, pixel3, pixel4, pixel5, ...]
 ^^^^^^^ Row0    ^^^^^^^^^^^^^ Row1
```

---

## 修复内容

### 修改1: LeftTriangle解码器

**文件**: `rust-diablo/src/tiles/decoder/triangle.rs`

**修改**: Line 57-163

```rust
pub fn decode_left_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    // ⚠️ KEY FIX: Output compact data (no padding to 32)
    let mut output = Vec::new();  // 动态Vec，不预分配
    let mut src = 0;

    // Lower half
    for i in 0..8 {
        src += 2; // Skip padding
        let width1 = 2 + i * 4;
        output.extend_from_slice(&raw_data[src..src + width1]);
        src += width1;
        // ⚠️ NO dst += 32! 移除padding
        
        let width2 = 4 + i * 4;
        output.extend_from_slice(&raw_data[src..src + width2]);
        src += width2;
        // ⚠️ NO dst += 32! 移除padding
    }
    
    // Upper half: 类似修改
    ...
    
    Ok(output)  // 返回紧凑数据
}
```

### 修改2: RightTriangle解码器

**文件**: `rust-diablo/src/tiles/decoder/triangle.rs`

**修改**: Line 188-277

类似LeftTriangle，改为紧凑输出。

---

## 预期效果

### 修复前
```
First 10 palette indices: [57, 57, 0, 0, 0, 0, 0, 0, 0, 0]
Total: 992, Transparent(a=0): 480, Opaque(a=255): 512
```
- 48% transparent（应该有纹理的地方变透明了）

### 修复后
```
First 10 palette indices: [57, 57, 45, 46, 47, 48, ...]
Total: 992, Transparent(a=0): ~240, Opaque(a=255): ~752
```
- 只有三角形斜边外才透明
- 三角形内部全部有纹理

---

## 其他修复

### 修改3: 清屏逻辑

**文件**: `rust-diablo/src/game.rs` (Line 1101)

```rust
// ✅ Clear screen every frame to prevent ghosting
self.engine.canvas_mut().set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
self.engine.canvas_mut().clear();
```

**问题**: 之前清屏代码被注释掉，导致每帧累积渲染

**症状**: 角色和特效有"鬼影"

---

## 测试步骤

### 1. 编译（已完成✅）

```bash
cd rust-diablo
cargo build
```

### 2. 运行并查看桥渲染

```bash
cargo run
```

**关注点**:
1. 桥的纹理是否正确显示？
2. 缺口小三角形是否显示为桥的颜色？
3. 是否还有"鬼影"效果？

### 3. 查看PNG导出（可选）

程序会自动导出到：
```
debug_textures/cpp_pieces/piece_0038_block_0.png
debug_textures/cpp_pieces/piece_0038_block_1.png
```

打开查看纹理是否正确。

---

## 影响范围

### 直接影响
- ✅ LeftTriangle和RightTriangle的渲染正确性
- ✅ 所有使用这两种TileType的piece（包括桥）

### 间接影响
- ✅ 整体地图渲染质量
- ✅ "鬼影"问题消失

### 不影响
- ✅ Square和TransparentSquare（使用不同解码器）
- ✅ Trapezoid（虽然内部调用Triangle，但需要单独验证）

---

## 参考

### C++实现
- `Source/levels/reencode_dun_cels.cpp` - Triangle解码算法
- `Source/engine/render/dun_render.cpp` - Triangle渲染

### Rust实现
- `rust-diablo/src/tiles/decoder/triangle.rs` - Triangle解码器（已修复）
- `rust-diablo/src/engine/mod.rs` - 渲染引擎
- `rust-diablo/src/game.rs` - 主循环和清屏（已修复）

---

## 下一步

1. **运行程序验证**
2. **截图对比C++和Rust的桥渲染**
3. **检查其他TileType是否也有类似问题**

---

**修复日期**: 2024-12-04  
**问题**: Triangle解码器输出格式错误（padding vs compact）  
**解决**: 改为compact输出，匹配C++格式




