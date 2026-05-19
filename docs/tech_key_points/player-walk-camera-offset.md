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

## 7. camera offset 开启后仍闪烁的补充修正

这次回看后，闪烁点不在 `GetOffsetForWalking(..., cameraMode=true)` 的符号上。原版 camera mode 会把角色 walking offset 取反，Rust 用 `offset -= player.walking_render_offset()`，方向是对的。

真正危险的是两个阶段性实现细节：

第一，F11 关闭 walking camera offset 时，Rust 仍然因为 `player.walking` 进入 walking viewport expansion。这样“关掉 camera offset”并不是回到静态视口，而是只关掉像素 offset、保留移动扫描窗口，排查时会出现两种几何混在一起的假象。修正后，walking expansion 只在 `walking_camera_offset` 开启时应用。

第二，walking camera offset 会让扫描窗口多扫到边缘 tile。Rust 原来对 out-of-bounds floor tile 调用 `draw_black_tile()`，但这个函数画的是 64x32 黑色矩形，不是原版 tile mask，也不是软件 framebuffer 的安全清边。矩形会覆盖相邻合法菱形 tile，移动时看起来像黑块闪烁。修正后，越界 tile 不再主动画黑矩形；每帧开头的 `engine.clear()` 已经负责黑色背景，合法 tile 只需要正常覆盖进来。

第三，Rust 曾经额外加入一层全局 walking overscan：把起始 tile 往 north+west 推，再把屏幕 offset 往左上推，并增加 rows/columns。它能增加覆盖范围，但也会改变整张图的扫描起点和 wall/content 绘制顺序；在 camera offset 从 0 开始滚动时，这种顺序变化会比原版更容易表现成抖动。修正后移除这层全局 overscan，只保留原版方向相关的 `CalcFirstTilePosition()` / `DrawGame()` expansion，以及 wall/content pass 的窄 padding。

由于手动验证仍报告启动默认路径会抖动，`walking_camera_offset` 暂时回到默认关闭。F11 仍可打开这条路径，用于继续对照原版；只有在手动验证稳定后才应再次改回默认开启。

## 8. 原版轻微抖动 vs Rust 放大抖动

进一步对照后，Rust 的严重抖动不应只归因于 tile mask 或 camera offset 的符号。原版确实也会在格间滚动时有轻微闪动，但它的输入、逻辑 tick、动画进度和渲染插值是一套固定节奏；Rust 现在把这些点更多地绑在当前渲染帧的 `dt` 上，所以同样的像素级合成误差会被放大。

关键差异如下：

- 原版默认 gameplay tick 是 20Hz，`gnTickDelay = 1000 / sgGameInitInfo.nTickRate`；渲染发生在两个 game tick 之间时，通过 `ProgressToNextGameTick` 把 0..128 的 tick 内进度补进 `AnimationInfo::getAnimationProgress()`。
- 原版行走通常是 8 帧，`Player::getAnimationFramesAndTicksPerFrame()` 对 `Walk` 使用 `_pWFrames` 和 `ticksPerFrame = 1`。在 20Hz 下，一步大约是 8 个 game tick，也就是 0.4 秒；城镇跑步还会通过 `StartWalkAnimation()` 跳过部分帧。
- Rust 此前 `DEFAULT_WALK_FRAME_DURATION = 0.10`，8 帧一步约 0.8 秒，明显慢于原版默认节奏。相机 offset 因此会在半像素/半格状态停留更久，边缘补画、遮罩和整数取整造成的闪动更容易被肉眼捕捉。
- 原版键盘/手柄入口 `WalkInDir()` 只是把目标发成 `CMD_WALKXY`，真正的开始、提交和连走由 path / player processing 在 game tick 中消费；鼠标持续行走的 `RepeatWalk()` 还会等到 walk frame 较晚时才重复发新目标。
- Rust 当前每帧直接轮询按键状态，并在 `Game::update()` 里先 `try_start_player_walk()`、再 `world.update(dt)`、结束后再 `try_start_player_walk()`。这样可以减少一步结束时的 Idle 闪现，但也意味着一步可能在某个渲染帧开头启动、立刻吃掉这一帧完整 `dt`，完成时又在同一帧重置 progress 并启动下一步。

