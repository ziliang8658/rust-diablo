# Step 7 玩家移动与 8 方向动画原版调研

**工作线**: `rust-diablo/`
**对应需求**: `step-7-player-movement-8direction-animation-requirements.md`

这份文档只做一件事: 把 DevilutionX 里“按下方向键 -> 启动行走 -> 播放对应方向动画 -> 在最后一帧提交格子 -> 渲染时带着行走偏移”这条链路拆开，变成 Rust 里能直接照着实现的材料。

---

## 1. 先看结论

原版不是“玩家先改到目标格，再播放一段 walk 动画”。它的真实顺序更像:

1. 输入或寻路决定方向。
2. `StartWalk()` 先启动 Walk 动画。
3. `HandleWalkMode()` 锁定 `_pdir`、`position.future`、`position.temp` 和 `_pmode`。
4. `NewPlrAnim()` 按方向选择对应的 8 方向 CL2 帧组。
5. `DoWalk()` 只有在最后一帧才提交 `position.tile = position.temp`。
6. `GetOffsetForWalking()` 和 `DrawPlayer()` 把“走到一半”变成屏幕上的像素偏移。

对 Rust 来说，这意味着两个核心点不能拆开做:

- 行走状态必须保存“起点格、目标格、进度、方向”。
- 动画系统必须知道“状态 + 方向”，不能只按 `Idle/Walk` 查表。

---

## 2. 原版行走链路

### 2.1 `StartWalk()` 先启动动画，再进入移动模式

`Source/player.cpp::StartWalk()` 的调用顺序很重要。它不是先改格子，而是:

- 调 `StartWalkAnimation(player, dir, pmWillBeCalled)`。
- 再调 `HandleWalkMode(player, dir)`。

`StartWalkAnimation()` 会调用 `NewPlrAnim(player, player_graphic::Walk, dir, ...)`，也就是在逻辑提交之前先切到对应方向的 Walk 动画。

### 2.2 `HandleWalkMode()` 锁定目标格和移动模式

`Source/player.cpp::HandleWalkMode()` 里有三个关键动作:

- `player._pdir = dir`
- `player.position.future = player.position.tile + dirModeParams.dir`
- `player.position.temp = player.position.tile + dirModeParams.dir`
- `player._pmode = dirModeParams.walkMode`

这里的 `WalkSettings` 不是单纯的方向表，它把 8 个方向折叠成 3 种 walk mode:

- `PM_WALK_SOUTHWARDS`
- `PM_WALK_NORTHWARDS`
- `PM_WALK_SIDEWAYS`

这就是原版为什么会在渲染里对南向、东向等情况做额外锚点修正。

### 2.3 `DoWalk()` 直到最后一帧才提交格子

`Source/player.cpp::DoWalk()` 的关键判断是:

- 如果 `AnimInfo.isLastFrame()` 还没到，就只更新子格偏移并返回 `false`。
- 只有到最后一帧，才:
  - 清掉旧格的 `dPlayer`
  - `position.tile = position.temp`
  - `occupyTile(position.tile, false)`
  - `StartStand(player, player.tempDirection)`
  - `ClearStateVariables(player)`

这说明“逻辑格子提交”与“动画播放”是绑定的，而不是独立的。

---

## 3. 动画系统原版语义

### 3.1 `PlayerAnimationData::spritesForDirection()`

`Source/player.h::PlayerAnimationData::spritesForDirection(Direction direction)` 直接用方向索引取 8 方向精灵表。也就是说，原版玩家动画不是“一条 walk 序列”，而是“同一个状态下的 8 组方向帧”。

### 3.2 `LoadPlrGFX()` 加载的是整张方向表

`Source/player.cpp::LoadPlrGFX()` 里，`LoadCl2Sheet(pszName, animationWidth)` 装载的是 sheet，不是单个线性帧列表。

它随后还会:

- 读取 `GraphicTRN`
- 读取 `ClassTRN`
- 把颜色变换应用到整个 sheet

这意味着 Rust 的 CL2 加载器如果只返回一个线性 `Vec<Frame>`，就还没有达到原版语义。

### 3.3 `NewPlrAnim()` 负责把方向和状态绑定起来

