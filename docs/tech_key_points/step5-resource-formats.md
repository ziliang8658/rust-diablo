# Step 5 技术要点：原版资源格式完整支持

> 本文档是 Step 5 的技术详解，包含所有资源格式的实现细节。

## 📦 原版资源格式完整列表

### 1. MPQ归档系统

**格式：** `.MPQ` (Mo'PaQ Archive)
**用途：** 游戏资源打包

**MPQ文件列表：**
- `DIABDAT.MPQ` - 主要游戏数据（完整版）
- `spawn.mpq` - 试玩版数据
- `hellfire.mpq` - Hellfire扩展数据
- `hfbard.mpq` - 吟游诗人职业
- `hfbarb.mpq` - 野蛮人职业
- `devilutionx.mpq` - DevilutionX额外数据
- `fonts.mpq` - 额外字体
- `[lang].mpq` - 语言包

**功能清单：**
1. MPQ归档读取
   - 使用 `mpq` Rust crate
   - 文件列表枚举
   - 文件哈希查找
   - 文件解压缩
2. MPQ优先级系统
   - 多MPQ文件加载
   - 优先级覆盖（Mod系统基础）
3. MPQ缓存系统
   - 已加载文件缓存
   - 内存管理

**文件结构：**
```
MPQ Archive
├── Header (32 bytes)
│   ├── Magic: "MPQ\x1A"
│   ├── Archive size
│   ├── Format version
│   └── Block table offset
├── Hash Table
│   └── 文件名哈希 → 文件索引
├── Block Table
│   └── 文件偏移、大小、压缩信息
└── File Data
    └── 压缩的文件数据
```

**Rust实现：**
```rust
use mpq::Archive;

pub struct MpqManager {
    archives: Vec<(Archive, i32)>, // (归档, 优先级)
}

impl MpqManager {
    pub fn find_file(&self, path: &str) -> Option<Vec<u8>> {
        // 按优先级从高到低查找
        for (archive, _priority) in self.archives.iter().rev() {
            if let Ok(data) = archive.read_file(path) {
                return Some(data);
            }
        }
        None
    }
}
```

---

### 2. PCX图像格式

**格式：** `.PCX` (PC Paintbrush)
**用途：** 256色图像（UI、背景等）

**PCX文件示例：**
- `ui_art/logo.pcx` - Logo
- `ui_art/title.pcx` - 标题画面
- `gendata/cut*.pcx` - 过场画面

**文件结构：**
```
PCX File
├── Header (128 bytes)
│   ├── Manufacturer: 0x0A
│   ├── Version: 5
│   ├── Encoding: 1 (RLE)
│   ├── BitsPerPixel: 8
│   ├── Width, Height
│   └── ...
├── Compressed Image Data (RLE encoded)
│   └── 256色索引值
└── Palette (768 bytes)
    └── 256 × RGB (3 bytes each)
```

**RLE压缩算法：**
```
规则：
- 如果字节 >= 0xC0: (字节 & 0x3F) 表示重复次数，下一个字节是值
- 否则：直接是像素值

示例：
0xC5 0x10 → 重复5次0x10
0x20      → 单个像素
```

**Rust实现：**
```rust
pub struct PcxImage {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>,      // 256色索引
    pub palette: Palette,
}

pub fn load_pcx(data: &[u8]) -> Result<PcxImage> {
    // 1. 读取头部
    let width = read_u16_le(&data[8..10]);
    let height = read_u16_le(&data[10..12]);
    
    // 2. 解压RLE数据
    let mut pixels = Vec::new();
    let mut i = 128;
    
    while pixels.len() < (width as usize * height as usize) {
        let byte = data[i];
        i += 1;
        
        if byte >= 0xC0 {
            let count = (byte & 0x3F) as usize;
            let value = data[i];
            i += 1;
            pixels.extend(std::iter::repeat(value).take(count));
        } else {
            pixels.push(byte);
        }
    }
    
    // 3. 读取调色板
    let palette_start = data.len() - 768;
    let palette = Palette::from_bytes(&data[palette_start..])?;
    
    Ok(PcxImage { width, height, pixels, palette })
}
```

---

### 3. CEL/CL2/CLX精灵格式

**格式演进：**
```
CEL (原始)
  ↓ 优化
CL2 (压缩, 32像素块)
  ↓ DevilutionX优化
CLX (简化头部, 运行时格式)
```

**精灵文件示例：**
- `plrgfx/warrior/wha/whaas.cl2` - 战士攻击动画
- `plrgfx/warrior/whs/whsas.cl2` - 战士站立动画
- `monsters/zombie/zombieA.cl2` - 僵尸动画

**CLX格式结构：**
```
CLX Frame
├── Header (6 bytes)
│   ├── Header Size (2 bytes)
│   ├── Width (2 bytes)
│   └── Height (2 bytes)
└── Pixel Data (RLE编码)
```

**CL2 RLE编码规则：**
```
- 负数: 重复下一个像素 |value| 次
- 正数: 接下来value个字节是原始像素
- 0x80: 透明像素（跳过）

示例：
-5, 0x10         → 5个0x10像素
3, 0x20, 0x21, 0x22 → 3个不同像素
0x80             → 透明
```

**Rust实现：**
```rust
pub struct ClxFrame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<Option<u8>>,  // None表示透明
}

pub fn load_clx(data: &[u8]) -> Result<Vec<ClxFrame>> {
    let frame_count = read_u32_le(&data[0..4]) as usize;
    let mut frames = Vec::new();
    
    for i in 0..frame_count {
        let offset = read_u32_le(&data[4 + i*4..8 + i*4]) as usize;
        
        let header_size = read_u16_le(&data[offset..offset+2]);
        let width = read_u16_le(&data[offset+2..offset+4]);
        let height = read_u16_le(&data[offset+4..offset+6]);
        
        let pixel_data_start = offset + header_size as usize;
        let pixels = decode_clx_pixels(&data[pixel_data_start..], width, height)?;
        
        frames.push(ClxFrame { width, height, pixels });
    }
    
    Ok(frames)
}

fn decode_clx_pixels(data: &[u8], width: u16, height: u16) -> Result<Vec<Option<u8>>> {
    let mut pixels = vec![None; (width * height) as usize];
    let mut data_idx = 0;
    let mut pixel_idx = 0;
    
    while pixel_idx < pixels.len() {
        let control = data[data_idx] as i8;
        data_idx += 1;
        
        if control < 0 {
            // 重复像素
            let count = (-control) as usize;
            let value = data[data_idx];
            data_idx += 1;
            for _ in 0..count {
                pixels[pixel_idx] = Some(value);
                pixel_idx += 1;
            }
        } else if control > 0 {
            // 原始像素
            let count = control as usize;
            for _ in 0..count {
                pixels[pixel_idx] = Some(data[data_idx]);
                data_idx += 1;
                pixel_idx += 1;
            }
        } else {
            // 透明像素
            pixels[pixel_idx] = None;
            pixel_idx += 1;
        }
    }
    
    Ok(pixels)
}
```

---

### 4. 调色板系统

**格式：** `.PAL` (Palette)
**用途：** 256色调色板

**调色板文件：**
- `levels/towndata/town.pal` - 城镇调色板
- `levels/l1data/l1_1.pal` - 教堂调色板
- `levels/l2data/l2.pal` - 地下墓穴调色板

**调色板布局：**
```
Diablo调色板分区：
0-127   (0x00-0x7F): 关卡特定颜色
128-255 (0x80-0xFF): 全局颜色

全局颜色分配：
128-135: 蓝色  (PAL8_BLUE)
136-143: 红色  (PAL8_RED)
144-151: 黄色  (PAL8_YELLOW)
152-159: 橙色  (PAL8_ORANGE)
...
```

**Rust实现：**
```rust
#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Palette {
    pub colors: [Color; 256],
}

impl Palette {
    pub fn from_file(data: &[u8]) -> Result<Self> {
        if data.len() != 768 {
            return Err(anyhow!("Invalid palette size"));
        }
        
        let mut colors = [Color { r: 0, g: 0, b: 0 }; 256];
        for i in 0..256 {
            colors[i] = Color {
                r: data[i*3],
                g: data[i*3 + 1],
                b: data[i*3 + 2],
            };
        }
        
        Ok(Palette { colors })
    }
    
    pub fn to_rgba(&self, index: u8, transparent: bool) -> [u8; 4] {
        let c = &self.colors[index as usize];
        let alpha = if transparent && index == 0 { 0 } else { 255 };
        [c.r, c.g, c.b, alpha]
    }
}
```

---

### 5. TRN颜色转换表

**格式：** `.TRN` (TRaNslation)
**用途：** 颜色映射转换（实现怪物/玩家变色）

**文件结构：**
```
TRN File
└── 256 bytes
    ├── [0]: 输入颜色0 → 输出颜色?
    ├── [1]: 输入颜色1 → 输出颜色?
    └── ...
```

**使用场景：**
1. 怪物变色：`僵尸基础精灵 + blue.trn → 蓝色僵尸`
2. 唯一怪物：`普通怪物 + unique.trn → 金色唯一怪物`
3. 玩家装备：`基础盔甲 + armor.trn → 不同颜色的盔甲`

**Rust实现：**
```rust
pub struct ColorTransform {
    pub map: [u8; 256],
}

impl ColorTransform {
    pub fn from_file(data: &[u8]) -> Result<Self> {
        if data.len() != 256 {
            return Err(anyhow!("Invalid TRN size"));
        }
        
        let mut map = [0u8; 256];
        map.copy_from_slice(data);
        Ok(ColorTransform { map })
    }
    
    pub fn apply(&self, color_index: u8) -> u8 {
        self.map[color_index as usize]
    }
    
    pub fn apply_to_pixels(&self, pixels: &mut [u8]) {
        for pixel in pixels.iter_mut() {
            *pixel = self.map[*pixel as usize];
        }
    }
}
```

---

### 6. 其他资源格式

**音频格式：**
- `.WAV` - 音效和音乐
- `.MP3` - 压缩音乐（可选）

**视频格式：**
- `.SMK` - Smacker视频（过场动画）

**数据格式：**
- `.DUN` - 地牢布局数据
- `.MIN` - 最小瓦片数据
- `.TIL` - 瓦片集数据
- `.SOL` - 碰撞数据（solid）

---

## 🔧 实现架构

### Rust模块结构

```
src/
├── resources/
│   ├── mod.rs              # 资源管理器主模块
│   ├── mpq.rs              # MPQ归档读取
│   ├── pcx.rs              # PCX图像加载
│   ├── cel.rs              # CEL精灵加载
│   ├── cl2.rs              # CL2精灵加载
│   ├── clx.rs              # CLX精灵加载
│   ├── palette.rs          # 调色板系统
│   ├── trn.rs              # 颜色转换
│   └── cache.rs            # 资源缓存
├── sprite/
│   ├── sprite_sheet.rs     # 精灵表（多帧动画）
│   └── texture.rs          # [已有] 扩展支持调色板
└── assets/
    └── mod.rs              # [已有] 扩展资源路径
```

### 资源管理器

```rust
pub struct ResourceManager {
    mpq_manager: mpq::MpqManager,
    palette_manager: palette::PaletteManager,
    texture_cache: HashMap<String, Texture>,
    sprite_cache: HashMap<String, Vec<ClxFrame>>,
    trn_cache: HashMap<String, ColorTransform>,
}

impl ResourceManager {
    pub fn new() -> Result<Self> {
        let mut mgr = Self { ... };
        
        // 加载核心MPQ
        mgr.mpq_manager.load_mpq("devilutionx.mpq", 9000)?;
        mgr.mpq_manager.load_mpq("DIABDAT.MPQ", 1000)?;
        mgr.mpq_manager.load_mpq("hellfire.mpq", 1000)?;
        
        Ok(mgr)
    }
    
    pub fn load_sprite(&mut self, path: &str) -> Result<&Vec<ClxFrame>> {
        if let Some(sprite) = self.sprite_cache.get(path) {
            return Ok(sprite);
        }
        
        // 从MPQ加载
        let data = self.mpq_manager.find_file(path)?;
        
        // 根据扩展名选择加载器
        let sprite = if path.ends_with(".cel") {
            load_cel(&data)?
        } else if path.ends_with(".cl2") {
            load_cl2(&data)?
        } else if path.ends_with(".clx") {
            load_clx(&data)?
        } else {
            return Err(anyhow!("Unknown format"));
        };
        
        self.sprite_cache.insert(path.to_string(), sprite);
        Ok(self.sprite_cache.get(path).unwrap())
    }
}
```

---

## 🧪 测试要求

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_mpq_loading() {
        let mut mgr = MpqManager::new();
        mgr.load_mpq("test_assets/test.mpq", 1000).unwrap();
        assert!(mgr.has_file("test/file.txt"));
    }
    
    #[test]
    fn test_pcx_loading() {
        let data = include_bytes!("../../test_assets/test.pcx");
        let pcx = load_pcx(data).unwrap();
        assert_eq!(pcx.width, 640);
        assert_eq!(pcx.palette.colors.len(), 256);
    }
    
    #[test]
    fn test_clx_frame_parsing() {
        let data = include_bytes!("../../test_assets/test.clx");
        let clx = load_clx(data).unwrap();
        assert_eq!(clx[0].width, 96);
    }
}
```

### 集成测试

```rust
// tests/resource_loading_test.rs

