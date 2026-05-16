# 玩家动画帧切换问题修复

## 问题描述

玩家在移动时，画面并未切换，动画帧没有正确更新。

## 问题根源

渲染代码使用了错误的帧索引方法：

1. **错误方法**：`current_frame_index()`
   - 只返回 `current_frame`，没有考虑动画分布逻辑和插值
   - 无法正确处理帧之间的平滑过渡

2. **正确方法**：`get_frame_to_use_for_rendering()`
   - 考虑了动画分布逻辑（Animation Distribution Logic）
   - 使用 `ticks_since_sequence_started` 和 `elapsed` 计算插值
   - 与 C++ 版本的 `getFrameToUseForRendering()` 等价

## C++ vs Rust 实现对比

### C++ 实现

```cpp
// Source/engine/animationinfo.h:64-67
ClxSprite currentSprite() const {
    return (*sprites)[getFrameToUseForRendering()];
}

// Source/engine/animationinfo.cpp:18-59
int8_t AnimationInfo::getFrameToUseForRendering() const {
    // 考虑动画分布逻辑和插值
    // 使用 ticksSinceSequenceStarted_ 和 getProgressToNextGameTick()
    // 计算用于渲染的帧索引
}
```

### Rust 实现（修复前）

```rust
// 错误：只返回 current_frame
pub fn current_frame_index(&self) -> Option<usize> {
    self.animations
        .get(&(self.current_state, self.current_direction))
        .map(|anim| anim.current_frame)  // ❌ 没有考虑插值
}
```

### Rust 实现（修复后）

```rust
// 正确：考虑动画分布逻辑和插值
pub fn get_frame_to_use_for_rendering(&self) -> Option<usize> {
    self.animations
        .get(&(self.current_state, self.current_direction))
        .map(|anim| anim.get_frame_to_use_for_rendering())  // ✅ 考虑插值
}

// Animation::get_frame_to_use_for_rendering()
pub fn get_frame_to_use_for_rendering(&self) -> usize {
    if self.relevant_frames_for_distributing == 0 {
        return self.current_frame.min(self.frames.len().saturating_sub(1));
    }

    // 使用分布逻辑计算帧索引（考虑插值）
    let ticks_since_start = self.ticks_since_sequence_started.max(0) as i32;
    let progress_to_next = (self.elapsed / self.frame_duration) * Self::BASE_VALUE_FRACTION as f32;
    let total_ticks = ticks_since_start + progress_to_next as i32;

    let absolute_frame = (total_ticks * self.tick_modifier as i32)
        / (Self::BASE_VALUE_FRACTION as i32 * Self::BASE_VALUE_FRACTION as i32);

    // ... 处理跳过的帧 ...

    frame_index.min(self.relevant_frames_for_distributing.saturating_sub(1))
}
```

## 修复内容

### 1. 添加新方法到 AnimationController

**文件**：`rust-diablo/src/sprite/animation.rs`

```rust
/// Get frame index to use for rendering
///
/// This method considers animation distribution logic and interpolation,
/// similar to C++ AnimationInfo::getFrameToUseForRendering().
/// This should be used instead of current_frame_index() when rendering.
pub fn get_frame_to_use_for_rendering(&self) -> Option<usize> {
    self.animations
        .get(&(self.current_state, self.current_direction))
        .map(|anim| anim.get_frame_to_use_for_rendering())
}
```

### 2. 更新渲染代码

**文件**：`rust-diablo/src/world/mod.rs` (3处)
**文件**：`rust-diablo/src/game.rs` (1处)

**修改前**：
```rust
let frame = anim.current_frame_index().unwrap_or(0);
```

**修改后**：
```rust
// Use get_frame_to_use_for_rendering() for proper frame interpolation
let frame = anim.get_frame_to_use_for_rendering().unwrap_or(0);
```

## 插值实现原理

### C++ 版本

1. **固定点数学**：使用 `baseValueFraction = 128` 进行固定点计算
2. **Tick 计数**：`ticksSinceSequenceStarted_` 累计动画序列开始后的 tick 数
3. **进度计算**：`getProgressToNextGameTick()` 获取到下一个 game tick 的进度
4. **帧索引计算**：
   ```cpp
   totalTicks = getProgressToNextGameTick() + ticksSinceSequenceStarted_;
   absoluteFrame = totalTicks * tickModifier / baseValueFraction / baseValueFraction;
   ```

### Rust 版本

1. **固定点数学**：使用 `BASE_VALUE_FRACTION = 128`（与 C++ 一致）
2. **Tick 计数**：`ticks_since_sequence_started` 累计动画序列开始后的 tick 数
3. **进度计算**：使用 `elapsed / frame_duration` 计算帧内进度
4. **帧索引计算**：
   ```rust
   let progress_to_next = (self.elapsed / self.frame_duration) * BASE_VALUE_FRACTION;
   let total_ticks = ticks_since_start + progress_to_next;
   let absolute_frame = (total_ticks * tick_modifier)
       / (BASE_VALUE_FRACTION * BASE_VALUE_FRACTION);
   ```

## 关键差异

| 方面 | C++ | Rust |
|------|-----|------|
| **固定点基础值** | `baseValueFraction = 128` | `BASE_VALUE_FRACTION = 128` ✅ |
| **Tick 计数** | `ticksSinceSequenceStarted_` | `ticks_since_sequence_started` ✅ |
| **进度计算** | `getProgressToNextGameTick()` | `elapsed / frame_duration * BASE_VALUE_FRACTION` ✅ |
| **帧索引方法** | `getFrameToUseForRendering()` | `get_frame_to_use_for_rendering()` ✅ |
| **渲染使用** | `currentSprite()` → `getFrameToUseForRendering()` | 修复前：`current_frame_index()` ❌<br>修复后：`get_frame_to_use_for_rendering()` ✅ |

## 测试验证

修复后，玩家移动时应该：
1. ✅ 动画帧正确切换
2. ✅ 帧之间平滑过渡（插值）
3. ✅ 行走动画循环播放
4. ✅ 方向变化时动画正确切换

## 参考代码

### C++ 原版
- `Source/engine/animationinfo.h:64-67` - `currentSprite()`
- `Source/engine/animationinfo.cpp:18-59` - `getFrameToUseForRendering()`
- `Source/engine/animationinfo.cpp:62-83` - `getAnimationProgress()`
- `Source/engine/render/scrollrt.cpp:1581-1598` - `GetOffsetForWalking()`

### Rust 实现
- `rust-diablo/src/sprite/animation.rs:256-282` - `get_frame_to_use_for_rendering()`
- `rust-diablo/src/sprite/animation.rs:409-418` - `AnimationController::get_frame_to_use_for_rendering()`
- `rust-diablo/src/world/mod.rs:941-949` - 渲染代码（修复后）
- `rust-diablo/src/game.rs:1296-1303` - 渲染代码（修复后）

## 总结

**问题**：渲染时使用了 `current_frame_index()`，没有考虑动画插值，导致画面不切换。

**解决方案**：使用 `get_frame_to_use_for_rendering()` 方法，该方法考虑了动画分布逻辑和插值，与 C++ 版本的 `getFrameToUseForRendering()` 等价。

**修复位置**：
- `rust-diablo/src/sprite/animation.rs` - 添加新方法
- `rust-diablo/src/world/mod.rs` - 更新 3 处渲染代码
- `rust-diablo/src/game.rs` - 更新 1 处渲染代码
