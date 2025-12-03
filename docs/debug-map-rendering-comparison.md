# 地图渲染问题诊断对比分析

## 问题描述

地图渲染实现了，但画面看起来非常奇怪，纹理似乎没有正确加载。

## 诊断思路

1. 自顶向下分析C++的地图加载和渲染流程
2. 自顶向下分析Rust的地图加载和渲染流程
3. 对比二者差异，找出问题所在

---

## 第一部分：C++地图加载与渲染流程分析

### 1. C++顶层入口点：LoadGameLevel()

**文件位置:** `Source/diablo.cpp` Line 3328-3413

**关键流程:**
```cpp
tl::expected<void, std::string> LoadGameLevel(bool firstflag, lvl_entry lvldir)
{
    // 1. 加载图形资源
    RETURN_IF_ERROR(LoadTrns());                    // 加载颜色表
    MakeLightTable();                               // 生成光照表
    RETURN_IF_ERROR(LoadLevelSOLData());            // 加载SOL数据（碰撞）
    
    // 2. 加载关卡图形数据
    RETURN_IF_ERROR(LoadLvlGFX());                  // ← 关键：加载CEL, MIN数据
    SetDungeonMicros(pDungeonCels, MicroTileLen);   // ← 关键：构建DPieceMicros
    
    // 3. 生成/加载地图
    if (setlevel) {
        RETURN_IF_ERROR(LoadGameLevelSetLevel(...));
    } else {
        RETURN_IF_ERROR(LoadGameLevelStandardLevel(...));
    }
    
    return {};
}
```

### 2. C++地图数据加载：PlaceDunTiles()

**文件位置:** `Source/levels/gendung.cpp` Line 690-707

**关键功能:** 加载DUN文件，填充`dungeon[DMAXX][DMAXY]`数组

```cpp
void PlaceDunTiles(const uint16_t *dunData, Point position, int floorId)
{
    const WorldTileSize size = GetDunSize(dunData);
    const uint16_t *tileLayer = &dunData[2];  // Skip width/height header
    
    for (WorldTileCoord j = 0; j < size.height; j++) {
        for (WorldTileCoord i = 0; i < size.width; i++) {
            // ⚠️ 关键：dunData存储的是MegaTile索引（1-based）
            auto tileId = static_cast<uint8_t>(Swap16LE(tileLayer[j * size.width + i]));
            if (tileId != 0) {
                dungeon[position.x + i][position.y + j] = tileId;  // ← 存储MegaTile ID
            } else if (floorId != 0) {
                dungeon[position.x + i][position.y + j] = floorId;
            }
        }
    }
}
```

**数据结构:**
- `dungeon[DMAXX][DMAXY]` (40x40): 存储MegaTile索引（1-based）
- `dPiece[MAXDUNX][MAXDUNY]` (112x112): 存储levelPieceId（MicroTile索引）

### 3. C++地图扩展：FillSector() / DRLG_LPass3()

**文件位置:** `Source/levels/town.cpp` Line 26-58

**关键功能:** 将MegaTile扩展为MicroTile（2x2扩展）

```cpp
void FillSector(const char *path, int xi, int yy)
{
    auto dunData = LoadFileInMem<uint16_t>(path);
    const WorldTileSize size = GetDunSize(dunData.get());
    const uint16_t *tileLayer = &dunData[2];
    
    for (WorldTileCoord j = 0; j < size.height; j++) {
        int xx = xi;
        for (WorldTileCoord i = 0; i < size.width; i++) {
            // ⚠️ 关键：从TIL数据获取MegaTile定义，每个MegaTile = 4个MicroTile
            const int tileId = Swap16LE(tileLayer[j * size.width + i]) - 1;  // Convert to 0-based
            if (tileId >= 0) {
                const MegaTile mega = pMegaTiles[tileId];  // ← 查找TIL数据
                v1 = Swap16LE(mega.micro1);  // ← 4个MicroTile索引
                v2 = Swap16LE(mega.micro2);
                v3 = Swap16LE(mega.micro3);
                v4 = Swap16LE(mega.micro4);
            }
            
            // ⚠️ 关键：2x2扩展到dPiece数组
            dPiece[xx + 0][yy + 0] = v1;  // Left-Top
            dPiece[xx + 1][yy + 0] = v2;  // Right-Top
            dPiece[xx + 0][yy + 1] = v3;  // Left-Bottom
            dPiece[xx + 1][yy + 1] = v4;  // Right-Bottom
            xx += 2;
        }
        yy += 2;
    }
}
```

