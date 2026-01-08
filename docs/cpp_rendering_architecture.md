# C++ Rendering Architecture Analysis

**目标**: 完全复制C++的渲染方式到Rust实现

## 📐 **顶层架构**

### **渲染调用链**

```
scrollrt_draw_game_screen()
  └─> DrawView(Surface, Point)
      └─> DrawGame(Surface, Point, Displacement)
          ├─> DrawFloor(Surface, Lightmap, Point, Point, rows, columns)
          │   └─> DrawFloorTile(Surface, Lightmap, Point, Point)
          │       └─> DrawCell(Surface, Lightmap, Point, Point, lightIndex) [blocks 0-1 only]
          │
          └─> DrawTileContent(Surface, Lightmap, Point, Point, rows, columns)
              └─> DrawDungeon(Surface, Lightmap, Point, Point)
                  └─> DrawCell(Surface, Lightmap, Point, Point, lightIndex) [all blocks]
```

### **关键发现**

1. **DrawGame()**: Line 1254-1322
   - ⚠️ **没有clear()调用**
   - 直接调用DrawFloor()和DrawTileContent()
   
2. **DrawFloor()**: Line 1049-1077
   - 只渲染 `IsFloor(tilePosition) == true` 的tile
   - 调用DrawFloorTile()，内部调用DrawCell()只处理blocks 0-1
   
3. **DrawTileContent()**: Line 1088-1141
   - 遍历所有可见tile
   - 对**每个tile**调用DrawDungeon()
   - rows += MicroTileLen (扩展渲染范围以包含高墙)

4. **DrawDungeon()**: Line 875-1073
   - 调用DrawCell()渲染tile本身
   - 然后渲染尸体、物品、物体、玩家等

---

## 🎨 **DrawCell 详细分析**

### **函数签名**

```cpp
void DrawCell(const Surface &out, const Lightmap &lightmap, 
              Point tilePosition, Point targetBufferPosition, 
              uint8_t lightTableIndex)
```

**Reference**: Source/engine/render/scrollrt.cpp Line 521-643

### **渲染逻辑**

```cpp
const LevelCelBlock *pMap = dPiece[tilePosition.x][tilePosition.y].map();

// Render blocks 0-1 (floor level)
if (!isFloor || tileType == TileType::TransparentSquare) {
    if (isFloor && tileType == TransparentSquare) {
        RenderTileFoliage(...);  // 草地等
    } else {
        RenderTile(...);  // 墙壁
    }
}

// Render blocks 2-15 (wall layers)
for (i = 2; i < MicroTileLen; i += 2) {
    RenderTile(...);  // 总是渲染，不检查isFloor
}
```

**关键点**:
- Blocks 0-1: 只在 `!isFloor || TransparentSquare` 时渲染
- Blocks 2-15: **总是渲染**，不管isFloor

---

## 🔢 **Tile数据格式: Compact**

### **C++使用Compact格式**

```cpp
// LeftTriangle: 512 bytes compact
// - Row 0 (width=2): 2 bytes
// - Row 1 (width=4): 4 bytes
// - Row 2 (width=6): 6 bytes
// ...
// - Row 15 (width=32): 32 bytes
// - Row 16 (width=30): 30 bytes
// ...
// - Row 30 (width=2): 2 bytes
// Total: 2+4+6+...+32+30+...+2 = 512 bytes
```

**对比Rust Padded格式**:
```rust
// Padded to 32 bytes per row: 992 bytes
// - Row 0: [##] + 30 padding bytes
// - Row 1: [####] + 28 padding bytes
// ...
```

**性能影响**:
- Compact: 512 bytes (内存节省)
- Padded: 992 bytes (代码简单)

---

## 🖼️ **像素渲染方式**

### **C++: 直接写framebuffer**

```cpp
// 8-bit indexed color
uint8_t *dst = framebuffer + y * pitch + x;

if (pixel_is_opaque) {
    *dst = palette_index;  // 写入
} else {
    // 透明: 什么都不做，保留dst原值
}
```

