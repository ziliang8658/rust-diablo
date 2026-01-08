# Direct Pixel Renderer 实施计划

**目标**: 完全复制C++的逐像素渲染方式

## 📋 **架构设计**

### **核心组件**

```rust
// 1. 8-bit Framebuffer
struct Framebuffer {
    pixels: Vec<u8>,      // 8-bit palette indices
    width: usize,
    height: usize,
    palette: Vec<Color>,  // 256-color palette
}

// 2. Direct Renderer
struct DirectRenderer {
    framebuffer: Framebuffer,
}

// 3. Tile Data (保持compact格式)
struct CompactTile {
    data: Vec<u8>,        // 512 bytes for triangle
    tile_type: TileType,
}
```

### **渲染流程**

```
1. 初始化: 创建8-bit framebuffer（一次）
2. 每帧:
   a. Phase 1: 渲染地板tile到framebuffer
   b. Phase 2: 渲染墙壁tile到framebuffer
   c. 转换framebuffer为RGBA texture
   d. 渲染texture到screen
```

---

## 🔧 **实施步骤**

### **Step 1: 创建Framebuffer模块**

**文件**: `rust-diablo/src/renderer/framebuffer.rs`

```rust
pub struct Framebuffer {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![0; width * height],
            width,
            height,
        }
    }
    
    /// 写入像素（如果palette_index != 0）
    pub fn set_pixel(&mut self, x: usize, y: usize, palette_index: u8) {
        if palette_index != 0 {  // 透明检查
            let offset = y * self.width + x;
            if offset < self.pixels.len() {
                self.pixels[offset] = palette_index;
            }
        }
        // palette_index == 0: 透明，不写入
    }
    
    /// 转换为RGBA数据
    pub fn to_rgba(&self, palette: &[Color; 256]) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.pixels.len() * 4);
        for &idx in &self.pixels {
            let color = palette[idx as usize];
            rgba.extend_from_slice(&[color.r, color.g, color.b, 255]);
        }
        rgba
    }
}
```

### **Step 2: 创建DirectRenderer**

**文件**: `rust-diablo/src/renderer/direct.rs`

```rust
pub struct DirectRenderer {
    framebuffer: Framebuffer,
    palette: [Color; 256],
}

impl DirectRenderer {
    pub fn new(width: usize, height: usize, palette: [Color; 256]) -> Self {
        Self {
            framebuffer: Framebuffer::new(width, height),
            palette,
        }
    }
    
    /// 渲染compact triangle tile
    pub fn render_left_triangle(
        &mut self,
        data: &[u8],  // 512 bytes compact
        screen_x: i32,
        screen_y: i32,
    ) {
        // 解码并渲染（逐像素）
        let widths = [2,4,6,8,10,12,14,16,18,20,22,24,26,28,30,32,
                      30,28,26,24,22,20,18,16,14,12,10,8,6,4,2];
        
        let mut src_pos = 0;
        for (row, &width) in widths.iter().enumerate() {
            let y = screen_y + row as i32;
            if y < 0 || y >= self.framebuffer.height as i32 {
                src_pos += width;
                continue;
            }
            
            for x in 0..width {
                let px = screen_x + x as i32;
                if px >= 0 && px < self.framebuffer.width as i32 {
                    self.framebuffer.set_pixel(
                        px as usize,
                        y as usize,
                        data[src_pos + x],
                    );
                }
            }
            src_pos += width;
        }
    }
    
    /// 渲染square tile
    pub fn render_square(
        &mut self,
        data: &[u8],  // 1024 bytes (32×32)
        screen_x: i32,
        screen_y: i32,
    ) {
        for row in 0..32 {
            let y = screen_y + row;
            if y < 0 || y >= self.framebuffer.height as i32 {
                continue;
            }
            
            for col in 0..32 {
                let x = screen_x + col;
                if x >= 0 && x < self.framebuffer.width as i32 {
                    let idx = (row * 32 + col) as usize;
                    self.framebuffer.set_pixel(
                        x as usize,
                        y as usize,
                        data[idx],
                    );
                }
            }
        }
    }
    
    /// 转换为SDL2 texture
    pub fn to_texture<'a>(
        &self,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Texture<'a>> {
        let rgba = self.framebuffer.to_rgba(&self.palette);
        
        let mut texture = texture_creator
            .create_texture_static(
                PixelFormatEnum::RGBA32,
                self.framebuffer.width as u32,
                self.framebuffer.height as u32,
            )?;
        
        texture.update(None, &rgba, self.framebuffer.width * 4)?;
        Ok(texture)
    }
}
```