**数据流:**
```
DUN文件 (MegaTile indices)
   ↓
dungeon[40][40] (MegaTile IDs, 1-based)
   ↓ (查找TIL数据)
pMegaTiles[tileId] → 4个MicroTile索引
   ↓ (2x2扩展)
dPiece[112][112] (levelPieceId, 即MicroTile索引)
```

### 4. C++微型瓦片数据构建：SetDungeonMicros()

**文件位置:** `Source/levels/gendung.cpp` Line 509-560

**关键功能:** 加载MIN文件，构建`DPieceMicros`数组

```cpp
void SetDungeonMicros(std::unique_ptr<std::byte[]> &dungeonCels, uint_fast8_t &microTileLen)
{
    // 1. 确定每个Piece的块数
    microTileLen = 10;
    size_t blocks = 10;
    if (leveltype == DTYPE_TOWN) {
        microTileLen = 16;
        blocks = 16;
    } else if (leveltype == DTYPE_HELL) {
        microTileLen = 12;
        blocks = 16;
    }
    
    // 2. 加载MIN数据
    size_t tileCount;
    const std::unique_ptr<uint16_t[]> levelPieces = LoadMinData(tileCount);  // ← 加载MIN文件
    
    // 3. 构建DPieceMicros数组
    for (size_t levelPieceId = 0; levelPieceId < tileCount / blocks; levelPieceId++) {
        uint16_t *pieces = &levelPieces[blocks * levelPieceId];
        for (uint32_t block = 0; block < blocks; block++) {
            // ⚠️ 关键：重新排序块索引
            const LevelCelBlock levelCelBlock { Swap16LE(pieces[blocks - 2 + (block & 1) - (block & 0xE)]) };
            DPieceMicros[levelPieceId].mt[block] = levelCelBlock;  // ← 存储到DPieceMicros
        }
    }
}
```

**LevelCelBlock结构:**
```cpp
struct LevelCelBlock {
    uint16_t data;
    
    bool hasValue() const { return data != 0; }
    TileType type() const { return static_cast<TileType>((data & 0x7000) >> 12); }
    uint16_t frame() const { return data & 0xFFF; }  // ← 1-based frame index
};
```

**数据结构:**
```
MIN文件: [u16; n]  (每个u16 = 块的编码)
   ↓
DPieceMicros[levelPieceId].mt[block]  (每个Piece有10-16个块)
   ↓
LevelCelBlock: { type: TileType, frame: u16 }
```

### 5. C++渲染流程：DrawCell()

**文件位置:** `Source/engine/render/scrollrt.cpp` Line 521-643

**关键流程:**

