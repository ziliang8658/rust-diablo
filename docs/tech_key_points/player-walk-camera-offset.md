# 玩家行走回退问题：方向、偏移和相机必须闭合

## 1. 现象

当前玩家按方向键移动时，看起来会先往目标方向走几帧，然后又被拉回屏幕中心。这个现象容易被误判成“行走动画倒放”或“帧序错误”，但真正的问题在坐标闭合：逻辑目标格、行走像素偏移、地图渲染相机偏移没有使用同一套规则。

最小观察方式：

- 运行 `cargo run`。
- 按住一个方向键，例如 `D` 或右方向键。
- 玩家 sprite 会先偏出中心；一步提交后，`tile_position` 更新、行走偏移清零，画面看起来像回退。

## 2. 原版锚点

这次修复只对齐玩家“一步行走”和屏幕偏移，不实现完整寻路或 `dPlayer` 正负索引排序。

关键原版锚点：

- `Source/player.cpp:93` `WalkSettings`：8 个方向决定玩家走向和 walk mode。
- `Source/player.cpp:125` `HandleWalkMode()`：开始一步移动时设置 `_pdir`、`position.future`、`position.temp`。
- `Source/player.cpp:404` `DoWalk()`：动画到最后一帧后才提交 `position.tile`。
- `Source/engine/displacement.hpp:247` `Displacement::fromDirection()`：Diablo 的方向不是普通屏幕上下左右，而是等距 tile 坐标位移。
- `Source/engine/render/scrollrt.cpp:1081` `CalcFirstTilePosition()`：本地玩家行走时，地图渲染会应用反向 camera offset。
- `Source/engine/render/scrollrt.cpp:1473` `GetOffsetForWalking()`：行走中的像素偏移来自 8 方向表和动画进度。

## 3. 根因

这次 bug 有三层叠加。

第一层是方向位移错误。Rust 里原来的 `Direction::to_tile_offset()` 用的是普通 2D 方格：

```rust
Direction::East => (1, 0)
Direction::North => (0, -1)
```

但 DevilutionX 的等距方向是：

```rust
Direction::East => (1, -1)
Direction::North => (-1, -1)
```

这样一来，动画偏移表说“East 应该在屏幕上横向走 64 像素”，逻辑目标格却只移动到另一套坐标里的 `(x + 1, y)`，提交时自然会跳。

第二层是实体渲染投影不一致。`World::render_entities()` 使用过 `engine::isometric::world_to_screen()`，它和当前 tile renderer 的行列推进方向符号相反。玩家自己因为相对玩家中心是 `(0, 0)`，这个问题不总是明显；一旦要比较“行走偏移”和“目标 tile 投影”，方向就对不上。

第三层是相机没有跟着格间偏移。Rust 当前做了 `player.walking_render_offset()`，让玩家 sprite 在行走中偏出中心；但是地图渲染仍停在旧 `tile_position`。等动画完成后，`tile_position` 提交到目标格，视图中心跳到新格，而 sprite 偏移清零，于是视觉上像玩家走出去后又回来了。

## 4. Rust 修复映射

修复后的规则是让三个量闭合：

```text
方向 -> tile offset -> target tile
方向 + progress -> walking_render_offset
target tile 相对 start tile 的投影 == walking_render_offset(progress=1.0)
```

具体改动：

- `src/engine/direction.rs`：`to_tile_offset()` 改成 DevilutionX 等距方向位移，和 `Source/engine/displacement.hpp::fromDirection()` 对齐。
- `src/world/mod.rs`：实体等距投影改用当前 tile renderer 同向的 `world_to_screen()`。
- `src/world/mod.rs`：地图渲染时对第一个实体，也就是当前玩家，应用 `-walking_render_offset()` 作为 camera offset。
- `src/world/mod.rs`：等距模式下本地玩家实体不再额外加自己的 walking offset，避免“地图也在动、玩家也在动”的双重位移。

这样在一步走到末尾时：

```text
提交前：view = start tile, camera offset = -target_projection
提交后：view = target tile, camera offset = 0
```

两者在屏幕上的地图位置相同，所以不会再出现提交瞬间的回退。

## 5. 西向移动时的边缘补画

另一个容易被误判成“相机移动太快”的现象，是玩家向西、向西北、向西南行走时，窗口边缘露出锯齿状黑三角。SDL 每帧都会清屏，所以这不是旧帧没刷新，而是行走中的视口已经把地图往反方向推开，但 tile 渲染范围没有补画即将滑入屏幕的那一圈菱形边缘。

DevilutionX 在 `Source/engine/render/scrollrt.cpp::CalcFirstTilePosition()` 里专门处理这个问题：

- `North` / `NorthEast`：首 tile 往北补一格，屏幕 offset 再上移 32 像素。
- `West` / `SouthWest`：首 tile 往西补一格，屏幕 offset 再左移 64 像素。
- `NorthWest`：首 tile 往西北补一格，屏幕 offset 再左移 32、上移 16 像素。

随后 `DrawGame()` 还会按方向增加 `columns` / `rows`，让补进来的 tile 真正进入 floor 和 wall 两个渲染阶段。Rust 现在把这套规则收敛到 `walking_viewport_expansion()`：它同时返回 `tile_shift`、`screen_offset`、`extra_columns` 和 `extra_rows`。这样修复点不会散落在 floor 循环和 wall 循环里，也能用单元测试直接对齐原版方向表。

