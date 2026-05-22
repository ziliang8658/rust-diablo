# Step 7 玩家移动与 8 方向动画 Rust 设计

**工作线**: `rust-diablo/`
**前置**: `step-7-player-movement-8direction-animation-requirements.md`、`step-7-player-movement-8direction-animation-original-research.md`

这份设计文档只描述 Rust 侧怎么落，不重复原版背景。

---

## 1. 设计目标

这一步要把下面三件事绑成一个最小但完整的系统:

1. 方向输入。
2. 8 方向动画。
3. 一步一格的行走生命周期。

实现后要满足:

- 方向能稳定映射到动画方向索引。
- Walk 不再是扁平帧序列，而是 `(state, direction)`。
- 行走未结束时不提交 `tile_position`。
- 渲染层能拿到当前 walking offset。

---

## 2. 现有 Rust 基线

当前仓库已经有这些基础:

- `src/engine/direction.rs` 里有 8 方向枚举和 tile 偏移。
- `src/sprite/animation.rs` 里有基础帧动画和状态切换。
- `src/entity/mod.rs` 里已经有 `walking`、`walk_direction` 和 `start_walk()`.
- `src/world/mod.rs` 里已经有实体渲染入口。
- `src/game.rs` 里已经把输入合成成 8 方向并尝试启动行走。

但这些基础还缺少三件关键事:

- 方向没有进入动画 key。
- 行走没有明确的 start/target/progress。
- 渲染没有消费 walking offset。

---

## 3. 数据结构设计

### 3.1 `Direction`

在 `src/engine/direction.rs` 里补这些能力:

- `to_walk_animation_index() -> Option<usize>`
- `from_walk_animation_index(index: usize) -> Option<Direction>`
- `walk_animation_order() -> [Direction; 8]`
- `animation_suffix() -> &'static str`

这里要保持和原版一致的顺序:

`South, SouthWest, West, NorthWest, North, NorthEast, East, SouthEast`

`Direction::None` 不能直接当成有效 walk 方向。

### 3.2 `Animation`

`src/sprite/animation.rs` 继续保留单个动画对象，但补两个能力:

- `duration()` 或等价方法，返回总时长。
- `progress()` 或等价方法，返回当前动画的 0.0..=1.0 进度。

### 3.3 `AnimationController`

控制器要从一维 key 变成二维 key:

- 旧逻辑: `AnimationState -> Animation`
- 新逻辑: `(AnimationState, Direction) -> Animation`

建议实现方式:

- 保留 `add_animation(state, animation)` 作为兼容入口，默认写到 South key。
- 新增 `add_directional_animation(state, direction, animation)`。
- 新增 `set_state_direction(state, direction)`。
- `current_frame_index()`、`current_frame_rect()`、`update()` 都按当前 state+direction 查找。

fallback 规则建议:

1. 精确命中 `(state, direction)`。
2. 回退到 `(state, South)`。
3. 再回退到同一 state 下任意已有方向。

这样就算某个方向的资源暂时缺失，也不会让系统直接崩掉。

### 3.4 `Entity`

`src/entity/mod.rs` 里建议补这些字段:

- `walk_start_tile: Option<Point>`
- `walk_target_tile: Option<Point>`
- `walk_progress: f32`
- `walk_duration: f32`

现有字段里:

- `direction` 代表当前面向。
- `walking` 代表是否处于一步行走中。
- `walk_direction` 代表当前步的方向。

### 3.5 步行偏移

建议在 `Entity` 上提供两个查询:

- `walking_render_offset()` - 给等轴测世界渲染用。
- `walking_pixel_offset(tile_size)` - 给普通 2D/平面预览用。

前者参考原版 `GetOffsetForWalking()` 的方向表，后者用于 town preview 或其他平面视图。

---

## 4. 行走状态机

### 4.1 `start_walk(direction)`

成功时要做这些事:

