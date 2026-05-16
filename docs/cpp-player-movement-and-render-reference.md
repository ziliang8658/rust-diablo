# C++ 玩家移动与渲染动画逻辑 — 提炼参考

本文档提炼 DevilutionX 中玩家移动与行走动画渲染的核心逻辑，便于与 Rust 实现对照或复刻。

**参考源码路径：**
- `Source/player.cpp` — 移动、DoWalk、HandleWalkMode、StartWalk、ProcessPlayers
- `Source/player.h` — Player、_pmode、_pdir、getRenderingOffset、occupyTile
- `Source/engine/render/scrollrt.cpp` — DrawCell 内玩家分支、GetOffsetForWalking、DrawPlayer
- `Source/engine/animationinfo.cpp` — processAnimation、getAnimationProgress
- `Source/engine/actor_position.hpp` — tile / temp / future、CalculateWalkingOffset

---

## 一、核心数据结构（提炼）

```text
// 玩家模式：站立 / 三种行走 / 攻击 / 施法 / 死亡 等
enum PLR_MODE {
    PM_STAND,
    PM_WALK_NORTHWARDS,   // 北、西北、东北 → 在源格画
    PM_WALK_SOUTHWARDS,   // 南、西南、东南 → 在目标格画再挪回源格
    PM_WALK_SIDEWAYS,     // 东、西 → 东在目标格画，西在源格画
    PM_ATTACK, PM_RATTACK, PM_SPELL, PM_BLOCK, PM_GOTHIT, PM_DEATH, ...
};

// 8 方向
enum Direction { South, SouthWest, West, NorthWest, North, NorthEast, East, SouthEast };

// 每方向对应：逻辑位移 dir + 使用的 walk 模式
struct DirectionSettings { Direction dir; PLR_MODE walkMode; };
WalkSettings[8] = {
    { South,     PM_WALK_SOUTHWARDS },
    { SouthWest, PM_WALK_SOUTHWARDS },
    { West,      PM_WALK_SIDEWAYS   },
    { NorthWest, PM_WALK_NORTHWARDS },
    { North,     PM_WALK_NORTHWARDS },
    { NorthEast, PM_WALK_NORTHWARDS },
    { East,      PM_WALK_SIDEWAYS   },
    { SouthEast, PM_WALK_SOUTHWARDS },
};

// 玩家位置（格子）
struct ActorPosition {
    Point tile;   // 当前所在格（逻辑位置）
    Point temp;   // 本步目标格，DoWalk 结束时 tile = temp
    Point future; // 本步目标格，与 temp 一致，用于寻路/占用
};

// 占格表：正 id = 站立占格，负 id = 移动中占目标格
int8_t dPlayer[MAXDUNX][MAXDUNY];
void occupyTile(Point tilePosition, bool isMoving) {
    dPlayer[tilePosition.x][tilePosition.y] = isMoving ? -(id+1) : (id+1);
}

// 动画状态（每 game tick 推进）
struct AnimationInfo {
    int8_t currentFrame;
    int8_t tickCounterOfCurrentFrame;
    int8_t ticksPerFrame;  // Walk 为 1
    int8_t numberOfFrames;
    // getAnimationProgress() 用 ticksSinceSequenceStarted + getProgressToNextGameTick 算出 0..baseValueFraction
};
```

---

## 二、移动逻辑（提炼代码）

### 2.1 开始一次行走

```text
// 输入/寻路决定方向 dir 后调用
fn StartWalk(player, dir, pmWillBeCalled) {
    StartWalkAnimation(player, dir, pmWillBeCalled);
    HandleWalkMode(player, dir);
}
```

#### StartWalkAnimation 具体实现

```text
// Source/player.cpp:144-151
fn StartWalkAnimation(player, dir, pmWillBeCalled) {
    numSkippedFrames = -2;
    if leveltype == DTYPE_TOWN && bRunInTown != 0 then numSkippedFrames = 2;
    if pmWillBeCalled then numSkippedFrames += 1;
    NewPlrAnim(player, player_graphic::Walk, dir,
               AnimationDistributionFlags::ProcessAnimationPending,
               numSkippedFrames);
}

// NewPlrAnim (player.cpp:2219-2235)：为 Walk 设定动画并启播
fn NewPlrAnim(player, graphic, dir, flags, numSkippedFrames, distributeFramesBeforeFrame, previewShownGameTickFragments) {
    LoadPlrGFX(player, graphic);   // 若未加载则加载该 graphic（Walk）的 CL2 精灵
    sprites = player.AnimationData[graphic].spritesForDirection(dir);  // 取该方向的帧列表
    player.getAnimationFramesAndTicksPerFrame(graphic, &numberOfFrames, &ticksPerFrame);
    // Walk: numberOfFrames = _pWFrames（来自 PlayerAnimData），ticksPerFrame = 1
    player.AnimInfo.setNewAnimation(sprites, numberOfFrames, ticksPerFrame,
                                    flags, numSkippedFrames, distributeFramesBeforeFrame,
                                    previewShownGameTickFragments);
}

// getAnimationFramesAndTicksPerFrame 对 Walk (player.cpp:1825-1827)
//   numberOfFrames = player._pWFrames;
//   ticksPerFrame 保持 1（默认值），即每 game tick 进一帧

// setNewAnimation 对 Walk 的效果 (animationinfo.cpp:86-183 简化)
//   sprites = 该方向 Walk 的 CL2 帧列表
//   currentFrame = numSkippedFrames（可能为负或 0，用于跳过前几帧加速起播）
//   tickCounterOfCurrentFrame = 0
//   ticksPerFrame = 1
//   ticksSinceSequenceStarted_ = 0（若 ProcessAnimationPending 则先置 -baseValueFraction，本 tick 会立刻 processAnimation 一次）
//   relevantFramesForDistributing_、tickModifier_ 等用于 getAnimationProgress / getFrameToUseForRendering 的插值
```

