# DirectRenderer vs TextureRenderer 对比分析

## 🎯 **核心差异**

| 特性 | DirectRenderer (C++风格) | TextureRenderer (当前) |
|------|-------------------------|----------------------|
| 数据格式 | 8-bit indexed (512B/1024B) | 32-bit RGBA (padded) |
| 渲染方式 | 逐像素写framebuffer | SDL2纹理渲染 |
| 透明处理 | 不写入（保留原值） | alpha=0 + BlendMode |
| 内存占用 | 512B/triangle | 992B/triangle (padded) + RGBA转换 |
| 性能 | 高（少内存拷贝） | 中（需RGBA转换） |
| C++相似度 | 100% | ~70% |

---

## 📐 **架构对比**

### **DirectRenderer架构**

```
Tile数据流 (C++完全复刻):

1. CEL文件 (压缩)
   ↓
2. Decoder → Indexed data (8-bit, compact)
   - LeftTriangle: 512 bytes (变宽行，连续存储)
   - Square: 1024 bytes (32x32连续)
   ↓
3. DirectRenderer.render_tile()
   - 逐像素读取indexed data
   - 写入8-bit framebuffer
   - 透明像素(0): 不写入 ← 关键！
   ↓
4. Framebuffer.to_rgba()
   - 应用palette: indexed → RGBA
   ↓
5. SDL2 Texture
   - 渲染到屏幕
```

**关键代码**:
```rust
// DirectRenderer - 透明像素不写入
pub fn set_pixel(&mut self, x: usize, y: usize, palette_index: u8) {
    if palette_index != 0 {  // 只写非透明像素
        self.pixels[y * width + x] = palette_index;
    }
    // palette_index == 0: 透明，什么都不做！
}
```

### **TextureRenderer架构**

```
Tile数据流 (SDL2纹理):

1. CEL文件 (压缩)
   ↓
2. Decoder → Indexed data (8-bit, padded)
   - LeftTriangle: 992 bytes (32 bytes/row × 31 rows)
   ↓
3. Palette.indices_to_rgba()
   - 立即转换: indexed → RGBA
   - palette_index 0 → RGBA(?, ?, ?, 0)
   ↓
4. apply_mask_to_rgba()
   - 应用MaskType (Left/Right/Transparent)
   - 修改alpha值实现透明
   ↓
5. Engine.draw_rgba_texture()
   - 创建SDL2 texture
   - texture.set_blend_mode(Blend)
   - canvas.copy_ex()
```

**关键代码**:
```rust
// TextureRenderer - 使用alpha通道
pub fn to_rgba(&self, index: u8, use_transparency: bool) -> [u8; 4] {
    if use_transparency && index == 0 {
        [0, 0, 0, 0]  // 透明：alpha=0
    } else {
        let color = self.colors[index as usize];
        [color.r, color.g, color.b, 255]  // 不透明：alpha=255
    }
}

// 然后SDL2的BlendMode处理alpha
canvas.copy_ex(&texture, ...)  // alpha=0的像素会blend
```

---

## 🔍 **透明处理机制差异**

### **DirectRenderer: 不写入（C++行为）**

```rust
framebuffer[y][x] = 之前的值  // 透明像素不写入

结果：
Frame 1: [黑色背景] ← 初始化
Frame 2: [地板tile] ← Phase 1渲染
Frame 3: [墙壁tile] ← Phase 2渲染（透明区域显示地板）
         透明像素 → 显示地板 ✓
```

**优点**:
- 完美分层
- 不需要clear()
- 透明区域自动显示下层

### **TextureRenderer: Alpha Blending（SDL2行为）**

```rust
texture[透明像素] = RGBA(0,0,0,0)
canvas.copy_ex(&texture, ...)  // SDL2混合

结果：
- alpha=0像素根据BlendMode处理
- 可能被GPU优化跳过
- 但texture本身占据整个矩形区域

问题：
如果在Phase 1和Phase 2之间clear():
  ↓ Phase 1渲染地板
  ↓ clear() ← 擦除地板！
  ↓ Phase 2渲染墙壁
  ↓ 墙壁透明区域 → 显示clear的颜色 ✗
```

**解决方案（当前）**:
- 移除clear()
- 依赖上一帧内容
- 但texture覆盖区域可能影响下层

---

## 📏 **数据格式对比**

### **Compact格式 (DirectRenderer)**

**LeftTriangle: 512 bytes**
```
内存布局（连续存储）:
[row0: 2字节][row1: 4字节][row2: 6字节]...[row15: 32字节]
[row16: 30字节]...[row30: 2字节]

优点：
- 节省内存 (512 vs 992)
- 完全匹配C++
- cache友好（连续）

缺点：
- 渲染时需要计算每行偏移
- 代码稍复杂
```

### **Padded格式 (TextureRenderer)**

**LeftTriangle: 992 bytes (32 bytes/row)**
```
内存布局（每行32字节）:
[row0: 2字节 + 30字节padding]
[row1: 4字节 + 28字节padding]
...
[row15: 32字节]
...
[row30: 2字节 + 30字节padding]

优点：
- 代码简单（固定row width）
- SDL2兼容（矩形区域）

缺点：
- 浪费内存 (~48%)
- 不匹配C++
```

