# Step 4.2: 碰撞检测与相机系统总结

## 📅 完成日期
2025年11月20日

## 🚀 已完成功能

在 Step 4.2 中，我们成功实现了游戏的核心空间交互系统：

1.  **碰撞检测系统 (`world/collision.rs`)**
    *   实现了 `CollisionMap` 结构，管理整个地图的行走性数据。
    *   定义了 `TileType` 枚举（Empty, Floor, Wall, Door, Obstacle）。
    *   实现了 `validate_move` 方法，防止实体穿过墙壁。
    *   实现了基于 AABB (轴对齐包围盒) 的矩形碰撞检测 `is_rect_walkable`。

2.  **相机系统 (`renderer/camera.rs`)**
    *   实现了 `Camera` 结构，支持视口管理和坐标转换。
    *   实现了 `follow` 方法，使相机能够平滑跟随玩家，并保持玩家在屏幕中心。
    *   实现了 `set_bounds` 方法，防止相机超出地图边界。
    *   实现了 `world_to_screen` 和 `screen_to_world` 坐标转换。
    *   实现了 `is_rect_visible` 方法，用于视口剔除（Culling）优化。

3.  **系统集成**
    *   **World 集成**：`World` 现在包含 `CollisionMap`，并在渲染时使用相机进行坐标变换。
    *   **Entity 集成**：`Entity::update` 方法更新，支持碰撞检测。移动时会先验证目标位置是否可行走。
    *   **Game 集成**：`Game` 循环中加入了相机的更新和边界设置。

## 📝 设计决策与实现细节

### 1. 碰撞检测策略
我们采用了**预先验证（Predictive Validation）**策略：
*   在实体移动之前，先计算目标位置。
*   检查目标位置的包围盒是否与任何不可行走瓦片重叠。
*   如果重叠，则拒绝移动（或后续可优化为滑动）。
*   这种方法比"先移动后修正"更简单且稳定，避免了物体卡在墙里的问题。

### 2. 相机坐标系
*   采用了**世界坐标**和**屏幕坐标**分离的设计。
*   实体和地图数据存储在世界坐标系中。
*   渲染时通过 `camera.world_to_screen()` 转换为屏幕坐标。
*   目前暂时使用正交投影（Orthographic），为后续 Step 5/6 的等距视角（Isometric）打下基础。相机模块的设计允许将来替换具体的投影算法而不影响其他系统。

### 3. 视口剔除 (Culling)
*   `World::render` 增加了视口剔除逻辑。
*   只渲染相机范围内的瓦片和实体。
*   这对于大型地图（如 Diablo 的地牢）是必不可少的性能优化。

## ⚠️ 遇到的问题与解决方案

1.  **Rect API 差异**
    *   **问题**：在实现过程中，误用了 `Rect` 的 `w`/`h` 字段，而实际上 `math::Rect` 定义的是 `width`/`height`。
    *   **解决**：修正了 `camera.rs` 和 `collision.rs` 中的字段访问，并重新进行了编译检查。

2.  **浮点数坐标精度**
    *   **问题**：实体移动使用浮点数累积，但碰撞检测基于整数网格。
    *   **解决**：在 `Entity` 中保留 `position_f` 用于平滑移动计算，但在碰撞检测后，如果发生碰撞修正，会同步更新 `position_f`，防止视觉位置与逻辑位置脱节。

3.  **子像素移动累积丢失 Bug** ⚠️ 严重问题
    *   **问题**：玩家在可行走的瓦片上按 WASD 键，但玩家位置没有任何变化。即使瓦片类型是 walkable，玩家仍然无法移动。
    *   **根本原因**：在 `Entity::update` 中，当整数位置未改变时（高帧率或低速度场景），浮点位置 `position_f` 没有被更新，导致子像素移动被丢弃，玩家被"卡住"。
    *   **解决**：修复了 `Entity::update` 方法，确保即使整数位置未改变，也要更新浮点位置以累积子像素移动。
    *   **详细文档**：参见 [bugfix-subpixel-movement-accumulation.md](bugfix-subpixel-movement-accumulation.md)

## 🎨 Tile 系统多样化

为了增强游戏的视觉效果，我们扩展了 Tile 系统，支持多种瓦片类型和图片渲染：

### 1. 瓦片类型扩展

在 `world/collision.rs` 中，`TileType` 枚举现在包含：
- `Empty` - 空地（黑色）
- `Floor` - 地板（可行走，深灰色）
- `Wall` - 墙壁（不可行走，灰色）
- `Grass` - 草地（可行走，绿色）
- `Water` - 水（不可行走，蓝色）
- `Door` - 门（预留，可行走）
- `Obstacle` - 障碍物（预留，不可行走）

### 2. 图片资源支持

每种瓦片类型都可以使用对应的图片资源：
- `tile_floor.png` - 地板纹理
- `tile_wall.png` - 墙壁纹理
- `tile_grass.png` - 草地纹理
- `tile_water.png` - 水面纹理

### 3. 渲染系统

`World::render` 方法现在：
1. 首先尝试使用 `TileType::sprite_id()` 获取纹理ID
2. 如果纹理存在，使用 `engine.draw_texture_by_id()` 渲染图片
3. 如果纹理不存在，回退到使用颜色渲染（`tile.color()`）

### 4. 资源加载

在 `Game::new()` 中，所有瓦片纹理在游戏初始化时被加载：
```rust
let tile_textures = vec![
    ("tile_floor", "tile_floor.png"),
    ("tile_wall", "tile_wall.png"),
    ("tile_grass", "tile_grass.png"),
    ("tile_water", "tile_water.png"),
];
```

### 5. 地图生成

`World::new()` 现在使用简单的噪声函数生成多样化的地图：
- 边界是墙壁
- 中心区域是地板
- 使用正弦/余弦函数生成草地和水域
- 随机分布一些障碍物

这为游戏提供了更丰富的视觉体验，也为后续的地图生成系统（Step 6）打下了基础。

## 🐵 玩家精灵图系统（猴子动画）

为了提升游戏的视觉效果，我们实现了一个完整的玩家精灵图系统，包含静止和移动动画。

### 1. 精灵图设计

**精灵表规格**：
- 尺寸：256x64 像素
- 帧数：4 帧（水平排列）
- 每帧大小：64x64 像素

**帧内容**：
- **帧 1 (0-63)**：静止的猴子（Idle 状态）
  - 站立的棕色猴子
  - 包含头部、身体、手臂、腿部
  - 简单的面部特征（眼睛、鼻子、嘴巴）
  
