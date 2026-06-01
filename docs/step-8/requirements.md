# Step 8 遮挡关系需求文档

## 1. 本 Step 要解决什么

当前 Rust 复刻已经有一部分遮挡相关能力，但还没有形成完整的 Diablo 1 式“按格扫描 + 地牢内容 + 实体/物品/对象穿插绘制”的遮挡关系。

本 Step 的目标是补齐地牢场景中的可验证遮挡语义：玩家、怪物、物品、对象、尸体、飞弹和墙体内容必须按照原版 `DrawDungeon()` / `DrawTileContent()` 的顺序进入同一条格子扫描渲染路径，而不是在地牢渲染结束后统一叠加。

本需求文档只定义“要做什么”和“验收什么”。实现前还需要补齐原版调研、Rust 设计和测试计划。

## 2. 当前分支评估

结论：本分支不是完全没有遮挡关系，但目前只实现了局部能力，还不能算完整遮挡系统。

### 已有能力

- `rust-diablo/src/world/mod.rs:1826` 附近已有 `wall_predraw` 判断，会参考原版墙后预绘制逻辑，把东侧墙后的内容提前画出来，避免移动 sprite 从墙边露出。
- `rust-diablo/src/world/mod.rs:1598` 附近的 `draw_tile_content_at()` 会先 `draw_cell_at()`，再调用 `render_entities_at_dpiece()`，说明地牢 wall/content pass 中已经能在格子扫描时插入实体绘制。
- `rust-diablo/src/world/mod.rs:1419` 的 `entity_render_order_tile()` 已经为向 South / East 方向移动的实体使用 target tile 做绘制排序，这对应原版移动实体正负索引的一部分意图。
- `rust-diablo/src/world/mod.rs:510` 附近已有测试覆盖 South / East 使用 target tile、North / West 保持 source tile 的最小排序规则。
- `rust-diablo/src/renderer/policy.rs:30` 避免在地牢墙体开启时走 `EntityOverlay`，说明代码已经意识到实体不应该简单叠在所有墙体之后。

### 仍然缺失

- 还没有 Rust 侧等价的 `dPlayer` / `dMonster` / `dObject` / `dItem` / `dCorpse` / missile occupancy 语义；实体列表只是遍历 `self.entities`，缺少按格索引和正负索引含义。
- `render_entities_at_dpiece()` 目前只处理通用 `Entity`，没有区分玩家、怪物、物品、对象、尸体、飞弹的原版前后绘制顺序。
- 对象还没有 `_oPreFlag` / post draw 语义，物品也没有 `_iPostDraw` 语义。
- 当前排序主要覆盖玩家/实体行走时的 source/target 锚点，尚未覆盖对象遮挡玩家、玩家遮挡物品、墙体遮挡移动怪物、飞弹前后层等组合。
- TownPreview 仍在 `Game::render_town()` 中单独画背景和玩家，不属于本 Step 的地牢遮挡路径。

## 3. 原版 C++ 行为锚点

后续调研文档必须至少覆盖这些锚点：

| 行为 | C++ 锚点 | 需求含义 |
| --- | --- | --- |
| 整体扫描 | `Source/engine/render/scrollrt.cpp:1132` `DrawGame()` | 先画 floor，再画 tile content；实体不是最后统一叠加。 |
| 地牢内容扫描 | `Source/engine/render/scrollrt.cpp:966` `DrawTileContent()` | 按可见格扫描，每个格调用 `DrawDungeon()`，并处理墙后预绘制。 |
| 墙后预绘制 | `Source/engine/render/scrollrt.cpp:985` 附近 | 遇到 x 轴墙且墙后可走时，先画墙后的格，再跳过正常扫描中的下一格。 |
| 单格内容顺序 | `Source/engine/render/scrollrt.cpp:753` `DrawDungeon()` | 每个格内部有固定顺序：cell、pre missile、corpse、pre object、pre item、dead player、player、monster、post missile、post object、post item。 |
| 玩家绘制 | `Source/engine/render/scrollrt.cpp:440` `DrawPlayer()` | 玩家 sprite 用当前动画帧、行走偏移和 tile center 计算屏幕位置。 |
| 移动玩家锚点 | `Source/engine/render/scrollrt.cpp:808` 附近 | South / East 类移动用特殊正负 `dPlayer` 标记和临时 tile/screen offset 保证绘制顺序。 |
| 怪物绘制 | `Source/engine/render/scrollrt.cpp:365` `DrawMonster()` 和 `Source/engine/render/scrollrt.cpp:836` 附近 | 怪物也有移动锚点和 tile occupancy 语义。 |
| 对象绘制 | `Source/engine/render/scrollrt.cpp:496` `DrawObject()` | 对象根据 `_oPreFlag` 分成玩家/怪物前后两个绘制阶段。 |
| 占格语义 | `Source/player.h:886`、`Source/monster.cpp:4981`、`Source/objects.cpp:736` | 原版用格子数组表达“这个格有哪些可绘制内容”和移动中的正负索引。 |