所以目前的判断是：键盘控制行走和动画速度都有关。更准确地说，键盘“每渲染帧直接起步”和动画“可变 dt + 慢于原版”的组合，在放大 camera offset 期间的抖动。下一轮验证应先记录每帧 `dt / walk_progress / current_frame / walking_render_offset / tile_position`，再分别试两件事：把 walk frame duration 调到约 `0.05s` 对齐 20Hz 原版节奏；把行走状态推进改成固定 20Hz tick，渲染只读取插值进度。两者分开试，才能判断主要放大源是速度差还是 variable-dt 边界。

本轮实验开关：

- `Shift+F11`：开启/关闭行走 trace。开启后，玩家正在行走、刚结束行走、按键请求方向或一步刚开始时，会采样输出 `raw_dt`、实际用于行走推进的 `update_dt`、输入方向、鼠标是否按住和鼠标位置、update 前是否在行走、一步前/后是否启动、当前帧、progress、walking offset、tile/start/target 和当前 frame duration。trace 会做低频采样，避免每帧 `println!` 本身制造帧时间尖峰。
- `F11`：仍只负责开启/关闭 walking camera offset。
- 默认/`Home` 步速回到 `0.05s/frame`，也就是 8 帧一步约 `0.4s`，接近原版 20Hz 正常速度。更快的 `0.04s/frame` 实验没有减少房屋闪烁，反而让人物行走观感变得不自然；`PageUp` 不再低于 `0.05s/frame`，`PageDown` 可以逐步调慢，按 5 次会回到旧的 `0.10s/frame`，用于肉眼 A/B。
- 行走 update 的 `dt` 只对真正的长卡顿做保护，不再把 `0.05s` 以上的普通低帧率帧全部截断。调大默认分辨率后，如果渲染帧间隔稳定落在 50ms 到 100ms 之间，行走仍应按真实时间推进；否则窗口越大、渲染越慢，人物速度会被人为拖慢。
- 鼠标对照：按住左键时，Rust 会按“屏幕中心的玩家 -> 鼠标位置”的 8 方向请求行走；键盘方向仍优先。这个实验不是完整点击寻路，只用来比较“键盘持续轮询”和“鼠标持续目标方向”在相同 camera offset 下的抖动差异。
- 房屋闪烁 trace：`Shift+F7` 把渲染限制到玩家视口中心附近的 2x2 dPiece，`Ctrl+F7` 开启采样日志。日志会每 8 帧输出一次 frame、view、scan、floor/cell pass、dPiece、piece、block、真实 tile type、floor 强制三角类型、DrawCell mask、屏幕坐标和 wall pre-draw 信息。把疑似闪烁的房屋边缘移动到屏幕中心附近，再打开这两个开关，可以观察同一块房屋在相邻采样帧里是否出现 pass 顺序、block、mask 或 screen pos 跳变。
- 实际渲染帧数：每次 `engine.present()` 后累计一帧，每秒输出一次 `[render-fps]`，包含这一秒提交的帧数、FPS、平均帧耗时、当前 scene、focus trace、walking camera offset 和 mask-aware DrawCell 状态。用它对比普通渲染、房屋闪烁 trace、F11 camera offset 开关下的实际渲染压力。
- 渲染阶段耗时：`World::render_with_texture_manager()` 每秒输出一次 `[render-perf]`，按同一批帧汇总 world render 总耗时、floor pass、wall/content pass，每帧 floor tile、wall cell、pre-draw cell 的平均数量，以及 micro/foliage 纹理缓存命中、缺失和当前 tile texture cache 大小。当前实验里 full render 约 20 FPS，而 `F9` floor-only 约 46 FPS；`Shift+F12` 关闭 mask-aware DrawCell 只小幅提升到约 23 FPS，所以下一步应优先确认 wall/content pass 的扫描量、cell 像素准备成本和大量小 SDL texture copy 成本，而不是继续调 walking camera offset 或 mask 透明规则。
- 纹理缓存快路径：`draw_cached_rgba_texture()` 原本只避免重复创建/上传 SDL texture，但 `World::render_micro_tile_with_kind()` 每次仍会先把 indexed pixels 重排、转 RGBA、应用 mask/toon，再把 RGBA 交给缓存。现在在计算出 `cache_key` 与目标 rect 后，先尝试 `Engine::draw_cached_texture()`；命中时直接 copy 已有 SDL texture，跳过本帧像素准备。实现上要注意 `TextureCache` 不能保存 `Engine::new()` 中局部 `texture_creator` 的转生引用，否则缓存上传会失败并静默回退到临时 texture，表现为 `texture_cache_size=0`、每帧全 miss。缓存现在由 `Engine` 在上传时传入自己持有的 `texture_creator`。另外，foliage 可能解码失败或全透明；这类结果也要做负缓存，否则即使 micro tile 已经全命中，仍会每帧重复尝试几十次 foliage。

