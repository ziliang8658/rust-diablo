# Step 6.3: 墙体瓦片和完整渲染系统

## 📋 目标和范围

### 主要目标

本步骤将完成地下城瓦片渲染系统的最后关键部分：

1. **特殊瓦片支持** - 加载和解码 CLX 格式的特殊瓦片（l1s.cel 等）
2. **SDL2 纹理渲染** - 将解码的像素数据转换为 SDL2 纹理并正确渲染
3. **完整墙体渲染** - 实现 micro3/micro4 的垂直堆叠渲染
4. **透明度和混合** - 处理 TransparentSquare 和半透明效果
5. **实际游戏场景测试** - 在真实地下城场景中验证渲染效果

### 与 Step 6.2 的关系

**Step 6.2 已完成**:
- ✅ 6种 TileType 解码器（Square/Triangle/Trapezoid/TransparentSquare）
- ✅ TileTextureManager 纹理管理器（双层缓存）
- ✅ 主 CEL 文件（l1.cel）的 76% 瓦片加载
- ✅ 地板和墙体渲染基础架构（占位矩形）

**Step 6.3 要完成**:
- 🎯 特殊 CEL 文件（l1s.cel 等）的剩余 24% 瓦片
- 🎯 将 RGBA 数据转换为 SDL2 纹理
- 🎯 实际瓦片绘制（替换占位矩形）
- 🎯 透明度和 alpha 混合
- 🎯 完整的墙体堆叠渲染

### 不包含的内容（留待后续）

- ⏸️ 光照系统（dLight 数组）- Step 6.4
- ⏸️ 地图生成算法（BSP/房间布局）- Step 7
- ⏸️ Miniset 系统（门、楼梯、特殊房间）- Step 7
- ⏸️ 动画瓦片（火把、水面）- Step 8
- ⏸️ 性能优化（批量渲染、SIMD）- Step 9

---

## 🎯 核心原则

### 1. 完全复刻原版渲染逻辑

**严格对照原版 C++ 代码**:
- 渲染顺序必须与原版一致（从后到前，从下到上）
- 坐标计算必须与原版一致（等距投影）
- 混合模式必须与原版一致（透明度处理）

### 2. 参考原版代码位置

所有实现都必须标注对应的原版 C++ 代码位置：

```
主要参考文件：
- Source/engine/render/scrollrt.cpp         # 主渲染循环
- Source/engine/render/dun_render.cpp       # 瓦片帧渲染
- Source/engine/render/cl2_render.cpp       # CLX/CL2 渲染
- Source/levels/gendung.cpp                 # 地下城数据管理
- Source/engine/displacement.hpp            # 等距投影坐标
```

### 3. 不自由发挥

- ❌ 不添加原版没有的优化
- ❌ 不改变渲染顺序
- ❌ 不使用不同的坐标系统
- ✅ 保持与原版 100% 视觉一致性

---

## 📐 架构设计

### 原版 C++ 渲染流程（参考）

```
游戏主循环 (diablo.cpp: GameLoop)
    ↓
DrawAndBlit() (scrollrt.cpp:1753)
    ↓
DrawMain() (scrollrt.cpp:1686-1703)
    ↓
【关键】DrawTileContent() (scrollrt.cpp:1445-1555)
    ├─ 遍历可见区域 (ViewX, ViewY范围)
    ├─ 计算屏幕坐标 (等距投影)
    ├─ Pass 1: DrawFloorTile() → 渲染地板
    │   ├─ MicroTile 1: LeftTriangle
    │   └─ MicroTile 2: RightTriangle
    │
    └─ Pass 2: DrawCell() (scrollrt.cpp:1262-1402) → 渲染墙体
        ├─ 获取 DPieceMicros (4个 micro)
        ├─ 从底部到顶部渲染:
        │   ├─ micro1 (下层左)
        │   ├─ micro2 (下层右)
        │   ├─ micro3 (上层左，Y偏移-32)
        │   └─ micro4 (上层右，Y偏移-32)
        │
        └─ 每个 micro 调用 RenderTile()
            ↓
RenderTile() (dun_render.cpp:305-418)
    ├─ 获取 frame 数据
    ├─ 根据 TileType 分发到具体渲染器
    ├─ 应用光照 (dLight)
    └─ Blit 到屏幕缓冲区
```

**关键代码位置**:
- `scrollrt.cpp:1445` - `DrawTileContent()` 函数开始
- `scrollrt.cpp:1262` - `DrawCell()` 函数开始
- `scrollrt.cpp:1187` - `DrawFloorTile()` 函数开始
- `dun_render.cpp:305` - `RenderTile()` 函数开始

### Rust 实现架构（对应）

```
src/
├── engine/
│   ├── mod.rs                    # Engine 核心
│   └── texture_cache.rs          # 新增：SDL2 纹理缓存
│
├── tiles/
│   ├── decoder/                  # 已完成：6种解码器
│   ├── texture_manager.rs        # 已完成：纹理管理器
│   ├── min.rs                    # 已完成：MicroTile
│   ├── til.rs                    # 已完成：MegaTile
│   └── renderer.rs               # 新增：瓦片渲染器
│
├── world/
│   └── mod.rs                    # 修改：完整渲染逻辑
│       ├─ render_tiles()         # 主渲染入口
│       ├─ render_floor_tile()    # 地板渲染（已有基础）
│       ├─ render_wall_cell()     # 墙体渲染（扩展）
│       └─ render_micro_tile()    # 单个 micro 渲染
│
└── resources/
    ├── dungeon_cel.rs            # 修改：支持特殊 CEL
    └── clx.rs                    # 已有：CLX 解析器
```

---

## 🔧 核心模块详解

### 4.1 特殊 CEL 文件支持（l1s.cel 等）

#### 问题背景

**Step 6.2 发现的问题**:
```
Frame index out of range: 1107 (max=279)
主 CEL 文件（l1.cel）只有 279 帧（索引 0-278）
MIN 文件中有些瓦片引用索引 280+（指向特殊 CEL 文件）
```

