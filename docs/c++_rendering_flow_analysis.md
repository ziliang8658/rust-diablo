# C++ 渲染流程完整分析

## 概述

本文档详细分析了 DevilutionX C++ 代码库的完整渲染流程，从游戏主循环到最终屏幕呈现的每一个步骤。

## 1. 游戏主循环 (diablo.cpp)

### 1.1 主循环入口

**文件**: `Source/diablo.cpp`  
**函数**: `RunGameLoop(interface_mode uMsg)`

主游戏循环位于 `RunGameLoop` 函数中，这是整个渲染流程的起点：

```cpp
while (gbRunGame) {
    // 1. 处理事件
    while (FetchMessage(&event, &modState)) {
        HandleMessage(event, modState);
    }
    
    // 2. 决定是否运行游戏逻辑
    const bool runGameLoop = nthread_has_500ms_passed(&drawGame);
    
    if (!runGameLoop) {
        if (processInput)
            ProcessInput();
        if (!drawGame)
            continue;
        RedrawViewport();
        DrawAndBlit();  // ← 渲染入口
        continue;
    }
    
    // 3. 运行游戏逻辑
    multi_process_network_packets();
    if (game_loop(gbGameLoopStartup))
        diablo_color_cyc_logic();
    
    // 4. 渲染
    if (drawGame)
        DrawAndBlit();  // ← 主要渲染调用
}
```

### 1.2 渲染触发条件

- **完整重绘**: `IsRedrawEverything()` 为真时，重绘整个屏幕
- **视口重绘**: `IsRedrawViewport()` 为真时，只重绘视口区域
- **部分重绘**: 根据 `PanelDrawComponent` 标志决定重绘哪些UI组件

## 2. 渲染入口函数 (scrollrt.cpp)

### 2.1 DrawAndBlit()

**文件**: `Source/engine/render/scrollrt.cpp`  
**行号**: 1993-2083

这是主要的渲染协调函数，负责：

1. **确定重绘范围**
   ```cpp
   if (IsRedrawEverything() || ...) {
       hgt = gnScreenHeight;  // 全屏
       drawHealth = true;
       drawMana = true;
       // ... 所有组件
   } else if (IsRedrawViewport()) {
       hgt = gnViewportHeight;  // 仅视口
   }
   ```

2. **渲染顺序**:
   ```cpp
   const Surface &out = GlobalBackBuffer();
   UndrawCursor(out);              // 1. 擦除旧光标
   DrawView(out, ViewPosition);    // 2. 绘制游戏世界
   DrawMainPanel(out);             // 3. 绘制主面板
   DrawLifeFlaskLower(out);        // 4. 绘制生命值
   DrawManaFlaskLower(out);        // 5. 绘制魔法值
   DrawMainPanelButtons(out);      // 6. 绘制控制按钮
   DrawInvBelt(out);               // 7. 绘制腰带
   DrawCursor(out);                // 8. 绘制新光标
   DrawFPS(out);                   // 9. 绘制FPS
   DrawMain(hgt, ...);            // 10. 绘制主UI
   RenderPresent();                // 11. 呈现到屏幕
   ```

### 2.2 DrawView()

**文件**: `Source/engine/render/scrollrt.cpp`  
**行号**: 1476-1554

负责绘制游戏世界视图：

```cpp
void DrawView(const Surface &out, Point startPosition) {
    Displacement offset = {};
    CalcFirstTilePosition(startPosition, offset);
    DrawGame(out, startPosition, offset);  // ← 核心渲染函数
    
    if (AutomapActive) {
        DrawAutomap(out.subregionY(0, gnViewportHeight));
    }
    
    // 调试网格
    // ...
    
    // UI叠加层
    DrawItemNameLabels(out);
    DrawMonsterHealthBar(out);
    DrawFloatingNumbers(out, startPosition, offset);
    // ...
}
```

## 3. 核心渲染函数 DrawGame()

**文件**: `Source/engine/render/scrollrt.cpp`  
**行号**: 1371-1469

这是整个渲染系统的核心，负责绘制游戏世界：

