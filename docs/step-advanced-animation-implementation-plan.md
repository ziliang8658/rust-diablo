# 高级动画系统实现计划

## 概述

本计划详细说明如何在 Rust 版本中实现以下高级动画功能：
1. **8方向动画系统** - 支持8个方向的动画播放
2. **帧跳过** - 支持快速攻击等特性
3. **动画分布逻辑** - 平滑渲染和帧跳过分布
4. **行走偏移插值** - 平滑的行走动画偏移

**参考代码**：
- C++ 版本：`Source/engine/animationinfo.h/cpp`
- C++ 版本：`Source/player.cpp::NewPlrAnim()`
- C++ 版本：`Source/engine/render/scrollrt.cpp::GetOffsetForWalking()`

---

## 实现计划

### 阶段 1: 扩展 Direction 系统支持动画索引

**文件**: `rust-diablo/src/engine/direction.rs`

**任务**:
1. 添加 `to_animation_index()` 方法，将 Direction 转换为 CL2 动画索引 (0-7)
2. 添加 `from_animation_index()` 方法，从索引创建 Direction
3. 添加 `to_walking_offset()` 方法，返回行走偏移量（像素）

**实现细节**:
```rust
impl Direction {
    /// 转换为CL2动画索引 (0-7)
    ///
    /// CL2文件中的方向顺序：
    /// 0: South (↓)
    /// 1: SouthWest (↙)
    /// 2: West (←)
    /// 3: NorthWest (↖)
    /// 4: North (↑)
    /// 5: NorthEast (↗)
    /// 6: East (→)
    /// 7: SouthEast (↘)
    pub fn to_animation_index(&self) -> usize {
        match self {
            Direction::South => 0,
            Direction::SouthWest => 1,
            Direction::West => 2,
            Direction::NorthWest => 3,
            Direction::North => 4,
            Direction::NorthEast => 5,
            Direction::East => 6,
            Direction::SouthEast => 7,
            Direction::None => 0, // 默认朝南
        }
    }

    /// 从动画索引创建Direction
    pub fn from_animation_index(index: usize) -> Self {
        match index {
            0 => Direction::South,
            1 => Direction::SouthWest,
            2 => Direction::West,
            3 => Direction::NorthWest,
            4 => Direction::North,
            5 => Direction::NorthEast,
            6 => Direction::East,
            7 => Direction::SouthEast,
            _ => Direction::South, // 默认
        }
    }

    /// 获取行走偏移量（像素）
    ///
    /// 参考 C++: Source/engine/render/scrollrt.cpp:1585
    /// 每个方向的移动偏移量（像素）
    pub fn to_walking_offset(&self) -> (f32, f32) {
        match self {
            Direction::South => (0.0, 32.0),
            Direction::SouthWest => (-32.0, 16.0),
            Direction::West => (-64.0, 0.0),
            Direction::NorthWest => (-32.0, -16.0),
            Direction::North => (0.0, -32.0),
            Direction::NorthEast => (32.0, -16.0),
            Direction::East => (64.0, 0.0),
            Direction::SouthEast => (32.0, 16.0),
            Direction::None => (0.0, 0.0),
        }
    }
}
```

---

### 阶段 2: 扩展 Animation 结构支持帧跳过和分布逻辑

**文件**: `rust-diablo/src/sprite/animation.rs`

**任务**:
1. 添加动画分布逻辑相关字段
2. 实现 `get_animation_progress()` 方法
3. 实现 `set_with_distribution()` 方法支持帧跳过
4. 修改 `update()` 方法支持分布逻辑

**新增字段**:
```rust
pub struct Animation {
    // ... 现有字段 ...

    // 动画分布逻辑相关
    /// 用于分布的帧数（0表示不使用分布逻辑）
    relevant_frames_for_distributing: usize,
    /// 从上一个动画跳过的帧数
    skipped_frames_from_previous: usize,
    /// Tick 修改器（固定点数学，base_value_fraction = 128）
    tick_modifier: u16,
    /// 动画序列开始后的 tick 数（固定点）
    ticks_since_sequence_started: i16,
    /// 是否被石化（暂停动画）
    is_petrified: bool,
}

impl Animation {
    /// 固定点数学的基础值（对应 C++ 的 baseValueFraction = 128）
    pub const BASE_VALUE_FRACTION: u8 = 128;

    /// 获取动画进度（0.0 - 1.0）
    ///
    /// 参考 C++: Source/engine/animationinfo.cpp:62-83
    pub fn get_animation_progress(&self) -> f32 {
        // 实现固定点数学计算
        // 返回 0.0 到 1.0 之间的值
    }

    /// 设置新动画，支持帧跳过和分布逻辑
    ///
    /// 参考 C++: Source/engine/animationinfo.cpp:85-182
    pub fn set_with_distribution(
        &mut self,
        frames: Vec<Rect>,
        frame_duration: f32,
        looping: bool,
        num_skipped_frames: usize,
        distribute_frames_before_frame: usize,
        process_animation_pending: bool,
    ) {
        // 实现动画分布逻辑
    }
}
```

---