```cpp
void DrawCell(const Surface &out, const Lightmap lightmap, 
              Point tilePosition, Point targetBufferPosition, int lightTableIndex)
{
    // 1. 获取levelPieceId
    const uint16_t levelPieceId = dPiece[tilePosition.x][tilePosition.y];  // ← 从dPiece查询
    const MICROS *pMap = &DPieceMicros[levelPieceId];  // ← 获取Piece数据
    
    // 2. 渲染地板（mt[0], mt[1]）
    if (const LevelCelBlock levelCelBlock { pMap->mt[0] }; levelCelBlock.hasValue()) {
        const TileType tileType = levelCelBlock.type();
        if (!isFloor || tileType == TileType::TransparentSquare) {
            RenderTile(out, bleedLightmap, targetBufferPosition,
                pDungeonCels.get(), levelCelBlock, maskType, tbl);  // ← 渲染左侧地板
        }
    }
    if (const LevelCelBlock levelCelBlock { pMap->mt[1] }; levelCelBlock.hasValue()) {
        // ... 渲染右侧地板
    }
    
    // 3. 渲染墙壁（mt[2] 到 mt[MicroTileLen-1]）
    targetBufferPosition.y -= TILE_HEIGHT;  // ← 向上移动
    for (uint_fast8_t i = 2, n = MicroTileLen; i < n; i += 2) {
        {
            const LevelCelBlock levelCelBlock { pMap->mt[i] };
            if (levelCelBlock.hasValue()) {
                RenderTile(out, bleedLightmap, targetBufferPosition,
                    pDungeonCels.get(), levelCelBlock, maskType, foliageTbl);  // ← 渲染左墙
            }
        }
        {
            const LevelCelBlock levelCelBlock { pMap->mt[i + 1] };
            if (levelCelBlock.hasValue()) {
                RenderTile(...);  // ← 渲染右墙
            }
        }
        targetBufferPosition.y -= TILE_HEIGHT;  // ← 继续向上
    }
}
```

**RenderTile关键逻辑:**
```cpp
// levelCelBlock.frame() 是1-based索引
uint16_t frameIndex = levelCelBlock.frame();  // ← 1-based
const uint8_t *frameData = GetDunFrame(pDungeonCels, frameIndex);  // ← 获取CEL帧数据
// ... 解码并渲染
```

---

## 第二部分：Rust地图加载与渲染流程分析

### 1. Rust纹理管理器加载：TileTextureManager::load_for_dungeon()

**文件位置:** `rust-diablo/src/tiles/texture_manager.rs` Line 88-172

```rust
pub fn load_for_dungeon(
    dungeon_type: DungeonType,
    mpq_manager: &mut MpqManager,
) -> Result<Self> {
    // 1. 加载CEL数据
    let cel_data = mpq_manager.find_file(&cel_path)?;
    let cel_sprite = DungeonCelSprite::from_bytes(&cel_data)?;
    
    // 2. 加载Palette
    let palette = Palette::from_mpq(mpq_manager, pal_path)?;
    
    // 3. 加载MIN数据
    let min_data = MinData::from_mpq(mpq_manager, min_path, dungeon_type)?;
    
    Ok(Self {
        cel_sprite,
        palette,
        min_data,
        decoded_cache: HashMap::new(),
        indexed_cache: HashMap::new(),
    })
}
```

### 2. Rust MIN数据加载：MinData::from_bytes()

**文件位置:** `rust-diablo/src/tiles/min.rs` Line 53-94

```rust
pub fn from_bytes(data: &[u8], dungeon_type: DungeonType) -> Result<Self> {
    let blocks_per_piece = match dungeon_type {
        DungeonType::Town => 16,
        DungeonType::Hell => 12,
        _ => 10,
    };
    
    let total_u16s = data.len() / 2;
    let num_pieces = total_u16s / blocks_per_piece;
    let mut pieces = Vec::with_capacity(num_pieces);
    
    // ⚠️ 第一步：按顺序读取MIN数据
    let mut offset = 0;
    for _ in 0..num_pieces {
        let mut mt = Vec::with_capacity(blocks_per_piece);
        for _ in 0..blocks_per_piece {
            let val = u16::from_le_bytes([data[offset], data[offset + 1]]);
            mt.push(LevelCelBlock::new(val));  // ← 存储原始数据
            offset += 2;
        }
        pieces.push(PieceMicros { mt });
    }
    
    // ⚠️ 第二步：重新排序（对应C++的SetDungeonMicros逻辑）
    for piece in &mut pieces {
        let raw_mt = piece.mt.clone();
        for block in 0..blocks_per_piece {
            // ⚠️ 关键公式：idx = blocks_per_piece - 2 + (block & 1) - (block & 0xE)
            let idx = blocks_per_piece as isize - 2 + (block as isize & 1) - (block as isize & 0xE);
            if idx >= 0 && (idx as usize) < raw_mt.len() {
                piece.mt[block] = raw_mt[idx as usize];
            }
        }
    }
    
    Ok(Self { pieces, blocks_per_piece })
}
```

