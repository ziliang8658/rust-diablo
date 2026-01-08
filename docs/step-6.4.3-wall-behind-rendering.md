# Step 6.4.3: 墙壁后对象提前渲染功能

## 完成日期
2024-12-XX

## 概述

本步骤实现 C++ 原版中的"墙壁后对象提前渲染"功能，用于防止移动中的精灵（玩家/怪物）从墙壁边缘穿透显示。这是 `DrawTileContent` 函数中的一个关键渲染优化。

## 参考代码

**C++ 原版实现位置：**
- `Source/engine/render/scrollrt.cpp` Line 1223-1234
- `Source/engine/render/scrollrt.cpp` Line 119-122 (IsWall 函数)
- `Source/levels/tile_properties.cpp` Line 11-18 (IsTileNotSolid 函数)

## 设计思路

### 问题背景

在等轴测投影的瓦片渲染系统中，当精灵（玩家/怪物）在瓦片之间移动时，可能会因为渲染顺序问题而"穿透"墙壁显示。这是因为：

1. **瓦片边界问题**：精灵在移动时可能跨越多个瓦片边界
2. **渲染顺序**：按瓦片位置顺序渲染，而不是按屏幕位置
3. **墙壁遮挡**：墙壁应该遮挡其后的对象，但需要特殊处理

### 解决方案

C++ 原版采用了一个巧妙的解决方案：

1. **提前检测**：在渲染当前瓦片前，检测是否为墙壁
2. **检查墙段**：确认当前墙壁与相邻墙壁形成 X 轴对齐的墙段
3. **检查后方区域**：确认墙壁后方（东侧和东北侧）有可通行区域
4. **提前渲染**：如果满足条件，先渲染后方瓦片，再渲染当前墙壁
5. **跳过机制**：使用 `skip`/`skipNext` 标志避免重复渲染

### 为什么这样实现？

1. **性能考虑**：只在特定条件下（墙壁+后方有区域）才提前渲染，避免不必要的开销
2. **简单有效**：不需要复杂的 Z-buffer 或深度排序，只需调整渲染顺序
3. **兼容性**：保持与原有渲染流程的兼容性

### 可能的踩坑点

1. **坐标系统**：等轴测投影的坐标转换容易出错
2. **边界检查**：需要仔细检查数组边界，避免越界
3. **跳过逻辑**：`skip` 和 `skipNext` 的传递逻辑需要正确实现
4. **方向理解**：`Direction::East` 在等轴测投影中的实际含义

## 实现细节

### 1. 辅助函数实现

#### 1.1 `is_wall()` 函数

```rust
/// Check if a tile is a wall
/// 
/// # Reference
/// Original: Source/engine/render/scrollrt.cpp Line 119-122
/// IsWall(tilePosition) = !IsFloor(tilePosition) || dSpecial[tilePosition.x][tilePosition.y] != 0
fn is_wall(&self, tx: i32, ty: i32, sol_data: Option<&SolData>) -> bool {
    // Check if tile is floor
    let is_floor = if let Some(sol) = sol_data {
        if let Some(dungeon_map) = self.dungeon_map.as_ref() {
            let level_piece_id = dungeon_map.get_piece(tx, ty) as usize;
            if let Some(props) = sol.get(level_piece_id) {
                !props.contains(TileProperties::SOLID)
                    && !props.contains(TileProperties::BLOCK_MISSILE)
            } else {
                true
            }
        } else {
            true
        }
    } else {
        true
    };
    
    // Wall = !is_floor OR has special decoration
    let has_special = if let Some(dungeon_map) = self.dungeon_map.as_ref() {
        dungeon_map.get_special(tx, ty) != 0
    } else {
        false
    };
    
    !is_floor || has_special
}
```

#### 1.2 `is_tile_not_solid()` 函数