**关键特性**:
- 透明像素: 不写入，framebuffer保持不变
- 这就是为什么C++不需要clear()

### **Rust: SDL2纹理渲染**

```rust
// 32-bit RGBA
rgba_pixels[...] = [r, g, b, alpha];
texture.update(rgba_pixels);
canvas.copy(&texture, ...);
```

**问题**:
- SDL2的copy()可能会覆盖下层，即使alpha=0
- BlendMode可能不够精确

---

## 📊 **坐标系统**

### **C++渲染方向**

```cpp
// 从下往上渲染 (bottom-to-top)
for (int i = 0; i < height; i++, dst -= dstPitch) {
    // 渲染当前行
    // dst -= dstPitch: 向上移动一行
}
```

**原因**: CLX格式是bottom-to-top存储

### **Rust/SDL2**

```
// 从上往下渲染 (top-to-bottom)
// Y轴向下
// 需要flip_v翻转
```

---

## 🎯 **完整复制C++的实现方案**

### **方案A: 使用SDL2 Surface直接写像素**

**优点**:
- 完全复制C++行为
- 透明像素不写入（保留下层）
- 不需要clear()

**缺点**:
- SDL2 Surface性能较差
- 需要手动管理像素访问

**实现**:
```rust
// 创建8-bit Surface (模拟C++的8-bit framebuffer)
let surface = Surface::new(width, height, PixelFormat::Index8);

// 逐像素渲染
for (x, y, palette_index) in pixels {
    if palette_index != 0 {  // 非透明
        surface.set_pixel(x, y, palette_index);
    }
    // 透明: 不写入
}

// 转换为texture渲染
let texture = surface.as_texture();
canvas.copy(&texture, ...);
```

### **方案B: 使用Render Target (推荐)**

**优点**:
- 保持GPU加速
- 完全控制渲染顺序
- 可以实现"不clear()"效果

**实现**:
```rust
// 创建render target
let target_texture = texture_creator.create_texture_target(
    PixelFormat::RGBA32, width, height
);

canvas.with_texture_canvas(&mut target_texture, |canvas| {
    // 第一帧: clear一次
    if first_frame {
        canvas.clear();
    }
    
    // Phase 1: 渲染地板
    for floor_tile in floor_tiles {
        render_tile(&floor_tile);
    }
    
    // Phase 2: 渲染墙壁
    for wall_tile in wall_tiles {
        render_tile(&wall_tile);
    }
});

// 渲染target到screen
canvas.copy(&target_texture, ...);
```

### **方案C: 保持Compact格式 + 直接渲染**

**完全复制C++**:
1. 保持Compact tile格式 (512 bytes)
2. 创建自定义渲染器
3. 逐像素写入，透明不写

---

## 🔬 **RenderTileFrame详细分析**

### **函数签名**

```cpp
void RenderTileFrame(const Surface &out, const Lightmap &lightmap, 
                     const Point &position, TileType tile, 
                     const uint8_t *src, int_fast16_t height,
                     MaskType maskType, const uint8_t *tbl)
```

**Reference**: Source/engine/render/dun_render.cpp Line 1048-1086

### **渲染分派**

```cpp
switch (maskType) {
    case MaskType::Solid:
        RenderTileDispatch<false>(tile, dst, dstPitch, src, tbl, lightmap, clip);
        break;
    case MaskType::Transparent:
        RenderTileDispatch<true>(tile, dst, dstPitch, src, tbl, lightmap, clip);
        break;
    case MaskType::Left:
        RenderLeftTrapezoidOrTransparentSquareDispatch<MaskType::Left>(...);
        break;
    case MaskType::Right:
        RenderRightTrapezoidOrTransparentSquareDispatch<MaskType::Right>(...);
        break;
}
```

### **透明处理机制**

```cpp
template <bool Transparent>
void RenderLine(uint8_t *dst, const uint8_t *src, unsigned width, ...) {
    for (unsigned i = 0; i < width; ++i) {
        if (Transparent) {
            // 使用blending table
            *dst = tbl[*src];
        } else {
            // 直接写入
            *dst = palette[*src];
        }
        dst++;
        src++;
    }
}
```