### 3. Rust地图数据：DungeonMap

**文件位置:** `rust-diablo/src/world/dungeon_map.rs`

```rust
pub struct DungeonMap {
    /// MegaTile indices (40x40), 1-based (0 = empty)
    pub dungeon: [[u8; DMAXY]; DMAXX],
    
    /// Expanded levelPieceId for each MicroTile position (112x112)
    pub d_piece: [[u16; MAXDUNY]; MAXDUNX],
}

impl DungeonMap {
    /// Expand MegaTile map to MicroTile map
    pub fn expand_to_d_piece(&mut self, til_data: &TilData, default_tile: usize) {
        // Step 1: Fill with default tile
        if let Some(default_mega) = til_data.get(default_tile) {
            let v1 = default_mega.micro1;
            let v2 = default_mega.micro2;
            let v3 = default_mega.micro3;
            let v4 = default_mega.micro4;
            
            for j in (0..MAXDUNY).step_by(2) {
                for i in (0..MAXDUNX).step_by(2) {
                    self.d_piece[i][j] = v1;
                    self.d_piece[i + 1][j] = v2;
                    self.d_piece[i][j + 1] = v3;
                    self.d_piece[i + 1][j + 1] = v4;
                }
            }
        }
        
        // Step 2: Expand dungeon to dPiece
        let mut yy = 16usize;
        for j in 0..DMAXY {
            let mut xx = 16usize;
            for i in 0..DMAXX {
                let tile_id = self.dungeon[i][j];
                if tile_id > 0 {
                    let tile_idx = (tile_id - 1) as usize;
                    if let Some(mega) = til_data.get(tile_idx) {
                        self.d_piece[xx][yy] = mega.micro1;
                        self.d_piece[xx + 1][yy] = mega.micro2;
                        self.d_piece[xx][yy + 1] = mega.micro3;
                        self.d_piece[xx + 1][yy + 1] = mega.micro4;
                    }
                }
                xx += 2;
            }
            yy += 2;
        }
    }
}
```

### 4. Rust瓦片获取：TileTextureManager::get_decoded_tile()

**文件位置:** `rust-diablo/src/tiles/texture_manager.rs` Line 182-225

```rust
pub fn get_decoded_tile(&mut self, piece_index: usize, block_index: usize) -> Result<&[u8]> {
    // Create a unique cache key
    let cache_key = piece_index * 16 + block_index;
    
    // Check cache first
    if self.decoded_cache.contains_key(&cache_key) {
        return Ok(&self.decoded_cache[&cache_key]);
    }
    
    // Get PieceMicros from MIN data
    let piece = self.min_data
        .get(piece_index)  // ← ⚠️ 关键：从MIN数据获取Piece
        .ok_or_else(|| anyhow::anyhow!("Piece index out of range: {}", piece_index))?;
    
    // Get the specific block
    let block = piece.mt.get(block_index)  // ← ⚠️ 关键：获取Block
        .ok_or_else(|| anyhow::anyhow!("Block index out of range: {}", block_index))?;
    
    if !block.has_value() {
        // Empty tile - return transparent
        let size = 32 * 32 * 4;  // RGBA
        let transparent = vec![0u8; size];
        self.decoded_cache.insert(cache_key, transparent);
        return Ok(&self.decoded_cache[&cache_key]);
    }
    
    // Get TileType and frame index
    let tile_type = block.tile_type();
    let frame_idx = block.frame() as usize;  // ← ⚠️ 关键：frame是1-based
    
    // Get decoded indexed pixels
    let indexed_pixels = self.get_indexed_tile(frame_idx, tile_type)?;
    
    // Apply palette to convert to RGBA
    let rgba_pixels = self.palette.indices_to_rgba(&indexed_pixels, true);
    
    // Cache and return
    self.decoded_cache.insert(cache_key, rgba_pixels);
    Ok(&self.decoded_cache[&cache_key])
}
```