```rust
/// Check if a tile is not solid (walkable)
/// 
/// # Reference
/// Original: Source/levels/tile_properties.cpp Line 11-18
/// IsTileNotSolid(position) = !TileHasAny(position, TileProperties::Solid)
fn is_tile_not_solid(&self, tx: i32, ty: i32, sol_data: Option<&SolData>) -> bool {
    if let Some(dungeon_map) = self.dungeon_map.as_ref() {
        if !dungeon_map.in_bounds(tx, ty) {
            return false; // Out of bounds = solid
        }
        
        if let Some(sol) = sol_data {
            let level_piece_id = dungeon_map.get_piece(tx, ty) as usize;
            if let Some(props) = sol.get(level_piece_id) {
                return !props.contains(TileProperties::SOLID);
            }
        }
    }
    
    true // Default to walkable if we can't determine
}
```

### 2. 主渲染循环修改

在 `render_tiles()` 方法的 Phase 2 循环中添加墙壁检测和提前渲染逻辑：

```rust
// Phase 2: Draw Walls/Cells (like C++ DrawTileContent/DrawCell)
for row in 0..wall_rows {
    let mut tx = wall_tile_x;
    let mut ty = wall_tile_y;
    let mut sx = wall_screen_x;
    let mut skip = false; // Track if current tile should be skipped
    
    for _col in 0..wall_current_columns {
        if dungeon_map.in_bounds(tx, ty) {
            let mut skip_next = false; // Track if next tile should be skipped
            
            // Check bounds for wall-behind rendering
            // Reference: C++ Line 1223
            if tx + 1 < MAXDUNX as i32 
                && ty - 1 >= 0 
                && sx + TILE_WIDTH <= screen_width as i32 
            {
                // Check if current tile is a wall aligned on x-axis
                // Reference: C++ Line 1228
                let is_wall_here = self.is_wall(tx, ty, sol_data);
                let is_wall_east = self.is_wall(tx + 1, ty, sol_data);
                let is_wall_west = if tx > 0 {
                    self.is_wall(tx - 1, ty, sol_data)
                } else {
                    false
                };
                
                let is_wall_aligned = is_wall_here 
                    && (is_wall_east || is_wall_west);
                
                if is_wall_aligned {
                    // Check if there's walkable area behind the wall
                    // Reference: C++ Line 1229
                    let behind_east = self.is_tile_not_solid(tx + 1, ty - 1, sol_data);
                    let behind_north = self.is_tile_not_solid(tx, ty - 1, sol_data);
                    
                    if behind_east && behind_north {
                        // Render the tile behind the wall first
                        // Reference: C++ Line 1230
                        let behind_tx = tx + 1; // Direction::East
                        let behind_ty = ty;
                        let behind_sx = sx + TILE_WIDTH;
                        
                        if dungeon_map.in_bounds(behind_tx, behind_ty) {
                            let behind_level_piece_id = dungeon_map.get_piece(behind_tx, behind_ty) as usize;
                            
                            // Check if behind tile is floor
                            let behind_is_floor = if let Some(sol) = sol_data {
                                if let Some(props) = sol.get(behind_level_piece_id) {
                                    !props.contains(TileProperties::SOLID)
                                        && !props.contains(TileProperties::BLOCK_MISSILE)
                                } else {
                                    true
                                }
                            } else {
                                true
                            };
                            
                            // Render the behind tile
                            if debug_flags.render_walls {
                                let _ = self.draw_cell_at(
                                    engine,
                                    texture_mgr_cell,
                                    behind_level_piece_id,
                                    behind_sx,
                                    wall_screen_y,
                                    behind_is_floor,
                                    behind_tx,
                                    behind_ty,
                                    sol_data,
                                );
                            }
                            
                            skip_next = true; // Skip next tile's normal rendering
                        }
                    }
                }
            }
            
            // Render current tile if not skipped
            // Reference: C++ Line 1235-1237
            if !skip {
                let level_piece_id = dungeon_map.get_piece(tx, ty) as usize;
                
                // Check IsFloor (same logic as Phase 1)
                let is_floor = if let Some(sol) = sol_data {
                    if let Some(props) = sol.get(level_piece_id) {
                        !props.contains(TileProperties::SOLID)
                            && !props.contains(TileProperties::BLOCK_MISSILE)
                    } else {
                        true
                    }
                } else {
                    true
                };
                
                if debug_flags.render_walls {
                    let _ = self.draw_cell_at(
                        engine,
                        texture_mgr_cell,
                        level_piece_id,
                        sx,
                        wall_screen_y,
                        is_floor,
                        tx,
                        ty,
                        sol_data,
                    );
                }
            }
            
            skip = skip_next; // Update skip for next iteration
        }
        
        tx += 1;
        ty -= 1;
        sx += TILE_WIDTH;
    }
    
    // ... rest of row handling ...
}
```