fn HandleWalkMode(player, dir) {
    (dir_mode, walk_mode) = WalkSettings[dir];
    if !PlrDirOK(player, dir) { return; }  // 目标格可走性

    player._pdir = dir;
    player.position.future = player.position.tile + dir_mode;
    // 占目标格为“移动中”（负 id）
    occupyTile(player.position.future, true);
    player.position.temp = player.position.tile + dir_mode;
    player._pmode = walk_mode;  // PM_WALK_NORTHWARDS / SOUTHWARDS / SIDEWAYS
}
```

### 2.2 每 game tick 推进（ProcessPlayers 内）

```text
for each player in active players {
    loop {
        tplayer = false;
        switch player._pmode {
            case PM_WALK_*:
                tplayer = DoWalk(player);
                break;
            case PM_STAND: ...
            case PM_ATTACK: ...
        }
        CheckNewPath(player, tplayer);  // 若 tplayer 可能立刻 StartWalk 下一步
    } while tplayer;

    player.AnimInfo.processAnimation();  // tick 计数 +1，必要时 currentFrame++
}
```

### 2.3 单步行走推进 DoWalk

```text
fn DoWalk(player) -> bool {
    if !player.AnimInfo.isLastFrame() {
        UpdatePlayerLightOffset(player);  // 用 getAnimationProgress 更新光照子格偏移
        return false;
    }

    // 动画到最后一帧：换格
    dPlayer[player.position.tile.x][player.position.tile.y] = 0;
    player.position.tile = player.position.temp;
    occupyTile(player.position.tile, false);  // 新格标为正 id
    StartStand(player, player.tempDirection);
    ClearStateVariables(player);
    return true;
}
```

---

## 三、动画逻辑（提炼代码）

### 3.1 每 tick 推进一帧

```text
fn processAnimation() {
    tickCounterOfCurrentFrame += 1;
    ticksSinceSequenceStarted += baseValueFraction;
    if tickCounterOfCurrentFrame >= ticksPerFrame {
        tickCounterOfCurrentFrame = 0;
        currentFrame += 1;
        if currentFrame >= numberOfFrames { currentFrame = 0; }
    }
}
```

### 3.2 行走进度（0..1，用于像素偏移）

```text
// 简化：无 distribution 时
fn getAnimationProgress() -> u8 {
    ticksSinceSequenceStarted = (currentFrame * ticksPerFrame + tickCounterOfCurrentFrame) * baseValueFraction;
    totalTicks = getProgressToNextGameTick() + ticksSinceSequenceStarted;
    progressInFrames = totalTicks * (baseValueFraction / ticksPerFrame);
    return (progressInFrames / numberOfFrames / baseValueFraction).min(baseValueFraction);
}
```

### 3.3 行走像素偏移（8 方向）

```text
MovingOffset[8] = {
    (0, 32), (-32, 16), (-64, 0), (-32, -16),
    (0, -32), (32, -16), (64, 0), (32, 16)
};  // South, SouthWest, West, NorthWest, North, NorthEast, East, SouthEast

fn GetOffsetForWalking(animationInfo, dir, cameraMode) -> (dx, dy) {
    progress = animationInfo.getAnimationProgress() / baseValueFraction;  // 0..1
    (dx, dy) = MovingOffset[dir] * progress;
    if cameraMode { (dx, dy) = (-dx, -dy); }
    return (dx, dy);
}
```

---

## 四、渲染逻辑（提炼代码）

### 4.1 按格绘制时的玩家分支（DrawCell 内）

```text
// 当前绘制到格子 tilePosition，屏幕坐标 targetBufferPosition
player = PlayerAtPosition(tilePosition);  // 查 dPlayer[tilePosition]，0 则无
if player == nil { return; }

