# Step 7 玩家移动与 8 方向动画测试计划

**工作线**: `rust-diablo/`
**前置**: `step-7-player-movement-8direction-animation-requirements.md`、`step-7-player-movement-8direction-animation-original-research.md`、`step-7-player-movement-8direction-animation-design.md`

这份测试计划只关注“怎么证明确实做对了”，不重复实现细节。

---

## 1. 测试目标

本 Step 的测试要覆盖四类行为:

1. 方向映射是否稳定。
2. 动画控制器是否真的按 state + direction 取帧。
3. 行走状态机是否在最后一帧才提交格子。
4. 渲染偏移是否能从逻辑上被计算出来。

---

## 2. 单元测试

### 2.1 `src/engine/direction.rs`

要补的测试:

- 8 个方向的动画索引双向映射。
- `Direction::None` 不应产生有效 walk index。
- 动画顺序常量和原版一致。
- 方向到 walk offset 的顺序和 C++ 一致。

### 2.2 `src/sprite/animation.rs`

要补的测试:

- 单个动画的进度推进。
- 非循环动画在末帧结束。
- `AnimationController` 能按 `(state, direction)` 找到动画。
- 当某个方向缺资源时，控制器能回退到可用方向。
- `set_state_direction()` 会切换并重置当前动画。

### 2.3 `src/entity/mod.rs`

要补的测试:

- `start_walk()` 成功后会记录起点、目标、方向和进度。
- `start_walk()` 失败时不会改 `tile_position`。
- `update()` 在步行未完成时不提交格子。
- `update()` 到最后一帧时才提交目标格。
- 停止后保持最后朝向，而不是重置成 `None`。
- `walking_render_offset()` 在 0% / 50% / 100% 时返回合理偏移。

### 2.4 CL2 sheet 解析

如果这次实现了方向 sheet 解析，要补:

- 单列表 CL2 还能正常解析。
- 多方向 sheet 能被拆成多个方向组。
- 方向组数量和顺序能正确读出。
- 方向组里每个 frame 的宽高和像素数据仍然正确。

---

## 3. 集成测试

### 3.1 纯逻辑集成测试

建议新增一个 `tests/step7_player_movement.rs` 或等价测试文件，验证:

- 方向输入组合能得到 8 方向。
- 走一步时不会立刻改格。
- 动画结束后才改格。
- 连续按键时能在完成一步后继续下一步。

### 3.2 渲染逻辑集成测试

如果不方便做像素级截图，也至少要验证:

- `world::render_entities()` 会根据 current state + direction 生成正确的纹理 key。
- walking 时会叠加 offset。
- idle 时不会叠加 offset。

这类测试可以先用纯逻辑方式检查生成的 key 和位置，不一定要真正打开 SDL 窗口。

---

## 4. 手动验证

### 4.1 必做场景

- 按 `W`
- 按 `W+D`
- 按 `D`
- 按 `S+D`
- 按 `S`
- 按 `S+A`
- 按 `A`
- 按 `W+A`

观察点:

- 面向是否和方向一致。
- Walk 动画是否会切到对应方向。
- 走到一半时是否能看到明显的子格偏移。
- 松开按键后是否保持最后朝向。

### 4.2 失败场景

- 目标格不可走时，玩家不应该进入 walking。
- 目标格不可走时，不应该切到错误方向的 Walk 动画。
- 没有 `Diabdat.mpq` 时，基础单元测试仍应该能跑。

---

## 5. 依赖和隔离

### 5.1 不应依赖的东西

核心单元测试不应依赖:

- 本机 MPQ
- SDL2 窗口
- 游戏资源是否真的存在

### 5.2 可以依赖的东西

如果要测方向 sheet 解析或真实资源纹理:

- 允许依赖本机 MPQ
- 但必须单独标注为手动测试或 ignored

---

## 6. 验收门槛

这一步通过的最低标准建议是:

- `cargo test` 通过。
- 方向映射测试通过。
- `AnimationController` 的 state + direction 测试通过。
- `Entity` 的一步行走状态机测试通过。
- 手动验证里 8 个方向的移动行为一致。

---

## 7. 未覆盖风险

以下内容这一步可以先不完全覆盖，但要在文档里明确知道它们存在:

- 不同职业/装备的完整动画差异。
- 屏幕滚动和相机连续跟随的完全原版一致性。
- 真实 CL2 sheet 在所有职业上的完整分组边界。
- 物体、怪物、施法、攻击动画的复用。