#[test]
fn test_load_player_sprite() {
    let mut res_mgr = ResourceManager::new().unwrap();
    let sprite = res_mgr.load_sprite("plrgfx/warrior/whs/whsas.cl2");
    assert!(sprite.is_ok());
}

#[test]
fn test_load_with_palette() {
    let mut res_mgr = ResourceManager::new().unwrap();
    let palette = res_mgr.load_palette("levels/towndata/town.pal").unwrap();
    let texture = res_mgr.load_pcx_with_palette("ui_art/logo.pcx", &palette).unwrap();
    assert!(texture.width() > 0);
}
```

### 性能测试

```rust
// benches/resource_benchmark.rs

fn benchmark_mpq_file_lookup(c: &mut Criterion) {
    let mut mgr = MpqManager::new();
    mgr.load_mpq("DIABDAT.MPQ", 1000).unwrap();
    
    c.bench_function("mpq_file_lookup", |b| {
        b.iter(|| black_box(mgr.find_file("plrgfx/warrior/whs/whsas.cl2")))
    });
}
```

---

## 🛠️ 工具和脚本

### 资源提取工具

```rust
// tools/extract_mpq.rs

fn main() -> Result<()> {
    let mpq_path = std::env::args().nth(1).expect("Usage: extract_mpq <mpq_file>");
    let output_dir = std::env::args().nth(2).unwrap_or("extracted".to_string());
    
    let mut mgr = MpqManager::new();
    mgr.load_mpq(&mpq_path, 1000)?;
    
    for file in mgr.list_files()? {
        let data = mgr.find_file(&file)?;
        let output_path = format!("{}/{}", output_dir, file);
        
        if let Some(parent) = Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&output_path, data)?;
        println!("Extracted: {}", file);
    }
    
    Ok(())
}
```

### 精灵查看器

```rust
// examples/sprite_viewer.rs