### 3.1 初始化阶段

```cpp
void DrawGame(const Surface &fullOut, Point position, Displacement offset) {
    // 1. 限制渲染区域到视口
    const Surface &out = !*GetOptions().Graphics.zoom
        ? fullOut.subregionY(0, gnViewportHeight)
        : fullOut.subregionY(0, (gnViewportHeight + 1) / 2);
    
    // 2. 计算需要渲染的瓦片行列数
    int columns = tileColumns;
    int rows = tileRows;
    
    // 3. 处理玩家行走时的额外区域
    if (MyPlayer->isWalking()) {
        // 根据行走方向扩展渲染区域
        switch (MyPlayer->_pdir) {
            case Direction::North:
            case Direction::South:
                rows += 2;
                break;
            // ...
        }
    }
    
    // 4. 构建光照贴图
    Lightmap lightmap = Lightmap::build(
        *GetOptions().Graphics.perPixelLighting,
        position, Point {} + offset,
        gnScreenWidth, gnViewportHeight, rows, columns,
        out.at(0, 0), out.pitch(),
        LightTables, FullyLitLightTable, FullyDarkLightTable,
        dLight, MicroTileLen
    );
```

### 3.2 两阶段渲染

渲染分为两个主要阶段：

#### 阶段1: 地板层 (DrawFloor)

```cpp
// 渲染调试: 地板层 (Phase 1)
if (g_RenderDebug.renderFloor) {
    DrawFloor(out, lightmap, position, Point {} + offset, rows, columns);
}
```

**功能**: 只渲染地板瓦片（`is_floor=true` 的瓦片）

**实现**: `DrawFloor()` 函数遍历所有瓦片，检查 `IsFloor(tilePosition)`，如果是地板则调用 `DrawFloorTile()` 渲染。

#### 阶段2: 墙壁/上层内容 (DrawTileContent)

```cpp
// 渲染调试: 墙壁/上层 (Phase 2)
if (g_RenderDebug.renderWalls) {
    DrawTileContent(out, lightmap, position, Point {} + offset, rows, columns);
}
```

**功能**: 渲染所有瓦片的内容，包括：
- 墙壁
- 对象
- 物品
- 怪物
- 玩家
- 投射物
- 尸体

**关键特性**: 
- 透明区域会显示下层内容（地板或前一帧内容）
- 从不调用 `clear()`，依赖瓦片覆盖整个屏幕

## 4. 瓦片渲染详细流程

### 4.1 DrawFloor() - 地板渲染

**文件**: `Source/engine/render/scrollrt.cpp`  
**行号**: 1141-1194

```cpp
void DrawFloor(const Surface &out, const Lightmap &lightmap, 
               Point tilePosition, Point targetBufferPosition, 
               int rows, int columns) {
    for (int i = 0; i < rows; i++) {
        for (int j = 0; j < columns; j++) {
            if (!InDungeonBounds(tilePosition)) {
                world_draw_black_tile(out, ...);
                continue;
            }
            
            const bool isFloor = IsFloor(tilePosition);
            if (isFloor) {
                DrawFloorTile(out, lightmap, tilePosition, targetBufferPosition);
            }
            // 非地板瓦片跳过（不绘制）
            
            tilePosition += Direction::East;
            targetBufferPosition.x += TILE_WIDTH;
        }
        // 移动到下一行（菱形网格的特殊处理）
    }
}
```

**关键点**:
- 只渲染 `is_floor=true` 的瓦片
- 非地板瓦片被跳过，保持前一帧内容或透明
- 使用菱形网格遍历（奇偶行偏移）

### 4.2 DrawTileContent() - 内容渲染

**文件**: `Source/engine/render/scrollrt.cpp`  
**行号**: 1205-1258