### 5. Rust渲染流程：World::render_with_texture_manager()

**文件位置:** `rust-diablo/src/world/mod.rs` Line 644-857

```rust
fn render_with_texture_manager(&self, engine: &mut Engine, _camera: &Camera) -> Result<()> {
    let texture_mgr_cell = self.texture_manager.as_ref().unwrap();
    let dungeon_map = self.dungeon_map.as_ref().unwrap();
    
    // Calculate view position
    let (view_x, view_y) = if let Some(player) = self.entities.first() {
        let px = (player.position.x / 32) as i32 + 16;
        let py = (player.position.y / 32) as i32 + 16;
        (px.clamp(16, MAXDUNX as i32 - 16), py.clamp(16, MAXDUNY as i32 - 16))
    } else {
        (56, 56)
    };
    
    // === Phase 1: Draw Floor ===
    for row in 0..tiles_y {
        // ... 遍历每个瓦片
        if dungeon_map.in_bounds(tx, ty) {
            let level_piece_id = dungeon_map.get_piece(tx, ty) as usize;  // ← 从dPiece获取
            if level_piece_id > 0 {
                self.draw_floor_at(engine, texture_mgr_cell, level_piece_id, sx, screen_pos_y)?;
            }
        }
    }
    
    // === Phase 2: Draw Walls/Content ===
    for row in 0..tiles_y {
        // ... 遍历每个瓦片
        if dungeon_map.in_bounds(tx, ty) {
            let level_piece_id = dungeon_map.get_piece(tx, ty) as usize;
            if level_piece_id > 0 {
                self.draw_cell_at(engine, texture_mgr_cell, level_piece_id, sx, screen_pos_y)?;
            }
        }
    }
}

fn draw_floor_at(&self, engine: &mut Engine, texture_mgr: &RefCell<TileTextureManager>,
                 level_piece_id: usize, screen_x: i32, screen_y: i32) -> Result<()> {
    // Draw mt[0] (left triangle) and mt[1] (right triangle)
    self.render_micro_tile(engine, texture_mgr, level_piece_id, 0, screen_x, screen_y)?;
    self.render_micro_tile(engine, texture_mgr, level_piece_id, 1, screen_x + 32, screen_y)?;
    Ok(())
}

fn draw_cell_at(&self, engine: &mut Engine, texture_mgr: &RefCell<TileTextureManager>,
                level_piece_id: usize, screen_x: i32, screen_y: i32) -> Result<()> {
    const TILE_HEIGHT: i32 = 32;
    let blocks_per_piece = texture_mgr.borrow().blocks_per_piece();
    
    // Draw blocks 2 and above (walls)
    let mut y = screen_y - TILE_HEIGHT;
    for i in (2..blocks_per_piece).step_by(2) {
        self.render_micro_tile(engine, texture_mgr, level_piece_id, i, screen_x, y)?;
        if i + 1 < blocks_per_piece {
            self.render_micro_tile(engine, texture_mgr, level_piece_id, i + 1, screen_x + 32, y)?;
        }
        y -= TILE_HEIGHT;
    }
    Ok(())
}

fn render_micro_tile(&self, engine: &mut Engine, texture_mgr: &RefCell<TileTextureManager>,
                     level_piece_id: usize, block_index: usize,
                     screen_x: i32, screen_y: i32) -> Result<()> {
    // Get decoded RGBA tile
    match texture_mgr.borrow_mut().get_decoded_tile(level_piece_id, block_index) {
        Ok(rgba_pixels) => {
            let width = 32u32;
            let height = (rgba_pixels.len() / 4 / 32) as u32;
            
            let texture_id = format!("tile_{}_{}", level_piece_id, block_index);
            let rect = Rect::new(screen_x, screen_y, width, height);
            engine.draw_rgba_texture(&texture_id, rgba_pixels, width, height, rect)?;
        }
        Err(_) => {
            // Empty or invalid block - skip
        }
    }
    Ok(())
}
```

---

## 第三部分：关键差异对比与问题分析

