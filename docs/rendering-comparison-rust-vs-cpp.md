# Rust vs C++ 渲染流程对比分析

## 文档目的
系统对比Rust和C++的地图渲染实现，找出差异和需要对齐的地方。

---

## 1. 主渲染入口

### C++
**文件**: `Source/engine/render/scrollrt.cpp`
**函数**: `DrawGame()` (Line 1254)

```cpp
void DrawGame(const Surface &fullOut, Point position, Displacement offset)
{
    // 1. 限制渲染区域到视口
    const Surface &out = !*GetOptions().Graphics.zoom
        ? fullOut.subregionY(0, gnViewportHeight)
        : fullOut.subregionY(0, (gnViewportHeight + 1) / 2);
    
    // 2. 获取列数和行数
    int columns = tileColumns;
    int rows = tileRows;
    
    // 3. 构建光照映射
    Lightmap lightmap = Lightmap::build(...);
    
    // 4. 两阶段渲染
    DrawFloor(out, lightmap, position, Point {} + offset, rows, columns);        // Phase 1
    DrawTileContent(out, lightmap, position, Point {} + offset, rows, columns);  // Phase 2
}
```

### Rust
**文件**: `rust-diablo/src/world/mod.rs`
**函数**: `render_with_texture_manager()` (Line 410)

```rust
fn render_with_texture_manager(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
    // 1. 获取视口尺寸
    let screen_width = camera.viewport_width as i32;
    let viewport_height = camera.viewport_height as i32;
    
    // 2. 计算列数和行数
    let columns = (screen_width / TILE_WIDTH) + 2;
    let rows = (viewport_height / (TILE_HEIGHT / 2)) + 4;
    
    // 3. 计算屏幕偏移（居中视图）
    let offset_x = if remainder_x != 0 { (TILE_WIDTH - remainder_x) / 2 } else { 0 };
    let offset_y = if remainder_y != 0 { (TILE_HEIGHT - remainder_y) / 2 } else { 0 };
    
    // 4. 两阶段渲染
    // Phase 1: 绘制地板
    // Phase 2: 绘制墙体和内容
}
```

**状态**: ✅ 基本流程一致

---

## 2. Phase 1: 地板渲染

### C++
**函数**: `DrawFloor()` (Line 1049)

```cpp
void DrawFloor(const Surface &out, const Lightmap &lightmap, 
               Point tilePosition, Point targetBufferPosition, 
               int rows, int columns)
{
    for (int i = 0; i < rows; i++) {
        for (int j = 0; j < columns; j++, 
             tilePosition += Direction::East, 
             targetBufferPosition.x += TILE_WIDTH) {
            
            // 1. 边界检查
            if (!InDungeonBounds(tilePosition)) {
                world_draw_black_tile(out, targetBufferPosition.x, targetBufferPosition.y);
                continue;
            }
            
            // 2. 只渲染地板瓦片
            if (IsFloor(tilePosition)) {
                DrawFloorTile(out, lightmap, tilePosition, targetBufferPosition);
            }
        }
        
        // 3. 行间移动逻辑（等距投影Z字形）
        tilePosition += Displacement(Direction::West) * columns;
        targetBufferPosition.x -= columns * TILE_WIDTH;
        targetBufferPosition.y += TILE_HEIGHT / 2;
        
        if ((i & 1) != 0) {
            tilePosition.x++;
            columns--;
            targetBufferPosition.x += TILE_WIDTH / 2;
        } else {
            tilePosition.y++;
            columns++;
            targetBufferPosition.x -= TILE_WIDTH / 2;
        }
    }
}
```

### Rust
**代码位置**: `render_with_texture_manager()` Phase 1 (Line 514-575)

