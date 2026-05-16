# Step 7 玩家移动的八方向动画系统需求文档

## 1. 本 Step 要解决什么

当前 Rust 复刻已经能用 WASD/方向键让玩家在地图上移动，也已经能加载 Idle/Walk 动画帧。但现在的行为还不像 Diablo 1：玩家行走动画没有按 8 个方向拆开，移动和动画之间也没有形成“开始走、在两个格子之间移动、动画完成后落到目标格”的稳定关系。

Step 7 的目标是把“玩家移动”和“玩家 8 方向动画”绑成一个可验证的小系统：按下某个方向后，玩家面向这个方向，播放这个方向的行走帧，在渲染上从当前格平滑走向目标格，走完后停在目标格并保持最后面向。

本需求只定义“要做什么”和“验收什么”。原版调研文档、Rust 设计文档、测试文档完成后，才能进入实现。

## 2. 当前状态

### Rust 侧已有基础

- `rust-diablo/src/engine/direction.rs` 已有 `Direction`，支持 8 方向输入、单位速度、tile offset 和基础测试。
- `rust-diablo/src/game.rs` 已经在 `Game::update()` 中把键盘输入合成为 8 方向，并调用 `player.start_walk(...)`。
- `rust-diablo/src/entity/mod.rs` 已有 `Entity::walking`、`walk_direction`、`start_walk()` 和 `update()`，但目前行走完成逻辑仍是简化版。
- `rust-diablo/src/entity/mod.rs` 和 `rust-diablo/src/game.rs` 已经为玩家配置 Idle/Walk 动画，但 Walk 仍更像一个扁平帧序列，不是按方向选择的帧组。

### 目前不够像原版的地方

- 方向输入和动画帧组之间缺少明确映射，容易出现“向东北走却播放南向帧”的问题。
- 当前移动状态只记录 `walking` 和 `walk_direction`，缺少明确的起点、目标格、行走进度和渲染偏移。
- `Entity::update()` 里还保留“先简化为立即移动/等待后续动画系统”的 TODO，Step 7 需要把这个口径收紧。
- 渲染位置主要来自 `tile_position`，缺少类似原版 `GetOffsetForWalking()` 的格间像素偏移。

## 3. 原版 C++ 行为锚点

本 Step 必须以 DevilutionX 当前源码为原版行为来源，至少覆盖下列锚点：

| 行为 | C++ 锚点 | 需求含义 |
| --- | --- | --- |
| 玩家方向和行走模式映射 | `Source/player.cpp:93` `WalkSettings` | 8 个方向不只是位移，还会映射到不同的 walk mode。 |
| 开始行走 | `Source/player.cpp:144` `StartWalkAnimation()` 和 `Source/player.cpp:157` `StartWalk()` | 开始一步移动时先启动 Walk 动画，再进入行走模式。 |
| 锁定目标格 | `Source/player.cpp:125` `HandleWalkMode()` | 设置 `_pdir`、`position.future`、`position.temp` 和 `_pmode`。 |
| 行走提交时机 | `Source/player.cpp:404` `DoWalk()` | 动画未到最后一帧时不提交格子；到最后一帧才把 `position.tile` 改成目标格。 |
| 方向精灵选择 | `Source/player.cpp:2218` `NewPlrAnim()` 和 `Source/player.h:195` `spritesForDirection()` | 玩家动画数据按 8 个方向取对应精灵列表。 |
| 当前帧选择 | `Source/engine/animationinfo.cpp:18` `getFrameToUseForRendering()` | 渲染帧可能考虑跳帧和分布逻辑；当前 Step 可先做简化，但要保留接口空间。 |
| 行走进度 | `Source/engine/animationinfo.cpp:62` `getAnimationProgress()` | 行走偏移来自动画进度，而不是随便按时间猜。 |
| 行走像素偏移 | `Source/engine/render/scrollrt.cpp:1473` `GetOffsetForWalking()` | 原版用 8 方向 offset 表把玩家画在两个格子之间。 |
| 玩家绘制 | `Source/engine/render/scrollrt.cpp:440` `DrawPlayer()` 和 `Source/player.h:784` `getRenderingOffset()` | 绘制时取当前 sprite，再叠加行走偏移和居中偏移。 |
| 按格渲染中的移动玩家 | `Source/engine/render/scrollrt.cpp:799` 的 `PlayerAtPosition()` 分支 | 原版会处理移动中玩家的正负 `dPlayer` 标记和南/东向绘制锚点。当前 Step 可简化，但必须明确差异。 |