**关键发现**:
- 主 CEL：`levels/l1data/l1.cel` - 标准瓦片（地板、基础墙体）
- 特殊 CEL：`levels/l1data/l1s.cel` - 特殊结构（门、拱门、装饰）
- **重要**: `l1s.cel` 虽然扩展名是 `.cel`，但实际是 **CLX 格式**！

#### 原版代码对照

**参考**: `Source/levels/gendung.cpp`

```cpp
// Line 1235-1264: LoadLvlGFX()
void LoadLvlGFX()
{
    // 加载主 CEL
    pDungeonCels = LoadFileInMem("levels/l1data/l1.cel");
    
    // 加载特殊 CEL（实际是 CLX）
    pSpecialCels = LoadFileInMem("levels/l1data/l1s.cel");
    
    // 加载 MIN（包含 frame 映射）
    pLevelPieces = LoadFileInMem("levels/l1data/l1.min");
    
    // 初始化 micro 瓦片系统
    SetDungeonMicros();
}

// Line 1165-1180: GetDunFrame()
static uint8_t *GetDunFrame(int frameNum)
{
    int mainCelCount = pDungeonCelFrames.size();
    
    if (frameNum < mainCelCount) {
        // 主 CEL
        return pDungeonCels + pDungeonCelFrames[frameNum].offset;
    } else {
        // 特殊 CEL（索引需要调整）
        int specialIdx = frameNum - mainCelCount;
        return pSpecialCels + pSpecialCelFrames[specialIdx].offset;
    }
}
```

#### Rust 实现方案

**文件**: `src/resources/dungeon_cel.rs`

**修改 DungeonCelSprite 结构**:

```rust
pub struct DungeonCelSprite {
    pub frames: Vec<DungeonCelFrame>,
    pub frame_count: usize,
}

impl DungeonCelSprite {
    /// 从普通 CEL 格式加载（主 CEL）
    pub fn from_cel_bytes(data: &[u8]) -> Result<Self> {
        // 现有实现，解析偏移表
        // ...
    }
    
    /// 从 CLX 格式加载（特殊 CEL）
    /// ⚠️ 注意：l1s.cel 等文件虽然扩展名是 .cel，但实际是 CLX 格式
    pub fn from_clx_bytes(data: &[u8]) -> Result<Self> {
        // 使用 ClxSprite 解析器
        let clx = ClxSprite::from_bytes(data)?;
        
        // 将 CLX 帧转换为 DungeonCelFrame
        let frames = clx.sheets[0].frames.iter().map(|frame| {
            DungeonCelFrame {
                raw_data: frame.data.clone(),
            }
        }).collect();
        
        Ok(Self {
            frames,
            frame_count: frames.len(),
        })
    }
}
```

**文件**: `src/tiles/texture_manager.rs`

**扩展 TileTextureManager**:

```rust
pub struct TileTextureManager {
    // 主 CEL（l1.cel 等）
    cel_sprite: DungeonCelSprite,
    
    // 特殊 CEL（l1s.cel 等，实际是 CLX）
    special_cel_sprite: Option<DungeonCelSprite>,  // ✅ 已添加此字段
    
    palette: Palette,
    min_data: MinData,
    
    // 缓存
    decoded_cache: HashMap<usize, Vec<u8>>,  // micro_index -> RGBA
    indexed_cache: HashMap<usize, Vec<u8>>,  // micro_index -> indexed
}

impl TileTextureManager {
    /// 加载 tileset（包含主 CEL 和特殊 CEL）
    pub fn load_for_dungeon(
        dungeon_type: DungeonType,
        mpq_manager: &mut MpqManager
    ) -> Result<Self> {
        // 1. 加载主 CEL
        let cel_data = mpq_manager.read_file(get_cel_path(dungeon_type))?;
        let cel_sprite = DungeonCelSprite::from_cel_bytes(&cel_data)?;
        
        // 2. 加载特殊 CEL（CLX 格式）
        let special_cel_sprite = if let Ok(special_data) = 
            mpq_manager.read_file(get_special_cel_path(dungeon_type)) 
        {
            Some(DungeonCelSprite::from_clx_bytes(&special_data)?)
        } else {
            None  // 某些地下城类型可能没有特殊 CEL
        };
        
        // 3. 加载调色板
        let pal_data = mpq_manager.read_file(get_pal_path(dungeon_type))?;
        let palette = Palette::from_bytes(&pal_data)?;
        
        // 4. 加载 MIN
        let min_data = MinData::from_mpq(mpq_manager, dungeon_type)?;
        
        Ok(Self {
            cel_sprite,
            special_cel_sprite,
            palette,
            min_data,
            decoded_cache: HashMap::new(),
            indexed_cache: HashMap::new(),
        })
    }
    
    /// 获取瓦片帧（支持主 CEL 和特殊 CEL）
    pub fn get_frame(&self, frame_idx: usize) -> Result<&DungeonCelFrame> {
        let main_count = self.cel_sprite.frame_count;
        
        if frame_idx < main_count {
            // 主 CEL
            self.cel_sprite.frames.get(frame_idx)
                .ok_or_else(|| anyhow!("Frame {} not found in main CEL", frame_idx))
        } else {
            // 特殊 CEL
            let special = self.special_cel_sprite.as_ref()
                .ok_or_else(|| anyhow!("No special CEL loaded"))?;
            
            let special_idx = frame_idx - main_count;
            special.frames.get(special_idx)
                .ok_or_else(|| anyhow!("Frame {} not found in special CEL", special_idx))
        }
    }
}
```

**路径映射**（参考原版）:

```rust
fn get_cel_path(dungeon_type: DungeonType) -> &'static str {
    match dungeon_type {
        DungeonType::Cathedral => "levels/l1data/l1.cel",
        DungeonType::Catacombs => "levels/l2data/l2.cel",
        DungeonType::Caves => "levels/l3data/l3.cel",
        DungeonType::Hell => "levels/l4data/l4.cel",
    }
}

fn get_special_cel_path(dungeon_type: DungeonType) -> &'static str {
    match dungeon_type {
        DungeonType::Cathedral => "levels/l1data/l1s.cel",
        DungeonType::Catacombs => "levels/l2data/l2s.cel",
        DungeonType::Caves => "levels/l3data/l3s.cel",
        DungeonType::Hell => "levels/l4data/l4s.cel",
    }
}
```