## 9. 房屋闪烁：floor pass 与 DrawCell 的像素级差异

最新手动观察是“原画面也有一点点闪烁，只是很不明显”，而 Rust 主要是房屋/屋顶这类透明上层在滚动时闪得更明显。这个现象更像是像素合成误差被 camera offset 放大，而不是键盘或鼠标输入本身造成的。

原版锚点如下：

- `Source/engine/render/scrollrt.cpp:652` `DrawFloorTile()`：floor pass 只画 block 0 和 block 1，并且固定把 block 0 当 `TileType::LeftTriangle`、block 1 当 `TileType::RightTriangle`，高度固定为 `DunFrameTriangleHeight = 31`。
- `Source/engine/render/scrollrt.cpp:521` `DrawCell()`：随后 wall/content pass 再按真实 micro tile 类型、透明属性和 `MaskType` 画墙体/屋顶上层。
- `Source/levels/reencode_dun_cels.cpp:189` `ReencodeFloorWithFoliage()`：原版在加载阶段会把某些 `TransparentSquare` floor frame 先拆成 floor triangle 与 foliage 数据，所以运行时 `DrawFloorTile()` 能安全地用强制三角形路径渲染。

Rust 目前没有完整复刻 `ReencodeDungeonCels()`。如果直接把原始 CEL 数据硬按 `LeftTriangle` / `RightTriangle` 解码，遇到 `TransparentSquare` RLE frame 反而会读错；所以本轮先做一个更小的等价映射：floor pass 仍按 MIN 里的真实类型解码，但在渲染前裁成原版 floor pass 的左右三角形，并把 floor 三角形纹理和普通 `DrawCell` 纹理分开缓存。这样透明屋顶或房屋上层透出下方像素时，底层 footprint 更接近原版，不会因为 Rust 之前把实际 square/trapezoid floor 直接画满而在滚动中暴露出多余像素。

同一轮顺手收紧 `DrawTileContent()` 的右边界 gate：原版条件是 `targetBufferPosition.x + TILE_WIDTH <= gnScreenWidth`，Rust 之前只判断 `screen_x <= screen_width`，会把已经超过一个 tile 宽度的东侧预绘纳入 wall/content pass。这个差异不会单独导致输入抖动，但会让房屋边缘的绘制顺序在滚动边界更容易变化。

默认窗口回到 `640x352`，也就是原版地牢画面区域，而不是完整 `640x480` 面板分辨率。`960x528` 等比例放大实验没有减少房屋闪烁，还会增加渲染负担并改变肉眼观察节奏；排查应继续聚焦在 floor pass、DrawCell、mask 和 wall/content 顺序的像素级差异上。

## 10. 最终定位：为什么看起来像 camera offset 抖动

这轮实验的最终结论是：Rust 版“房屋闪得比原版严重”，主因已经不再是 `walking_camera_offset` 的开关逻辑，也不是键盘输入本身，而是 full render 时 wall/content 路径长期跑在低帧率，导致可变 `dt` 下的滚动和像素级合成误差被显著放大。

证据链按顺序如下：

- 在 cache 修复前，full render 的 `[render-perf]` 大致是 `avg_world_ms ~= 48ms`、`avg_floor_ms ~= 20ms`、`avg_wall_ms ~= 28ms`，对应 `[render-fps]` 约 20 FPS；而 `F9` floor-only 大约能到 40 到 46 FPS，说明瓶颈集中在 wall/content pass，而不是整个世界更新或 camera offset 逻辑。
- `Shift+F12` 关闭 mask-aware DrawCell 后，full render 只从约 20.5 FPS 提高到约 23.5 FPS。这说明 mask 透明规则有成本，但不是把抖动放大的主因；真正的大头仍是 wall/content pass 每帧数百个 cell 的像素准备和提交。
- 新增缓存命中统计后，full render 曾稳定出现 `micro_cache_hit/frame=0`、`micro_cache_miss/frame` 接近 950、`texture_cache_size=0`。这说明“理论上已经做了 texture cache”，但运行时缓存根本没有成功保存任何 tile texture。
- 根因最终定位到 `TextureCache` 生命周期错误：它保存了 `Engine::new()` 里局部 `texture_creator` 的转生引用。`Engine::new()` 返回后，这个引用已经失效，缓存上传会失败并静默退回“每帧现做临时 texture”，于是 wall/content pass 每一帧都在重复做 indexed 重排、RGBA 转换、mask/toon 和小纹理上传。
- 修复后，full render 立即变成 `fps=165.0`、`avg_world_ms=0.95`、`avg_wall_ms=0.83`、`micro_cache_hit/frame=909`、`micro_cache_miss/frame=0`、`texture_cache_size=930`。这条对比基本坐实：之前被误认为“camera offset 一开就更抖”的现象，本质上是 full render 只有约 20 FPS 时，格间滚动在 variable-dt 下停留更久，房屋/屋顶这类透明上层的像素差异更容易被肉眼捕捉。