### ⚠️ 潜在问题1：Frame索引（0-based vs 1-based）

**C++ (正确):**
```cpp
// LevelCelBlock.frame() 返回 1-based 索引
uint16_t frame() const { return data & 0xFFF; }  // 1-based

// GetDunFrame() 期望 1-based 索引
const uint8_t *GetDunFrame(const uint8_t *dungeonCels, uint16_t frameIndex);
```

**Rust (可能有问题):**
```rust
// LevelCelBlock.frame() 返回 1-based 索引
pub fn frame(&self) -> u16 {
    self.data & 0xFFF  // 1-based
}

// 但get_indexed_tile使用frame作为数组索引（0-based）
fn get_indexed_tile(&mut self, frame_idx: usize, tile_type: TileType) -> Result<&[u8]> {
    if frame_idx < main_cel_len {
        // ⚠️ 问题：frame_idx应该是1-based，但这里当作0-based使用
        (&self.cel_sprite.frames[frame_idx], "main")  // ← 数组访问是0-based
    }
    // ...
}
```

**问题分析:**
- C++的`frame()`返回1-based索引，但`GetDunFrame()`内部会处理转换
- Rust的`frame()`也返回1-based索引，但`get_indexed_tile()`直接用作数组索引（0-based）
- **这会导致所有帧都偏移1，frame=1会访问frames[1]而不是frames[0]**

**修复方案:**
```rust
fn get_indexed_tile(&mut self, frame_idx: usize, tile_type: TileType) -> Result<&[u8]> {
    // ⚠️ 修复：frame_idx是1-based，需要转换为0-based
    if frame_idx == 0 {
        bail!("Invalid frame index: 0 (frame indices are 1-based)");
    }
    let array_idx = frame_idx - 1;  // ← 转换为0-based
    
    if array_idx < main_cel_len {
        (&self.cel_sprite.frames[array_idx], "main")
    } else if let Some(ref special_cel) = self.special_cel_sprite {
        let adjusted_idx = array_idx - main_cel_len;
        if adjusted_idx >= special_cel.frames.len() {
            bail!("Frame index out of range: {}", frame_idx);
        }
        (&special_cel.frames[adjusted_idx], "special")
    } else {
        bail!("Frame index out of range: {}", frame_idx);
    }
}
```

### ⚠️ 潜在问题2：地图数据加载不完整

**检查清单:**

1. **DUN文件加载:**
   - ✅ C++使用`PlaceDunTiles()`填充`dungeon[][]`
   - ❓ Rust是否正确填充`DungeonMap.dungeon`？

2. **TIL数据扩展:**
   - ✅ C++使用`FillSector()`或`DRLG_LPass3()`扩展到`dPiece`
   - ❓ Rust是否调用`DungeonMap::expand_to_d_piece()`？

3. **MIN数据索引:**
   - ✅ C++的`dPiece`值是levelPieceId（MIN中的piece索引）
   - ❓ Rust的`d_piece`值是否正确？

**检查Rust是否缺少地图初始化步骤:**
```rust
// 需要确认是否执行了：
let mut dungeon_map = DungeonMap::new();

// 1. 加载DUN文件到dungeon[40][40]
// (需要实现PlaceDunTiles等价函数)

// 2. 扩展到d_piece[112][112]
dungeon_map.expand_to_d_piece(&til_data, default_tile);
```

### ⚠️ 潜在问题3：levelPieceId的含义混淆

**C++数据流:**
```
dungeon[i][j] = MegaTile ID (1-based, 来自DUN文件)
   ↓ (通过TIL查找)
pMegaTiles[tileId-1] = { micro1, micro2, micro3, micro4 }  (4个levelPieceId)
   ↓
dPiece[x][y] = levelPieceId (MIN中的piece索引)
   ↓
DPieceMicros[levelPieceId].mt[block] = LevelCelBlock
   ↓
LevelCelBlock.frame() = CEL帧索引 (1-based)
```