- **帧 2-4 (64-255)**：猴子乘坐云朵（Walk 状态）
  - 帧 2 (64-127)：猴子在云朵上，云朵位置居中
  - 帧 3 (128-191)：猴子在云朵上，云朵轻微左移
  - 帧 4 (192-255)：猴子在云朵上，云朵轻微右移
  - 通过云朵位置和猴子手臂的微小变化形成动画效果

### 2. 精灵图生成工具

创建了 `assets/create_monkey_sprite.py` 脚本来自动生成精灵图：

**功能**：
- 使用 PIL/Pillow 库绘制像素艺术风格的猴子
- 自动生成 4 帧动画
- 支持透明背景（RGBA）
- 包含详细的绘制函数（`draw_monkey_idle` 和 `draw_monkey_on_cloud`）

**使用方法**：
```bash
cd rust-diablo/assets
python create_monkey_sprite.py
```

**输出**：`sprites/player.png` (256x64 像素)

### 3. 动画系统集成

**Entity::create_player 更新**：
- 实体大小：从 32x32 更新为 64x64（匹配精灵图）
- Idle 动画：单帧（帧 1），循环播放
- Walk 动画：3 帧循环（帧 2→3→4→3），帧率 0.12 秒/帧（约 8 FPS）

**动画状态切换**：
- 静止时（`Direction::None`）→ `AnimationState::Idle` → 显示静止猴子
- 移动时（`Direction::is_moving()`）→ `AnimationState::Walk` → 显示猴子乘坐云朵动画

### 4. 渲染系统更新

**World::render 改进**：
- 使用 `AnimationController::current_frame_rect()` 获取当前动画帧
- 将动画帧的 `Rect` 作为源矩形传递给 `draw_texture_by_id`
- 正确渲染精灵表中的不同帧，而不是整个纹理

**代码示例**：
```rust
let src_rect = entity.animation
    .as_ref()
    .and_then(|anim| anim.current_frame_rect());

engine.draw_texture_by_id(sprite_id, src_rect, dst_rect)?;
```

### 5. 代码清理

移除了临时方案：
- 删除了 `game.rs` 中基于动画状态改变颜色和大小的临时代码
- 现在完全依赖精灵图系统进行视觉表现
- 代码更加清晰和可维护

### 6. 技术要点

**精灵表（Sprite Sheet）的优势**：
- **性能**：单张纹理比多张纹理更高效
- **内存**：减少纹理切换开销
- **组织**：所有动画帧集中管理

**动画帧坐标计算**：
- 帧索引从 0 开始
- X 坐标 = 帧索引 × 帧宽度
- Y 坐标 = 0（单行精灵表）

**动画循环**：
- Walk 动画使用 4 帧：2→3→4→3
- 最后一帧回到第 3 帧，形成平滑循环
- 避免从帧 4 直接跳回帧 2 的视觉跳跃

### 7. 未来扩展

这个系统为后续功能打下了基础：
- **方向动画**：可以为不同方向（8 方向）添加不同的精灵帧
- **攻击动画**：添加 `AnimationState::Attack` 的精灵帧
- **受击动画**：添加 `AnimationState::Hit` 的精灵帧
- **死亡动画**：添加 `AnimationState::Death` 的精灵帧

### 8. 参考资源

- **精灵图生成**：`rust-diablo/assets/create_monkey_sprite.py`
- **实体定义**：`rust-diablo/src/entity/mod.rs::create_player()`
- **渲染逻辑**：`rust-diablo/src/world/mod.rs::render()`
- **动画系统**：`rust-diablo/src/sprite/animation.rs`

## 📚 练习任务

完成 Step 4.2 的核心功能后，你可以尝试以下练习来加深理解和提升技能：

### 🎯 基础练习（推荐先完成）

#### 练习 1：滑墙效果（Wall Sliding）
**目标**：改进碰撞响应，让玩家在碰到墙时能够沿墙滑动，而不是完全停止。

**任务**：
1. 修改 `CollisionMap::validate_move` 方法
2. 当检测到碰撞时，尝试分别检查 X 和 Y 方向的移动
3. 如果 X 方向可行走，允许 X 方向移动；如果 Y 方向可行走，允许 Y 方向移动
4. 这样玩家在斜向撞墙时，可以沿墙滑动

**提示**：
- 当前代码已经有部分实现（检查 X 和 Y 方向），但可以优化
- 参考原版代码：`Source/player.cpp::PlrDoWalk()` 中的碰撞处理

**预期效果**：
- 玩家斜向撞墙时，可以沿墙滑动
- 移动感觉更自然流畅

---

#### 练习 2：相机平滑跟随（Camera Smoothing）
**目标**：实现相机的平滑跟随，而不是瞬间跳转。

**任务**：
1. 在 `Camera` 结构中添加 `target_position: Point` 字段
2. 修改 `follow` 方法，设置目标位置而不是直接设置位置
3. 实现 `update` 方法，使用线性插值（Lerp）平滑移动相机
4. 在 `Game::update` 中调用 `camera.update(dt)`

**提示**：
- 使用公式：`new_pos = current_pos + (target_pos - current_pos) * lerp_factor`
- `lerp_factor` 可以是一个常量（如 0.1）或基于 `dt` 计算
- 参考：`Source/engine/render/scrollrt.cpp` 中的相机平滑逻辑

**预期效果**：
- 相机跟随更平滑，不会突然跳转
- 提供更好的视觉体验

---

#### 练习 3：添加更多瓦片类型
**目标**：扩展瓦片系统，添加新的瓦片类型和对应的图片。

**任务**：
1. 在 `TileType` 枚举中添加新类型（如 `Stone`, `Sand`, `Lava` 等）
2. 在 `color()` 方法中添加对应的颜色
3. 在 `sprite_id()` 方法中添加对应的纹理ID
4. 在 `is_walkable()` 中设置行走性
5. 创建或下载对应的图片资源（32x32 像素）
6. 在 `Game::new()` 中加载新纹理
7. 在 `World::new()` 的地图生成中使用新瓦片类型

**提示**：
- 可以使用在线资源（如 OpenGameArt.org）获取免费瓦片图片
- 确保图片大小是 32x32 像素（或 tile_size）
- 考虑不同瓦片的游戏性（哪些可行走，哪些不可行走）

**预期效果**：
- 地图更加多样化
- 视觉效果更丰富

---

#### 练习 4：自定义猴子精灵图动画
**目标**：修改猴子精灵图，添加更多动画帧或改进现有动画。