---

## 🖼️ **渲染流程对比**

### **DirectRenderer**

```rust
// Phase 1: 地板
for floor_tile in floor_tiles {
    for pixel in tile.indexed_pixels {
        if pixel != 0 {  // 非透明
            framebuffer[y][x] = pixel;  // 写入
        }
        // pixel == 0: 跳过，不写入
    }
}

// Phase 2: 墙壁
for wall_tile in wall_tiles {
    for pixel in tile.indexed_pixels {
        if pixel != 0 {
            framebuffer[y][x] = pixel;  // 覆盖或写入
        }
        // pixel == 0: 跳过，保留Phase 1的内容
    }
}

// 转换为RGBA
for indexed in framebuffer {
    rgba[i] = palette[indexed];
}

// 渲染
canvas.copy(&framebuffer_texture, ...);
```

**特点**:
- 单次texture更新
- 透明自然分层
- 完全不需要clear()

### **TextureRenderer**

```rust
// Phase 1: 地板
for floor_tile in floor_tiles {
    let rgba = indexed_to_rgba(tile.pixels, palette);
    let texture = create_texture(rgba);
    canvas.copy_ex(&texture, ...);  // 每个tile一个texture
}

// Phase 2: 墙壁
for wall_tile in wall_tiles {
    let rgba = indexed_to_rgba(tile.pixels, palette);
    apply_mask(&rgba, mask_type);  // 修改alpha
    let texture = create_texture(rgba);
    canvas.copy_ex(&texture, ...);  // 覆盖到地板上
}
```

**特点**:
- 多次texture创建/更新
- 依赖BlendMode处理alpha
- 如果clear()会破坏分层

---

## ⚡ **性能对比**

### **内存占用**

**场景**: 100个可见tile

| 方案 | Triangle | Square | 总计 |
|------|----------|--------|------|
| Direct | 512B × 100 = 50KB | 1024B × 50 = 50KB | ~100KB |
| Texture (padded) | 992B × 100 = 97KB | 1024B × 50 = 50KB | ~147KB |
| Texture (RGBA) | 3968B × 100 = 387KB | 4096B × 50 = 200KB | ~587KB |

**结论**: DirectRenderer节省 ~85% 内存！

### **渲染性能**

| 操作 | DirectRenderer | TextureRenderer |
|------|----------------|-----------------|
| Decode | 一次 | 一次 |
| RGBA转换 | 一次（整个framebuffer） | 每个tile |
| Texture创建 | 一次（framebuffer） | 每个tile |
| GPU调用 | 一次copy | N次copy_ex |

**结论**: DirectRenderer减少 ~99% 的GPU调用！

---

## 🐛 **可能的问题点**

### **DirectRenderer的潜在问题**

1. **Y坐标系统**
   - C++: bottom-to-top
   - SDL2: top-to-bottom
   - **需要转换**: `y = screen_y + (height - 1 - row)`

2. **Row数据偏移**
   - Compact格式需要手动计算每行起始位置
   - 如果width数组错误 → 数据错位

3. **边界检查**
   - 屏幕外的像素需要裁剪
   - 否则越界访问

### **TextureRenderer的潜在问题**

1. **Alpha Blending**
   - BlendMode设置不正确
   - alpha=0像素仍可能覆盖下层

2. **Padding数据**
   - Padding区域可能有脏数据
   - 需确保padding为0

3. **Texture缓存**
   - 每帧创建texture开销大
   - 需要缓存机制

---

## 🎯 **当前黑色artifacts的可能原因**

### **如果是DirectRenderer问题**

1. **Row width错误**
   ```rust
   // 验证
   widths.iter().sum() == 512?
   ```

2. **Y坐标偏移**
   ```rust
   // 当前: y = screen_y + (30 - row)
   // 是否应该: y = screen_y + row?
   ```

3. **Left/Right对齐错误**
   ```rust
   // Left: x = screen_x + x
   // Right: x = screen_x + (32 - width) + x
   ```

### **如果是Decoder问题**

1. **Padding跳过错误**
   - 应该跳过的padding没跳过
   - 不应该跳过的跳过了

2. **数据边界**
   - raw_data读取越界
   - 输出数据截断

---

## 🔬 **精确诊断建议**

### **Step 1: 验证width数组**

```rust
const WIDTHS: [usize; 31] = [...];
let sum: usize = WIDTHS.iter().sum();
println!("Width sum: {} (expected 512)", sum);
```

### **Step 2: 可视化导出的PNG**

打开 `debug_tiles/*.png`：
- 是否是正确的三角形？
- 数据是否连续？
- 是否有空洞？

### **Step 3: 对比texture_manager版本**

```
F3 OFF: TextureManager (有artifacts)
F3 ON:  DirectRenderer (有artifacts)

如果两者artifacts位置相同 → Decoder问题
如果位置不同 → Renderer问题
```

---

## 💡 **下一步行动**

请告诉我：
1. `debug_tiles`目录下生成了哪些PNG文件？
2. 打开这些PNG，tile形状是否正确？
3. 游戏中黑色artifacts和导出的PNG有关联吗？

这样我能精确定位问题！🔍