**Rust需要确认:**
- `d_piece[x][y]`存储的是什么？
  - 应该是levelPieceId（MIN中的piece索引）
  - 不是MegaTile ID
- `get_decoded_tile(piece_index, block_index)`中：
  - `piece_index`应该是levelPieceId
  - `block_index`是0-15之间的块索引

### ⚠️ 潜在问题4：坐标系统

**C++坐标系:**
```
dungeon[DMAXX][DMAXY] = dungeon[x][y]  (列优先？)
dPiece[MAXDUNX][MAXDUNY] = dPiece[x][y]  (列优先？)
```

**Rust坐标系:**
```rust
pub dungeon: [[u8; DMAXY]; DMAXX],  // dungeon[x][y]
pub d_piece: [[u16; MAXDUNY]; MAXDUNX],  // d_piece[x][y]
```

**问题:** 需要确认C++和Rust的数组索引顺序是否一致。

---

## 诊断步骤建议

### 步骤1：添加详细日志

在Rust代码中添加日志，输出关键数据：

```rust
// 在get_decoded_tile中：
println!("get_decoded_tile: piece_index={}, block_index={}", piece_index, block_index);
println!("  piece.mt.len()={}", piece.mt.len());
println!("  block.data={:#06x}, has_value={}", block.data, block.has_value());
println!("  tile_type={:?}, frame={} (1-based)", block.tile_type(), block.frame());
println!("  frame_idx used for array access={}", frame_idx);

// 在render_with_texture_manager中：
println!("Rendering at dPiece({}, {}), level_piece_id={}", tx, ty, level_piece_id);
```

### 步骤2：验证地图数据

```rust
// 打印dungeon和d_piece的内容
println!("\n=== Dungeon Map (first 10x10) ===");
for j in 0..10 {
    for i in 0..10 {
        print!("{:3} ", dungeon_map.dungeon[i][j]);
    }
    println!();
}

println!("\n=== dPiece Map (16-26, 16-26) ===");
for j in 16..26 {
    for i in 16..26 {
        print!("{:4} ", dungeon_map.d_piece[i][j]);
    }
    println!();
}
```

### 步骤3：验证MIN数据

```rust
// 打印MIN数据的前几个piece
println!("\n=== MIN Data (first 5 pieces) ===");
for piece_idx in 0..5.min(min_data.len()) {
    println!("Piece {}: ", piece_idx);
    if let Some(piece) = min_data.get(piece_idx) {
        for (block_idx, block) in piece.mt.iter().enumerate() {
            println!("  Block {}: data={:#06x}, type={:?}, frame={}",
                block_idx, block.data, block.tile_type(), block.frame());
        }
    }
}
```

### 步骤4：对比C++输出

在C++代码中添加类似的日志，对比输出结果。

### 步骤5：单独测试第一个瓦片

```rust
// 强制渲染特定的tile
let test_piece_id = 1;
for block_idx in 0..16 {
    match texture_mgr.borrow_mut().get_decoded_tile(test_piece_id, block_idx) {
        Ok(rgba) => {
            println!("Tile {}/{}: {} bytes", test_piece_id, block_idx, rgba.len());
            // 检查是否全透明
            let non_transparent = rgba.chunks(4).filter(|p| p[3] != 0).count();
            println!("  Non-transparent pixels: {}", non_transparent);
        }
        Err(e) => {
            println!("Tile {}/{}: Error - {}", test_piece_id, block_idx, e);
        }
    }
}
```

---

## 总结

**最可能的问题:**

1. **Frame索引偏移1**（1-based vs 0-based）
2. **地图数据未正确初始化**（缺少dungeon→dPiece的扩展步骤）
3. **levelPieceId含义混淆**（MegaTile ID vs MIN piece索引）

**立即检查:**

1. 确认`get_indexed_tile()`中frame_idx是否正确转换（1-based → 0-based）
2. 确认`dungeon_map`是否正确加载DUN文件并扩展到`d_piece`
3. 添加日志输出，对比C++和Rust的中间数据

**下一步:**

根据日志输出，定位具体的数据差异，然后逐个修复。