```cpp
void DrawTileContent(const Surface &out, const Lightmap &lightmap,
                     Point tilePosition, Point targetBufferPosition,
                     int rows, int columns) {
    rows += MicroTileLen;  // 扩展行数以处理微瓦片
    
    for (int i = 0; i < rows; i++) {
        for (int j = 0; j < columns; j++) {
            if (InDungeonBounds(tilePosition)) {
                // 特殊处理：墙壁后的对象
                if (IsWall(tilePosition) && ...) {
                    // 先渲染墙壁后的对象
                    DrawDungeon(out, lightmap, 
                               tilePosition + Direction::East, ...);
                }
                
                // 渲染当前瓦片的所有内容
                DrawDungeon(out, lightmap, tilePosition, targetBufferPosition);
            }
            // 移动到下一个瓦片
        }
        // 移动到下一行
    }
}
```

### 4.3 DrawDungeon() - 单个瓦片内容渲染

**文件**: `Source/engine/render/scrollrt.cpp`  
**行号**: 963-1363

这是渲染单个瓦片所有内容的函数，渲染顺序如下：

```cpp
void DrawDungeon(const Surface &out, const Lightmap &lightmap,
                 Point tilePosition, Point targetBufferPosition) {
    const int lightTableIndex = dLight[tilePosition.x][tilePosition.y];
    
    // 1. 绘制瓦片单元格（墙壁、地板装饰等）
    DrawCell(out, lightmap, tilePosition, targetBufferPosition, lightTableIndex);
    
    // 2. 绘制投射物（PreFlag）
    if (MissilePreFlag) {
        DrawMissile(out, tilePosition, targetBufferPosition, true, lightTableIndex);
    }
    
    // 3. 绘制尸体
    if (lightTableIndex < LightsMax && bDead != 0) {
        // 绘制尸体精灵
    }
    
    // 4. 绘制对象（PreFlag）
    if (object != nullptr && object->_oPreFlag) {
        DrawObject(out, *object, tilePosition, targetBufferPosition, lightTableIndex);
    }
    
    // 5. 绘制物品（非PostDraw）
    if (bItem > 0 && !Items[bItem - 1]._iPostDraw) {
        DrawItem(out, bItem - 1, targetBufferPosition, lightTableIndex);
    }
    
    // 6. 绘制死去的玩家
    if (TileContainsDeadPlayer(tilePosition)) {
        DrawDeadPlayer(out, tilePosition, targetBufferPosition, lightTableIndex);
    }
    
    // 7. 绘制玩家
    if (PlayerAtPosition(tilePosition)) {
        DrawPlayer(out, *player, tilePosition, playerRenderPosition, lightTableIndex);
    }
    
    // 8. 绘制怪物
    if (MonsterAtPosition(tilePosition)) {
        DrawMonster(out, *monster, tilePosition, monsterRenderPosition, lightTableIndex);
    }
    
    // 9. 绘制投射物（PostFlag）
    DrawMissile(out, tilePosition, targetBufferPosition, false, lightTableIndex);
    
    // 10. 绘制对象（非PreFlag）
    if (object != nullptr && !object->_oPreFlag) {
        DrawObject(out, *object, tilePosition, targetBufferPosition, lightTableIndex);
    }
    
    // 11. 绘制物品（PostDraw）
    if (bItem > 0 && Items[bItem - 1]._iPostDraw) {
        DrawItem(out, bItem - 1, targetBufferPosition, lightTableIndex);
    }
}
```

**渲染顺序的重要性**:
- `PreFlag` 对象在玩家/怪物之前渲染（在背后）
- `PostDraw` 物品在玩家/怪物之后渲染（在前面）
- 投射物分为 Pre 和 Post 两个阶段

### 4.4 DrawCell() - 瓦片单元格渲染

**文件**: `Source/engine/render/scrollrt.cpp`  
**行号**: 534-955

渲染瓦片的基础结构（墙壁、地板装饰等）：

```cpp
void DrawCell(const Surface &out, const Lightmap lightmap,
              Point tilePosition, Point targetBufferPosition,
              int lightTableIndex) {
    const uint16_t levelPieceId = dPiece[tilePosition.x][tilePosition.y];
    const MICROS *pMap = &DPieceMicros[levelPieceId];
    
    // 渲染4个微瓦片（MicroTiles）
    for (int i = 0; i < 4; i++) {
        const LevelCelBlock levelCelBlock { pMap->mt[i] };
        if (levelCelBlock.hasValue()) {
            // 根据微瓦片类型和位置选择渲染方式
            RenderTile(out, lightmap, position, 
                      pDungeonCels.get(), levelCelBlock,
                      maskType, tbl);
        }
    }
}
```