**任务**：
1. 修改 `assets/create_monkey_sprite.py` 脚本
2. 添加新的动画帧（如跳跃、攻击等）
3. 或者改进现有动画（更流畅的云朵移动、猴子表情变化等）
4. 更新 `Entity::create_player` 中的动画帧坐标
5. 添加新的 `AnimationState`（如 `Jump`, `Attack` 等）

**提示**：
- 精灵表可以扩展到 8 帧或更多（256x128 或更大）
- 可以参考像素艺术教程改进绘制效果
- 确保动画帧的坐标计算正确
- 参考技术文档：[精灵表系统技术详解](../tech_key_points/sprite-sheet-system.md)

**预期效果**：
- 更丰富的玩家动画
- 更好的视觉表现

---

### 🚀 进阶练习（挑战性）

#### 练习 5：相机边界缓冲（Camera Dead Zone）
**目标**：实现相机"死区"，玩家在屏幕中心一定区域内移动时，相机不跟随。

**任务**：
1. 在 `Camera` 中添加 `dead_zone: Rect` 字段
2. 修改 `follow` 方法：
   - 计算玩家相对于屏幕中心的位置
   - 如果玩家在死区内，相机不移动
   - 如果玩家超出死区，相机跟随
3. 添加 `set_dead_zone` 方法设置死区大小

**提示**：
- 死区通常是一个以屏幕中心为原点的矩形
- 参考：许多平台游戏（如《超级马里奥》）使用这种技术
- 可以让玩家在屏幕中心自由移动，只有接近边缘时才移动相机

**预期效果**：
- 玩家在屏幕中心移动时，相机保持静止
- 提供更稳定的视野

---

#### 练习 6：碰撞检测优化（Spatial Partitioning）
**目标**：优化碰撞检测性能，使用空间划分技术。

**任务**：
1. 实现简单的网格划分（Grid-based spatial partitioning）
2. 将地图划分为多个网格单元
3. 只检测实体所在网格及其相邻网格的碰撞
4. 在 `CollisionMap` 中添加网格索引

**提示**：
- 网格大小可以是 tile_size 的倍数（如 4x4 瓦片）
- 使用 `HashMap<(i32, i32), Vec<Entity>>` 存储网格实体
- 参考：游戏开发中的空间数据结构（Spatial Data Structures）

**预期效果**：
- 大型地图上的碰撞检测性能提升
- 为后续大量实体（怪物、物品）做准备

---

#### 练习 7：相机震动效果（Camera Shake）
**目标**：实现相机震动效果，用于受击、爆炸等场景。

**任务**：
1. 在 `Camera` 中添加震动相关字段：
   - `shake_intensity: f32` - 震动强度
   - `shake_duration: f32` - 震动持续时间
   - `shake_elapsed: f32` - 已过时间
2. 实现 `shake` 方法，启动震动
3. 在 `update` 方法中应用随机偏移
4. 震动随时间衰减

**提示**：
- 使用随机数生成器（`rand` crate）生成随机偏移
- 震动强度随时间衰减：`intensity = max_intensity * (1.0 - elapsed / duration)`
- 在 `world_to_screen` 中应用偏移

**预期效果**：
- 受击时相机震动，增强打击感
- 提升游戏体验

---

### 🎨 创意练习（自由发挥）

#### 练习 8：迷你地图（Minimap）
**目标**：在屏幕角落显示迷你地图。

**任务**：
1. 创建一个 `Minimap` 结构
2. 在屏幕右上角绘制小地图
3. 显示已探索的区域
4. 显示玩家位置（用不同颜色标记）
5. 可以按 Tab 键切换显示/隐藏

**提示**：
- 使用 `Engine::draw_rect` 绘制小地图
- 可以记录哪些瓦片已被探索
- 参考：`Source/automap.cpp` 中的自动地图系统

---

#### 练习 9：动态障碍物
**目标**：实现可以移动或破坏的障碍物。

**任务**：
1. 创建 `Obstacle` 实体类型
2. 实现障碍物的移动（如推箱子）
3. 实现障碍物的破坏（如可破坏的墙）
4. 更新碰撞检测，考虑动态障碍物

**提示**：
- 障碍物也是 `Entity`，可以添加到 `World::entities`
- 需要检查实体之间的碰撞
- 参考：`Source/objects.cpp` 中的游戏对象系统

---

#### 练习 10：8方向精灵图动画
**目标**：为猴子精灵图添加8个方向的动画。

**任务**：
1. 扩展精灵表为多行布局（8行，每行一个方向）
2. 修改 `create_monkey_sprite.py` 生成8方向的精灵图
3. 在 `Direction` 和 `AnimationState` 之间建立映射
4. 根据移动方向选择对应的动画帧行
5. 更新渲染系统以支持多行动画

**提示**：
- 精灵表布局：`[N][NE][E][SE]` 第一行，`[S][SW][W][NW]` 第二行
- 或者使用 8 行布局，每行一个方向
- 参考：`Source/engine/direction.hpp` 和原版 Diablo 的方向系统
- 参考技术文档：[精灵表系统技术详解](../tech_key_points/sprite-sheet-system.md)

**预期效果**：
- 玩家移动时显示对应方向的动画
- 更真实的移动效果

---

### 🔥 高挑战性练习（专家级）

#### 练习 11：完整A*路径查找算法
**目标**：实现完整的A*路径查找算法，用于怪物AI和玩家点击移动。

**任务**：
1. 创建 `src/engine/pathfinding.rs` 模块
2. 实现A*算法的核心组件：
   - `Node` 结构（位置、g值、h值、f值、父节点）
   - `OpenSet` 和 `ClosedSet`（使用优先队列和哈希表）
   - `heuristic` 函数（曼哈顿距离或欧几里得距离）
   - `reconstruct_path` 函数（从目标节点回溯路径）
3. 实现 `find_path` 函数：
   - 输入：起点、终点、碰撞地图
   - 输出：路径点列表或方向序列
4. 支持8方向移动（包括对角线）
5. 处理对角线移动的额外成本（如原版代码中的 `PathDiagonalStepCost = 101`）

**参考原版代码**：
- `Source/engine/path.cpp` - A*算法实现
- `Source/engine/path.h` - 路径查找接口
- 原版使用 `PathAxisAlignedStepCost = 100` 和 `PathDiagonalStepCost = 101`