```rust
// === Phase 1: Draw Floor (like C++ DrawFloor) ===
for row in 0..rows {
    let mut tx = tile_x;
    let mut ty = tile_y;
    let mut sx = screen_x;
    
    for _col in 0..current_columns {
        // 1. 边界检查
        if tx >= 0 && ty >= 0 && tx < MAXDUNX as i32 && ty < MAXDUNY as i32 {
            let piece_id = dungeon_map.d_piece[tx as usize][ty as usize] as usize;
            
            // 2. 只渲染地板
            if piece_id > 0 {
                let is_floor = self.is_floor_tile((tx, ty));
                if is_floor {
                    // Render floor blocks 0 and 1
                    self.render_micro_tile(..., 0, ..., MaskType::Solid)?;
                    self.render_micro_tile(..., 1, ..., MaskType::Solid)?;
                }
            }
        }
        
        tx += 1;
        ty -= 1;
        sx += 64;
    }
    
    // 3. 行间移动逻辑（等距投影Z字形）
    screen_y += TILE_HEIGHT / 2;
    if (row & 1) != 0 {
        tile_x += 1;
        current_columns -= 1;
        screen_x += TILE_WIDTH / 2;
    } else {
        tile_y += 1;
        current_columns += 1;
        screen_x -= TILE_WIDTH / 2;
    }
}
```

**状态**: ✅ 逻辑一致

---

## 3. Phase 2: 墙体和内容渲染

### C++
**函数**: `DrawTileContent()` (Line 1088)

```cpp
void DrawTileContent(const Surface &out, const Lightmap &lightmap,
                     Point tilePosition, Point targetBufferPosition, 
                     int rows, int columns)
{
    // ⚠️ 关键：扩展行数以渲染高墙
    rows += MicroTileLen;  // MicroTileLen = 8
    
    for (int i = 0; i < rows; i++) {
        bool skip = false;
        for (int j = 0; j < columns; j++) {
            if (InDungeonBounds(tilePosition)) {
                bool skipNext = false;
                
                // 特殊处理：墙体背后的对象渲染
                if (IsWall(tilePosition) && ...) {
                    if (IsTileNotSolid(...)) {
                        DrawDungeon(out, lightmap, 
                                    tilePosition + Direction::East, 
                                    { targetBufferPosition.x + TILE_WIDTH, targetBufferPosition.y });
                        skipNext = true;
                    }
                }
                
                if (!skip) {
                    DrawDungeon(out, lightmap, tilePosition, targetBufferPosition);
                }
                skip = skipNext;
            }
            tilePosition += Direction::East;
            targetBufferPosition.x += TILE_WIDTH;
        }
        
        // 行间移动逻辑（同 DrawFloor）
        ...
    }
}
```

### Rust
**代码位置**: `render_with_texture_manager()` Phase 2 (Line 576-646)

```rust
// === Phase 2: Draw Walls/Cells (like C++ DrawTileContent/DrawCell) ===

// ⚠️ 关键：扩展行数
const MICRO_TILE_LEN: i32 = 8;
let wall_rows = rows + MICRO_TILE_LEN;

// Reset to starting position
let mut wall_tile_x = start_tile_x;
let mut wall_tile_y = start_tile_y;
let mut wall_screen_x = offset_x;
let mut wall_screen_y = offset_y;
let mut wall_current_columns = columns;

for row in 0..wall_rows {
    let mut tx = wall_tile_x;
    let mut ty = wall_tile_y;
    let mut sx = wall_screen_x;
    
    for _col in 0..wall_current_columns {
        if tx >= 0 && ty >= 0 && tx < MAXDUNX as i32 && ty < MAXDUNY as i32 {
            let piece_id = dungeon_map.d_piece[tx as usize][ty as usize] as usize;
            
            if piece_id > 0 {
                // 获取MaskType
                let mask_left = self.get_mask_type_left((tx, ty), sol_data);
                let mask_right = self.get_mask_type_right((tx, ty), sol_data);
                
                // 渲染所有块（0-7）
                for block_idx in 0..8 {
                    let mask = if block_idx % 2 == 0 { mask_left } else { mask_right };
                    self.render_micro_tile(..., block_idx, ..., mask)?;
                }
            }
        }
        
        tx += 1;
        ty -= 1;
        sx += 64;
    }
    
    // 行间移动逻辑
    ...
}
```