### **Step 3: 修改Tile Decoder保持Compact格式**

**当前**: 解码后padding到32 bytes/row (992 bytes)
**改为**: 保持compact格式 (512 bytes)

```rust
// decoder/triangle.rs
pub fn decode_left_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    // 返回512 bytes compact数据，不padding
    let mut output = Vec::new();
    
    // 直接复制compact data
    for row in 0..31 {
        let width = get_row_width(row);
        output.extend_from_slice(&raw_data[src..src+width]);
        src += width;
    }
    
    output  // 512 bytes
}
```

### **Step 4: 集成到World渲染**

```rust
// world/mod.rs
impl World {
    pub fn render(&self, engine: &mut Engine, camera: &Camera) {
        let mut direct_renderer = DirectRenderer::new(
            engine.screen_width(),
            engine.screen_height(),
            self.palette.clone(),
        );
        
        // Phase 1: Floor tiles
        for tile in floor_tiles {
            direct_renderer.render_tile(tile.data, tile.screen_x, tile.screen_y);
        }
        
        // Phase 2: Wall tiles
        for tile in wall_tiles {
            direct_renderer.render_tile(tile.data, tile.screen_x, tile.screen_y);
        }
        
        // 转换为texture并渲染
        let texture = direct_renderer.to_texture(engine.texture_creator())?;
        engine.canvas_mut().copy(&texture, None, None)?;
    }
}
```

---

## ⚡ **性能优化**

### **优化1: 重用Framebuffer**

```rust
// 不是每帧创建，而是重用
struct DirectRenderer {
    framebuffer: Framebuffer,
    texture: Option<Texture>,  // 重用texture
}

impl DirectRenderer {
    pub fn clear(&mut self) {
        // 只在第一帧clear
        static mut FIRST_FRAME: bool = true;
        unsafe {
            if FIRST_FRAME {
                self.framebuffer.pixels.fill(0);
                FIRST_FRAME = false;
            }
        }
    }
}
```

### **优化2: SIMD加速**

```rust
// 使用SIMD批量转换palette
#[cfg(target_arch = "x86_64")]
unsafe fn palette_to_rgba_simd(pixels: &[u8], palette: &[Color; 256]) -> Vec<u8> {
    // AVX2加速
    ...
}
```

### **优化3: 多线程渲染**

```rust
// 分块渲染
use rayon::prelude::*;

tiles.par_chunks(64).for_each(|chunk| {
    for tile in chunk {
        render_tile_to_local_buffer(tile);
    }
});
```

---

## 📊 **预期效果**

| 指标 | 目标 |
|------|------|
| 内存 | 512B/tile (vs 992B当前) |
| 性能 | 60 FPS @ 800×600 |
| C++相似度 | 100% |
| 透明正确性 | 完美 |

---

## ✅ **验收标准**

1. [ ] 河流和路面过渡自然
2. [ ] 透明区域正确显示下层
3. [ ] 无黑色/蓝色artifacts
4. [ ] 性能不低于当前版本
5. [ ] 代码可读性良好

---

## 🚀 **实施时间表**

- **Day 1-2**: Framebuffer + DirectRenderer核心
- **Day 3-4**: Tile Decoder改造（保持compact）
- **Day 5-6**: 集成到World渲染
- **Day 7**: 性能优化和测试

---

## 📝 **注意事项**

1. **坐标系统**: C++是bottom-to-top，SDL2是top-to-bottom
2. **Palette管理**: 确保palette正确加载
3. **边界检查**: 防止越界访问
4. **内存安全**: 使用safe Rust，避免unsafe除非必要