## 4. 目标

### 4.1 行为目标

- 地牢渲染必须能在同一条 tile content 扫描中绘制墙体内容和实体内容。
- 玩家、怪物、对象、物品、尸体和飞弹必须有明确的格子归属。
- 同一格中多类内容的绘制顺序必须对齐原版 `DrawDungeon()` 的语义。
- 移动中的玩家和怪物必须有 source tile、target tile、render anchor 三者的明确关系。
- South / SouthWest / SouthEast / East 这类会影响遮挡优先级的移动方向必须使用原版等价的 target-priority 规则。
- 墙后预绘制必须继续可用，并且不能因为新增对象/怪物/物品路径而退化。

### 4.2 视觉目标

- 玩家走到墙后时，sprite 不应从墙边或墙体透明区错误露出。
- 玩家走到对象后方时，对象能遮挡玩家；玩家走到对象前方时，玩家能遮挡对象。
- 怪物和玩家相邻移动时，靠屏幕下方/前方的一方应遮挡后方的一方。
- 物品、尸体和飞弹的前后层不应全部简单压在玩家上方或下方。
- 开关墙体层、实体层、wall predraw 调试开关时，差异应该可解释且可定位。

### 4.3 学习目标

- 文档必须解释 Diablo 1 为什么不用通用 z-buffer，而是用 tile scan order 和 occupancy arrays 得到稳定遮挡。
- 文档必须解释正负 `dPlayer` / `dMonster` 索引解决了什么问题，以及 Rust 侧如何等价表达。
- 文档必须把“墙后预绘制”标为遮挡系统中的局部补丁，而不是完整排序方案本身。

## 5. 非目标

- 不修改 C++ 主线，只以 `Source/` 作为行为参考。
- 不实现现代 3D z-buffer、任意多边形深度测试或自由场景图排序。
- 不在本 Step 重新设计资源解码、调色板、光照或 CL2/CLX 帧格式。
- 不要求一次性实现完整战斗、AI、掉落或对象交互；这些系统只需要提供遮挡所需的最小渲染数据。
- 不把 TownPreview 的背景图渲染纳入本 Step，除非后续明确要复刻 town 地图的按格遮挡。

## 6. 功能需求

### FR-1 必须建立格子占用/绘制索引

- Rust 侧必须有一个明确结构表达每个 dPiece/world tile 上的可绘制内容。
- 至少要能表示 player、monster、object、item、corpse、missile 的存在与索引。
- 移动中的 player/monster 必须能表达 source tile 和 target tile，不能只靠 `Entity::tile_position` 推断。
- 如果暂时不完全复刻 `dPlayer` / `dMonster` 的正负整数编码，设计文档必须说明 Rust 等价结构如何覆盖同样语义。

### FR-2 地牢实体必须嵌入 tile content pass

- 当地牢地图存在且 wall/content layer 开启时，实体不能走全屏 `EntityOverlay` 兜底路径。
- 每个格子必须在 `draw_cell_at()` 后按原版顺序绘制这个格的实体内容。
- `RenderPolicy` 的 pass 选择必须继续防止地牢实体被最后统一叠加。
- 对非地牢调试场景，可以保留 `EntityOverlay`。

### FR-3 单格内部顺序必须可测试

- 单格内容顺序至少要拆成这些阶段：cell、pre missile、corpse、pre object、pre item、dead player、player、monster、post missile、post object、post item。
- Rust 可以先用空实现或测试替身覆盖尚未有实际资源的阶段，但顺序结构必须稳定。
- 每个阶段必须能被单元测试记录和断言，不能只靠肉眼截图判断。

### FR-4 对象和物品必须支持前后绘制标记