**算法要点**：
- **A*算法**：`f(n) = g(n) + h(n)`
  - `g(n)`：从起点到当前节点的实际成本
  - `h(n)`：从当前节点到终点的启发式估计（曼哈顿距离）
  - `f(n)`：节点的总估计成本
- **优先队列**：使用二叉堆（Binary Heap）维护开放集
- **路径重构**：从目标节点回溯到起点

**预期效果**：
- 可以计算任意两点之间的最短路径
- 为后续怪物AI和点击移动功能打下基础

---

#### 练习 12：点击移动系统（Click-to-Move）
**目标**：实现点击地面移动，玩家自动寻路到目标位置。

**任务**：
1. 在 `Game` 中添加鼠标点击处理
2. 将屏幕坐标转换为世界坐标
3. 使用A*算法计算路径
4. 实现路径跟随系统：
   - 存储路径点队列
   - 每帧移动到下一个路径点
   - 到达路径点后移除并继续下一个
5. 处理路径中断（如被障碍物阻挡）

**参考原版代码**：
- `Source/controls/plrctrls.cpp` - 玩家控制
- `Source/player.cpp::PlrDoWalk()` - 玩家移动

**预期效果**：
- 点击地面，玩家自动寻路移动
- 路径显示（可选：用线条显示路径）

---

#### 练习 13：动态路径重规划（Dynamic Path Replanning）
**目标**：当路径被动态障碍物阻挡时，自动重新规划路径。

**任务**：
1. 在路径跟随过程中检测障碍物
2. 如果当前路径被阻挡，触发重新规划
3. 从当前位置重新计算到目标的路径
4. 优化：只重新规划剩余路径，而不是整条路径

**算法挑战**：
- 如何高效检测路径是否被阻挡？
- 何时触发重规划？（每帧检查 vs 事件驱动）
- 如何处理频繁重规划的性能问题？

**预期效果**：
- 玩家移动时，如果路径被阻挡，自动绕路
- 提供流畅的移动体验

---

### 🧮 算法练习题板块

以下练习题将经典算法与游戏开发相结合，既锻炼算法能力，又解决实际问题。

#### 练习 14：Dijkstra算法 - 多目标路径查找
**目标**：使用Dijkstra算法找到到多个目标点的最短路径。

**任务**：
1. 实现Dijkstra算法（单源最短路径）
2. 找到从起点到所有可行走瓦片的最短距离
3. 应用场景：
   - 找到最近的物品
   - 找到最近的敌人
   - 找到最近的出口
4. 可视化：在地图上用颜色深浅表示距离

**算法要点**：
- Dijkstra算法：从起点开始，逐步扩展到所有可达节点
- 使用优先队列维护待处理节点
- 时间复杂度：O(V log V + E)，其中V是节点数，E是边数

**参考原版代码**：
- `Source/engine/path.cpp` - 路径查找相关

**预期效果**：
- 可以快速找到最近的物品或敌人
- 为游戏AI提供距离信息

---

#### 练习 15：Flood Fill算法 - 区域连通性检测
**目标**：使用Flood Fill算法检测地图区域的连通性。

**任务**：
1. 实现Flood Fill算法（洪水填充）
2. 检测地图中所有连通区域
3. 为每个区域分配唯一ID
4. 应用场景：
   - 验证地图生成是否连通
   - 检测房间是否被墙壁完全隔离
   - 找到所有可到达的区域

**算法要点**：
- Flood Fill：从起点开始，递归或迭代填充所有连通节点
- 可以使用深度优先搜索（DFS）或广度优先搜索（BFS）
- 标记已访问节点，避免重复处理

**预期效果**：
- 可以检测地图是否连通
- 为地图生成算法提供验证工具

---

#### 练习 16：Bresenham算法 - 视线检测（Line of Sight）
**目标**：使用Bresenham直线算法实现视线检测。

**任务**：
1. 实现Bresenham直线算法
2. 从起点到终点绘制直线，检查路径上的所有瓦片
3. 如果路径上有不可行走的瓦片，则视线被阻挡
4. 应用场景：
   - 怪物是否能看到玩家
   - 玩家是否能看到目标
   - 远程攻击的视线检查

**算法要点**：
- Bresenham算法：高效绘制直线，只使用整数运算
- 逐像素检查路径上的瓦片
- 时间复杂度：O(max(|dx|, |dy|))

**参考原版代码**：
- `Source/vision.cpp` - 视线检测系统

**预期效果**：
- 可以检测两点之间是否有视线
- 为怪物AI和战斗系统提供基础

---

#### 练习 17：JPS算法 - 跳点搜索（Jump Point Search）
**目标**：实现JPS算法，优化A*在网格地图上的性能。

**任务**：
1. 研究JPS算法原理
2. 实现跳点识别逻辑
3. 在A*基础上集成JPS优化
4. 性能对比：JPS vs 标准A*

**算法挑战**：
- JPS是A*的优化版本，专门针对网格地图
- 通过"跳点"跳过不必要的节点
- 可以显著减少搜索节点数量

**学习价值**：
- 理解算法优化的思路
- 学习如何针对特定场景优化算法

**预期效果**：
- 路径查找性能提升（减少搜索节点）
- 理解算法优化的实际应用

---

#### 练习 18：流场寻路（Flow Field Pathfinding）
**目标**：实现流场寻路算法，用于大量单位的移动。

**任务**：
1. 为地图生成流场（Flow Field）
2. 流场存储每个瓦片到目标的方向
3. 实体只需查询流场即可移动，无需单独计算路径
4. 应用场景：
   - 大量怪物同时寻路
   - 群体移动优化

**算法要点**：
- 流场：预计算所有瓦片到目标的方向
- 使用Dijkstra算法生成流场
- 实体移动时只需查询流场，O(1)复杂度

**预期效果**：
- 支持大量实体同时寻路
- 性能优于为每个实体单独计算路径

---

#### 练习 19：分层路径查找（Hierarchical Pathfinding）
**目标**：实现分层路径查找，优化大型地图的寻路性能。

**任务**：
1. 将地图划分为多个区域（如房间）
2. 第一层：区域之间的路径
3. 第二层：区域内部的详细路径
4. 结合两层路径得到完整路径

**算法要点**：
- 分层抽象：将大问题分解为小问题
- 区域抽象：将地图划分为多个子图
- 路径组合：将粗粒度路径细化为详细路径

**预期效果**：
- 大型地图上的寻路性能提升
- 理解分层抽象的设计思想

---

#### 练习 20：势场寻路（Potential Field Pathfinding）
**目标**：实现基于势场的寻路算法，用于动态环境。