### 阶段 3: 扩展 AnimationController 支持8方向

**文件**: `rust-diablo/src/sprite/animation.rs`

**任务**:
1. 修改 AnimationController 存储结构，支持 (AnimationState, Direction) 键
2. 添加 `set_direction()` 方法
3. 修改 `set_state()` 方法，同时设置方向
4. 更新所有相关方法

**新的数据结构**:
```rust
/// 动画控制器 - 支持8方向
#[derive(Clone)]
pub struct AnimationController {
    /// 动画集合: (状态, 方向) -> 动画
    animations: HashMap<(AnimationState, Direction), Animation>,
    current_state: AnimationState,
    current_direction: Direction,
    previous_state: AnimationState,
    previous_direction: Direction,
}

impl AnimationController {
    /// 设置当前方向
    pub fn set_direction(&mut self, direction: Direction) {
        if direction != self.current_direction {
            self.previous_direction = self.current_direction;
            self.current_direction = direction;
            // 如果当前状态有该方向的动画，重置动画
            if let Some(anim) = self.animations.get_mut(&(self.current_state, direction)) {
                anim.reset();
            }
        }
    }

    /// 设置动画状态和方向
    pub fn set_state_and_direction(&mut self, state: AnimationState, direction: Direction) {
        let changed = state != self.current_state || direction != self.current_direction;
        if changed {
            self.previous_state = self.current_state;
            self.previous_direction = self.current_direction;
            self.current_state = state;
            self.current_direction = direction;
            if let Some(anim) = self.animations.get_mut(&(state, direction)) {
                anim.reset();
            }
        }
    }

    /// 添加动画（指定状态和方向）
    pub fn add_animation(&mut self, state: AnimationState, direction: Direction, animation: Animation) {
        self.animations.insert((state, direction), animation);
    }
}
```

---

### 阶段 4: 修改 CL2 加载逻辑支持8方向解析

**文件**: `rust-diablo/src/resources/cl2.rs`

**任务**:
1. 添加 `from_bytes_multi_direction()` 方法，解析包含8方向的 CL2 文件
2. 返回 `Vec<Vec<ClxFrame>>`，外层是8个方向，内层是每个方向的帧

**实现**:
```rust
impl Cl2Sprite {
    /// 解析包含8方向的CL2文件
    ///
    /// CL2文件结构：
    /// - 文件头：8个方向的偏移量（每个4字节）
    /// - 每个方向：帧数 + 帧偏移表 + 帧数据
    ///
    /// 返回: Vec[8][frames]，8个方向，每个方向包含多个帧
    pub fn from_bytes_multi_direction(data: &[u8], frame_width: u16) -> Result<Vec<Vec<ClxFrame>>> {
        // 1. 读取8个方向的偏移量
        // 2. 对每个方向调用 from_bytes_single_direction()
        // 3. 返回8个方向的帧列表
    }

    /// 解析单个方向的CL2数据
    fn from_bytes_single_direction(data: &[u8], offset: usize, frame_width: u16) -> Result<Vec<ClxFrame>> {
        // 复用现有的解析逻辑
    }
}
```

---

### 阶段 5: 修改资源加载逻辑支持8方向纹理

**文件**: `rust-diablo/src/game.rs`

**任务**:
1. 修改 CL2 加载代码，使用 `from_bytes_multi_direction()`
2. 为每个方向的每一帧创建纹理
3. 纹理 ID 格式：`{sprite_name}_{state}_{direction}_{frame_index}`
4. 更新动画配置，为每个方向创建动画

**实现**:
```rust
// 在 game.rs 的 Phase 5 中
match Cl2Sprite::from_bytes_multi_direction(&idle_data.unwrap(), frame_width) {
    Ok(idle_directions) => {
        // idle_directions: Vec[8][frames]
        for (dir_idx, direction_frames) in idle_directions.iter().enumerate() {
            let direction = Direction::from_animation_index(dir_idx);
            for (frame_idx, frame) in direction_frames.iter().enumerate() {
                let texture_id = format!("{}_idle_{}_{}",
                    sprite_name, dir_idx, frame_idx);
                // 创建纹理...
            }
        }
    }
}
```

---

### 阶段 6: 实现行走偏移插值

**文件**: `rust-diablo/src/entity/mod.rs`

**任务**:
1. 添加 `get_walking_offset()` 方法
2. 在渲染时应用偏移

**实现**:
```rust
impl Entity {
    /// 获取行走偏移量（像素）
    ///
    /// 参考 C++: Source/engine/render/scrollrt.cpp:1581-1598
    pub fn get_walking_offset(&self, tile_size: u32) -> (f32, f32) {
        if !self.walking {
            return (0.0, 0.0);
        }

        if let Some(ref anim) = self.animation {
            // 获取动画进度 (0.0 - 1.0)
            let progress = if let Some(current_anim) = anim.get_current_animation() {
                current_anim.get_animation_progress()
            } else {
                0.0
            };

            // 获取方向的行走偏移
            let (dx, dy) = if let Some(walk_dir) = self.walk_direction {
                walk_dir.to_walking_offset()
            } else {
                self.direction.to_walking_offset()
            };

            // 根据动画进度插值
            (dx * progress, dy * progress)
        } else {
            (0.0, 0.0)
        }
    }
}
```