所以要把原因拆成三层理解：

- 现象层：开启行走 camera offset 后，房屋边缘和屋顶透明区域更容易被看成“抖动”。
- 近因层：低帧率使每帧位移更大、每个半格/半像素状态停留更久，原版本来就有的一点点闪动会被放大。
- 根因层：Rust 的 wall/content 渲染路径在缓存失效时退化成“每帧重新准备几乎全部 micro tile 像素”，把 full render 压到约 20 FPS。

这也解释了为什么“调步速”“切键盘/鼠标”“改默认分辨率”都没有真正解决问题：这些实验最多只是在改变抖动被观察到的节奏，不能消除 full render 低帧率这个放大器。键盘持续轮询、动画 `dt` 和 floor/DrawCell 像素 footprint 差异仍然会影响最终观感，但它们在这轮实验里已经从“主嫌疑”降级成“次级放大项”；先把 wall/content 缓存路径修正到正常命中，才有资格继续细看与原版相比剩下的轻微闪烁。

顺手补上的 foliage 负缓存属于同一类尾项优化：某些 foliage frame 可能解码失败或本来就是全透明，如果不记住“这块不需要再试”，即使 micro tile 已经全命中，仍会每帧重复浪费几十次 foliage 尝试。它不是这次 20 FPS 的主因，但能避免后续分析被残留噪音干扰。

## 11. 当前简化和后续债务

这次修复仍然是学习型最小闭环，不等于完整复刻原版移动渲染。

- 还没有实现原版 `dPlayer` 正负索引对南向、东向移动玩家的绘制排序。
- 还没有把光源偏移做到原版 `ChangeLightOffset()` 那种格间位置。
- 还没有实现点击寻路和路径队列，只处理键盘触发的一步一格移动。
- 还没有复刻 `AnimationDistributionFlags` 和固定 game tick 的全部细节，当前仍用 `dt` 推进行走动画。
- 还没有完整复刻加载阶段的 `ReencodeDungeonCels()`；floor 三角形裁剪是先把运行时像素 footprint 对齐到原版的过渡方案。

## 12. 验证

自动测试重点验证两个不依赖 MPQ 的规则：

- `Direction::to_tile_offset()` 的 8 方向位移必须和 DevilutionX 等距方向一致。
- `walking_render_offset(1.0)` 必须等于目标 tile 相对起点的屏幕投影。
- `calc_viewport_geometry()` 必须对齐原版 `CalcViewportGeometry()` 的关键数值；当前默认地牢视口 `640x352` 和旧完整面板对照 `640x480` 都保留测试覆盖。
- `walking_viewport_expansion()` 必须覆盖原版 `CalcFirstTilePosition()` / `DrawGame()` 对 8 个方向的视口补画规则。
- `wall_content_scan_padding()` 必须确认 wall/content pass 左右各多扫一列，作为学习阶段的保守补画保护。
- `should_predraw_east_wall_content()` 必须覆盖原版 `DrawTileContent()` 对 x 轴连续墙、背后可走区域和右边界的 gate 条件。
- floor pass 的 block 0/1 必须走独立的 floor 三角形缓存，不能和普通 `DrawCell` / foliage 纹理缓存互相污染。

手动验证建议：

- `cargo run`
- 在主场景中按住 8 个方向移动。
- 观察每一步结束时，玩家不应再出现“走出去后回退”的跳变。
- 重点观察西、西北、西南移动时，窗口边缘不应再出现锯齿状未渲染黑三角。
- 按 `W` 让房屋或连续墙进入窗口上方/斜向边缘，观察不应再出现“floor 有、墙体或房屋边缘缺失”的断裂；再按 `S` 走回时，房屋也不应出现重新弹回来的感觉。
- 如果缺少 `Diabdat.mpq` 或 CL2 资源，只能验证 fallback 行为；真实战士动画需要本机资源存在。
