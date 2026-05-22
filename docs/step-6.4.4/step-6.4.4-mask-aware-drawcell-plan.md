# Step 6.4.4: mask-aware DrawCell 渲染路径前置文档

## 需求

本 step 要让 Rust 复刻的 `DrawCell` 更接近 DevilutionX 的像素级合成规则。当前 Rust 渲染已经按 floor/wall 两阶段绘制 micro tile，并把 palette index 0 作为透明像素跳过；但 `DrawCell` 里原版的 `MaskType::Solid / Transparent / Left / Right` 还没有进入 Rust 的像素合成路径。结果是桥、墙上半部、透明区域和移动摄像机叠加时，某些本该半透明或局部半透明的 tile 仍按普通 alpha 贴图处理。

本 step 的可观察目标：

- 新增一个可切换的 mask-aware `DrawCell` 路径，默认开启。
- 在该路径中，`DrawCell` 先按原版规则为每个 micro tile 决定 mask，再把 mask 应用到 RGBA 像素 alpha。
- 自动测试覆盖 mask 选择和像素 alpha 结果。
- walking camera offset 保持可切换；只有在手动验证稳定后才重新默认打开。

本 step 暂不做：

- 不实现完整 8-bit 软件 framebuffer。
- 不实现 `dTransVal` / `TransList` 的完整区域透明系统；当前先以 SOL 的 `TRANSPARENT`、`TRANSPARENT_LEFT`、`TRANSPARENT_RIGHT` 作为 Rust 可用输入。
- 不重写 lighting table、per-pixel lightmap 或 C++ 的精确透明混色表。

## 原版调研

原版锚点：

- `Source/engine/render/scrollrt.cpp::DrawCell()`，约 521-643 行。
- `Source/engine/render/dun_render.hpp::MaskType`，约 28-90 行。
- `Source/engine/render/dun_render.cpp::RenderTileFrame()`，约 1048-1080 行。

原版控制流：

1. `DrawCell()` 读取 `dPiece[tilePosition]` 得到 `levelPieceId`，再取 `DPieceMicros[levelPieceId]`。
2. 它先判断 `transparency = TileHasAny(tilePosition, TileProperties::Transparent) && TransList[dTransVal[x][y]]`。
3. 对 block 0 和 block 1，原版使用两个 lambda：
   - `getFirstTileMaskLeft(tile)`
   - `getFirstTileMaskRight(tile)`
4. `LeftTrapezoid` / `RightTrapezoid` / `TransparentSquare` 会根据 `TransparentLeft` / `TransparentRight` 决定 `MaskType::Left` 或 `MaskType::Right`，否则可能是 `Solid`。
5. 其他上层 wall block 在透明 tile 上统一走 `MaskType::Transparent`，否则走 `MaskType::Solid`。
6. `RenderTileFrame()` 根据 mask 分发到底层 renderer，`Solid` 直接写，`Transparent` 半透明混合，`Left` / `Right` 只让上半部的一侧进入半透明混合。

这说明 `MaskType` 不是 tile shape 本身，而是“这些已经解码出来的非零像素要用不透明还是半透明方式合成”。

## Rust 设计

Rust 当前入口：

- `src/world/mod.rs::draw_cell_at()`
- `src/world/mod.rs::render_micro_tile()`
- `src/debug.rs::RenderDebugFlags`
- `src/game.rs` 的 F 键调试开关

新增结构：

- `DrawCellMask`：内部 enum，对应原版 `MaskType::{Solid, Transparent, Left, Right}`。
- `RenderDebugFlags::mask_aware_draw_cell`：调试开关，默认开启。

核心映射：

| 原版 C++ | Rust 实现 | 说明 |
| --- | --- | --- |
| `MaskType::Solid` | `DrawCellMask::Solid` | 非零像素保持 alpha 255 |
| `MaskType::Transparent` | `DrawCellMask::Transparent` | 非零像素 alpha 降为半透明 |
| `MaskType::Left` | `DrawCellMask::Left` | 下半部不透明，上半部左侧半透明 |
| `MaskType::Right` | `DrawCellMask::Right` | 下半部不透明，上半部右侧半透明 |
| `TileHasAny(...Transparent...)` | `SolData::get(piece_id)` | 当前 step 用 SOL flag 近似，后续再接 `TransList` |

实现步骤：

1. 在 `draw_tile_content_at()` 把当前 tile 的 `TileProperties` 传入 `draw_cell_at()`。
2. 在 `draw_cell_at()` 计算 block 0、block 1 和上层 blocks 的 mask。
3. 在 `render_micro_tile()` 增加 mask 参数；旧路径保留为 `Solid`，由 `mask_aware_draw_cell` 控制。
4. 在 RGBA 转换后、toon filter 前，把 mask 应用到非零 alpha 像素。
5. 通过 F12/Shift+F12 或附近调试键位保留切换能力，状态打印包含该 flag。
6. 自动测试通过后，保留 F11 手动验证 walking camera offset；若手动验证仍抖动，默认值继续保持 `false`。

## 测试计划

自动测试：

- `DrawCellMask::first_left()`：非透明 tile 总是 `Solid`。
- `DrawCellMask::first_left()`：透明 `TransparentSquare` + `TRANSPARENT_LEFT` 得到 `Left`。
- `DrawCellMask::first_right()`：透明 `TransparentSquare` + `TRANSPARENT_RIGHT` 得到 `Right`。
- `DrawCellMask::upper_block()`：透明 tile 得到 `Transparent`，普通 tile 得到 `Solid`。
- `apply_draw_cell_mask_to_rgba()`：`Solid` 不改 alpha，`Transparent` 把非零 alpha 改为半透明。
- `apply_draw_cell_mask_to_rgba()`：`Left` / `Right` 只影响上半部对应斜边一侧。

验证命令：

```powershell
cd rust-diablo
cargo fmt
cargo test world::tests::test_draw_cell_mask
cargo test
```

手动验证：

```powershell
cd rust-diablo
cargo run
```

观察点：

- F5 打开 wall layer，F6 打开 entity layer。
- F11 可以切换 walking camera offset。
- 新的 mask-aware DrawCell 开关可以比较桥/墙透明区域。
- walking camera offset 当前应保持默认 OFF，确认不抖后再通过 F11 打开验证。

## 风险

- 当前没有完整 `TransList[dTransVal]`，所以“是否透明”的触发条件仍比原版粗。
- SDL alpha blend 只能近似 C++ 8-bit light/transparent table，无法做到完全逐表一致。
- 若贴图缓存 key 没区分 mask 类型，切换 mask 会复用旧纹理；实现时必须把 mask 纳入 cache key。