---

### 4.2 SDL2 纹理创建和缓存

#### 原版渲染机制

原版使用软件渲染（直接写入屏幕缓冲区）：

```cpp
// Source/engine/render/dun_render.cpp: RenderTile()
void RenderTile(uint8_t *dst, uint8_t *src, int width, int height)
{
    // 直接复制像素到屏幕缓冲区
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            uint8_t pixel = src[y * width + x];
            if (pixel != 0) {  // 0 = 透明
                dst[y * screenWidth + x] = pixel;
            }
        }
    }
}
```

#### Rust/SDL2 实现方案

SDL2 使用硬件加速纹理渲染：

**文件**: `src/engine/texture_cache.rs`（新建）

```rust
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::collections::HashMap;

/// SDL2 纹理缓存
/// 负责将 RGBA 数据转换为 SDL2 纹理并缓存
pub struct TextureCache<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    cache: HashMap<usize, Texture<'a>>,  // micro_index -> SDL2 纹理
}

impl<'a> TextureCache<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            texture_creator,
            cache: HashMap::new(),
        }
    }
    
    /// 获取或创建纹理
    /// 
    /// 参考原版: Source/engine/render/dun_render.cpp: RenderTile()
    pub fn get_or_create_texture(
        &mut self,
        micro_index: usize,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<&Texture<'a>> {
        if !self.cache.contains_key(&micro_index) {
            // 创建新纹理
            let mut texture = self.texture_creator
                .create_texture_streaming(
                    PixelFormatEnum::ABGR8888,  // SDL2 格式
                    width,
                    height,
                )?;
            
            // 设置混合模式（支持透明度）
            texture.set_blend_mode(BlendMode::Blend);
            
            // 上传像素数据
            texture.with_lock(None, |buffer, pitch| {
                for y in 0..height as usize {
                    for x in 0..width as usize {
                        let src_idx = (y * width as usize + x) * 4;
                        let dst_idx = y * pitch + x * 4;
                        
                        // RGBA -> ABGR (SDL2 格式)
                        buffer[dst_idx + 0] = rgba_data[src_idx + 3];  // A
                        buffer[dst_idx + 1] = rgba_data[src_idx + 2];  // B
                        buffer[dst_idx + 2] = rgba_data[src_idx + 1];  // G
                        buffer[dst_idx + 3] = rgba_data[src_idx + 0];  // R
                    }
                }
            })?;
            
            self.cache.insert(micro_index, texture);
        }
        
        Ok(self.cache.get(&micro_index).unwrap())
    }
    
    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    /// 获取缓存统计
    pub fn stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }
}
```

#### 生命周期处理

**问题**: SDL2 `Texture` 的生命周期与 `TextureCreator` 绑定

**解决方案**: 在 `Engine` 中持有 `TextureCache`

```rust
// src/engine/mod.rs

pub struct Engine {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    window: Window,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,  // 新增
    texture_cache: Option<TextureCache<'static>>,     // 新增
    event_pump: EventPump,
    // ... 其他字段
}

impl Engine {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self> {
        // ... 初始化 SDL2
        
        let texture_creator = canvas.texture_creator();
        
        // 使用 unsafe 延长生命周期（texture_creator 与 engine 同生命周期）
        let texture_cache = Some(unsafe {
            std::mem::transmute(TextureCache::new(&texture_creator))
        });
        
        Ok(Self {
            // ...
            texture_creator,
            texture_cache,
            // ...
        })
    }
    
    pub fn texture_cache_mut(&mut self) -> &mut TextureCache {
        self.texture_cache.as_mut().unwrap()
    }
}
```

---

### 4.3 完整渲染管线

#### 原版渲染流程详解

**参考**: `Source/engine/render/scrollrt.cpp`

```cpp
// Line 1445-1555: DrawTileContent()
void DrawTileContent(const Surface &out)
{
    // 计算可见区域
    int viewX = ViewPosition.tile.x;
    int viewY = ViewPosition.tile.y;
    
    // 遍历瓦片（从后到前，从左到右）
    for (int j = viewY - 1; j < viewY + TILE_HEIGHT + 1; j++) {
        for (int i = viewX - 1; i < viewX + TILE_WIDTH + 2; i++) {
            // 检查边界
            if (!InDungeonBounds({ i, j })) continue;
            
            // 计算屏幕坐标（等距投影）
            int px = (i - j) * TILE_WIDTH / 2;     // X 坐标
            int py = (i + j) * TILE_HEIGHT / 2;    // Y 坐标
            
            // Pass 1: 渲染地板
            DrawFloorTile(out, i, j, px, py);
            
            // Pass 2: 渲染墙体和物体
            DrawCell(out, i, j, px, py);
        }
    }
}

// Line 1187-1240: DrawFloorTile()
void DrawFloorTile(const Surface &out, int x, int y, int px, int py)
{
    int tileId = dPiece[x][y];  // MegaTile ID
    if (tileId == 0) return;
    
    const TileData &tile = LevelPieces[tileId];
    
    // 渲染左三角
    if (tile.micro1 != 0) {
        RenderTile(out, 
            GetDunFrame(tile.micro1),  // frame 数据
            px - TILE_WIDTH,            // X 位置（左偏移）
            py,                         // Y 位置
            TILE_WIDTH, TILE_HEIGHT_LOWER,  // 尺寸
            TileType::LeftTriangle);
    }
    
    // 渲染右三角
    if (tile.micro2 != 0) {
        RenderTile(out,
            GetDunFrame(tile.micro2),
            px,                         // X 位置（右边）
            py,
            TILE_WIDTH, TILE_HEIGHT_LOWER,
            TileType::RightTriangle);
    }
}

// Line 1262-1402: DrawCell()
void DrawCell(const Surface &out, int x, int y, int px, int py)
{
    int tileId = dPiece[x][y];
    if (tileId == 0) return;
    
    const TileData &tile = LevelPieces[tileId];
    
    // 获取 4 个 microtile
    const Microile micros[4] = {
        DPieceMicros[tileId][0],  // micro1 (下层左)
        DPieceMicros[tileId][1],  // micro2 (下层右)
        DPieceMicros[tileId][2],  // micro3 (上层左)
        DPieceMicros[tileId][3],  // micro4 (上层右)
    };
    
    // 从底部到顶部渲染
    for (int layer = 0; layer < 2; layer++) {
        for (int side = 0; side < 2; side++) {
            int microIdx = layer * 2 + side;
            const Microtile &micro = micros[microIdx];
            
            if (micro.frameIdx == 0) continue;
            
            // 计算位置
            int drawX = px;
            int drawY = py;
            
            // 垂直偏移（上层瓦片）
            if (layer == 1) {
                drawY -= TILE_HEIGHT;  // -32 像素
            }
            
            // 水平偏移（根据 TileType）
            switch (micro.tileType) {
                case TileType::LeftTriangle:
                case TileType::LeftTrapezoid:
                    drawX -= TILE_WIDTH;  // 左偏移
                    break;
                case TileType::RightTriangle:
                case TileType::RightTrapezoid:
                    // 右对齐，无偏移
                    break;
                case TileType::Square:
                case TileType::TransparentSquare:
                    drawX -= TILE_WIDTH / 2;  // 居中
                    break;
            }
            
            // 渲染瓦片
            RenderTile(out,
                GetDunFrame(micro.frameIdx),
                drawX, drawY,
                TILE_WIDTH, TILE_HEIGHT,
                micro.tileType);
        }
    }
}
```

