# 精灵表（Sprite Sheet）系统技术文档

## 概述

精灵表系统是游戏渲染的核心组件，用于高效地管理和渲染动画精灵。本文档详细说明了 Rust Diablo 项目中精灵表系统的实现和使用。

## 什么是精灵表？

精灵表（Sprite Sheet）是一张包含多个动画帧的大图片，所有帧按顺序排列在一张纹理中。相比每帧使用单独的图片文件，精灵表有以下优势：

1. **性能优化**：减少纹理切换次数
2. **内存效率**：单张纹理比多张纹理占用更少内存
3. **加载速度**：只需加载一次纹理
4. **组织管理**：所有动画帧集中管理

## 系统架构

### 1. 精灵表生成

**工具**：`assets/create_monkey_sprite.py`

**功能**：
- 使用 PIL/Pillow 库绘制像素艺术
- 自动生成多帧动画
- 支持透明背景（RGBA）

**示例**：猴子精灵表
```
尺寸：256x64 像素
帧数：4 帧
每帧：64x64 像素
布局：[帧1][帧2][帧3][帧4]
```

### 2. 纹理加载

**位置**：`src/engine/mod.rs`

**流程**：
1. 游戏初始化时加载精灵图
2. 使用 `Engine::load_texture(id, path)` 加载
3. 存储在 `TextureManager` 中
4. 通过 ID 访问（如 "player"）

**代码示例**：
```rust
let player_sprite_path = AssetPaths::sprite("player.png");
engine.load_texture("player", &player_sprite_path)?;
```

### 3. 动画系统

**位置**：`src/sprite/animation.rs`

**核心组件**：

#### Animation
- 管理单个动画序列
- 包含帧列表、当前帧索引、帧持续时间
- 支持循环和非循环动画

#### AnimationController
- 管理多个动画状态（Idle, Walk, Attack 等）
- 处理状态切换
- 提供当前帧查询接口

**代码示例**：
```rust
let mut anim_controller = AnimationController::new();

// 添加 Idle 动画（单帧）
let idle_anim = Animation::new(
    vec![Rect::new(0, 0, 64, 64)],  // 帧1
    0.2,
    true  // 循环
);
anim_controller.add_animation(AnimationState::Idle, idle_anim);

// 添加 Walk 动画（多帧）
let walk_anim = Animation::new(
    vec![
        Rect::new(64, 0, 64, 64),   // 帧2
        Rect::new(128, 0, 64, 64),  // 帧3
        Rect::new(192, 0, 64, 64),  // 帧4
    ],
    0.12,  // 每帧 0.12 秒
    true   // 循环
);
anim_controller.add_animation(AnimationState::Walk, walk_anim);
```

### 4. 渲染系统

**位置**：`src/world/mod.rs::render()`

**流程**：
1. 获取实体的当前动画帧矩形
2. 将世界坐标转换为屏幕坐标
3. 使用源矩形和目标矩形绘制纹理

**代码示例**：
```rust
// 获取当前动画帧
let src_rect = entity.animation
    .as_ref()
    .and_then(|anim| anim.current_frame_rect());

// 计算屏幕位置
let screen_pos = camera.world_to_screen(entity.position);
let dst_rect = Rect::from_center(screen_pos, entity.size.0, entity.size.1);

// 绘制纹理
engine.draw_texture_by_id("player", src_rect, dst_rect)?;
```

## 坐标系统

### 精灵表坐标

精灵表中的坐标是**像素坐标**，从左上角 (0, 0) 开始：

```
精灵表：256x64 像素
┌─────────────────────────────────────────┐
│ 帧1 (0,0)    │ 帧2 (64,0)  │ 帧3 (128,0) │ 帧4 (192,0) │
│ 64x64        │ 64x64       │ 64x64      │ 64x64       │
└─────────────────────────────────────────┘
```

### 帧坐标计算

对于水平排列的精灵表：
```rust
fn frame_rect(frame_index: usize, frame_width: u32, frame_height: u32) -> Rect {
    let x = frame_index as i32 * frame_width as i32;
    let y = 0;
    Rect::new(x, y, frame_width, frame_height)
}
```

### 多行动画

如果精灵表有多行，需要计算行和列：

```rust
fn frame_rect(
    frame_index: usize,
    frames_per_row: usize,
    frame_width: u32,
    frame_height: u32
) -> Rect {
    let col = frame_index % frames_per_row;
    let row = frame_index / frames_per_row;
    let x = col as i32 * frame_width as i32;
    let y = row as i32 * frame_height as i32;
    Rect::new(x, y, frame_width, frame_height)
}
```