fn main() -> Result<()> {
    let sprite_path = std::env::args().nth(1).expect("Usage: sprite_viewer <sprite_file>");
    
    let mut res_mgr = ResourceManager::new()?;
    let sprite = res_mgr.load_sprite(&sprite_path)?;
    
    // 创建窗口
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video.window("Sprite Viewer", 800, 600).build()?;
    let mut canvas = window.into_canvas().build()?;
    
    let mut current_frame = 0;
    // ... 渲染循环 ...
    
    Ok(())
}
```

---

## 🔗 依赖更新

```toml
[dependencies]
# 已有
sdl2 = "0.36"
anyhow = "1.0"

# 新增
mpq = "0.8"  # MPQ归档支持

[dev-dependencies]
criterion = "0.5"  # 性能测试
```

---

## 📚 参考资源

### 文件格式文档
- [Diablo 1 File Formats](https://github.com/savagesteel/d1-file-formats)
- [MPQ Format Specification](http://www.zezula.net/en/mpq/mpqformat.html)
- [PCX Format Specification](http://www.fileformat.info/format/pcx/)

### 原版代码参考
- `Source/mpq/mpq_reader.cpp` - MPQ实现
- `Source/engine/load_pcx.cpp` - PCX加载
- `Source/engine/clx_sprite.hpp` - CLX格式
- `Source/engine/palette.cpp` - 调色板系统

---

**文档版本：** 1.0  
**最后更新：** 2025-11-17  
**关联文档：** [MASTER_PLAN.md](../MASTER_PLAN.md), [STEP5_RESOURCE_FORMAT_PLAN.md](../STEP5_RESOURCE_FORMAT_PLAN.md)