**任务**：
1. 为地图生成势场（目标点为正，障碍物为负）
2. 实体沿着势场梯度移动
3. 动态更新势场（障碍物移动时）
4. 应用场景：
   - 动态障碍物环境
   - 群体避障

**算法要点**：
- 势场：每个位置有一个势能值
- 梯度下降：沿着势能下降最快的方向移动
- 动态更新：障碍物移动时重新计算势场

**预期效果**：
- 支持动态环境的寻路
- 自然的避障行为

---

### 🎨 图形学专属练习题板块

以下练习题专注于图形学技术，结合原版 Diablo 的渲染系统，深入理解游戏图形渲染原理。

#### 练习 21：2D光照系统（Lighting System）
**目标**：实现2D光照系统，为游戏添加动态光照效果。

**任务**：
1. 创建 `src/renderer/lighting.rs` 模块
2. 实现光照数据结构：
   - `Light` 结构（位置、半径、强度、颜色）
   - `Lightmap` 结构（光照贴图缓冲区）
3. 实现光照计算：
   - 点光源光照（距离衰减）
   - 光照贴图生成
   - 光照混合（多个光源叠加）
4. 在渲染时应用光照：
   - 将光照贴图与精灵纹理混合
   - 支持完全黑暗、部分光照、完全光照三种模式

**参考原版代码**：
- `Source/lighting.cpp` - 光照系统主文件
- `Source/engine/render/light_render.cpp` - 光照渲染实现
- `Source/lighting.h` - 光照系统接口
- 原版使用 `LightType` 枚举：`FullyDark`, `PartiallyLit`, `FullyLit`, `PerPixel`

**图形学要点**：
- **光照衰减**：`intensity = base_intensity / (1 + distance^2)`
- **光照贴图**：预计算每个像素的光照强度
- **颜色混合**：`final_color = sprite_color * light_intensity`

**预期效果**：
- 玩家周围有光照范围
- 远离光源的区域变暗
- 多个光源可以叠加

---

#### 练习 22：基于模式的阴影系统（Pattern-Based Shadows）
**目标**：实现基于模式的阴影系统，为地图添加静态阴影。

**任务**：
1. 研究原版的阴影模式系统
2. 实现阴影模式匹配：
   - 定义阴影模式结构（2x2瓦片模式）
   - 实现模式匹配算法
   - 根据周围瓦片类型应用阴影
3. 在地图生成后应用阴影：
   - 遍历所有瓦片
   - 匹配阴影模式
   - 替换为阴影瓦片

**参考原版代码**：
- `Source/levels/drlg_l1.cpp::ApplyShadowsPatterns()` - 第一层阴影
- `Source/levels/drlg_l2.cpp::ApplyShadowsPatterns()` - 第二层阴影
- `Source/levels/crypt.cpp::ApplyCryptShadowsPatterns()` - 地牢阴影

**图形学要点**：
- **模式匹配**：使用2x2瓦片窗口检测阴影模式
- **阴影类型**：不同瓦片类型产生不同阴影
- **预处理**：在地图生成时一次性应用阴影

**预期效果**：
- 墙壁和障碍物产生阴影
- 阴影增强空间深度感
- 地图看起来更有层次

---

#### 练习 23：三角形光栅化（Triangle Rasterization）
**目标**：实现三角形光栅化算法，用于光照渲染。

**任务**：
1. 实现三角形光栅化算法：
   - 使用扫描线算法（Scanline Algorithm）
   - 支持半空间方法（Half-Space Method）
   - 处理三角形填充
2. 应用场景：
   - 光照区域的三角形渲染
   - 等轴测视图的光照计算
3. 优化：
   - 使用整数运算避免浮点误差
   - 边界处理（避免越界）

**参考原版代码**：
- `Source/engine/render/light_render.cpp::RenderTriangle()` - 三角形渲染
- 原版使用半空间方法绘制三角形
- 参考：https://web.archive.org/web/20050408192410/http://sw-shader.sourceforge.net/rasterizer.html

**图形学要点**：
- **扫描线算法**：从上到下逐行填充
- **边函数**：使用边函数判断点是否在三角形内
- **插值**：在三角形内插值颜色或纹理坐标

**预期效果**：
- 可以渲染任意三角形
- 为光照系统提供基础渲染能力

---

#### 练习 24：视线系统（Line of Sight / Vision System）
**目标**：实现视线系统，用于视野计算和光照遮挡。

**任务**：
1. 实现视线检测算法：
   - 使用光线追踪（Ray Casting）
   - 支持8方向视线检测
   - 处理对角线遮挡
2. 实现视野计算：
   - 从观察点发射多条光线
   - 检测光线是否被阻挡
   - 标记可见区域
3. 应用场景：
   - 玩家视野范围
   - 怪物视野检测
   - 光照遮挡计算

**参考原版代码**：
- `Source/vision.cpp` - 视线系统实现
- `Source/engine/vision.hpp` - 视线系统接口
- 原版使用预计算的视线射线表（VisionRays）

**图形学要点**：
- **光线追踪**：从起点到终点逐像素检测
- **遮挡检测**：检查路径上的瓦片是否阻挡视线
- **对角线处理**：需要检查对角线相邻瓦片

**预期效果**：
- 可以计算任意两点的视线
- 为游戏AI和光照系统提供基础

---

#### 练习 25：粒子系统（Particle System）
**目标**：实现2D粒子系统，用于视觉效果（火焰、烟雾、魔法效果等）。

**任务**：
1. 创建 `src/renderer/particle.rs` 模块
2. 实现粒子数据结构：
   - `Particle` 结构（位置、速度、生命周期、颜色、大小）
   - `ParticleEmitter` 结构（发射器位置、发射率、粒子类型）
3. 实现粒子更新：
   - 物理模拟（重力、速度衰减）
   - 生命周期管理
   - 颜色渐变
4. 实现粒子渲染：
   - 批量渲染优化
   - 支持不同粒子形状（点、圆、精灵）

**参考原版代码**：
- `Source/missiles.cpp` - 投射物系统（类似粒子）
- `Source/objects.cpp` - 对象动画（火焰效果）

**图形学要点**：
- **粒子更新**：`position += velocity * dt`
- **生命周期**：`alpha = 1.0 - (age / lifetime)`
- **批量渲染**：使用纹理图集减少绘制调用

**预期效果**：
- 可以创建火焰、烟雾等视觉效果
- 为战斗系统提供视觉反馈

---