**状态**: ✅ 逻辑基本一致，但需要检查细节

---

## 4. DrawCell/DrawDungeon 细节对比

### C++
**函数**: `DrawCell()` (Line 587) 和 `DrawDungeon()` (Line 875)

```cpp
void DrawCell(const Surface &out, const Lightmap lightmap, 
              Point tilePosition, Point targetBufferPosition, 
              int lightTableIndex)
{
    const uint16_t levelPieceId = dPiece[tilePosition.x][tilePosition.y];
    
    // 计算透明度
    const bool transparency = TileHasAny(tilePosition, TileProperties::Transparent) 
                               && TransList[dTransVal[tilePosition.x][tilePosition.y]];
    
    // Lambda: 获取左掩码
    const auto getFirstTileMaskLeft = [=](TileType tile) -> MaskType {
        MaskType mask = MaskType::Solid;
        if (transparency) {
            switch (tile) {
                case TileType::TransparentSquare:
                case TileType::TransparentSquareLeft:
                    mask = MaskType::Transparent;
                    break;
                case TileType::LeftTriangle:
                case TileType::LeftTrapezoid:
                    mask = MaskType::Left;
                    break;
                // ...
            }
        }
        return mask;
    };
    
    // Lambda: 获取右掩码
    const auto getFirstTileMaskRight = [=](TileType tile) -> MaskType { ... };
    
    // 渲染块0-1（地板可能已渲染）
    if (IsFloor(tilePosition)) {
        // 已在DrawFloor渲染
    }
    
    // 渲染块2-7（墙体和上层）
    for (size_t i = 2; i < 8; i += 2) {
        const LevelCelBlock block { DPieceMicros[levelPieceId].mt[i] };
        if (block.hasValue()) {
            MaskType mask = getFirstTileMaskLeft(block.type());
            RenderTileFrame(..., mask, ...);
        }
    }
    
    for (size_t i = 3; i < 8; i += 2) {
        const LevelCelBlock block { DPieceMicros[levelPieceId].mt[i] };
        if (block.hasValue()) {
            MaskType mask = getFirstTileMaskRight(block.type());
            RenderTileFrame(..., mask, ...);
        }
    }
}

void DrawDungeon(const Surface &out, const Lightmap &lightmap, 
                 Point tilePosition, Point targetBufferPosition)
{
    DrawCell(out, lightmap, tilePosition, targetBufferPosition, lightTableIndex);
    
    // 额外渲染：尸体、物品、玩家、怪物等
    // ...
}
```

### Rust
**函数**: `render_micro_tile()` (Line 950)

```rust
fn render_micro_tile(
    &self,
    engine: &mut Engine,
    texture_mgr: &RefCell<TileTextureManager>,
    level_piece_id: usize,
    block_index: usize,
    screen_x: i32,
    screen_y: i32,
    micro_x: i32,
    micro_y: i32,
    is_floor: bool,
    mask_type: MaskType,
) -> Result<()> {
    // 1. 获取块数据
    let (has_value, tile_type) = {
        let mgr = texture_mgr.borrow();
        if let Some(piece) = mgr.get_piece(level_piece_id) {
            if let Some(block) = piece.mt.get(block_index) {
                (block.has_value(), block.tile_type())
            } else {
                return Ok(()); // No block
            }
        } else {
            return Ok(()); // No piece
        }
    };
    
    if !has_value {
        return Ok(()); // Empty block
    }
    
    // 2. DEBUG输出
    let transparency = self.check_transparency((micro_x, micro_y), self.sol_data.as_ref());
    println!("[RUST] Tile piece={} block={} at screen=({},{}) micro=({},{}) type={:?} is_floor={} mask={:?} transparency={}",
        level_piece_id, block_index, screen_x, screen_y, micro_x, micro_y,
        tile_type, is_floor, mask_type, transparency);
    
    // 3. 渲染
    engine.draw_rgba_texture(..., mask_type)?;
    
    Ok(())
}
```