**关键**:
- `Transparent=false`: 完全不透明，直接写入像素
- `Transparent=true`: 使用blending table (半透明效果)
- **RLE编码透明**: 根本不调用RenderLine，直接跳过

### **MaskType::Left/Right 的prefix机制**

```cpp
// Left mask (从下往上，每行增加2个透明像素)
constexpr int8_t InitialPrefix<MaskType::Left> = -32;
constexpr int8_t PrefixIncrement<MaskType::Left> = 2;

// 对于每一行(从bottom到top):
int transparent_pixels = max(0, prefix);
prefix += 2;

// 只渲染非透明部分
RenderLine(dst + transparent_pixels, src, width - transparent_pixels, ...);
```

---

## 🎯 **完整实施方案**

### **推荐方案: Render Target + 逐像素渲染**

#### **Phase 1: 保持当前架构，优化渲染顺序**

**当前问题**:
1. ✅ Phase 1和Phase 2顺序正确
2. ✅ 不使用clear()
3. ❌ SDL2纹理可能覆盖下层（即使alpha=0）

**优化措施**:
```rust
// 1. 确保blend mode正确
canvas.set_blend_mode(BlendMode::Blend);
texture.set_blend_mode(BlendMode::Blend);

// 2. 对于alpha=0的像素，不渲染该tile区域
// 或者使用set_color_mod/set_alpha_mod

// 3. 验证TransparentSquare的处理
```

#### **Phase 2: 实现自定义像素渲染器（如需要）**

```rust
struct DirectRenderer {
    framebuffer: Vec<u8>,  // 8-bit indexed color
    palette: Vec<Color>,
    width: usize,
    height: usize,
}

impl DirectRenderer {
    fn render_pixel(&mut self, x: usize, y: usize, palette_index: u8) {
        if palette_index != 0 {  // 非透明
            let offset = y * self.width + x;
            self.framebuffer[offset] = palette_index;
        }
        // palette_index == 0: 透明，不写入
    }
    
    fn to_texture(&self, texture_creator: &TextureCreator) -> Texture {
        // 转换为RGBA
        let rgba = self.framebuffer.iter()
            .map(|&idx| self.palette[idx as usize])
            .collect::<Vec<_>>();
        // 创建texture
        ...
    }
}
```

#### **Phase 3: Compact格式支持（性能优化）**

```rust
// 保持512字节compact格式
// 渲染时动态解包到正确位置
fn render_left_triangle_compact(data: &[u8; 512], dst: &mut [u8]) {
    let widths = [2,4,6,8,...,32,30,...,2];
    let mut src_pos = 0;
    
    for (row, &width) in widths.iter().enumerate() {
        let dst_row = row * 32;
        // 左对齐
        dst[dst_row..dst_row+width].copy_from_slice(&data[src_pos..src_pos+width]);
        src_pos += width;
    }
}
```

---

## 📊 **性能对比**

| 方案 | 内存 | 性能 | 复杂度 | C++相似度 |
|------|------|------|--------|-----------|
| 当前(Padded+SDL2) | 992B/tile | 快 | 低 | 70% |
| Render Target | 992B/tile | 快 | 中 | 85% |
| Direct Pixel | 512B/tile | 中 | 高 | 95% |
| Compact+Direct | 512B/tile | 最快 | 最高 | 100% |

---

## 🚀 **实施步骤**

### **Step 1: 诊断当前问题** ✅
- [x] 确认不使用clear()
- [x] 验证Phase 1/2顺序
- [ ] 检查河流piece的渲染细节

### **Step 2: 修复透明渲染**
- [ ] 验证BlendMode设置
- [ ] 检查TransparentSquare的alpha值
- [ ] 测试河流+路面过渡

### **Step 3: 如需要，实现Direct Renderer**
- [ ] 创建8-bit framebuffer
- [ ] 实现逐像素渲染
- [ ] 转换为SDL2 texture

### **Step 4: 性能优化**
- [ ] 支持Compact格式
- [ ] 优化渲染循环
- [ ] 缓存常用数据