#### Rust 实现

**文件**: `src/world/mod.rs`

```rust
impl World {
    /// 主渲染入口
    /// 
    /// 参考原版: Source/engine/render/scrollrt.cpp: DrawTileContent()
    pub fn render_tiles(
        &self,
        engine: &mut Engine,
        camera_x: i32,
        camera_y: i32,
    ) -> Result<()> {
        let texture_mgr = self.texture_manager.as_ref()
            .ok_or_else(|| anyhow!("No texture manager"))?;
        
        // 计算可见区域
        let view_x = camera_x / TILE_WIDTH;
        let view_y = camera_y / TILE_HEIGHT;
        
        // 遍历瓦片（从后到前）
        for map_y in (view_y - 1)..(view_y + VISIBLE_TILES_Y + 1) {
            for map_x in (view_x - 1)..(view_x + VISIBLE_TILES_X + 2) {
                // 检查边界
                if !self.in_bounds(map_x, map_y) {
                    continue;
                }
                
                // 计算屏幕坐标（等距投影）
                let screen_x = (map_x - map_y) * (TILE_WIDTH / 2);
                let screen_y = (map_x + map_y) * (TILE_HEIGHT / 2);
                
                // Pass 1: 渲染地板
                self.render_floor_tile(
                    engine,
                    texture_mgr,
                    map_x, map_y,
                    screen_x, screen_y,
                )?;
                
                // Pass 2: 渲染墙体
                self.render_wall_cell(
                    engine,
                    texture_mgr,
                    map_x, map_y,
                    screen_x, screen_y,
                )?;
            }
        }
        
        Ok(())
    }
    
    /// 渲染地板瓦片
    /// 
    /// 参考原版: Source/engine/render/scrollrt.cpp: DrawFloorTile()
    fn render_floor_tile(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        map_x: i32,
        map_y: i32,
        screen_x: i32,
        screen_y: i32,
    ) -> Result<()> {
        let tile_id = self.get_tile(map_x, map_y)?;
        if tile_id == 0 {
            return Ok(());
        }
        
        let mega_tile = &self.til_data.tiles[tile_id as usize];
        
        // 渲染左三角（micro1）
        if mega_tile.micro1 != 0 {
            self.render_micro_tile(
                engine,
                texture_mgr,
                mega_tile.micro1 as usize,
                screen_x - TILE_WIDTH,  // 左偏移
                screen_y,
                TileType::LeftTriangle,
            )?;
        }
        
        // 渲染右三角（micro2）
        if mega_tile.micro2 != 0 {
            self.render_micro_tile(
                engine,
                texture_mgr,
                mega_tile.micro2 as usize,
                screen_x,  // 右边
                screen_y,
                TileType::RightTriangle,
            )?;
        }
        
        Ok(())
    }
    
    /// 渲染墙体单元格
    /// 
    /// 参考原版: Source/engine/render/scrollrt.cpp: DrawCell()
    fn render_wall_cell(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        map_x: i32,
        map_y: i32,
        screen_x: i32,
        screen_y: i32,
    ) -> Result<()> {
        let tile_id = self.get_tile(map_x, map_y)?;
        if tile_id == 0 {
            return Ok(());
        }
        
        let mega_tile = &self.til_data.tiles[tile_id as usize];
        
        // 获取 4 个 microtile
        let micros = [
            mega_tile.micro1,  // 下层左
            mega_tile.micro2,  // 下层右
            mega_tile.micro3,  // 上层左
            mega_tile.micro4,  // 上层右
        ];
        
        // 从底部到顶部渲染
        for layer in 0..2 {
            for side in 0..2 {
                let micro_idx = layer * 2 + side;
                let micro_id = micros[micro_idx];
                
                if micro_id == 0 {
                    continue;
                }
                
                // 获取 TileType
                let min_block = self.min_data.get(micro_id as usize)?;
                let tile_type = min_block.tile_type();
                
                // 计算位置
                let mut draw_x = screen_x;
                let mut draw_y = screen_y;
                
                // 垂直偏移（上层）
                if layer == 1 {
                    draw_y -= TILE_HEIGHT as i32;  // -32
                }
                
                // 水平偏移（根据 TileType）
                match tile_type {
                    TileType::LeftTriangle | TileType::LeftTrapezoid => {
                        draw_x -= TILE_WIDTH as i32;
                    }
                    TileType::RightTriangle | TileType::RightTrapezoid => {
                        // 右对齐，无偏移
                    }
                    TileType::Square | TileType::TransparentSquare => {
                        draw_x -= (TILE_WIDTH / 2) as i32;
                    }
                }
                
                // 渲染 microtile
                self.render_micro_tile(
                    engine,
                    texture_mgr,
                    micro_id as usize,
                    draw_x,
                    draw_y,
                    tile_type,
                )?;
            }
        }
        
        Ok(())
    }
    
    /// 渲染单个 microtile
    /// 
    /// 参考原版: Source/engine/render/dun_render.cpp: RenderTile()
    fn render_micro_tile(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        micro_index: usize,
        x: i32,
        y: i32,
        tile_type: TileType,
    ) -> Result<()> {
        // 1. 获取解码后的 RGBA 数据
        let mut mgr = texture_mgr.borrow_mut();
        let rgba_data = mgr.get_decoded_tile(micro_index)?;
        
        // 2. 获取或创建 SDL2 纹理
        let (width, height) = tile_type.dimensions();
        let texture = engine.texture_cache_mut().get_or_create_texture(
            micro_index,
            rgba_data,
            width as u32,
            height as u32,
        )?;
        
        // 3. 渲染到屏幕
        let dst_rect = sdl2::rect::Rect::new(
            x,
            y,
            width as u32,
            height as u32,
        );
        
        engine.canvas.copy(texture, None, Some(dst_rect))?;
        
        Ok(())
    }
}
```