## 6. 房屋边缘只剩 floor 的补画

房屋或连续墙处在窗口边缘时，有时 floor 已经进入可见列，但墙体所属的相邻 tile 还没有按正确顺序进入 wall/content pass，于是画面看起来像“地板有了，房屋边缘没了”。这不是 SOL 把墙误判成 floor，而是 `DrawTileContent()` 的扫描顺序缺了一段原版特殊处理。

DevilutionX 在 `Source/engine/render/scrollrt.cpp::DrawTileContent()` 里有一个 x 轴连续墙 gate：

- 当前 tile 必须是墙。
- 当前 tile 的水平邻居之一也必须是墙，说明它属于一段 x 轴连续墙。
- `tile + (1, -1)` 和 `tile + (0, -1)` 必须是非 solid，说明墙后面有可走区域。
- 当前屏幕位置的 `x + TILE_WIDTH` 仍在视口内。

满足这些条件时，原版会先画 `tilePosition + Direction::East`，然后把 `skipNext` 设为 true。下一轮扫描到这个东侧 tile 时会跳过，避免重复绘制。这个顺序保证了边缘墙体、墙后对象和移动中的角色不会因为 tile 边界刚好卡在视口边缘而漏画或穿出墙面。

Rust 现在用 `should_predraw_east_wall_content()` 复刻这段 gate，并在 wall/content pass 中维护 `skip` / `skip_next`。真正绘制时走 `draw_tile_content_at()`，也就是同一个入口同时负责 cell 和该格上的实体，避免 floor pass、wall pass、entity pass 三套顺序互相打架。

这次进一步把 Rust 的基础视口几何改为对齐原版 `CalcViewportGeometry()`：先根据玩家在视口中心的位置算 `tilesToTop` / `tilesToLeft`，再应用原版“从短行开始渲染”的半行对齐分支，最后由 `startPosition` 推出 `tileOffset`，并从 `renderStart` 反推 `tileRows` / `tileColumns`。

这个改动解决的是另一类边缘漏画：按 `W` 进入房屋边缘时，floor pass 可能已经画到了可见地面，但高墙/房屋上层真正所属的 base tile 仍因为起始半行不准而没有进入 wall/content pass。原版不会靠“粗略 rows/columns + 多扫几格”兜底，而是先把第一行、屏幕 offset 和扫描网格相位算准，再让 `DrawTileContent()` 额外 `rows += MicroTileLen`，保证上层 micro tiles 能从屏幕外延伸进来。

Rust 当前仍保留 wall/content pass 的一列 overscan，作为学习阶段对实体网格、对象层和完整 panel/zoom 几何尚未复刻前的保守保护；但 floor 和 wall/content 的基础起点、offset、rows/columns 已经改成来自同一个 `calc_viewport_geometry()`，不再是两套不一致的估算。

## 7. 当前简化和后续债务

这次修复仍然是学习型最小闭环，不等于完整复刻原版移动渲染。

- 还没有实现原版 `dPlayer` 正负索引对南向、东向移动玩家的绘制排序。
- 还没有把光源偏移做到原版 `ChangeLightOffset()` 那种格间位置。
- 还没有实现点击寻路和路径队列，只处理键盘触发的一步一格移动。
- 还没有复刻 `AnimationDistributionFlags` 和固定 game tick 的全部细节，当前仍用 `dt` 推进行走动画。

## 8. 验证

自动测试重点验证两个不依赖 MPQ 的规则：

- `Direction::to_tile_offset()` 的 8 方向位移必须和 DevilutionX 等距方向一致。
- `walking_render_offset(1.0)` 必须等于目标 tile 相对起点的屏幕投影。
- `calc_viewport_geometry()` 必须对齐原版 `CalcViewportGeometry()` 的关键数值，例如 640x480 下 `tile_shift=(-13,-4)`、`tile_offset=(0,-17)`、`columns=10`、`rows=33`。
- `walking_viewport_expansion()` 必须覆盖原版 `CalcFirstTilePosition()` / `DrawGame()` 对 8 个方向的视口补画规则。
- `wall_content_scan_padding()` 必须确认 wall/content pass 左右各多扫一列，作为学习阶段的保守补画保护。
- `should_predraw_east_wall_content()` 必须覆盖原版 `DrawTileContent()` 对 x 轴连续墙、背后可走区域和右边界的 gate 条件。

手动验证建议：

- `cargo run`
- 在主场景中按住 8 个方向移动。
- 观察每一步结束时，玩家不应再出现“走出去后回退”的跳变。
- 重点观察西、西北、西南移动时，窗口边缘不应再出现锯齿状未渲染黑三角。
- 按 `W` 让房屋或连续墙进入窗口上方/斜向边缘，观察不应再出现“floor 有、墙体或房屋边缘缺失”的断裂；再按 `S` 走回时，房屋也不应出现重新弹回来的感觉。
- 如果缺少 `Diabdat.mpq` 或 CL2 资源，只能验证 fallback 行为；真实战士动画需要本机资源存在。