## 5. 底层渲染系统

### 5.1 CLX 精灵渲染

**文件**: `Source/engine/render/clx_render.hpp`

CLX 是优化后的精灵格式（从 CEL/CL2 转换而来）：

- `ClxDraw()`: 基础渲染
- `ClxDrawLight()`: 带光照的渲染
- `ClxDrawTRN()`: 带调色板转换的渲染
- `ClxDrawBlended()`: 半透明渲染

### 5.2 光照系统

**文件**: `Source/engine/render/light_render.hpp`

- **光照表**: `LightTables[]` - 预计算的光照查找表
- **每像素光照**: `Lightmap` - 可选的每像素光照计算
- **光照索引**: `dLight[x][y]` - 每个瓦片的光照强度索引

### 5.3 瓦片渲染

**文件**: `Source/engine/render/dun_render.hpp`

- `RenderTileFrame()`: 底层瓦片帧渲染
- `RenderTile()`: 瓦片渲染（带类型和遮罩）
- `RenderTileFoliage()`: 地板装饰渲染

**瓦片类型**:
- `LeftTriangle`: 左三角形
- `RightTriangle`: 右三角形
- `LeftTrapezoid`: 左梯形
- `RightTrapezoid`: 右梯形
- `TransparentSquare`: 透明方形

**遮罩类型**:
- `Solid`: 完全不透明
- `Transparent`: 完全透明
- `Right`: 右上角透明
- `Left`: 左上角透明

## 6. 后处理和呈现

### 6.1 缩放处理

**文件**: `Source/engine/render/scrollrt.cpp`  
**函数**: `Zoom()`

如果启用了缩放模式，将视口区域放大2倍：

```cpp
if (*GetOptions().Graphics.zoom) {
    Zoom(fullOut.subregionY(0, gnViewportHeight));
}
```

### 6.2 呈现到屏幕

**文件**: `Source/engine/dx.cpp`  
**函数**: `RenderPresent()`

```cpp
void RenderPresent() {
    SDL_Surface *surface = GetOutputSurface();
    
    if (renderer != nullptr) {
        // 使用 SDL Renderer
        SDL_RenderClear(renderer);
        SDL_UpdateTexture(texture.get(), nullptr, surface->pixels, surface->pitch);
        SDL_RenderCopy(renderer, texture.get(), nullptr, nullptr);
        SDL_RenderPresent(renderer);
    } else {
        // 直接更新窗口表面
        SDL_UpdateWindowSurface(ghMainWnd);
    }
    
    LimitFrameRate();  // 限制帧率
}
```

## 7. 渲染优化特性

### 7.1 脏矩形系统

**文件**: `Source/engine/backbuffer_state.hpp`

- `RedrawEverything()`: 标记需要重绘整个屏幕
- `RedrawViewport()`: 标记需要重绘视口
- `RedrawComponent()`: 标记需要重绘特定UI组件

### 7.2 增量渲染

- 只有变化的部分才重绘
- UI组件独立标记和重绘
- 视口和面板分离渲染

### 7.3 性能优化

- **编译优化**: 关键渲染文件使用 `-O2` 即使在 Debug 模式
- **内联函数**: 大量使用 `DVL_ALWAYS_INLINE`
- **菱形网格遍历**: 高效的瓦片遍历算法

## 8. 渲染流程总结图