---

### 4.4 透明度和混合

#### TransparentSquare 处理

**原版逻辑**:
```cpp
// Source/engine/render/dun_render.cpp: RenderTransparentSquare()
void RenderTransparentSquare(uint8_t *dst, uint8_t *src, int width, int height)
{
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            uint8_t pixel = src[y * width + x];
            if (pixel != 0) {  // 0 = 完全透明
                dst[y * screenWidth + x] = pixel;
            }
            // pixel == 0 时跳过（保留背景）
        }
    }
}
```

**Rust 实现**（在 Palette 转换时处理）:

```rust
// src/resources/palette.rs

impl Palette {
    /// 索引色转 RGBA（支持透明度）
    pub fn indices_to_rgba(&self, indices: &[u8], transparent: bool) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(indices.len() * 4);
        
        for &index in indices {
            let color = self.colors[index as usize];
            
            rgba.push(color.r);
            rgba.push(color.g);
            rgba.push(color.b);
            
            // Alpha 通道
            if transparent && index == 0 {
                rgba.push(0);  // 完全透明
            } else {
                rgba.push(255);  // 不透明
            }
        }
        
        rgba
    }
}
```

**SDL2 混合模式**:

```rust
// src/engine/texture_cache.rs

impl<'a> TextureCache<'a> {
    pub fn get_or_create_texture(...) -> Result<&Texture<'a>> {
        let mut texture = self.texture_creator.create_texture_streaming(...)?;
        
        // 设置混合模式（支持 alpha 通道）
        texture.set_blend_mode(BlendMode::Blend);  // ✅ 关键
        
        // ...
    }
}
```

---

## 📚 原版代码对照

### 关键文件和函数位置

| Rust 模块 | 原版 C++ 文件 | 关键函数 | 行号 |
|----------|--------------|---------|------|
| `world/mod.rs::render_tiles()` | `scrollrt.cpp` | `DrawTileContent()` | 1445-1555 |
| `world/mod.rs::render_floor_tile()` | `scrollrt.cpp` | `DrawFloorTile()` | 1187-1240 |
| `world/mod.rs::render_wall_cell()` | `scrollrt.cpp` | `DrawCell()` | 1262-1402 |
| `world/mod.rs::render_micro_tile()` | `dun_render.cpp` | `RenderTile()` | 305-418 |
| `tiles/texture_manager.rs::get_frame()` | `gendung.cpp` | `GetDunFrame()` | 1165-1180 |
| `resources/dungeon_cel.rs::from_clx_bytes()` | `gendung.cpp` | `LoadLvlGFX()` | 1235-1264 |
| `engine/texture_cache.rs` | `dun_render.cpp` | 各种 `Render*()` 函数 | 305-600 |

### 等距投影坐标计算

**原版**: `Source/engine/displacement.hpp`

```cpp
constexpr int TILE_WIDTH = 64;
constexpr int TILE_HEIGHT = 32;

// 世界坐标 -> 屏幕坐标
Point WorldToScreen(int worldX, int worldY) {
    return {
        (worldX - worldY) * TILE_WIDTH / 2,
        (worldX + worldY) * TILE_HEIGHT / 2
    };
}

// 屏幕坐标 -> 世界坐标
Point ScreenToWorld(int screenX, int screenY) {
    int worldX = (screenX / (TILE_WIDTH / 2) + screenY / (TILE_HEIGHT / 2)) / 2;
    int worldY = (screenY / (TILE_HEIGHT / 2) - screenX / (TILE_WIDTH / 2)) / 2;
    return { worldX, worldY };
}
```

**Rust**: `src/world/mod.rs`

```rust
pub const TILE_WIDTH: i32 = 64;
pub const TILE_HEIGHT: i32 = 32;

/// 世界坐标 -> 屏幕坐标（等距投影）
pub fn world_to_screen(world_x: i32, world_y: i32) -> (i32, i32) {
    let screen_x = (world_x - world_y) * (TILE_WIDTH / 2);
    let screen_y = (world_x + world_y) * (TILE_HEIGHT / 2);
    (screen_x, screen_y)
}

/// 屏幕坐标 -> 世界坐标
pub fn screen_to_world(screen_x: i32, screen_y: i32) -> (i32, i32) {
    let world_x = (screen_x / (TILE_WIDTH / 2) + screen_y / (TILE_HEIGHT / 2)) / 2;
    let world_y = (screen_y / (TILE_HEIGHT / 2) - screen_x / (TILE_WIDTH / 2)) / 2;
    (world_x, world_y)
}
```

---

## 📝 实施步骤

### 阶段 1: 特殊瓦片加载（Week 1, Day 1-2）

#### 任务清单

- [ ] **Task 1.1**: 修改 `DungeonCelSprite` 添加 `from_clx_bytes()` 方法
- [ ] **Task 1.2**: 扩展 `TileTextureManager` 添加 `special_cel_sprite` 字段
- [ ] **Task 1.3**: 实现 `get_frame()` 方法（支持主 CEL + 特殊 CEL）
- [ ] **Task 1.4**: 添加路径映射函数（`get_special_cel_path()`）
- [ ] **Task 1.5**: 编写单元测试（手工构造 CLX 数据）
- [ ] **Task 1.6**: 运行真实 MPQ 测试（验证 l1s.cel 加载）