---

### 阶段 7: 更新渲染逻辑应用行走偏移

**文件**: `rust-diablo/src/world/mod.rs`

**任务**:
1. 在渲染实体时获取行走偏移
2. 应用偏移到渲染位置
3. 更新纹理 ID 格式以包含方向

**实现**:
```rust
// 在 render_entities 中
let (walk_offset_x, walk_offset_y) = entity.get_walking_offset(self.tile_size);
let screen_pos = Point::new(
    screen_pos.x + walk_offset_x as i32,
    screen_pos.y + walk_offset_y as i32,
);

// 构建纹理 ID（包含方向）
let direction_index = entity.direction.to_animation_index();
let texture_id = format!("{}_{}_{}_{}",
    base_sprite_id, state_name, direction_index, frame_index);
```

---

### 阶段 8: 更新 Entity::update() 使用新的动画系统

**文件**: `rust-diablo/src/entity/mod.rs`

**任务**:
1. 修改 `start_walk()` 使用新的 `set_state_and_direction()`
2. 更新 `update()` 方法，正确处理方向切换

**实现**:
```rust
pub fn start_walk<F>(&mut self, direction: Direction, is_walkable_fn: Option<F>) -> bool {
    // ...
    if let Some(ref mut anim) = self.animation {
        anim.set_state_and_direction(AnimationState::Walk, direction);
    }
    // ...
}
```

---

## 实现检查清单

### 阶段 1: Direction 扩展
- [ ] 添加 `to_animation_index()`
- [ ] 添加 `from_animation_index()`
- [ ] 添加 `to_walking_offset()`
- [ ] 添加单元测试

### 阶段 2: Animation 扩展
- [ ] 添加分布逻辑字段
- [ ] 实现 `get_animation_progress()`
- [ ] 实现 `set_with_distribution()`
- [ ] 修改 `update()` 支持分布逻辑
- [ ] 添加单元测试

### 阶段 3: AnimationController 扩展
- [ ] 修改数据结构支持 (State, Direction) 键
- [ ] 添加 `set_direction()`
- [ ] 添加 `set_state_and_direction()`
- [ ] 更新所有相关方法
- [ ] 添加单元测试

### 阶段 4: CL2 多方向解析
- [ ] 实现 `from_bytes_multi_direction()`
- [ ] 实现 `from_bytes_single_direction()`
- [ ] 添加错误处理
- [ ] 添加单元测试

### 阶段 5: 资源加载更新
- [ ] 修改 game.rs 使用新的 CL2 解析
- [ ] 更新纹理 ID 格式
- [ ] 为每个方向创建动画
- [ ] 测试资源加载

### 阶段 6: 行走偏移实现
- [ ] 实现 `get_walking_offset()`
- [ ] 集成动画进度计算
- [ ] 添加单元测试

### 阶段 7: 渲染更新
- [ ] 更新纹理 ID 格式
- [ ] 应用行走偏移
- [ ] 测试渲染效果

### 阶段 8: Entity 更新
- [ ] 更新 `start_walk()`
- [ ] 更新 `update()`
- [ ] 测试移动和动画

---

## 测试计划

### 单元测试
1. Direction 转换测试
2. Animation 分布逻辑测试
3. AnimationController 方向切换测试
4. CL2 多方向解析测试

### 集成测试
1. 8方向动画播放测试
2. 行走偏移插值测试
3. 帧跳过功能测试
4. 动画分布逻辑测试

### 视觉测试
1. 8方向行走动画流畅性
2. 行走偏移平滑度
3. 帧跳过效果
4. 动画切换流畅度

---

## 参考实现细节

### C++ 动画分布逻辑关键点

1. **固定点数学**: 使用 `baseValueFraction = 128` 进行固定点计算
2. **帧跳过**: `numSkippedFrames` 跳过指定数量的帧
3. **分布帧数**: `distributeFramesBeforeFrame` 指定在哪个帧之前分布
4. **预览片段**: `previewShownGameTickFragments` 处理预览动画的时间

### 行走偏移关键点

1. **偏移量**: 每个方向有固定的像素偏移量
2. **进度插值**: 根据动画进度 (0-128) 插值计算偏移
3. **相机模式**: 相机模式下偏移需要取反

---

## 预期代码量

- Direction 扩展: ~50 行
- Animation 扩展: ~200 行
- AnimationController 扩展: ~150 行
- CL2 多方向解析: ~150 行
- 资源加载更新: ~100 行
- 行走偏移实现: ~50 行
- 渲染更新: ~50 行
- Entity 更新: ~50 行
- **总计**: ~800 行代码

---

## 注意事项

1. **向后兼容**: 确保现有代码仍然可以工作
2. **性能**: 固定点数学计算需要注意性能
3. **测试**: 每个阶段都要进行充分测试
4. **文档**: 更新相关文档说明新功能

---

**计划版本**: v1.0
**创建日期**: 2025-01-XX
**预计完成时间**: 2-3 天