#### 练习 26：后处理效果（Post-Processing Effects）
**目标**：实现后处理效果系统，为游戏添加视觉增强。

**任务**：
1. 实现渲染目标（Render Target）：
   - 创建离屏渲染缓冲区
   - 将场景渲染到纹理
2. 实现后处理效果：
   - **模糊效果**（Blur）：高斯模糊
   - **颜色调整**（Color Grading）：亮度、对比度、饱和度
   - **屏幕抖动**（Screen Shake）：相机震动效果
   - **淡入淡出**（Fade）：场景切换效果
3. 应用后处理：
   - 将处理后的纹理渲染到屏幕

**图形学要点**：
- **高斯模糊**：使用卷积核进行图像模糊
- **颜色调整**：`output = (input - 0.5) * contrast + 0.5 + brightness`
- **双缓冲**：使用两个纹理交替处理

**预期效果**：
- 可以添加各种视觉特效
- 提升游戏画面质量

---

#### 练习 27：纹理混合（Texture Blending）
**目标**：实现多种纹理混合模式，用于复杂视觉效果。

**任务**：
1. 实现混合模式：
   - **Alpha混合**：`result = src * alpha + dst * (1 - alpha)`
   - **加法混合**：`result = src + dst`（用于发光效果）
   - **乘法混合**：`result = src * dst`（用于阴影）
   - **屏幕混合**：`result = 1 - (1 - src) * (1 - dst)`
2. 应用场景：
   - 光照与纹理混合
   - 阴影渲染
   - 魔法效果叠加

**图形学要点**：
- **混合方程**：`C_final = C_src * F_src + C_dst * F_dst`
- **混合因子**：不同的混合模式使用不同的因子
- **性能**：使用硬件混合加速

**预期效果**：
- 可以实现复杂的视觉效果
- 提升渲染质量

---

#### 练习 28：视差滚动（Parallax Scrolling）
**目标**：实现视差滚动效果，增强画面深度感。

**任务**：
1. 实现多层背景系统：
   - 定义多个背景层（前景、中景、背景）
   - 每层有不同的滚动速度
2. 实现视差计算：
   - 根据相机位置计算各层偏移
   - `offset = camera_pos * parallax_factor`
3. 渲染多层背景：
   - 从后到前依次渲染
   - 保持正确的深度顺序

**图形学要点**：
- **视差因子**：距离越远的层，滚动速度越慢
- **深度感**：通过不同速度创造3D错觉
- **无缝循环**：背景需要无缝拼接

**预期效果**：
- 背景有深度感
- 增强游戏的视觉沉浸感

---

#### 练习 29：等轴测投影（Isometric Projection）
**目标**：实现等轴测投影，将3D坐标转换为2D屏幕坐标。

**任务**：
1. 实现等轴测变换：
   - 3D到2D坐标转换
   - `screen_x = (world_x - world_y) * tile_width / 2`
   - `screen_y = (world_x + world_y) * tile_height / 2 + world_z * height_factor`
2. 实现深度排序：
   - 根据等轴测坐标排序渲染顺序
   - 确保前面的物体遮挡后面的物体
3. 应用场景：
   - 等轴测地图渲染
   - 3D物体在2D屏幕上的投影

**参考原版代码**：
- `Source/engine/render/scrollrt.cpp` - 等轴测渲染
- 原版 Diablo 使用等轴测视图

**图形学要点**：
- **投影矩阵**：等轴测投影的数学变换
- **深度排序**：使用Z-order或画家算法
- **瓦片对齐**：确保瓦片正确对齐

**预期效果**：
- 可以实现等轴测视图
- 为原版 Diablo 风格的渲染做准备

---

#### 练习 30：调色板动画（Palette Animation）
**目标**：实现调色板动画，通过改变调色板实现颜色循环效果。

**任务**：
1. 实现调色板系统：
   - `Palette` 结构（256色RGB值）
   - 调色板加载和切换
2. 实现调色板动画：
   - 颜色循环（Color Cycling）
   - 颜色渐变（Color Fade）
   - 颜色闪烁（Color Flash）
3. 应用场景：
   - 魔法效果颜色变化
   - 水面波动效果
   - 火焰动画

**参考原版代码**：
- `Source/engine/palette.cpp` - 调色板系统
- 原版使用256色调色板

**图形学要点**：
- **调色板索引**：使用8位索引而非24位颜色
- **颜色循环**：周期性改变调色板中的颜色
- **性能**：调色板动画比纹理动画更高效

**预期效果**：
- 可以实现流畅的颜色动画
- 为原版资源格式支持做准备

---

### ❓ 问答题板块

以下问答题帮助你深入理解系统设计原理和代码实现细节，培养系统思维和代码分析能力。

#### 问答题 1：为什么使用浮点位置（position_f）而不是直接使用整数位置？
**问题**：在 `Entity` 结构中，我们同时维护了 `position: Point`（整数）和 `position_f: (f32, f32)`（浮点）。为什么需要两个位置字段？

**思考方向**：
1. 高帧率下的移动精度问题
2. 速度与帧率的关系
3. 子像素移动的累积
4. 碰撞检测的精度要求

**参考答案要点**：
- 在高帧率（如 144 FPS）下，每帧移动距离可能小于 1 像素
- 如果只使用整数位置，小数部分会被截断，导致移动卡顿
- `position_f` 累积小数移动，`position` 用于碰撞检测和渲染
- 参考：[子像素移动累积丢失问题](bugfix-subpixel-movement-accumulation.md)

---

#### 问答题 2：为什么动画系统使用状态机而不是简单的帧索引？
**问题**：为什么使用 `AnimationController` 管理多个 `AnimationState`，而不是直接用数组索引切换动画？

**思考方向**：
1. 状态转换的复杂性
2. 动画切换的平滑性
3. 代码的可维护性
4. 未来扩展性（攻击、受击、死亡等状态）

**参考答案要点**：
- 不同动画状态有不同的帧序列和播放逻辑
- 状态机可以处理状态转换规则（如：Idle → Walk，Walk → Idle）
- 便于添加新状态而不影响现有代码
- 参考：`src/sprite/animation.rs::AnimationController`

---

#### 问答题 3：碰撞检测为什么先检查 X 和 Y 方向，而不是直接检查目标位置？
**问题**：在 `CollisionMap::validate_move` 中，为什么分别检查 X 方向和 Y 方向的移动，而不是直接检查目标位置？

**思考方向**：
1. 斜向移动的碰撞处理
2. 滑墙效果（Wall Sliding）
3. 移动的平滑性
4. 玩家体验