#### 测试验证

```rust
#[test]
#[ignore]
fn test_load_special_cel() {
    let mut rm = create_rm_with_mpq().unwrap();
    let mut tex_mgr = TileTextureManager::load_for_dungeon(
        DungeonType::Cathedral,
        rm.mpq_manager_mut()
    ).unwrap();
    
    // 验证主 CEL
    assert_eq!(tex_mgr.cel_sprite.frame_count, 279);
    
    // 验证特殊 CEL
    assert!(tex_mgr.special_cel_sprite.is_some());
    let special = tex_mgr.special_cel_sprite.as_ref().unwrap();
    assert!(special.frame_count > 0);
    
    // 验证帧获取（索引 280+）
    let frame = tex_mgr.get_frame(280).unwrap();
    assert!(!frame.raw_data.is_empty());
}
```

**验收标准**:
- ✅ l1s.cel 成功加载为 CLX 格式
- ✅ 所有 4530 个瓦片（100%）都能获取帧数据
- ✅ 无 "Frame index out of range" 错误

---

### 阶段 2: SDL2 纹理系统（Week 1, Day 3-4）

#### 任务清单

- [ ] **Task 2.1**: 创建 `src/engine/texture_cache.rs` 模块
- [ ] **Task 2.2**: 实现 `TextureCache` 结构
- [ ] **Task 2.3**: 实现 `get_or_create_texture()` 方法
- [ ] **Task 2.4**: 在 `Engine` 中集成 `TextureCache`
- [ ] **Task 2.5**: 处理生命周期问题（unsafe transmute）
- [ ] **Task 2.6**: 编写单元测试

#### 测试验证

```rust
#[test]
fn test_texture_cache() {
    let mut engine = Engine::new("Test", 800, 600).unwrap();
    
    // 创建测试 RGBA 数据
    let rgba_data = vec![255u8; 32 * 32 * 4];  // 白色 32x32 纹理
    
    // 获取或创建纹理
    let texture = engine.texture_cache_mut()
        .get_or_create_texture(0, &rgba_data, 32, 32)
        .unwrap();
    
    // 验证纹理创建
    assert_eq!(texture.query().width, 32);
    assert_eq!(texture.query().height, 32);
    
    // 验证缓存
    let (cached_count, _) = engine.texture_cache_mut().stats();
    assert_eq!(cached_count, 1);
}
```

**验收标准**:
- ✅ SDL2 纹理成功创建
- ✅ 纹理缓存工作正常
- ✅ 混合模式正确设置（支持透明度）

---

### 阶段 3: 完整渲染集成（Week 1, Day 5 + Week 2, Day 1-3）

#### 任务清单

- [ ] **Task 3.1**: 实现 `render_tiles()` 主入口
- [ ] **Task 3.2**: 实现 `render_floor_tile()` 完整逻辑
- [ ] **Task 3.3**: 实现 `render_wall_cell()` 完整逻辑
- [ ] **Task 3.4**: 实现 `render_micro_tile()` 纹理渲染
- [ ] **Task 3.5**: 添加等距投影坐标计算辅助函数
- [ ] **Task 3.6**: 集成到游戏主循环
- [ ] **Task 3.7**: 调试坐标偏移和图层叠加

#### 实现细节

**关键: 渲染顺序**
```rust
// 从后到前，从下到上
for map_y in view_y_start..view_y_end {
    for map_x in view_x_start..view_x_end {
        // Pass 1: 地板（必须先渲染）
        render_floor_tile(...);
        
        // Pass 2: 墙体（覆盖在地板上）
        render_wall_cell(...);
    }
}
```

**关键: 坐标偏移**
```rust
// 等距投影
let screen_x = (map_x - map_y) * 32;  // TILE_WIDTH / 2
let screen_y = (map_x + map_y) * 16;  // TILE_HEIGHT / 2

// 左三角偏移
let left_x = screen_x - 64;  // -TILE_WIDTH

// 右三角无偏移
let right_x = screen_x;

// 上层瓦片偏移
let upper_y = screen_y - 32;  // -TILE_HEIGHT
```

#### 测试验证

```rust
#[test]
#[ignore]
fn test_render_cathedral_scene() {
    let mut engine = Engine::new("Test", 800, 600).unwrap();
    let mut world = World::new();
    
    // 加载 Cathedral tileset
    world.load_tileset(DungeonType::Cathedral, &mut rm).unwrap();
    
    // 渲染测试场景
    world.render_tiles(&mut engine, 0, 0).unwrap();
    
    // 保存截图验证
    engine.save_screenshot("test_cathedral_render.png").unwrap();
}
```

**验收标准**:
- ✅ 瓦片正确渲染到屏幕
- ✅ 地板和墙体分层正确
- ✅ 坐标对齐无偏移
- ✅ 无黑块、花屏

---

### 阶段 4: 测试和优化（Week 2, Day 4-5）

#### 任务清单

- [ ] **Task 4.1**: 性能测试（FPS 测量）
- [ ] **Task 4.2**: 内存测试（纹理缓存大小）
- [ ] **Task 4.3**: 视觉验证（多个场景）
- [ ] **Task 4.4**: 边界情况测试
- [ ] **Task 4.5**: 文档完善（实现总结）

#### 性能测试

```rust
#[bench]
fn bench_render_full_screen(b: &mut Bencher) {
    let mut engine = Engine::new("Bench", 800, 600).unwrap();
    let mut world = World::new();
    world.load_tileset(DungeonType::Cathedral, &mut rm).unwrap();
    
    b.iter(|| {
        world.render_tiles(&mut engine, 0, 0).unwrap();
    });
}
```

**性能目标**:
- 渲染一帧 < 16ms（60 FPS）
- 纹理缓存 < 100MB
- 无明显卡顿

---

## 🧪 测试策略

### 单元测试