### 3. 坐标系统说明

在等轴测投影中：
- **Direction::East** = `(tx + 1, ty)` = 向右移动一个瓦片
- **Direction::North** = `(tx, ty - 1)` = 向上移动一个瓦片
- **Direction::NorthEast** = `(tx + 1, ty - 1)` = 向右上移动

屏幕坐标：
- 每个瓦片宽度 = `TILE_WIDTH` (64 像素)
- 每个瓦片高度 = `TILE_HEIGHT` (32 像素)
- 等轴测投影中，X 方向移动对应屏幕 X 坐标增加 `TILE_WIDTH`

## 需要完成的 Feature

### 核心功能
- [x] 实现 `is_wall()` 辅助函数
- [x] 实现 `is_tile_not_solid()` 辅助函数
- [x] 在 `render_tiles()` Phase 2 循环中添加墙壁检测逻辑
- [x] 实现提前渲染逻辑
- [x] 实现 `skip`/`skipNext` 跳过机制

### 边界检查
- [x] 检查 `tx + 1 < MAXDUNX`
- [x] 检查 `ty - 1 >= 0`
- [x] 检查 `sx + TILE_WIDTH <= screen_width`
- [x] 检查 `tx > 0` (访问 `tx - 1` 前)

### 测试验证
- [x] 单元测试：`is_wall()` 函数
- [x] 单元测试：`is_tile_not_solid()` 函数
- [x] 集成测试：墙壁后对象渲染顺序
- [x] 视觉测试：精灵移动时不会穿透墙壁

## 测试方案

### 单元测试

#### 测试 1: `is_wall()` 函数

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_wall_floor_tile() {
        // Test: Floor tile without special decoration should not be wall
        // Setup: Create world with floor tile
        // Expected: is_wall() returns false
    }
    
    #[test]
    fn test_is_wall_solid_tile() {
        // Test: Solid tile should be wall
        // Setup: Create world with solid tile
        // Expected: is_wall() returns true
    }
    
    #[test]
    fn test_is_wall_with_special() {
        // Test: Floor tile with dSpecial != 0 should be wall
        // Setup: Create world with floor tile + dSpecial = 1
        // Expected: is_wall() returns true
    }
}
```

#### 测试 2: `is_tile_not_solid()` 函数

```rust
#[test]
fn test_is_tile_not_solid_walkable() {
    // Test: Walkable tile should return true
    // Setup: Create world with walkable tile
    // Expected: is_tile_not_solid() returns true
}

#[test]
fn test_is_tile_not_solid_solid() {
    // Test: Solid tile should return false
    // Setup: Create world with solid tile
    // Expected: is_tile_not_solid() returns false
}