**参考答案要点**：
- 斜向撞墙时，允许沿墙滑动（只允许一个方向的移动）
- 分别检查可以保留部分移动，而不是完全停止
- 提供更自然的移动体验
- 参考：`src/world/collision.rs::validate_move`

---

#### 问答题 4：相机系统为什么需要边界限制（bounds）？
**问题**：`Camera` 结构中有 `bounds: Option<Rect>` 字段，为什么需要限制相机的移动范围？

**思考方向**：
1. 地图边界处理
2. 防止相机超出地图范围
3. 小地图的相机行为
4. 用户体验

**参考答案要点**：
- 防止相机跟随玩家移动到地图外，显示空白区域
- 在小地图上，相机应该限制在可见区域内
- 提供更好的视觉体验
- 参考：`src/renderer/camera.rs::set_bounds`

---

#### 问答题 5：为什么精灵表（Sprite Sheet）比单独图片文件更高效？
**问题**：为什么将所有动画帧放在一张大图片中，而不是每个帧一个文件？

**思考方向**：
1. 纹理切换的开销
2. 内存使用
3. 加载时间
4. GPU 性能

**参考答案要点**：
- 减少纹理切换（Texture Binding）次数，这是 GPU 的昂贵操作
- 单张大纹理比多张小纹理占用更少内存（减少纹理元数据）
- 只需加载一次，减少磁盘 I/O
- 参考：[精灵表系统技术详解](../tech_key_points/sprite-sheet-system.md)

---

#### 问答题 6：为什么使用 Delta Time（dt）而不是固定帧数？
**问题**：为什么在 `Entity::update` 中使用 `dt`（时间差）计算移动，而不是假设固定 60 FPS？

**思考方向**：
1. 帧率独立性（Frame Rate Independence）
2. 不同硬件的性能差异
3. 游戏速度的一致性
4. 调试和测试

**参考答案要点**：
- 确保游戏在不同帧率下运行速度一致
- 高帧率设备不会运行更快，低帧率设备不会运行更慢
- 便于测试和调试（可以模拟不同帧率）
- 公式：`distance = speed * dt`

---

#### 问答题 7：为什么方向系统需要归一化（Normalization）？
**问题**：在 `Direction::to_unit_vector` 中，为什么需要将向量归一化为单位向量？

**思考方向**：
1. 对角线移动的速度问题
2. 速度一致性
3. 数学原理
4. 游戏体验

**参考答案要点**：
- 对角线移动的向量长度是 `√2 ≈ 1.414`，比轴向移动快
- 归一化后，所有方向的移动速度一致
- 公式：`unit_vector = vector / length(vector)`
- 参考：`src/engine/direction.rs::to_unit_vector`

---

#### 问答题 8：视口剔除（Viewport Culling）为什么重要？
**问题**：为什么在渲染前检查 `camera.is_rect_visible`，而不是直接渲染所有实体？

**思考方向**：
1. 性能优化
2. 渲染调用次数
3. 大型地图的处理
4. GPU 负载

**参考答案要点**：
- 只渲染相机范围内的实体，减少不必要的绘制调用
- 在大型地图上，可能有数百个实体，但只有少数在视野内
- 减少 CPU 和 GPU 的工作量
- 参考：`src/world/mod.rs::render`

---

### 🔬 前沿研究课题板块

以下练习题结合工业界和学术界的前沿研究，拓展视野并了解最新技术发展。

#### 研究课题 1：实体组件系统（ECS）架构重构
**目标**：研究现代游戏引擎的 ECS 架构，重构当前的实体系统。

**任务**：
1. 研究 ECS 架构原理：
   - Entity（实体）：唯一标识符
   - Component（组件）：数据容器
   - System（系统）：处理逻辑
2. 分析当前 `Entity` 结构的局限性
3. 设计 ECS 架构方案
4. 实现原型并对比性能

**前沿技术**：
- **EnTT**（C++ ECS 库）：https://github.com/skypjack/entt
- **Bevy**（Rust 游戏引擎）：https://bevyengine.org/
- **Unity DOTS**：Unity 的数据导向技术栈

**参考文献**：
1. Sander, T. (2014). "Data-Oriented Design". Game Engine Architecture.
2. Fabian, S. (2019). "EnTT: A Fast and Reliable Entity-Component-System". GitHub.
3. Bevy Contributors. (2020). "Bevy: A data-driven game engine built in Rust". bevyengine.org.

**学习价值**：
- 理解现代游戏引擎架构
- 学习数据导向设计（DOD）
- 性能优化思路

---

#### 研究课题 2：延迟渲染（Deferred Rendering）在 2D 游戏中的应用
**目标**：研究延迟渲染技术，优化多光源渲染性能。

**任务**：
1. 研究延迟渲染原理：
   - G-Buffer（几何缓冲区）
   - 光照计算阶段
   - 最终合成
2. 分析 2D 游戏中的延迟渲染优势
3. 实现简单的延迟渲染管线
4. 性能对比：前向渲染 vs 延迟渲染

**前沿技术**：
- **G-Buffer 优化**：减少缓冲区大小
- **Tile-Based Deferred Rendering**：分块延迟渲染
- **Light Culling**：光源剔除

**参考文献**：
1. Hargreaves, S., & Harris, M. (2004). "Deferred Shading". GPU Gems.
2. Lauritzen, A. (2007). "Deferred Rendering for Current and Future Rendering Pipelines". GPU Gems 3.
3. Olsson, O., & Assarsson, U. (2011). "Tiled Shading". Journal of Graphics Tools.

**学习价值**：
- 理解现代渲染管线
- 学习 GPU 优化技术
- 为多光源系统做准备

---

#### 研究课题 3：程序化内容生成（PCG）在地图生成中的应用
**目标**：研究高级 PCG 算法，改进当前的地图生成系统。

**任务**：
1. 研究 PCG 算法：
   - **Wave Function Collapse**：波函数坍缩
   - **L-Systems**：Lindenmayer 系统
   - **Cellular Automata**：细胞自动机
2. 分析原版 Diablo 的地图生成算法
3. 实现一种 PCG 算法原型
4. 对比生成质量和性能

**前沿技术**：
- **Wave Function Collapse**：https://github.com/mxgmn/WaveFunctionCollapse
- **Procedural Dungeon Generation**：程序化地牢生成
- **Constraint-Based Generation**：基于约束的生成