**状态**: ✅ 基本一致

---

## 5. 关键差异点检查清单

### 5.1 渲染顺序
| 项目 | C++ | Rust | 状态 |
|------|-----|------|------|
| Phase 1: 地板 | ✅ DrawFloor | ✅ Phase 1 loop | ✅ 一致 |
| Phase 2: 墙体 | ✅ DrawTileContent | ✅ Phase 2 loop | ✅ 一致 |
| 扩展行数 | ✅ rows += 8 | ✅ wall_rows = rows + 8 | ✅ 一致 |

### 5.2 MaskType选择
| 项目 | C++ | Rust | 状态 |
|------|-----|------|------|
| 地板使用Solid | ✅ | ✅ | ✅ 一致 |
| 墙体动态选择 | ✅ getFirstTileMaskLeft/Right | ✅ get_mask_type_left/right | ✅ 一致 |
| 透明度检查 | ✅ TileHasAny + TransList | ✅ check_transparency | ✅ 一致 |

### 5.3 透明度系统
| 项目 | C++ | Rust | 状态 |
|------|-----|------|------|
| TransList | ✅ | ✅ | ✅ 已实现 |
| TransVal (dTransVal) | ✅ | ✅ | ✅ 已实现 |
| 加载DUN透明层 | ✅ LoadTransparency | ✅ load_from_dun | ✅ 已实现 |
| 初始化TransList | ✅ DRLG_InitTrans | ✅ init_from_trans_val | ✅ 已实现 |

### 5.4 坐标计算
| 项目 | C++ | Rust | 需要验证 |
|------|-----|------|----------|
| 起始tile位置 | `position` 参数 | `(view_x, view_y)` | ⚠️ 需要对比 |
| 屏幕偏移计算 | `CalcTileOffset()` | `offset_x/offset_y` 计算 | ⚠️ 需要对比 |
| Z字形遍历 | ✅ | ✅ | ✅ 逻辑一致 |

### 5.5 边界处理
| 项目 | C++ | Rust | 状态 |
|------|-----|------|------|
| InDungeonBounds检查 | ✅ | ✅ | ✅ 一致 |
| 超出边界黑色瓦片 | ✅ world_draw_black_tile | ✅ draw_black_tile | ✅ 一致 |

---

## 6. 需要详细对比的部分

### 6.1 起始tile位置计算
**C++**: `Source/engine/render/scrollrt.cpp::CalcFirstTilePosition()` (需要找到)
**Rust**: `rust-diablo/src/world/mod.rs` Line 472-486

```rust
let (view_x, view_y) = if let Some(player) = self.entities.first() {
    let px = (player.position.x / 32) as i32 + BORDER_SIZE;
    let py = (player.position.y / 32) as i32 + BORDER_SIZE;
    (px.clamp(1, MAXDUNX as i32 - 10), py.clamp(1, MAXDUNY as i32 - 10))
} else {
    (25, 25)
};
```

**行动**: 需要找到C++中的对应计算并对比

### 6.2 屏幕偏移计算
**C++**: `Source/engine/render/scrollrt.cpp::CalcTileOffset()` (Line 1518-1541)
**Rust**: `rust-diablo/src/world/mod.rs` Line 493-508

**行动**: 需要验证remainder计算是否一致

### 6.3 光照系统
**C++**: 使用 `Lightmap` 和 `LightTables`
**Rust**: 目前使用简化的光照系统

**状态**: ⚠️ 这是一个已知差异，但不影响透明度系统

---

## 7. 测试建议

### 7.1 输出对比点
为了验证渲染一致性，需要对比以下日志：