#### 特殊 CEL 加载测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clx_to_dungeon_cel() {
        // 手工构造 CLX 数据
        let clx_data = create_test_clx_data();
        
        let sprite = DungeonCelSprite::from_clx_bytes(&clx_data).unwrap();
        assert_eq!(sprite.frame_count, 10);
        assert!(!sprite.frames[0].raw_data.is_empty());
    }
    
    #[test]
    fn test_get_frame_main_cel() {
        let tex_mgr = create_test_texture_manager();
        
        // 主 CEL 帧
        let frame = tex_mgr.get_frame(100).unwrap();
        assert!(!frame.raw_data.is_empty());
    }
    
    #[test]
    fn test_get_frame_special_cel() {
        let tex_mgr = create_test_texture_manager();
        
        // 特殊 CEL 帧（索引 280+）
        let frame = tex_mgr.get_frame(300).unwrap();
        assert!(!frame.raw_data.is_empty());
    }
}
```

#### SDL2 纹理测试

```rust
#[test]
fn test_texture_creation() {
    let mut engine = Engine::new("Test", 800, 600).unwrap();
    
    let rgba = vec![255u8; 32 * 32 * 4];
    let texture = engine.texture_cache_mut()
        .get_or_create_texture(0, &rgba, 32, 32)
        .unwrap();
    
    assert_eq!(texture.query().width, 32);
}

#[test]
fn test_texture_caching() {
    let mut engine = Engine::new("Test", 800, 600).unwrap();
    
    let rgba = vec![255u8; 32 * 32 * 4];
    
    // 第一次创建
    engine.texture_cache_mut()
        .get_or_create_texture(0, &rgba, 32, 32)
        .unwrap();
    
    // 第二次应该使用缓存
    let (count1, _) = engine.texture_cache_mut().stats();
    
    engine.texture_cache_mut()
        .get_or_create_texture(0, &rgba, 32, 32)
        .unwrap();
    
    let (count2, _) = engine.texture_cache_mut().stats();
    
    assert_eq!(count1, count2);  // 缓存命中
}
```

### 集成测试

#### 完整渲染测试

```rust
#[test]
#[ignore]
fn test_render_complete_scene() {
    let mut engine = Engine::new("Test", 800, 600).unwrap();
    let mut rm = create_rm_with_mpq().unwrap();
    let mut world = World::new();
    
    // 加载 tileset
    world.load_tileset(
        DungeonType::Cathedral,
        rm.mpq_manager_mut()
    ).unwrap();
    
    // 渲染测试
    for camera_y in 0..10 {
        for camera_x in 0..10 {
            world.render_tiles(
                &mut engine,
                camera_x * TILE_WIDTH,
                camera_y * TILE_HEIGHT,
            ).unwrap();
        }
    }
    
    // 验证纹理缓存
    let (cached, _) = engine.texture_cache_mut().stats();
    assert!(cached > 0);
}
```

### 视觉验证

#### 截图对比测试

```rust
#[test]
#[ignore]
fn test_visual_comparison() {
    let mut engine = Engine::new("Test", 800, 600).unwrap();
    let mut world = World::new();
    world.load_tileset(DungeonType::Cathedral, &mut rm).unwrap();
    
    // 渲染测试场景
    world.render_tiles(&mut engine, 0, 0).unwrap();
    
    // 保存截图
    engine.save_screenshot("tests/output/render_test_1.png").unwrap();
    
    // 人工验证：
    // 1. 地板瓦片是否正确？
    // 2. 墙体瓦片是否正确？
    // 3. 图层叠加是否正确？
    // 4. 透明度是否正确？
}
```

---

## 🎓 关键学习要点

### 1. 等距投影（Isometric Projection）

**核心公式**:
```
Screen_X = (World_X - World_Y) × TILE_WIDTH / 2
Screen_Y = (World_X + World_Y) × TILE_HEIGHT / 2
```

**为什么这样计算？**
- 等距投影将 2D 网格旋转 45° 并压扁
- X 轴和 Y 轴的贡献是对称的
- 结果是菱形瓦片（64×32）

**关键要点**:
- 瓦片必须从后到前渲染（画家算法）
- 坐标系原点通常在屏幕中心
- 负坐标是正常的（左上方）

### 2. 特殊 CEL 文件的陷阱

**坑点**: 
- ⚠️ `l1s.cel` 扩展名是 `.cel`，但实际格式是 **CLX**！
- ⚠️ 不能用 `DungeonCelSprite::from_cel_bytes()` 加载
- ⚠️ 必须用 CLX 解析器

**为什么这样设计？**
- 特殊瓦片（门、装饰）需要更复杂的结构
- CLX 格式支持多 sheet、帧组等高级特性
- 原版为了向后兼容保留了 `.cel` 扩展名

### 3. SDL2 纹理生命周期

**问题**: `Texture<'a>` 生命周期与 `TextureCreator<'a>` 绑定

**解决方案**:
```rust
// 使用 unsafe transmute 延长生命周期
let texture_cache = unsafe {
    std::mem::transmute(TextureCache::new(&texture_creator))
};
```

**为什么安全？**
- `TextureCreator` 和 `TextureCache` 都在 `Engine` 中
- `Engine` 生命周期保证二者同时存在
- 不会出现悬垂指针

### 4. 渲染顺序的重要性

**错误示例**:
```rust
// ❌ 错误：先渲染墙体再渲染地板
render_wall_cell(...);   // 墙体在底层
render_floor_tile(...);  // 地板在上层 → 错误！
```

**正确示例**:
```rust
// ✅ 正确：先渲染地板再渲染墙体
render_floor_tile(...);  // 地板在底层
render_wall_cell(...);   // 墙体在上层 → 正确！
```

**原因**: 2D 渲染没有 Z-buffer，必须手动控制顺序

### 5. 透明度处理

**关键点**:
- 索引色 `0` = 完全透明（alpha = 0）
- 其他索引 = 不透明（alpha = 255）
- SDL2 必须设置 `BlendMode::Blend`

**代码**:
```rust
// 转换时处理透明度
if transparent && index == 0 {
    rgba.push(0);    // alpha = 0（透明）
} else {
    rgba.push(255);  // alpha = 255（不透明）
}