```
RunGameLoop()
    │
    ├─→ DrawAndBlit()
    │       │
    │       ├─→ UndrawCursor()
    │       │
    │       ├─→ DrawView()
    │       │       │
    │       │       ├─→ CalcFirstTilePosition()
    │       │       │
    │       │       └─→ DrawGame()  [核心渲染]
    │       │               │
    │       │               ├─→ Lightmap::build()  [构建光照]
    │       │               │
    │       │               ├─→ DrawFloor()  [阶段1: 地板]
    │       │               │       └─→ DrawFloorTile()
    │       │               │               └─→ RenderTileFrame()
    │       │               │
    │       │               └─→ DrawTileContent()  [阶段2: 内容]
    │       │                       └─→ DrawDungeon()  [每个瓦片]
    │       │                               │
    │       │                               ├─→ DrawCell()
    │       │                               ├─→ DrawMissile()
    │       │                               ├─→ DrawObject()
    │       │                               ├─→ DrawItem()
    │       │                               ├─→ DrawPlayer()
    │       │                               └─→ DrawMonster()
    │       │
    │       ├─→ DrawMainPanel()
    │       ├─→ DrawLifeFlaskLower()
    │       ├─→ DrawManaFlaskLower()
    │       ├─→ DrawCursor()
    │       │
    │       └─→ RenderPresent()
    │               │
    │               ├─→ SDL_UpdateTexture()  [如果有Renderer]
    │               ├─→ SDL_RenderPresent()
    │               └─→ LimitFrameRate()
```

## 9. 关键数据结构

### 9.1 Surface

**文件**: `Source/engine/surface.hpp`

渲染目标缓冲区，封装了 SDL_Surface：

```cpp
class Surface {
    SDL_Surface *surface;
    // 提供像素访问、区域裁剪等功能
};
```

### 9.2 Lightmap

**文件**: `Source/engine/render/light_render.hpp`

光照贴图，支持：
- 传统光照表查找
- 每像素光照计算

### 9.3 瓦片数据

- `dPiece[x][y]`: 瓦片ID
- `dLight[x][y]`: 光照索引
- `dItem[x][y]`: 物品索引
- `dCorpse[x][y]`: 尸体索引
- `dObject[x][y]`: 对象索引

## 10. 重要设计决策

### 10.1 不调用 clear()

**关键**: 渲染系统从不调用 `clear()` 清空缓冲区

**原因**:
1. 依赖瓦片覆盖整个屏幕
2. 透明区域显示下层内容（地板或前一帧）
3. 这是实现正确分层渲染的唯一方式

### 10.2 两阶段渲染

**阶段1 (DrawFloor)**: 只渲染地板
**阶段2 (DrawTileContent)**: 渲染所有内容

**好处**:
- 墙壁透明区域可以显示地板
- 河流等非地板区域保持前一帧内容
- 正确的视觉层次

### 10.3 菱形网格遍历

瓦片使用菱形（等距）网格布局，遍历时需要特殊处理：
- 奇偶行偏移
- 每行瓦片数可能不同

## 11. 参考代码位置

| 功能 | 文件 | 行号范围 |
|------|------|----------|
| 主循环 | `Source/diablo.cpp` | 898-1000 |
| 渲染入口 | `Source/engine/render/scrollrt.cpp` | 1993-2083 |
| 核心渲染 | `Source/engine/render/scrollrt.cpp` | 1371-1469 |
| 地板渲染 | `Source/engine/render/scrollrt.cpp` | 1141-1194 |
| 内容渲染 | `Source/engine/render/scrollrt.cpp` | 1205-1258 |
| 瓦片渲染 | `Source/engine/render/scrollrt.cpp` | 963-1363 |
| 呈现 | `Source/engine/dx.cpp` | 231-288 |
| CLX渲染 | `Source/engine/render/clx_render.hpp` | - |
| 瓦片渲染 | `Source/engine/render/dun_render.hpp` | - |

## 12. 总结

DevilutionX 的渲染系统是一个精心设计的2D等距渲染引擎，具有以下特点：

1. **分层渲染**: 地板和内容分离，确保正确的视觉层次
2. **增量更新**: 只重绘变化的部分，提高性能
3. **菱形网格**: 高效的等距瓦片遍历
4. **光照系统**: 支持传统光照表和每像素光照
5. **优化渲染**: 关键路径高度优化

这个系统为 Rust 重写提供了完整的参考实现。






