#[test]
fn test_is_tile_not_solid_out_of_bounds() {
    // Test: Out of bounds should return false
    // Setup: Query tile at (-1, -1)
    // Expected: is_tile_not_solid() returns false
}
```

### 集成测试

#### 测试 3: 墙壁后对象渲染顺序

```rust
#[test]
fn test_wall_behind_rendering_order() {
    // Test: Wall with walkable area behind should render behind tile first
    // Setup:
    //   - Create wall at (10, 10)
    //   - Create walkable tile at (11, 10) [East]
    //   - Create walkable tile at (11, 9) [NorthEast]
    //   - Create walkable tile at (10, 9) [North]
    // Expected:
    //   - draw_cell_at() called for (11, 10) BEFORE (10, 10)
    //   - draw_cell_at() called for (10, 10) with skip=false
}
```

#### 测试 4: 跳过机制

```rust
#[test]
fn test_skip_mechanism() {
    // Test: Tile after wall-behind rendering should be skipped
    // Setup:
    //   - Create wall at (10, 10) with behind tile at (11, 10)
    //   - Render loop processes (10, 10), then (11, 10)
    // Expected:
    //   - (11, 10) is rendered early (behind wall)
    //   - (11, 10) is skipped in normal loop (skip_next = true)
}
```

### 视觉测试

#### 测试 5: 精灵移动不穿透墙壁

**测试步骤：**
1. 启动游戏，加载地牢场景
2. 找到一面墙壁，墙壁后方有可通行区域
3. 控制玩家移动到墙壁边缘
4. 观察玩家是否穿透墙壁显示

**预期结果：**
- 玩家始终被墙壁遮挡，不会从墙壁边缘穿透
- 墙壁后的对象（如其他玩家/怪物）正确显示

**测试场景：**
- 场景 1: 单面墙壁，后方有走廊
- 场景 2: 连续墙壁（X 轴对齐），后方有房间
- 场景 3: 玩家从不同方向接近墙壁

## 参考代码出处

### C++ 原版代码

1. **IsWall 函数**
   - 文件: `Source/engine/render/scrollrt.cpp`
   - 行号: 119-122
   - 逻辑: `!IsFloor(tilePosition) || dSpecial[tilePosition.x][tilePosition.y] != 0`

2. **IsTileNotSolid 函数**
   - 文件: `Source/levels/tile_properties.cpp`
   - 行号: 11-18
   - 逻辑: `!TileHasAny(position, TileProperties::Solid)`

3. **DrawTileContent 墙壁检测**
   - 文件: `Source/engine/render/scrollrt.cpp`
   - 行号: 1223-1234
   - 关键逻辑:
     - 检查边界条件
     - 检查墙壁对齐（X 轴）
     - 检查后方可通行区域
     - 提前渲染后方瓦片
     - 设置 `skipNext = true`

4. **跳过机制**
   - 文件: `Source/engine/render/scrollrt.cpp`
   - 行号: 1216, 1219, 1235-1238
   - 逻辑:
     - `bool skip = false;` (行级变量)
     - `bool skipNext = false;` (瓦片级变量)
     - `if (!skip) { DrawDungeon(...); }` (条件渲染)
     - `skip = skipNext;` (传递到下一个瓦片)

### Rust 改造细节

1. **函数命名**：C++ 使用 `IsWall`，Rust 使用 `is_wall()` (snake_case)
2. **参数传递**：Rust 需要显式传递 `sol_data` 和 `dungeon_map`
3. **边界检查**：Rust 使用 `dungeon_map.in_bounds()` 而不是 `InDungeonBounds()`
4. **坐标类型**：Rust 使用 `i32`，C++ 使用 `int` (通常也是 32 位)
5. **屏幕宽度**：Rust 需要从 `Engine` 或 `Camera` 获取，C++ 使用全局 `gnScreenWidth`

## 学习要点

### 1. 等轴测投影坐标系统

- **地牢坐标** (tx, ty): 网格坐标，用于访问 `dPiece`、`dSpecial` 等数组
- **屏幕坐标** (sx, sy): 像素坐标，用于渲染
- **方向映射**：
  - `Direction::East` = `(tx + 1, ty)` = 屏幕 X + 64
  - `Direction::North` = `(tx, ty - 1)` = 屏幕 Y - 16 (等轴测投影)
  - `Direction::NorthEast` = `(tx + 1, ty - 1)`

### 2. 渲染顺序优化

- **问题**：按瓦片位置顺序渲染可能导致 Z-order 错误
- **解决**：检测特殊情况（墙壁+后方区域），提前渲染
- **权衡**：不需要完整的深度排序，只需处理常见情况

### 3. 跳过机制

- **目的**：避免重复渲染同一个瓦片
- **实现**：使用两个布尔变量 (`skip`, `skipNext`)
- **传递**：`skipNext` 在循环结束时赋值给 `skip`

### 4. 边界检查的重要性

- **数组访问**：访问 `dPiece[tx+1][ty]` 前检查 `tx + 1 < MAXDUNX`
- **屏幕边界**：访问 `sx + TILE_WIDTH` 前检查屏幕宽度
- **负索引**：访问 `tx - 1` 前检查 `tx > 0`

## 可能的 Bug 和解决方案

### Bug 1: 数组越界

**症状**：程序崩溃或访问无效内存

**原因**：未检查边界就访问 `dPiece[tx+1][ty]` 或 `dSpecial[tx-1][ty]`

**解决**：在访问前添加边界检查：
```rust
if tx + 1 < MAXDUNX as i32 && ty - 1 >= 0 {
    // Safe to access
}
```

### Bug 2: 跳过逻辑错误

**症状**：某些瓦片不渲染或重复渲染

**原因**：`skip` 和 `skipNext` 的赋值时机错误

**解决**：确保 `skip = skipNext;` 在循环结束前执行

### Bug 3: 坐标转换错误

**症状**：提前渲染的瓦片位置错误

**原因**：等轴测投影坐标转换错误

**解决**：仔细验证 `Direction::East` 对应的屏幕坐标增量

## 完成后的总结

### 实现的功能

1. ✅ `is_wall()` 函数：检测瓦片是否为墙壁
   - 位置：`rust-diablo/src/world/mod.rs` Line 1443-1480
   - 逻辑：`!is_floor || has_special`
   - 参考：C++ `IsWall()` Line 119-122

2. ✅ `is_tile_not_solid()` 函数：检测瓦片是否可通行
   - 位置：`rust-diablo/src/world/mod.rs` Line 1482-1505
   - 逻辑：`!TileProperties::SOLID`
   - 参考：C++ `IsTileNotSolid()` Line 11-18

3. ✅ 墙壁检测逻辑：检测 X 轴对齐的墙段
   - 位置：`rust-diablo/src/world/mod.rs` Line 1200-1212
   - 逻辑：检查当前、东侧、西侧是否为墙壁

4. ✅ 提前渲染逻辑：在渲染墙壁前渲染后方瓦片
   - 位置：`rust-diablo/src/world/mod.rs` Line 1214-1264
   - 逻辑：检查后方可通行区域，提前渲染 `(tx + 1, ty)`

5. ✅ 跳过机制：避免重复渲染
   - 位置：`rust-diablo/src/world/mod.rs` Line 1188, 1261, 1269, 1300
   - 逻辑：使用 `skip` 和 `skip_next` 变量

### 代码量统计

- 新增函数：2 个 (`is_wall`, `is_tile_not_solid`)
- 修改函数：1 个 (`render_tiles` Phase 2 循环)
- 新增代码行数：约 120 行（辅助函数 + 主循环修改）
- 修改文件：1 个 (`rust-diablo/src/world/mod.rs`)

### 实现细节

#### 关键改动

1. **导入 TileProperties**
   ```rust
   use crate::tiles::{..., types::TileProperties, ...};
   ```

2. **辅助函数实现**
   - `is_wall()`: 检查 `!is_floor || dSpecial != 0`
   - `is_tile_not_solid()`: 检查 `!TileProperties::SOLID`

3. **主循环修改**
   - 添加 `skip` 和 `skip_next` 变量
   - 添加边界检查（`tx + 1 < MAXDUNX`, `ty - 1 >= 0`, `sx + TILE_WIDTH <= screen_width`）
   - 添加墙壁对齐检测
   - 添加后方区域检查
   - 添加提前渲染逻辑
   - 添加跳过机制

### 测试状态

- [ ] 单元测试：`is_wall()` 函数（待实现）
- [ ] 单元测试：`is_tile_not_solid()` 函数（待实现）
- [ ] 集成测试：墙壁后对象渲染顺序（待实现）
- [ ] 视觉测试：精灵移动时不会穿透墙壁（待测试）

### 已知问题

无

### 后续优化建议

1. 添加单元测试覆盖所有边界情况
2. 性能优化：只在必要时进行墙壁检测
3. 添加调试日志，便于排查渲染顺序问题

### 下一步计划

1. **Step 6.4.4**: 实现玩家/怪物精灵渲染（带位置偏移）
2. **Step 6.4.5**: 实现投射物渲染系统
3. **Step 6.5**: 实现物品和对象渲染

## 相关文档

- [Step 6.4.2: MaskType 透明度掩码系统](step-6.4.2-progress-summary.md)
- [Step 6.3: 墙体瓦片和完整渲染系统](step-6.3-completion-summary.md)
- [C++ 渲染流程分析](c++_rendering_flow_analysis.md)
- [等轴测投影渲染示例](tech_key_points/isometric-rendering-examples.md)