- 记录 `walk_start_tile = tile_position`
- 记录 `walk_target_tile = tile_position + direction`
- 记录 `walk_direction = Some(direction)`
- 归零 `walk_progress`
- 从动画控制器里切到 `Walk + direction`
- 初始化 `walk_duration`

失败时:

- 不改 `tile_position`
- 不进入 walking
- 不切错动画

### 4.2 `update(dt)`

如果 `walking == true`:

- 推进 `walk_progress`
- 更新当前 walk 动画
- 当进度到 1.0 或 walk 动画结束时，提交 `tile_position = walk_target_tile`
- 清空临时步行字段
- 切回 `Idle + current facing direction`

这一步是原版 `DoWalk()` 在 Rust 中的等价物。

### 4.3 停止移动

停止移动后，必须保留最后一次有效移动方向，不能每帧把方向重置回 `None`。

也就是说，`Game::update()` 里“没有输入”时不应该再调用一次 `set_direction(Direction::None)`。

---

## 5. 资源分组设计

### 5.1 CL2 sheet 的 Rust 表达

现在的 `Cl2Sprite` 只适合单列表达。Step 7 需要一个能表示方向 sheet 的数据结构，建议新增:

- `Cl2DirectionalSpriteSheet`

它的内部形态可以是:

- `Vec<Cl2Sprite>`
- 或 `Vec<Vec<ClxFrame>>`

重点不是名字，而是它要能表达:

- 第 0 组 South
- 第 1 组 SouthWest
- ...
- 第 7 组 SouthEast

### 5.2 纹理 ID 命名

建议纹理 ID 使用这类形态:

- `warrior_idle_south_0`
- `warrior_walk_north_east_3`

这样渲染层可以直接按 `state + direction + frame` 取纹理。

如果未来还要兼容旧的非方向纹理，可以在渲染时先试方向版，再试旧版。

---

## 6. 渲染集成

### 6.1 `world::render_entities()`

世界渲染里，实体绘制要做两件事:

- 根据 state + direction + frame 取纹理。
- 如果实体正在 walking，把 `walking_render_offset()` 加到绘制位置上。

对等轴测场景来说，偏移必须按原版 `MovingOffset[8]` 的语义来做，而不是用 `tile_position` 直接硬跳。

### 6.2 `game::render_town()`

城镇预览场景可以先保留原有渲染路径，但最好也把 walking offset 接进来，避免 town 里也出现“逻辑已经在走、画面还钉在原地”的情况。

如果城镇暂时没有方向纹理，也可以先用旧纹理做 fallback。

---

## 7. 风险控制

### 7.1 最容易出错的地方

- `Direction` 的方向顺序和原版不一致。
- 只改动画，不改行走提交时机。
- 只改逻辑，不改渲染偏移。
- 无输入时把方向清成 `None`，导致 idle 朝向丢失。
- Walk 资源仍然只按一维帧表加载，方向分组其实没生效。

### 7.2 这个 Step 的边界

这一步只做玩家行走链路，不碰:

- 怪物 AI
- 物体遮挡
- 施法/攻击/受击/死亡动画
- 完整网络同步
- 完整路径队列

---

## 8. 建议实现顺序

1. 先补 `Direction` 的方向索引和方向顺序常量。
2. 再把 `AnimationController` 改成二维 key。
3. 然后把 `Entity` 的一步行走状态机补完整。
4. 再把 `world::render_entities()` 和 `game::render_town()` 接上 offset。
5. 最后把 CL2 sheet 读取和 player 资源加载切成方向表。

---

## 9. 预期验收点

实现完成后，代码层应该能清楚回答这些问题:

- 当前朝向是什么?
- 当前在走哪一格?
- 当前 walk 走到哪一步?
- 渲染时偏移是多少?
- 当前播放的是哪一个方向的帧组?

如果这些问题都能从代码里直接查出来，Step 7 就算真正落地了。
