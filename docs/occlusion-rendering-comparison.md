# Rust vs C++ 遮挡渲染实现对比文档

## 📋 目录

1. [概述](#概述)
2. [C++ 遮挡实现机制](#c-遮挡实现机制)
3. [Rust 当前实现状态](#rust-当前实现状态)
4. [关键差异分析](#关键差异分析)
5. [实现目标对比](#实现目标对比)
6. [实现路径建议](#实现路径建议)
7. [总结](#总结)

---

## 概述

本文档对比分析 C++ 原版和 Rust 实现中的物体遮挡（Occlusion）渲染机制，评估 Rust 实现是否能够达到与 C++ 相同的遮挡效果。

**关键问题**：当玩家在建筑物后面时，如何确保建筑物正确遮挡玩家？

---

## C++ 遮挡实现机制

### 1. 核心原理：等距网格遍历顺序 + 分层渲染

C++ 通过**等距网格的 zigzag 遍历顺序**和**固定的分层渲染顺序**实现遮挡，无需显式的遮挡检测。

### 2. 等距网格遍历顺序（Zigzag Traversal）

**位置**：`Source/engine/render/scrollrt.cpp::DrawTileContent()` (1049-1101行)

```cpp
for (int i = 0; i < rows; i++) {
    for (int j = 0; j < columns; j++) {
        // 从西到东遍历当前行
        DrawDungeon(out, lightmap, tilePosition, targetBufferPosition);
        tilePosition += Direction::East;  // X坐标增加，Y坐标减少
    }

    // 移动到下一行
    targetBufferPosition.y += TILE_HEIGHT / 2;  // 屏幕Y增加16像素

    if ((i & 1) != 0) {  // 奇数行
        tilePosition.x++;  // X坐标增加
        columns--;
        targetBufferPosition.x += TILE_WIDTH / 2;
    } else {  // 偶数行
        tilePosition.y++;  // Y坐标增加（更靠前）
        columns++;
        targetBufferPosition.x -= TILE_WIDTH / 2;
    }
}
```

**关键点**：
- 等距投影中，**Y 坐标越大 = 在屏幕上越靠前**
- 遍历顺序保证 **Y 值小的瓦片先渲染，Y 值大的瓦片后渲染**
- 因此，位于更靠前（Y 更大）的建筑物会在更靠后（Y 更小）的玩家之后渲染，从而遮挡玩家

### 3. 分层渲染顺序（Layer-based Rendering）

**位置**：`Source/engine/render/scrollrt.cpp::DrawDungeon()` (813-971行)

C++ 在 `DrawDungeon` 函数中按以下**固定顺序**渲染每一层：

```
1. DrawCell()                    - 墙壁和地形（最远层）
2. DrawMissile(PreFlag=true)     - 背景魔法效果
3. DrawCorpse()                  - 尸体
4. DrawObject(_oPreFlag=true)    - 地面物体（桌子、椅子等）
5. DrawItem(!_iPostDraw)         - 地面物品
6. DrawDeadPlayer()              - 死去的玩家
7. DrawPlayer()                  - 玩家 ⭐
8. DrawMonster()                 - 怪物
9. DrawMissile(PreFlag=false)    - 前景魔法效果
10. DrawObject(!_oPreFlag)       - 遮挡物体（门框、柱子等）⭐
11. DrawItem(_iPostDraw)         - 前置物品（悬挂物品）
```

**关键点**：
- 玩家在第 **7 层**渲染
- 遮挡物体（`_oPreFlag=false`）在第 **10 层**渲染
- 由于遍历顺序，如果建筑物瓦片的 Y 坐标大于玩家瓦片的 Y 坐标，建筑物会在玩家之后渲染，从而遮挡玩家

### 4. 关键标志：`_oPreFlag` 和 `_iPostDraw`

#### `_oPreFlag`（Object PreFlag）

**定义**：`Source/objects.h::Object::_oPreFlag` (56行)

- `true`：在玩家/怪物**之前**渲染（背后）
- `false`：在玩家/怪物**之后**渲染（前面，用于遮挡）

**示例**：
- 门打开时：`_oPreFlag = true`（`Source/objects.cpp:1058`）
- 门关闭时：`_oPreFlag = false`（`Source/objects.cpp:1106`）

#### `_iPostDraw`（Item PostDraw）

**定义**：`Source/items.h::Item::_iPostDraw` (201行)

- `false`：在玩家/怪物**之前**渲染（地面物品）
- `true`：在玩家/怪物**之后**渲染（悬挂物品）

### 5. 特殊情况处理

#### 5.1 墙壁后物体的提前渲染

**位置**：`Source/engine/render/scrollrt.cpp:1063-1076`

```cpp
if (IsWall(tilePosition) && ...) {
    if (IsTileNotSolid(tilePosition + Displacement { 1, -1 }) &&
        IsTileNotSolid(tilePosition + Displacement { 0, -1 })) {
        // 提前渲染墙壁后方的瓦片（包括玩家）
        const Point behindTile = tilePosition + Direction::East;
        DrawDungeon(out, lightmap, behindTile, ...);
        skipNext = true;  // 避免重复渲染
    }
}
```

**目的**：防止移动中的精灵从墙壁边缘穿透显示。

#### 5.2 移动角色的偏移处理

**位置**：`Source/engine/render/scrollrt.cpp:870-898`

当玩家移动时，使用负 ID 和位置偏移确保正确渲染：

```cpp
if (player->_pmode == PM_WALK_SOUTHWARDS || ...) {
    playerId = -playerId;  // 使用负ID标记移动中
}

if (dPlayer[tilePosition.x][tilePosition.y] == playerId) {
    // 计算偏移，将精灵渲染到来源瓦片
    if (player->_pmode == PM_WALK_SOUTHWARDS) {
        tempTargetBufferPosition += { 0, -TILE_HEIGHT };
        tempTilePosition += Opposite(player->_pdir);
    }
    DrawPlayer(out, *player, tempTilePosition, tempTargetBufferPosition, ...);
}
```

---

## Rust 当前实现状态

### 1. 渲染架构

**位置**：`rust-diablo/src/world/mod.rs`

Rust 实现分为两个主要阶段：

#### Phase 1: 地板层渲染

**位置**：`rust-diablo/src/world/mod.rs::render_with_texture_manager()` (595-650行)

```rust
if self.render_debug.render_floor {
    // 遍历所有瓦片，只渲染地板（is_floor == true）
    for row in 0..floor_rows {
        for _col in 0..floor_current_columns {
            let is_floor = Self::is_floor(sol_data, level_piece_id);
            if is_floor {
                self.draw_floor_at(...);  // 只渲染 blocks 0-1
            }
        }
    }
}
```

✅ **已实现**：与 C++ `DrawFloor()` 对应

#### Phase 2: 墙壁/上层内容渲染

**位置**：`rust-diablo/src/world/mod.rs::render_with_texture_manager()` (680-750行)

```rust
if self.render_debug.render_walls {
    for row in 0..wall_rows {
        for _col in 0..wall_current_columns {
            // 渲染所有瓦片（包括墙壁和地板）
            self.draw_cell_at(...);  // 渲染所有 blocks
        }
    }
}
```

✅ **已实现**：与 C++ `DrawTileContent()` 对应，但**只渲染了墙壁和地板**

#### Phase 3: 实体渲染（玩家/怪物）

**位置**：`rust-diablo/src/world/mod.rs::render_entities()` (362-473行)

```rust
fn render_entities(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
    for entity in &self.entities {
        // 计算屏幕位置（使用等距投影）
        let screen_pos = ...;
        // 渲染实体
        engine.draw_texture_by_id(...);
    }
}
```

⚠️ **问题**：实体在**所有瓦片渲染之后**单独渲染，这意味着：
- 实体**总是**在最上层
- **无法**被建筑物遮挡

### 2. 等距网格遍历顺序

**位置**：`rust-diablo/src/world/mod.rs::render_with_texture_manager()` (687-749行)

```rust
for row in 0..wall_rows {
    for _col in 0..wall_current_columns {
        // 从西到东遍历
        tx += 1;
        ty -= 1;
        sx += 64;
    }

    wall_screen_y += TILE_HEIGHT / 2;

    if (row & 1) != 0 {  // 奇数行
        wall_tile_x += 1;
        wall_current_columns -= 1;
        wall_screen_x += TILE_WIDTH / 2;
    } else {  // 偶数行
        wall_tile_y += 1;
        wall_current_columns += 1;
        wall_screen_x -= TILE_WIDTH / 2;
    }
}
```

✅ **已实现**：与 C++ 的 zigzag 遍历顺序一致

### 3. 缺失的功能

#### ❌ 未实现：DrawDungeon 分层渲染

Rust **没有**实现 C++ 的 `DrawDungeon()` 函数，该函数负责：
- 在单个瓦片内按顺序渲染：墙壁 → 尸体 → 物体 → 物品 → 玩家 → 怪物 → 遮挡物体

**当前状态**：
- ✅ 渲染墙壁（`draw_cell_at`）
- ❌ **未渲染**尸体
- ❌ **未渲染**物体（`DrawObject`）
- ❌ **未渲染**物品（`DrawItem`）
- ❌ **未渲染**玩家（在瓦片内）
- ❌ **未渲染**怪物（在瓦片内）
- ❌ **未渲染**遮挡物体

#### ❌ 未实现：`_oPreFlag` 和 `_iPostDraw` 标志

Rust 代码中**没有**找到：
- `_oPreFlag` 或 `pre_flag` 字段
- `_iPostDraw` 或 `post_draw` 字段

#### ❌ 未实现：墙壁后物体提前渲染

虽然文档中有计划（`rust-diablo/docs/step-6.4.3-wall-behind-rendering.md`），但代码中**未实现**。

#### ❌ 未实现：移动角色的偏移处理

Rust 的实体渲染**没有**处理移动中的偏移逻辑。

---

## 关键差异分析

### 1. 渲染顺序差异

| 方面 | C++ | Rust |
|------|-----|------|
| **瓦片遍历** | ✅ Zigzag 遍历 | ✅ Zigzag 遍历 |
| **分层渲染** | ✅ 在 `DrawDungeon` 中按层渲染 | ❌ 实体单独渲染，在瓦片之后 |
| **玩家渲染位置** | ✅ 在瓦片遍历中（第7层） | ❌ 在所有瓦片之后 |
| **物体渲染** | ✅ 在瓦片遍历中（第4/10层） | ❌ 未实现 |
| **遮挡效果** | ✅ 通过遍历顺序自动实现 | ❌ **无法实现**（实体总是在最上层） |

### 2. 数据结构差异

| 数据结构 | C++ | Rust |
|----------|-----|------|
| **dPlayer[x][y]** | ✅ 存储玩家ID | ❌ 未实现 |
| **dMonster[x][y]** | ✅ 存储怪物ID | ❌ 未实现 |
| **dItem[x][y]** | ✅ 存储物品索引 | ❌ 未实现 |
| **dObject[x][y]** | ✅ 存储物体ID | ❌ 未实现 |
| **Object::_oPreFlag** | ✅ 控制渲染时机 | ❌ 未实现 |
| **Item::_iPostDraw** | ✅ 控制渲染时机 | ❌ 未实现 |

### 3. 遮挡效果对比

#### C++ 实现

```
场景：玩家在 (5, 3)，建筑物在 (6, 5)

渲染过程：
1. 遍历到 (5, 3) → DrawDungeon() → 渲染玩家（第7层）
2. 继续遍历...
3. 遍历到 (6, 5) → DrawDungeon() → 渲染建筑物（第10层，_oPreFlag=false）
   结果：建筑物覆盖玩家，玩家被遮挡 ✅
```

#### Rust 当前实现

```
场景：玩家在 (5, 3)，建筑物在 (6, 5)

渲染过程：
1. Phase 1: 渲染所有地板
2. Phase 2: 渲染所有墙壁（包括建筑物）
3. Phase 3: render_entities() → 渲染玩家
   结果：玩家覆盖建筑物，玩家**无法被遮挡** ❌
```

---

## 实现目标对比

### C++ 实现目标

✅ **完全实现**：
1. 等距网格遍历确保深度排序
2. 分层渲染确保正确的遮挡关系
3. 通过 `_oPreFlag` 和 `_iPostDraw` 控制渲染时机
4. 移动角色的特殊处理
5. 墙壁后物体的提前渲染

### Rust 实现目标

❌ **当前状态**：
1. ✅ 等距网格遍历（已实现）
2. ❌ 分层渲染（未实现）
3. ❌ 标志控制（未实现）
4. ❌ 移动角色处理（未实现）
5. ❌ 墙壁后物体提前渲染（未实现）

**结论**：Rust **目前无法实现**与 C++ 相同的遮挡效果。

---

## 实现路径建议

### 阶段 1：实现 DrawDungeon 分层渲染

**目标**：在 `draw_cell_at` 之后，按顺序渲染每一层内容。

**需要实现**：

1. **创建 `draw_dungeon_at` 函数**

```rust
fn draw_dungeon_at(
    &self,
    engine: &mut Engine,
    texture_mgr: &RefCell<TileTextureManager>,
    tile_x: i32,
    tile_y: i32,
    screen_x: i32,
    screen_y: i32,
    light_table_index: u8,
) -> Result<()> {
    // 1. DrawCell (墙壁)
    self.draw_cell_at(...);

    // 2. DrawMissile(PreFlag=true) - 背景魔法
    // TODO: 实现

    // 3. DrawCorpse - 尸体
    // TODO: 实现

    // 4. DrawObject(_oPreFlag=true) - 前置物体
    // TODO: 实现

    // 5. DrawItem(!_iPostDraw) - 前置物品
    // TODO: 实现

    // 6. DrawDeadPlayer - 死去的玩家
    // TODO: 实现

    // 7. DrawPlayer - 玩家
    // TODO: 实现（需要 dPlayer 数组）

    // 8. DrawMonster - 怪物
    // TODO: 实现（需要 dMonster 数组）

    // 9. DrawMissile(PreFlag=false) - 前景魔法
    // TODO: 实现

    // 10. DrawObject(!_oPreFlag) - 遮挡物体
    // TODO: 实现

    // 11. DrawItem(_iPostDraw) - 后置物品
    // TODO: 实现
}
```

2. **修改 Phase 2 渲染循环**

```rust
// 在 render_with_texture_manager() 的 Phase 2 中
for row in 0..wall_rows {
    for _col in 0..wall_current_columns {
        // 替换 draw_cell_at 为 draw_dungeon_at
        self.draw_dungeon_at(
            engine,
            texture_mgr_cell,
            dpiece_x,
            dpiece_y,
            sx,
            wall_screen_y,
            light_table_index,
        )?;
    }
}
```

3. **移除独立的实体渲染**

```rust
// 在 render() 函数中
pub fn render(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
    self.render_with_texture_manager(engine, camera)?;
    // ❌ 移除这行：self.render_entities(engine, camera)?;
    Ok(())
}
```

### 阶段 2：实现数据结构

**需要添加**：

1. **dPlayer 数组**：存储每个瓦片上的玩家ID
2. **dMonster 数组**：存储每个瓦片上的怪物ID
3. **dItem 数组**：存储每个瓦片上的物品索引
4. **dObject 数组**：存储每个瓦片上的物体ID
5. **Object::pre_flag 字段**：控制物体渲染时机
6. **Item::post_draw 字段**：控制物品渲染时机

### 阶段 3：实现特殊处理

1. **墙壁后物体提前渲染**（参考 `step-6.4.3-wall-behind-rendering.md`）
2. **移动角色的偏移处理**（参考 C++ Line 870-898）

---

## 总结

### 当前状态

| 功能 | C++ | Rust | 状态 |
|------|-----|------|------|
| 等距网格遍历 | ✅ | ✅ | ✅ 已实现 |
| 分层渲染 | ✅ | ❌ | ❌ 未实现 |
| 玩家在瓦片内渲染 | ✅ | ❌ | ❌ 未实现 |
| 物体渲染 | ✅ | ❌ | ❌ 未实现 |
| 遮挡效果 | ✅ | ❌ | ❌ **无法实现** |

### 结论

**Rust 目前无法实现与 C++ 相同的遮挡效果**，因为：

1. ❌ 实体（玩家/怪物）在**所有瓦片之后**单独渲染，总是位于最上层
2. ❌ **没有**实现 `DrawDungeon` 的分层渲染逻辑
3. ❌ **没有**实现 `_oPreFlag` 和 `_iPostDraw` 标志
4. ❌ **没有**实现物体和物品的渲染

### 实现可行性

✅ **完全可行**：Rust 可以完全复刻 C++ 的遮挡实现，但需要：

1. 实现 `DrawDungeon` 分层渲染函数
2. 添加必要的数据结构（dPlayer, dMonster, dItem, dObject）
3. 实现物体和物品的渲染逻辑
4. 移除独立的实体渲染，改为在瓦片遍历中渲染

### 参考文档

- C++ 渲染架构分析：`rust-diablo/docs/cpp_rendering_architecture.md`
- 墙壁后渲染实现：`rust-diablo/docs/step-6.4.3-wall-behind-rendering.md`
- C++ 渲染流程分析：`rust-diablo/docs/c++_rendering_flow_analysis.md`

---

**文档版本**：1.0
**最后更新**：2024-12-XX
**作者**：AI Assistant
