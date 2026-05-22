# Step 6.4.4: 让 DrawCell 记住“怎么合成”

## 本节要做什么

这一节解决一个很容易被误会的问题：地下城 tile 已经解码成 RGBA 了，为什么画面还是和原版不一样？

答案是，解码只告诉我们“哪些像素存在”，但原版 `DrawCell()` 还会告诉 renderer“这些像素要不透明写入，还是半透明混合”。这个决定就是 `MaskType`。如果 Rust 只把 palette index 0 当透明，而不处理 `MaskType::Transparent / Left / Right`，桥、墙上半部和透明墙区域在移动镜头下就会更容易露出错误的底色。

本节完成的结果：

- `draw_cell_at()` 会像原版一样为 micro tile 选择 mask。
- `render_micro_tile()` 会在上传 SDL texture 前，把 mask 作用到 RGBA alpha。
- Shift+F12 可以切换 mask-aware DrawCell。
- mask-aware 默认开启，walking camera offset 仍保留为 F11 手动验证路径。

## 先做一个最小实验

我们不用先打开整张地图，只看一个 32x32 的 RGBA buffer。假设每个像素 alpha 都是 255：

```rust
let mut pixels = [1u8, 1, 1, 255].repeat(32 * 32);
World::apply_draw_cell_mask_to_rgba(&mut pixels, 32, 32, DrawCellMask::Right);
```

如果 mask 是 `Solid`，alpha 不变。
如果 mask 是 `Transparent`，所有非零 alpha 像素都变成半透明。
如果 mask 是 `Right`，下半部不变，上半部右侧沿斜线变成半透明。

这个小实验对应新增测试：

```powershell
cargo test --lib world::tests::test_draw_cell_mask
```

## 观察问题

旧路径的问题不在 tile 解码本身，而在“合成语义”丢失了。

Rust 旧逻辑只有一条规则：palette index 0 -> alpha 0，其他像素 -> alpha 255。
原版还有第二层规则：某些 tile 在透明区域里，非零像素也要半透明混合；某些第一层 micro tile 只在上半部一侧半透明。

所以旧路径能画出形状，却不能稳定复现原版透明墙、桥边、墙上半部与玩家移动镜头叠加时的像素关系。

## 引入原版实现

原版锚点：

- `Source/engine/render/scrollrt.cpp::DrawCell()`，约 521-643 行。
- `Source/engine/render/dun_render.hpp::MaskType`，约 28-90 行。
- `Source/engine/render/dun_render.cpp::RenderTileFrame()`，约 1048-1080 行。

原版 `DrawCell()` 做了三件关键事：

1. 通过 `TileHasAny(tilePosition, TileProperties::Transparent)` 和 `TransList[dTransVal[x][y]]` 判断当前 tile 是否处于透明区域。
2. 对 block 0 / 1 使用 `getFirstTileMaskLeft()` 和 `getFirstTileMaskRight()`，让 `TransparentSquare` 与 trapezoid 可以走 `Left` / `Right` 局部 mask。
3. 对 block 2 以上的 wall layers，透明区域统一传 `MaskType::Transparent`，普通区域传 `MaskType::Solid`。

这一步回答的是“怎么合成”，不是“tile 长什么形状”。

## 映射到 Rust

| 原版 C++ | Rust 实现 | 等价关系 | 当前差异 |
| --- | --- | --- | --- |
| `MaskType::Solid` | `DrawCellMask::Solid` | 非零像素保持不透明 | 仍用 SDL RGBA texture |
| `MaskType::Transparent` | `DrawCellMask::Transparent` | 非零像素改成半透明 alpha | 近似原版透明表 |
| `MaskType::Left` | `DrawCellMask::Left` | 上半部左侧半透明 | 以 alpha 近似混色 |
| `MaskType::Right` | `DrawCellMask::Right` | 上半部右侧半透明 | 以 alpha 近似混色 |
| `TileHasAny(...Transparent...)` | `SolData` 的 `TRANSPARENT` flag | 当前可用的透明输入 | 还没接完整 `TransList` |

关键实现文件：

- `src/world/mod.rs`
- `src/debug.rs`
- `src/game.rs`

## 一步一步改进

第一步，加 `DrawCellMask`。
它只表达四种合成模式，并提供 `first_left()`、`first_right()`、`upper_block()` 三个小函数，对齐原版 `DrawCell()` 的分支。

第二步，把 mask 传进 `render_micro_tile()`。
这样 block 0、block 1、block 2+ 都可以携带自己的合成规则，而不是统一当普通透明贴图。

第三步，在 RGBA buffer 上应用 mask。
我们保留 palette index 0 的透明含义，只调整非零 alpha 像素。这样 tile shape 不会被破坏，mask 只影响“已存在像素”的不透明/半透明合成。

第四步，把 cache key 纳入 mask。
同一个 piece/block 在 `Solid` 和 `Transparent` 下会产生不同 alpha，因此 texture cache 不能复用同一个 key。

第五步，保留 walking camera offset 的手动验证路径。
mask-aware 路径默认开启后，移动镜头下的 tile 合成更接近原版；但 camera offset 仍需要肉眼验证扫描稳定性，所以默认保持关闭，F11 用来临时打开测试。

## 验证结果

已通过：

```powershell
cargo fmt
cargo test --lib world::tests::test_draw_cell_mask
```

结果：4 个新增测试全部通过。

还运行了：

```powershell
cargo test --lib
```

结果：新增 world 测试通过，库内总计 176 个测试中 168 个通过、2 个 ignored、6 个失败。失败项集中在既有 `drlg_l1`、CL2/CLX、Dungeon CEL 和缺少 `assets/Diabdat.mpq` 的 MPQ 测试；它们不属于本 step 改动面。

完整 `cargo test` 还会编译集成测试，目前会被既有 `tests/visual_validation.rs` 和 `tests/test_town_pal_no_decrypt.rs` 的接口/资源问题拦住。

## 常见误区

- 不要把 `MaskType` 理解成 tile 几何形状。几何形状来自 `TileType` 和解码结果；`MaskType` 只决定非零像素怎么合成。
- 不要让 texture cache 只按 piece/block 缓存。mask 和 toon filter 都会改变输出像素。
- `Left` / `Right` 只影响上半部斜边区域，下半部应该保持不透明。
- 当前 Rust 还没有完整 `dTransVal` / `TransList`，所以透明触发条件仍是阶段性近似。

## 练习题

1. 入门题：为什么 `TransparentSquare` 在 `isFloor` 时会走 foliage，而不是普通 tile render？
2. 代码题：把 `DrawCellMask::TRANSPARENT_ALPHA` 改成 96 或 160，运行城镇桥附近观察差异。
3. 调试题：如果 Shift+F12 切换后画面没变化，应该先检查 cache key 还是 mask 选择？
4. 挑战题：给 `World` 接入真正的 `TransVal` / `TransList`，让 `transparency` 判断从 SOL 近似升级成原版条件。

## 回到大图景

这个 step 不是最终像素完美 renderer，但它把 Rust 渲染从“只知道透明色”推进到了“知道原版 DrawCell 的合成意图”。这为后续接入完整区域透明、8-bit framebuffer、原版 lighting table 和更严格截图对比打下了基础。