## 4. 目标

### 4.1 玩法目标

- 玩家可以使用 WASD 或方向键触发 8 个方向的移动：North、NorthEast、East、SouthEast、South、SouthWest、West、NorthWest。
- 玩家移动时播放对应方向的 Walk 动画，不再把所有 Walk 帧当成同一个方向的连续帧。
- 玩家停止移动时切回 Idle，并保持最后一次有效移动方向作为面向。
- 当目标格不可走时，玩家不进入行走状态，不播放错误方向的行走动画，也不改变 `tile_position`。

### 4.2 视觉目标

- 行走过程中玩家在屏幕上从起点格向目标格移动，而不是逻辑上瞬移到目标格后再播放动画。
- 行走偏移必须使用明确的 8 方向 offset 表，方向含义要和原版锚点对齐。
- Idle/Walk 切换不能出现明显的首帧错向、方向闪烁或帧段越界。
- 缺少 MPQ 或玩家 CL2 资源时，程序仍能用 fallback 显示并给出清楚提示，不影响基础测试运行。

### 4.3 学习目标

- 读者能从这个 Step 学会：Diablo 1 为什么把“方向、动画、格子提交、渲染偏移”绑在一起。
- 文档要明确说明“原版按固定 game tick 推进，Rust 当前用 `dt` 推进”的差异。
- 实现后需要补一份教学文档，用最小可运行实验展示从单方向动画到 8 方向动画的演进。

## 5. 非目标

- 不实现怪物、NPC、战斗、攻击、施法、受击、格挡、死亡等其他角色动画。
- 不实现点击寻路、路径队列、自动绕路或网络同步。
- 不重写 MPQ/CL2/CLX 解码器，只在现有解码结果上组织方向帧组。
- 不引入骨骼动画、动画混合、物理引擎或现代化动作系统；这些只能作为后期扩展记录。
- 不在本 Step 强制完全复刻 `AnimationDistributionFlags` 的所有细节，但接口和文档必须说明哪些是当前简化。
- 不解决物体遮挡、怪物遮挡或完整 `dPlayer` 正负索引渲染顺序；如果需要最小兼容，只做不破坏现有遮挡渲染的改动。

## 6. 功能需求

### FR-1 方向映射必须稳定

- 必须定义一个从 `Direction` 到 Diablo 动画方向索引的显式映射。
- 映射顺序必须参考原版 8 方向顺序：`South`、`SouthWest`、`West`、`NorthWest`、`North`、`NorthEast`、`East`、`SouthEast`。
- `Direction::None` 不能映射成有效 Walk 方向；需要 fallback 时应使用最后面向方向。
- 必须有反向映射测试，防止后续调整 enum 顺序时破坏动画索引。

### FR-2 动画控制器必须支持状态加方向

- 动画查询不能只按 `AnimationState::Walk` 查找，必须能表达 `(AnimationState::Walk, Direction::East)` 这样的组合。
- Idle 和 Walk 至少要支持玩家当前使用的资源；如果某个方向缺帧，需要有明确 fallback 策略并记录日志。
- 切换方向时必须重置或调整当前动画帧，避免继续播放上一个方向的帧段。
- 停止移动后，Idle 使用最后面向方向，而不是强行回到默认南向。

### FR-3 CL2 帧必须按方向分组

- 玩家 CL2 加载后必须能得到每个方向的帧范围或帧列表。
- 不能把 `wmnaw.cl2` 的所有帧直接扁平配置成一个 Walk 动画。
- 如果当前解码器只返回线性帧列表，设计文档必须说明如何根据原版帧数和方向数切分。
- 帧宽、高、锚点和居中规则必须继续沿用当前资源加载和渲染体系，不为了方向动画做无关重写。

### FR-4 行走状态必须有完整生命周期

- `start_walk(direction)` 成功时必须记录起点格、目标格、方向和行走进度。
- 行走中不能再次启动新的 step，除非后续设计明确支持队列；当前 Step 默认一次只走一格。
- 行走未完成时，逻辑 `tile_position` 是否保持起点格，或引入 `current_tile/target_tile` 双字段，必须在设计文档中明确。
- 行走完成时才提交目标格，并切回 Idle。
- 碰撞失败时不应进入 Walk 状态，也不应消耗当前动画状态。