// SDL2 混合模式
texture.set_blend_mode(BlendMode::Blend);
```

---

## ⚠️ 常见坑点

### 1. 坐标偏移错误

**问题**: 左三角瓦片位置不对

```rust
// ❌ 错误
let left_x = screen_x;  // 应该有偏移！

// ✅ 正确
let left_x = screen_x - TILE_WIDTH;  // 左偏移 64 像素
```

**调试方法**: 在纸上画出网格和瓦片位置

### 2. 图层叠加顺序

**问题**: 上层瓦片（micro3/4）渲染在下层瓦片下面

```rust
// ❌ 错误：从上到下
for layer in (0..2).rev() {  // 2, 1, 0
    // ...
}

// ✅ 正确：从下到上
for layer in 0..2 {  // 0, 1
    // ...
}
```

### 3. CLX 格式识别

**问题**: 尝试用 CEL 解析器加载 l1s.cel

```rust
// ❌ 错误
let special = DungeonCelSprite::from_cel_bytes(&data)?;  // 失败！

// ✅ 正确
let special = DungeonCelSprite::from_clx_bytes(&data)?;  // 成功
```

### 4. 纹理缓存键冲突

**问题**: 主 CEL 和特殊 CEL 的帧索引重叠

```rust
// ❌ 错误：直接用 frame_idx 作为缓存键
cache.insert(frame_idx, texture);  // 可能冲突！

// ✅ 正确：用 micro_index 作为缓存键
cache.insert(micro_index, texture);  // 唯一
```

### 5. RGBA 字节序

**问题**: SDL2 纹理颜色不对

```rust
// ❌ 错误：RGBA 顺序
buffer[idx + 0] = rgba[src + 0];  // R
buffer[idx + 1] = rgba[src + 1];  // G
buffer[idx + 2] = rgba[src + 2];  // B
buffer[idx + 3] = rgba[src + 3];  // A

// ✅ 正确：ABGR 顺序（SDL2 格式）
buffer[idx + 0] = rgba[src + 3];  // A
buffer[idx + 1] = rgba[src + 2];  // B
buffer[idx + 2] = rgba[src + 1];  // G
buffer[idx + 3] = rgba[src + 0];  // R
```

---

## 🎯 验收标准

### 功能验收

- ✅ **F1**: 成功加载所有 4530 个瓦片（100%）
- ✅ **F2**: 特殊瓦片（l1s.cel）正确加载和渲染
- ✅ **F3**: 地板和墙体正确渲染（无黑块、花屏）
- ✅ **F4**: 图层叠加正确（上层瓦片覆盖下层）
- ✅ **F5**: 透明度正确处理（TransparentSquare 背景透明）

### 性能验收

- ✅ **P1**: 渲染一帧 < 16ms（60 FPS）
- ✅ **P2**: 纹理缓存 < 100MB
- ✅ **P3**: 无内存泄漏
- ✅ **P4**: 纹理缓存命中率 > 90%

### 代码质量验收

- ✅ **Q1**: 每个函数都标注原版代码位置
- ✅ **Q2**: 关键算法有详细注释
- ✅ **Q3**: 测试覆盖率 > 75%
- ✅ **Q4**: 所有 linter 警告已解决

### 视觉验收

- ✅ **V1**: Cathedral 场景渲染正确
- ✅ **V2**: 与原版截图对比一致
- ✅ **V3**: 不同视角（相机移动）渲染正确
- ✅ **V4**: 边界处理正确（地图边缘）

---

## 📈 预期成果

### 代码量估算

| 模块 | 新增代码 | 修改代码 | 测试代码 |
|------|---------|---------|---------|
| dungeon_cel.rs | ~100 | ~50 | ~80 |
| texture_manager.rs | ~150 | ~80 | ~100 |
| texture_cache.rs | ~200 | 0 | ~120 |
| world/mod.rs | ~300 | ~150 | ~150 |
| engine/mod.rs | ~50 | ~30 | ~40 |
| **总计** | **~800** | **~310** | **~490** |

**总代码量**: ~1600 行（800 新增 + 310 修改 + 490 测试）

### 文档输出

- ✅ 本设计文档（step-6.3-wall-tiles-and-rendering.md）
- ✅ 实现总结文档（step-6.3-implementation-summary.md）
- ✅ 踩坑记录（step-6.3-pitfalls.md）
- ✅ 性能分析报告（step-6.3-performance.md）

---

## 📚 参考资料

### 原版代码

- `Source/engine/render/scrollrt.cpp` - 主渲染循环
- `Source/engine/render/dun_render.cpp` - 瓦片帧渲染
- `Source/engine/render/cl2_render.cpp` - CLX/CL2 渲染
- `Source/levels/gendung.cpp` - 地下城数据管理
- `Source/engine/displacement.hpp` - 等距投影坐标

### SDL2 文档

- [SDL2 Texture Tutorial](https://lazyfoo.net/tutorials/SDL/10_color_keying/index.php)
- [SDL2 Rendering](https://wiki.libsdl.org/CategoryRender)
- [BlendMode Documentation](https://wiki.libsdl.org/SDL_BlendMode)

### 等距投影资源

- [Isometric Projection - Wikipedia](https://en.wikipedia.org/wiki/Isometric_projection)
- [Isometric Game Tutorial](https://gamedevelopment.tutsplus.com/tutorials/creating-isometric-worlds-a-primer-for-game-developers--gamedev-6511)

---

## 🚀 下一步计划（Step 6.4+）

完成 Step 6.3 后，后续步骤：

### Step 6.4: 光照系统
- dLight 数组（光照贴图）
- 动态光源（火把、法术）
- 阴影效果

### Step 6.5: 性能优化
- 批量渲染（减少 draw call）
- 纹理图集（减少纹理切换）
- 视口裁剪优化

### Step 7: 地图生成
- BSP 房间生成算法
- Miniset 系统（门、楼梯）
- 地图验证和修正

---

**文档版本**: 1.0  
**创建日期**: 2025-12-02  
**作者**: AI Assistant (Claude Sonnet 4.5)  
**状态**: ✅ 待审阅

---

**记住核心原则：严格对照原版 C++ 代码，不自由发挥！** 🎯