- Object 必须能表达类似原版 `_oPreFlag` 的 pre/post draw 语义。
- Item 必须能表达类似原版 `_iPostDraw` 的 pre/post draw 语义。
- 如果当前还没有正式 Object/Item 类型，本 Step 可以先引入最小渲染描述结构，但不得把它混进通用 `EntityType` 后失去阶段语义。

### FR-5 移动 actor 的排序锚点必须对齐原版

- 玩家和怪物向 South / SouthWest / SouthEast / East 移动时，绘制顺序必须使用 target tile priority。
- 玩家和怪物向 North / NorthWest / NorthEast / West 移动时，默认保持 source tile priority，除非原版调研证明需要调整。
- render anchor 必须能从 scan tile 反推到 sprite 应该出现的屏幕位置，避免 target-priority 导致 sprite 瞬移。
- 当前 `entity_render_order_tile()` 的测试可以保留，但需要扩展到 player 和 monster 两类 actor。

### FR-6 墙后预绘制必须保持兼容

- `wall_predraw` 逻辑仍应只在原版相同条件下触发：x 轴墙、墙后可走、目标格在视口范围内。
- 预绘制 behind tile 时，必须绘制该 tile 的完整 content，包括对象/玩家/怪物/物品/飞弹等阶段。
- 正常扫描到被预绘制的 tile 时必须跳过，避免重复绘制。
- 必须有测试证明新增实体阶段不会破坏已有 `skip_next` 行为。

### FR-7 调试和验证必须可控

- 新增遮挡诊断日志必须通过 debug flag 或 trace mode 控制，默认不刷屏。
- 可以增加 focus 2x2 / selected tile 的遮挡阶段 trace，输出 tile、stage、entity id、anchor 和 screen position。
- 截图或 dump 输出不得写入提交的大型资源目录。

## 7. 验收标准

- `cargo test` 通过。
- 新增测试覆盖单格绘制阶段顺序。
- 新增测试覆盖移动 player/monster 在 South/East 类方向使用 target-priority，在 North/West 类方向保持 source-priority。
- 新增测试覆盖 wall predraw 触发时 behind tile content 先绘制、正常扫描跳过。
- 在有 MPQ 资源的手动验证环境中，玩家在墙后移动不会从墙边穿出。
- 在测试对象前后移动时，玩家与对象的遮挡关系随 tile 前后关系变化。
- 缺少 MPQ 或 SDL2 资源时，普通单元测试仍然可以运行；依赖真实资源的验证必须标为手动或 ignored。

## 8. 文档交付要求

实现前必须补齐：

- `original-research.md`：按原版调用链解释 `DrawGame()`、`DrawTileContent()`、`DrawDungeon()`、移动玩家/怪物正负索引、对象/物品 pre/post draw。
- `design.md`：说明 Rust 侧 occupancy、render stage、actor anchor、wall predraw 和现有 `Entity`/`World` 的边界。
- `test-plan.md`：列出单元测试、集成测试、截图验证和资源依赖。

实现完成并验证后必须补齐：

- `teaching.md`：用中文解释从“统一叠加 sprite”演进到“按格穿插绘制”的原因、步骤和常见错误。

## 9. 风险和约束

- 这是高风险渲染顺序改动，必须小步推进，优先加测试替身再接真实资源。
- 当前 Step 7 已经涉及玩家移动和方向动画，遮挡实现必须避免破坏已有 walking offset、directional animation 和 camera offset 行为。
- 如果 occupancy 结构设计过早绑定完整游戏逻辑，后续对象、怪物、物品系统会被渲染层反向限制；设计文档必须保持渲染数据和玩法状态的边界。
- 原版 line number 可能随上游变动，调研文档应同时记录函数名和关键代码片段语义。

## 10. 最直接的下一步

先写 `original-research.md`。重点不要泛泛讨论 painter's algorithm，而是沿着真实调用链追踪一帧地牢画面：

1. `DrawGame()` 如何分 floor 和 tile content 两阶段。
2. `DrawTileContent()` 如何扫描格子并处理墙后预绘制。
3. `DrawDungeon()` 如何安排一个格内的 cell、飞弹、尸体、对象、物品、玩家、怪物。
4. 移动玩家/怪物为什么需要 source/target tile 和正负索引。
5. Rust 当前 `render_with_texture_manager()`、`draw_tile_content_at()`、`render_entities_at_dpiece()` 已经对应到哪一步，还缺哪一步。
