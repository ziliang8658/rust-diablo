# Wall 渲染顺序和玩家遮挡

## 1. 这次要解决什么

现象很直观：只画地面时，玩家移动和地面滚动都很顺；打开 wall/cell 层以后，房子、墙体这类高 tile 会像被快速覆盖又露出来，玩家移动也会带出抖动感。

这不是单个 tile 解码错位的问题，而是渲染顺序问题。旧 Rust 流程是：

```text
画完整屏 floor
画完整屏 wall/cell
最后统一画玩家和实体
```

这样玩家永远在所有墙体上面。移动时，玩家 sprite 会反复盖住本该位于前景的房子和墙，肉眼看起来就像 wall tile 高频闪动。

## 2. 原版怎么做

DevilutionX 原版入口在 `Source/engine/render/scrollrt.cpp`：

```text
DrawGame()
  DrawFloor(...)
  DrawTileContent(...)
```

关键点是 `DrawTileContent()` 不只是画 wall。它按等距菱形扫描顺序逐格调用 `DrawDungeon()`。而 `DrawDungeon()` 的顺序是：

```text
当前 tile 的 wall/cell
尸体、物品、pre object
玩家、怪物
missile、post object、post item、special
```

所以玩家不是最后统一画一遍，而是插在 tile 遍历中。后面扫描到的前景 wall 仍然可以盖住玩家，这就是 Diablo 这种 2D 等距场景不用 z-buffer 也能成立的原因。

原版还有一个小修正：`DrawTileContent()` 遇到某些沿 x 轴排列的墙时，会先把墙后方的 East tile 提前 `DrawDungeon()`，然后下一格跳过。这个逻辑用于避免移动中的玩家或怪物从墙边缘穿出来。

## 3. Rust 里怎么映射

这次 Rust 侧改成同一条规则：

```text
World::render()
  render_with_texture_manager()
    Phase 1: DrawFloor
    Phase 2: DrawTileContent-style scan
      draw_tile_content_at()
        draw_cell_at()
        render_entities_at_dpiece()
```

主要映射关系：

| 原版 C++ | Rust 实现 | 作用 |
| --- | --- | --- |
| `DrawGame()` 先 floor 后 content | `render_with_texture_manager()` 两阶段 | 保持地面先铺底，墙体和实体再参与排序 |
| `DrawTileContent()` 逐格扫描 | wall phase 的 zigzag 循环 | 用 tile 顺序表达遮挡关系 |
| `DrawDungeon()` | `draw_tile_content_at()` | 当前 tile 内先 wall/cell，再实体 |
| `PlayerAtPosition()` / `dPlayer` | `render_entities_at_dpiece()` | 当前简化为从实体列表按 tile 查找 |
| 南/东向移动负 id 优先级 | `entity_render_order_tile()` | 南、东、东南、西南移动时用目标格参与排序 |
| 墙后方 tile 提前绘制 | `should_predraw_east_wall_content()` | 对齐原版墙边缘防穿帮逻辑 |

这里仍然是学习型最小闭环，不是完整复刻。Rust 还没有完整的 `dPlayer`、`dMonster`、`dObject`、`dItem` 网格，也没有对象 `_oPreFlag` / 物品 `_iPostDraw` 这类更细的分层标记。所以这次先解决“玩家最后统一画导致 wall 闪动”的主因。

## 4. 为什么这个修正能消除闪动

旧实现里，房子 wall 已经画好了，但玩家最后又覆盖它。玩家移动每一帧的位置和动画帧都在变，于是房子边缘被不断擦掉、露出、再擦掉。

新实现里，玩家只在自己的排序 tile 被画一次。扫描到前景墙体所在 tile 时，墙体会在玩家之后画上来。结果是：

```text
背景 floor -> 背景 wall -> 玩家 -> 前景 wall
```

这和原版的核心视觉模型一致。墙体不再依赖“最后一层玩家”之后还能不能补画回来，因此不会出现高频闪动。

## 5. 验证

已补的单元测试集中验证排序规则：

```powershell
cargo test --lib world::tests
```

覆盖点：

- `walking_render_offset(1.0)` 仍然等于目标 tile 的屏幕投影。
- 行走方向导致的 viewport 扩展仍和原版 `DrawGame()` 规则一致。
- 南向和东向移动实体使用目标格参与排序。
- 北向和西向移动实体仍使用起点格参与排序。
- x 轴墙段、后方可通行、屏幕边界三类条件满足时才触发 East tile 提前绘制。

完整 `cargo test` 当前仍会失败，但失败点在既有集成测试和资源测试上：`tests/test_tiles_integration.rs` 还在调用旧的 `MinData::from_bytes(data)` 签名，部分 CL2/CEL/MPQ 测试也依赖尚未稳定的资源或本机 `Diabdat.mpq`。这次渲染顺序相关的 `world::tests` 已通过。

## 6. 后续练习

下一步最自然的是把当前实体列表查找替换成更接近原版的渲染网格：

```text
dPlayer / dMonster / dObject / dItem
```

有了这些数组后，`draw_tile_content_at()` 就可以继续拆出 object、item、monster、missile 的细分层，逐步靠近原版 `DrawDungeon()`。