## 动画更新

### Delta Time 驱动

动画使用真实时间（delta time）而非帧数，确保帧率独立：

```rust
impl Animation {
    pub fn update(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        
        while self.elapsed >= self.frame_duration {
            self.elapsed -= self.frame_duration;
            self.current_frame += 1;
            
            if self.current_frame >= self.frames.len() {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                    self.finished = true;
                }
            }
        }
        
        frame_changed
    }
}
```

### 帧率计算

**帧持续时间** = 1.0 / 目标帧率

示例：
- 8 FPS → `frame_duration = 0.125` 秒
- 12 FPS → `frame_duration = 0.083` 秒
- 24 FPS → `frame_duration = 0.042` 秒

## 最佳实践

### 1. 精灵表设计

**推荐尺寸**：
- 使用 2 的幂次方（64, 128, 256, 512）
- 避免过大的纹理（建议 ≤ 2048x2048）
- 保持帧大小一致

**布局建议**：
- 水平排列：适合时间序列动画
- 垂直排列：适合方向性动画（上下左右）
- 网格排列：适合复杂动画集合

### 2. 动画循环

**平滑循环**：
```rust
// 好的循环：2→3→4→3→2→3→4→3...
vec![
    Rect::new(64, 0, 64, 64),   // 帧2
    Rect::new(128, 0, 64, 64),  // 帧3
    Rect::new(192, 0, 64, 64),  // 帧4
    Rect::new(128, 0, 64, 64),  // 帧3（重复，平滑过渡）
]
```

**避免跳跃**：
```rust
// 不好的循环：2→3→4→2（会有视觉跳跃）
vec![
    Rect::new(64, 0, 64, 64),
    Rect::new(128, 0, 64, 64),
    Rect::new(192, 0, 64, 64),
    // 缺少过渡帧
]
```

### 3. 性能优化

**视口剔除**：
- 只渲染相机范围内的实体
- 减少不必要的绘制调用

**纹理缓存**：
- 使用纹理 ID 而非路径
- 避免重复加载相同纹理

**批量绘制**：
- 未来可以按纹理分组，批量绘制
- 减少状态切换

### 4. 调试技巧

**显示帧信息**：
```rust
if let Some(frame_index) = anim_controller.current_frame_index() {
    println!("Current frame: {}", frame_index);
}
```

**显示动画状态**：
```rust
println!("Animation state: {:?}", anim_controller.current_state());
```

**验证帧坐标**：
```rust
if let Some(rect) = anim_controller.current_frame_rect() {
    println!("Frame rect: {:?}", rect);
}
```

## 扩展方向

### 1. 方向性动画

为 8 个方向添加不同的动画帧：

```rust
enum Direction {
    North, NorthEast, East, SouthEast,
    South, SouthWest, West, NorthWest,
}

// 精灵表布局：
// [N][NE][E][SE]
// [S][SW][W][NW]
```

### 2. 动画事件

在特定帧触发事件：

```rust
impl Animation {
    pub fn on_frame(&self, frame_index: usize, callback: fn()) {
        if self.current_frame == frame_index {
            callback();
        }
    }
}
```

### 3. 动画混合

支持动画之间的平滑过渡：

```rust
pub fn blend_animations(
    from: &Animation,
    to: &Animation,
    t: f32  // 0.0 = from, 1.0 = to
) -> Animation {
    // 实现动画混合逻辑
}
```

## 参考实现

### 原版 Diablo

- **精灵格式**：CEL/CL2/CLX
- **调色板**：256 色调色板
- **方向**：8 方向动画
- **状态**：Idle, Walk, Attack, Hit, Death

### 我们的实现

- **精灵格式**：PNG 精灵表
- **颜色**：RGBA（真彩色）
- **方向**：当前单方向，可扩展
- **状态**：Idle, Walk（可扩展 Attack, Hit, Death）

## 相关文件

- **精灵图生成**：`assets/create_monkey_sprite.py`
- **动画系统**：`src/sprite/animation.rs`
- **实体定义**：`src/entity/mod.rs`
- **渲染逻辑**：`src/world/mod.rs`
- **纹理管理**：`src/sprite/texture.rs`
- **引擎接口**：`src/engine/mod.rs`

## 总结

精灵表系统是游戏渲染的基础，通过合理的设计和实现，可以：

1. ✅ 高效管理动画资源
2. ✅ 提供流畅的动画效果
3. ✅ 支持复杂的动画状态机
4. ✅ 为未来扩展打下基础

随着项目的发展，这个系统将支持更多功能，如方向性动画、动画事件、动画混合等。