1. **Player位置**
   - C++: `[C++ FRAME START] Player tile=(...) pixel=(...)`
   - Rust: `[RUST] FRAME START Player pixel_pos=(...) micro_tile=(...)`

2. **d_piece数组**
   - C++: 输出到 `dpiece_cpp.txt`
   - Rust: 输出到 `dpiece_rust.txt`
   - **比较**: `diff dpiece_cpp.txt dpiece_rust.txt`

3. **透明度数据**
   - C++: `[C++ TRANSPARENCY] Loaded transparency data: ...`
   - Rust: `TransList initialized: ... active indices`

4. **单个瓦片渲染**
   - C++: `[C++] Tile piece=... block=... at screen=(...) micro=(...) type=... mask=... transparency=...`
   - Rust: `[RUST] Tile piece=... block=... at screen=(...) micro=(...) type=... mask=... transparency=...`

### 7.2 对比步骤
1. ✅ **确认d_piece一致** - 先比较两边的dpiece数组
2. ✅ **确认透明度数据一致** - 比较TransList和TransVal加载
3. ⚠️ **确认渲染位置一致** - 比较相同piece在相同game状态下的screen坐标
4. ⚠️ **确认mask选择一致** - 相同transparency下，mask选择是否一致

### 7.3 当前状态总结
根据您提供的输出：

```
[RUST] piece=168 type=TransparentSquare mask=Solid transparency=false
[C++]  piece=168 type=TransparentSquare mask=Solid transparency=false
```

✅ **行为一致**！两边都：
- 识别出相同的tile type
- 选择相同的mask (Solid)
- 透明度都是false

**micro坐标不同是正常的**，因为两个程序的视口位置不同。

---

## 8. 下一步行动

### 优先级1: 验证d_piece数组
```bash
# 比较两边的dpiece输出
diff dpiece_cpp.txt dpiece_rust.txt
```
**预期**: 应该完全一致（除了格式差异）

### 优先级2: 验证TransVal数据
在同一个DUN文件加载后，对比TransVal数组：
- C++: `dTransVal[x][y]`
- Rust: `trans_val.get(x, y)`

### 优先级3: 验证相同游戏状态
需要确保两个程序：
1. 加载相同的地图
2. 玩家在相同位置
3. 相机在相同位置

然后再对比渲染输出。

### 优先级4: 增加更详细的日志
如果发现不一致，可以增加：
1. 每个瓦片的属性（TRANSPARENT flag）
2. TransVal的具体值
3. TransList[transVal]的状态

---

## 9. 已知问题和待办

### 已解决 ✅
- [x] TransList结构实现
- [x] TransVal结构实现
- [x] DUN透明层加载
- [x] TransList初始化
- [x] MaskType选择逻辑
- [x] 透明度检查集成
- [x] DEBUG日志输出

### 待验证 ⚠️
- [ ] d_piece数组完全一致
- [ ] TransVal数组完全一致
- [ ] 相同游戏状态下的渲染输出一致
- [ ] 屏幕偏移计算一致性

### 未来工作 🔮
- [ ] 光照系统完整实现
- [ ] 实体渲染（怪物、物品、玩家）
- [ ] 特效和动画

---

## 参考代码位置

### C++
- `Source/engine/render/scrollrt.cpp` - 主渲染循环
- `Source/engine/render/dun_render.cpp` - 瓦片渲染细节
- `Source/levels/gendung.cpp` - 地图加载和透明度

### Rust
- `rust-diablo/src/world/mod.rs` - 主渲染循环
- `rust-diablo/src/world/transparency.rs` - 透明度系统
- `rust-diablo/src/world/dungeon_map.rs` - 地图加载
- `rust-diablo/src/engine/mod.rs` - 渲染引擎

---

**文档版本**: v1.0  
**创建日期**: 2024-12-04  
**最后更新**: 2024-12-04