playerId = player.getId() + 1;
if player._pmode == PM_WALK_SOUTHWARDS || (player._pmode == PM_WALK_SIDEWAYS && player._pdir == East) {
    playerId = -playerId;
}
if dPlayer[tilePosition.x][tilePosition.y] != playerId { return; }

// 南/东走：当前迭代的是目标格，把绘制锚点改回源格
drawTile = tilePosition;
drawBufferPos = targetBufferPosition;
if player._pmode == PM_WALK_SOUTHWARDS {
    switch player._pdir {
        SouthWest: drawBufferPos += (TILE_WIDTH/2, -TILE_HEIGHT/2);
        South:     drawBufferPos += (0, -TILE_HEIGHT);
        SouthEast: drawBufferPos += (-TILE_WIDTH/2, -TILE_HEIGHT/2);
    }
    drawTile += Opposite(player._pdir);
} else if player._pmode == PM_WALK_SIDEWAYS && player._pdir == East {
    drawBufferPos += (-TILE_WIDTH, 0);
    drawTile += Opposite(player._pdir);
}

DrawPlayer(out, player, drawTile, drawBufferPos, lightTableIndex);
```

### 4.2 DrawPlayer：最终绘制位置

```text
fn DrawPlayer(out, player, tilePosition, targetBufferPosition, lightTableIndex) {
    sprite = player.currentSprite();  // 用 getFrameToUseForRendering() 取帧
    spriteBufferPosition = targetBufferPosition + player.getRenderingOffset(sprite);
    ClxDraw(out, spriteBufferPosition, sprite);
}

fn getRenderingOffset(sprite) -> (dx, dy) {
    dx = -CalculateSpriteTileCenterX(sprite.width());  // (width - TILE_WIDTH) / 2
    dy = 0;
    if player.isWalking() {
        (wx, wy) = GetOffsetForWalking(player.AnimInfo, player._pdir);
        dx += wx; dy += wy;
    }
    return (dx, dy);
}
```

---

## 五、流程串联（时间顺序）

1. **输入** → `StartWalk(player, dir)` → `HandleWalkMode`：设 `_pdir`、`position.future`/`temp`、`occupyTile(future, true)`、`_pmode = WalkSettings[dir].walkMode`；`NewPlrAnim(Walk, dir)` 启播行走动画。
2. **每 game tick**：`ProcessPlayers` → 对每个玩家先 `DoWalk`（未到最后一帧则只更新光照并 return false；到最后一帧则 `tile = temp`、清旧格、`occupyTile(tile, false)`、`StartStand`），再 `CheckNewPath`（可能连续 `StartWalk`），最后 `AnimInfo.processAnimation()`。
3. **每渲染帧**：按格子遍历调用 `DrawCell`；当 `dPlayer[tile] == ±playerId` 时，按 3 种 walk 模式决定是否把绘制锚点从当前格改到源格，再 `DrawPlayer`；`DrawPlayer` 内用 `targetBufferPosition + getRenderingOffset(sprite)` 得到 `spriteBufferPosition` 并绘制精灵。

---

## 六、与 Rust 实现的差异摘要

| 项目           | C++ | Rust |
|----------------|-----|------|
| 时间驱动       | 固定 game tick，每 tick 一次 processAnimation | 可变 dt，elapsed += dt |
| 换格时机       | DoWalk 中 isLastFrame() 时 tile = temp | walk_progress >= 1 时 tile_position = target |
| 占格与绘制顺序 | dPlayer + occupyTile；按格画；南/东在目标格画再挪回源格 | 无 dPlayer；按实体画在当前格 |
| 8 方向 / 3 模式 | WalkSettings 映射 3 种 _pmode，参与绘制分支 | 仅 8 方向，无 3 模式 |
| 行走进度       | getAnimationProgress()（tick + 帧内插值） | walk_progress 或 get_animation_progress() |
| 偏移计算       | MovingOffset[8] * progress + getRenderingOffset | to_walking_offset() * progress，加到 dst_rect |

---

## 七、参考文件清单

| 逻辑         | 文件与函数/位置 |
|--------------|------------------|
| 开始行走     | player.cpp: StartWalk, HandleWalkMode, WalkInDirection, WalkSettings |
| 每 tick 推进 | player.cpp: ProcessPlayers, DoWalk |
| 占格         | player.cpp: occupyTile; player.h: occupyTile 声明 |
| 动画推进     | engine/animationinfo.cpp: processAnimation, getAnimationProgress |
| 行走偏移     | engine/render/scrollrt.cpp: GetOffsetForWalking |
| 按格画玩家   | engine/render/scrollrt.cpp: DrawCell 内 dPlayer 分支、DrawPlayer |
| 绘制偏移     | player.h: getRenderingOffset; scrollrt.cpp: DrawPlayer |