`Source/player.cpp::NewPlrAnim()` 做了三件事:

- 确保对应 graphic 的精灵表已加载。
- 根据方向取 `spritesForDirection(dir)`。
- 调 `AnimInfo.setNewAnimation(...)`。

这对应到 Rust 的设计，就是动画控制器不能只按 `AnimationState` 查表，而要能表达 `(AnimationState, Direction)`。

---

## 4. 行走进度和渲染偏移

### 4.1 `AnimationInfo::getAnimationProgress()`

`Source/engine/animationinfo.cpp::getAnimationProgress()` 返回的是一个固定点进度，原版用它来做“走到一半”的插值，而不是猜测时间。

### 4.2 `GetOffsetForWalking()` 的 8 方向表

`Source/engine/render/scrollrt.cpp::GetOffsetForWalking()` 有一个固定的 8 方向像素表:

- South
- SouthWest
- West
- NorthWest
- North
- NorthEast
- East
- SouthEast

它会把方向偏移乘上动画进度，再除以固定点基数，得到当前子格偏移。

### 4.3 `DrawPlayer()` 把“中心锚点 + 行走偏移”合在一起

`Source/engine/render/scrollrt.cpp::DrawPlayer()` 里，最终位置不是简单的 tile 坐标，而是:

- `targetBufferPosition`
- 加上 `player.getRenderingOffset(sprite)`

而 `player.getRenderingOffset()` 里又会叠加:

- `-CalculateSpriteTileCenterX(sprite.width())`
- `GetOffsetForWalking(AnimInfo, _pdir)`

这就是为什么 Rust 端不能只在逻辑上改 `tile_position`，还必须给渲染层一个明确的 walking offset。

---

## 5. Rust 当前代码和原版的差距

当前 `rust-diablo/` 里，相关基础已经有了，但还不够像原版:

- `src/engine/direction.rs` 已经有 8 方向输入和基础偏移。
- `src/sprite/animation.rs` 现在还是按 `AnimationState` 一维查表。
- `src/entity/mod.rs` 还在用“walking + walk_direction + 立即提交”的简化逻辑。
- `src/game.rs` 的输入处理会在无输入时把方向重置掉，这会破坏“停止后保持最后朝向”。
- `src/world/mod.rs` 的实体渲染还没把 walking offset 真正接进去。

所以 Step 7 的目标不是再做一个“更大一点的动画系统”，而是把这条链条完整接起来。

---

## 6. Rust 侧最直接的映射

我会按下面的等价关系实现:

| 原版概念 | Rust 对应 |
| --- | --- |
| `Direction` 8 方向顺序 | `Direction::to_animation_index()` / `from_animation_index()` |
| `PlayerAnimationData::spritesForDirection()` | `AnimationController` 中的 `(AnimationState, Direction)` 查表 |
| `position.tile / future / temp` | `tile_position + walk_start_tile + walk_target_tile` |
| `DoWalk()` 最后一帧提交 | `Entity::update()` 在 walk 动画结束时提交目标格 |
| `GetOffsetForWalking()` | `Entity::walking_render_offset()` |
| `DrawPlayer()` | `world::render_entities()` 和 `game::render_town()` 的最终绘制位置 |

---

## 7. 本 Step 里我不会碰的东西

- 不做怪物、物体、施法、战斗动画。
- 不改完整的 pathfinding。
- 不重写整个 CL2/CLX 资源体系，只补“方向 sheet”的最小读取路径。
- 不把相机、光照、网络同步一起重构掉。

---

## 8. 直接可用的实现锚点

如果要落代码，最关键的 C++ 锚点是:

- `Source/player.cpp::StartWalk()`
- `Source/player.cpp::HandleWalkMode()`
- `Source/player.cpp::DoWalk()`
- `Source/player.cpp::NewPlrAnim()`
- `Source/engine/animationinfo.cpp::getAnimationProgress()`
- `Source/engine/render/scrollrt.cpp::GetOffsetForWalking()`
- `Source/engine/render/scrollrt.cpp::DrawPlayer()`
- `Source/player.h::PlayerAnimationData::spritesForDirection()`

这份 Step 的 Rust 实现应该逐条能对上这些锚点，而不是只对上“看起来能走”。