### FR-5 渲染必须使用行走进度偏移

- 必须提供类似 `walking_render_offset()` 的查询能力，让渲染层取得当前行走偏移。
- 偏移计算必须参考原版 `MovingOffset[8]`，并按当前行走进度缩放。
- 行走进度范围必须明确为 `0.0..=1.0` 或等价固定点，不允许多个模块各自猜测进度含义。
- 玩家 sprite 的 tile center / draw rect 计算必须保持和现有渲染系统兼容。

### FR-6 相机和光照要有兼容口径

- 如果相机仍跟随 `tile_position`，需求文档必须说明这会产生什么视觉差异。
- 最小验收可以先只要求玩家 sprite 平滑移动；相机和光源可以在设计文档中分阶段接入。
- Step 7 不能破坏 Step 6.4 光照系统的玩家光源更新接口。

### FR-7 测试必须覆盖行为而不依赖本机资源

- 方向映射、动画状态切换、行走进度、碰撞失败都必须能用纯单元测试覆盖。
- 依赖 `Diabdat.mpq` 或玩家 CL2 的测试必须标注为手动或 ignored，不能让普通 `cargo test` 因缺资源失败。
- 手动验证必须包含 8 个方向的移动检查和 Idle 保持方向检查。

### FR-8 调试输出必须可关闭

- 允许添加方向、帧号、行走进度的调试显示或日志。
- 调试输出默认不能刷屏。
- 如果加入截图或像素对比辅助，输出路径不能指向提交大型资源或构建产物的目录。

## 7. 验收标准

- `cargo test` 通过，并新增覆盖方向映射和移动状态机的测试。
- 按 `W`、`W+D`、`D`、`S+D`、`S`、`S+A`、`A`、`W+A` 时，玩家面向和行走方向一致。
- Walk 动画按方向取帧，不能出现 8 个方向帧连续串播的现象。
- 停止按键后玩家切回 Idle，并保持最后方向。
- 目标格不可走时，玩家停在原格，动画不进入错误的 Walk 循环。
- 行走过程中渲染位置能观察到从起点到目标格的偏移，而不是只在目标格闪现。
- 没有 `Diabdat.mpq` 时，基础测试仍可运行；手动资源验证明确标注未跑或跑过的环境。

## 8. 文档交付要求

本需求文档之后，进入实现前还必须补齐：

- `step-7-player-movement-8direction-animation-original-research.md`：逐条解释 C++ 原版如何处理方向、动画、格子提交和渲染偏移。
- `step-7-player-movement-8direction-animation-design.md`：说明 Rust 侧类型、数据流、状态机、资源分组、错误处理和阶段拆分。
- `step-7-player-movement-8direction-animation-test-plan.md`：列出单元测试、集成测试、手动验证、资源依赖和未覆盖风险。

实现完成并验证后，还必须补：

- `step-7-player-movement-8direction-animation-teaching.md`：用 TinyRenderer 式写法，从一个单方向最小实验开始，逐步引入 8 方向、行走进度和原版对齐。

## 9. 风险和约束

- CL2 帧分组可能和“帧数除以 8”的直觉不完全一致，必须用原版数据结构和实际资源验证。
- 原版固定 game tick 与 Rust `dt` 更新模型不同，行走进度如果没有统一口径，容易出现动画速度和移动速度脱节。
- 如果提前改动渲染排序或 `d_player` 语义，可能影响已有遮挡渲染文档和实现；当前 Step 只做最小兼容。
- 资源缺失、SDL2、本机 MPQ 路径差异会影响手动验证，自动测试必须隔离这些依赖。
- 当前仓库已有多份 Step 6.4/Step 7 旧文档，后续文档需要明确以本 Step 7 需求为新口径，避免继续把 8 方向动画归到 Step 6.4。

## 10. 最直接的下一步

先写原版调研文档，重点不是泛泛介绍动画系统，而是沿着一条真实行走链路追踪：

1. 输入或寻路决定方向。
2. `StartWalk()` 启动 Walk 动画。
3. `HandleWalkMode()` 设置方向、目标格和 walk mode。
4. `NewPlrAnim()` 按方向选择 CL2 帧组。
5. `DoWalk()` 到最后一帧提交格子。
6. `GetOffsetForWalking()` 和 `DrawPlayer()` 把行走进度变成屏幕上的像素偏移。