**参考文献**：
1. Gumin, M. (2016). "Wave Function Collapse Algorithm". GitHub.
2. Togelius, J., et al. (2011). "Procedural Content Generation in Games: A Textbook and an Overview of Current Research". FDG.
3. Shaker, N., et al. (2016). "Procedural Content Generation in Games". Springer.

**学习价值**：
- 理解程序化生成原理
- 学习约束满足问题（CSP）
- 为地图系统扩展做准备

---

#### 研究课题 4：机器学习在游戏 AI 中的应用
**目标**：研究机器学习技术，实现智能怪物 AI。

**任务**：
1. 研究游戏 AI 技术：
   - **Behavior Trees**：行为树
   - **State Machines**：状态机
   - **Reinforcement Learning**：强化学习
2. 分析原版 Diablo 的怪物 AI
3. 实现简单的行为树系统
4. 探索强化学习在游戏中的应用

**前沿技术**：
- **Behavior Trees**：https://www.gamasutra.com/blogs/ChrisSimpson/20140717/221339/Behavior_trees_for_AI_How_they_work.php
- **Unity ML-Agents**：Unity 机器学习代理
- **OpenAI Gym**：强化学习环境

**参考文献**：
1. Champandard, A. J. (2007). "Behavior Trees for Next-Gen Game AI". AiGameDev.com.
2. Julian, T., et al. (2018). "Unity ML-Agents: Machine Learning for Game Development". Unity Technologies.
3. Mnih, V., et al. (2015). "Human-level control through deep reinforcement learning". Nature.

**学习价值**：
- 理解现代 AI 技术
- 学习行为树设计
- 探索机器学习应用

---

#### 研究课题 5：实时全局光照（Real-Time Global Illumination）
**目标**：研究实时全局光照技术，实现更真实的光照效果。

**任务**：
1. 研究全局光照技术：
   - **Light Propagation Volumes (LPV)**：光传播体积
   - **Voxel Cone Tracing**：体素锥追踪
   - **Screen-Space Global Illumination (SSGI)**：屏幕空间全局光照
2. 分析 2D 游戏中的全局光照简化方案
3. 实现简单的全局光照原型
4. 性能优化和效果对比

**前沿技术**：
- **LPV**：用于实时全局光照
- **SSGI**：屏幕空间技术
- **Ray Tracing**：光线追踪（硬件加速）

**参考文献**：
1. Kaplanyan, A., et al. (2010). "Cascaded Light Propagation Volumes for Real-Time Indirect Illumination". I3D.
2. Ritschel, T., et al. (2009). "Imperfect Shadow Maps for Efficient Computation of Indirect Illumination". SIGGRAPH.
3. McGuire, M., & Mara, M. (2014). "Efficient GPU Screen-Space Ray Tracing". Journal of Computer Graphics Techniques.

**学习价值**：
- 理解高级光照技术
- 学习实时渲染优化
- 为光照系统扩展做准备

---

#### 研究课题 6：网络同步与预测（Network Synchronization & Prediction）
**目标**：研究多人游戏的网络同步技术。

**任务**：
1. 研究网络同步技术：
   - **Client-Side Prediction**：客户端预测
   - **Server Reconciliation**：服务器协调
   - **Lag Compensation**：延迟补偿
2. 分析原版 Diablo 的多人游戏实现
3. 设计网络同步方案
4. 实现简单的多人游戏原型

**前沿技术**：
- **Deterministic Lockstep**：确定性锁步
- **Rollback Netcode**：回滚网络代码
- **GGPO**：Good Game, Peace Out（网络库）

**参考文献**：
1. Smed, J., & Hakonen, H. (2006). "Algorithms and Networking for Computer Games". Wiley.
2. Fiedler, G. (2001). "What Every Programmer Needs to Know About Game Networking". Gaffer On Games.
3. Rollback Netcode. (2020). "Rollback Netcode Explained". YouTube.

**学习价值**：
- 理解网络游戏架构
- 学习延迟处理技术
- 为多人游戏功能做准备

---

## 📝 练习提交指南

## 📝 练习提交指南

完成练习后，你可以：

1. **代码审查**：将你的实现代码分享给我，我会进行代码审查并提供改进建议
2. **问题讨论**：如果遇到问题，可以随时询问，我会提供提示和指导
3. **功能扩展**：如果你有更好的想法，可以自由发挥，我会协助实现

**练习难度说明**：
- 🎯 **基础练习**：适合初学者，帮助理解核心概念
- 🚀 **进阶练习**：需要更深入的理解，适合有一定经验的开发者
- 🎨 **创意练习**：自由发挥，可以结合自己的想法扩展功能
- 🔥 **高挑战性练习**：专家级难度，需要深入理解算法和系统设计
- 🧮 **算法练习题**：结合经典算法与游戏开发，锻炼算法能力
- 🎨 **图形学练习题**：专注于图形学技术，深入理解游戏渲染原理
- ❓ **问答题**：培养系统思维和代码分析能力，深入理解设计原理
- 🔬 **前沿研究课题**：结合工业界和学术界最新研究，拓展技术视野

**推荐练习顺序**：
1. **入门阶段**：先完成基础练习 1-4，巩固核心概念
2. **提升阶段**：然后尝试进阶练习 5-7，提升技能
3. **创意阶段**：挑战创意练习 8-10，发挥创造力
4. **专家阶段**：完成高挑战性练习 11-13，深入系统设计
5. **算法阶段**：挑战算法练习题 14-20，结合算法与游戏开发
6. **图形学阶段**：挑战图形学练习题 21-30，深入图形渲染技术
7. **思考阶段**：完成问答题，深入理解系统设计原理
8. **研究阶段**：探索前沿研究课题，了解最新技术发展

**算法练习题学习路径**：
- **入门算法**：练习 14（Dijkstra）、练习 15（Flood Fill）、练习 16（Bresenham）
- **进阶算法**：练习 17（JPS）、练习 18（流场寻路）
- **高级算法**：练习 19（分层路径查找）、练习 20（势场寻路）

## 🔜 下一步计划 (Step 5)

完成练习后，我们将进入 **Step 5: 原版资源格式完整支持**。这是连接 Rust 实现与原版游戏数据的关键步骤。

*   **MPQ 归档读取**：实现读取 `DIABDAT.MPQ`。
*   **调色板系统**：支持 256 色调色板和 CLUT (Color Lookup Table)。
*   **CEL/CL2 精灵渲染**：解析并渲染 Diablo 原始的精灵格式。
*   这将使我们的游戏画面从现在的"彩色方块"进化为真正的 Diablo 画面。

