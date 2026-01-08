# 图形学与游戏调试技巧大全

## 概述

本文档总结了在 Rust Diablo 项目开发过程中积累的图形学和游戏调试经验，涵盖从像素级调试到系统级调试的各种技巧。

## 1. 数据数值化对比

### 1.1 像素值对比

**适用场景**: 像素排列、颜色转换、光照计算等

**方法**:
- 将像素数据输出到文本文件，逐像素对比
- 使用十六进制或十进制格式，便于查找差异
- 按行/列组织，便于定位问题区域

**示例**:
```rust
// 输出 indexed pixels
for row in 0..height {
    let row_data: Vec<String> = pixels[row*width..(row+1)*width]
        .iter()
        .map(|p| format!("{:3}", p))
        .collect();
    println!("row {}: {}", row, row_data.join(","));
}
```

**优势**:
- 精确到像素级别
- 可以自动化对比
- 便于发现规律

**局限**:
- 数据量大时难以阅读
- 需要理解数据格式

### 1.2 坐标值对比

**适用场景**: 坐标转换、碰撞检测、渲染位置

**方法**:
- 输出关键坐标点（世界坐标、屏幕坐标、纹理坐标）
- 对比 C++ 和 Rust 的坐标转换结果
- 记录坐标变换链

**示例**:
```rust
println!("World: ({}, {}) -> Screen: ({}, {})", 
    world_x, world_y, screen_x, screen_y);
```

### 1.3 状态值对比

**适用场景**: 游戏状态、动画帧、资源索引

**方法**:
- 输出状态机状态、动画帧索引、资源ID等
- 对比状态转换时机
- 记录状态历史

## 2. 可视化调试工具

### 2.1 颜色编码调试

**原理**: 用不同颜色表示不同的状态或值

**实现**:
```rust
// 用颜色表示 tile type
let debug_color = match tile_type {
    TileType::Square => Color::RED,
    TileType::LeftTriangle => Color::GREEN,
    TileType::RightTriangle => Color::BLUE,
    // ...
};
```

**应用场景**:
- Tile 类型可视化
- 碰撞区域可视化
- 光照等级可视化
- 渲染层级可视化

**优势**:
- 直观快速
- 可以实时查看
- 适合大面积问题定位

### 2.2 边界框/线框渲染

**原理**: 绘制几何体的边界，而不是填充

**实现**:
```rust
// 绘制 tile 边界框
fn draw_tile_bounds(&self, engine: &mut Engine, rect: Rect) {
    let color = Color::YELLOW;
    // 绘制四条边
    engine.draw_line(rect.left(), rect.top(), rect.right(), rect.top(), color);
    engine.draw_line(rect.right(), rect.top(), rect.right(), rect.bottom(), color);
    engine.draw_line(rect.right(), rect.bottom(), rect.left(), rect.bottom(), color);
    engine.draw_line(rect.left(), rect.bottom(), rect.left(), rect.top(), color);
}
```

**应用场景**:
- 碰撞检测区域
- 渲染区域验证
- 坐标系统验证
- 裁剪区域可视化

### 2.3 网格/辅助线渲染

**原理**: 绘制参考网格，帮助理解坐标系统

**实现**:
```rust
// 绘制等轴测网格
fn draw_isometric_grid(&self, engine: &mut Engine) {
    let color = Color::GRAY;
    // 绘制水平线
    for y in 0..viewport_height {
        if y % TILE_HEIGHT == 0 {
            engine.draw_line(0, y, viewport_width, y, color);
        }
    }
    // 绘制对角线
    // ...
}
```

**应用场景**:
- 等轴测投影验证
- Tile 对齐检查
- 坐标系统理解

### 2.4 热力图/渐变图

**原理**: 用颜色强度表示数值大小

**实现**:
```rust
// 光照热力图
fn draw_light_heatmap(&self, engine: &mut Engine, light_levels: &[u8]) {
    for (i, &level) in light_levels.iter().enumerate() {
        let intensity = level as f32 / 15.0; // 0-1
        let color = Color::rgb(
            (intensity * 255.0) as u8,
            0,
            ((1.0 - intensity) * 255.0) as u8
        );
        // 绘制像素
    }
}
```

**应用场景**:
- 光照分布
- 性能热点
- 密度分布

## 3. 分层/隔离渲染

### 3.1 单层渲染

**原理**: 只渲染特定层级，隔离问题

**实现**:
```rust
// 只渲染 floor tiles
if render_layer == RenderLayer::Floor {
    self.render_floor_tiles(engine, texture_mgr)?;
}

// 只渲染 walls
if render_layer == RenderLayer::Walls {
    self.render_walls(engine, texture_mgr)?;
}
```

**应用场景**:
- 定位渲染层级问题
- 性能分析
- 视觉对比

### 3.2 单对象渲染

**原理**: 只渲染特定对象，排除干扰

**实现**:
```rust
// 只渲染特定 piece
if piece_id == DEBUG_PIECE_ID {
    self.render_piece(engine, texture_mgr, piece_id)?;
}
```

**优势**:
- 快速定位问题对象
- 便于对比 C++ 和 Rust
- 减少输出干扰

### 3.3 区域裁剪

**原理**: 只渲染屏幕的特定区域

**实现**:
```rust
// 只渲染中心区域
let center_x = viewport_width / 2;
let center_y = viewport_height / 2;
let tolerance = 200;

if (screen_x - center_x).abs() < tolerance && 
   (screen_y - center_y).abs() < tolerance {
    // render...
}
```

## 4. 时间轴/帧分析

### 4.1 帧计数器

**原理**: 记录每帧的渲染信息

**实现**:
```rust
struct FrameDebug {
    frame_count: u64,
    render_calls: usize,
    tiles_rendered: usize,
    draw_calls: usize,
}

impl FrameDebug {
    fn log_frame(&self) {
        println!("Frame {}: {} tiles, {} draw calls", 
            self.frame_count, 
            self.tiles_rendered, 
            self.draw_calls);
    }
}
```

**应用场景**:
- 性能分析
- 渲染统计
- 帧率问题定位

### 4.2 时间戳记录

**原理**: 记录关键操作的时间点

**实现**:
```rust
use std::time::Instant;

let start = Instant::now();
self.render_floor_tiles(engine, texture_mgr)?;
let duration = start.elapsed();
println!("Floor rendering took: {:?}", duration);
```

**应用场景**:
- 性能瓶颈定位
- 优化效果验证
- 异步操作调试

### 4.3 帧回放

**原理**: 记录帧数据，可以回放分析

**实现**:
```rust
struct FrameRecorder {
    frames: Vec<FrameData>,
}

struct FrameData {
    frame_number: u64,
    player_pos: Point,
    tiles: Vec<TileData>,
    // ...
}
```

**应用场景**:
- 重现偶发问题
- 对比不同实现
- 自动化测试

## 5. 状态机可视化

### 5.1 状态转换日志

**原理**: 记录状态机的所有转换

**实现**:
```rust
enum PlayerState {
    Idle,
    Walking,
    Attacking,
    // ...
}

impl PlayerState {
    fn transition(&mut self, new_state: Self) {
        println!("State transition: {:?} -> {:?}", self, new_state);
        *self = new_state;
    }
}
```

### 5.2 状态可视化

**原理**: 在屏幕上显示当前状态

**实现**:
```rust
fn draw_debug_info(&self, engine: &mut Engine) {
    let state_text = format!("State: {:?}", self.current_state);
    engine.draw_text(10, 10, &state_text, Color::WHITE);
}
```

## 6. 坐标系统可视化

### 6.1 坐标轴绘制

**原理**: 绘制世界坐标轴和屏幕坐标轴

**实现**:
```rust
fn draw_coordinate_axes(&self, engine: &mut Engine) {
    // 世界坐标原点
    let world_origin = self.world_to_screen(Point::new(0, 0));
    
    // 绘制 X 轴（红色）
    engine.draw_line(
        world_origin.x, world_origin.y,
        world_origin.x + 100, world_origin.y,
        Color::RED
    );
    
    // 绘制 Y 轴（绿色）
    engine.draw_line(
        world_origin.x, world_origin.y,
        world_origin.x, world_origin.y - 100,
        Color::GREEN
    );
}
```

### 6.2 坐标转换可视化

**原理**: 显示坐标转换链

**实现**:
```rust
fn debug_coordinate_transform(&self, world_pos: Point) {
    let screen_pos = self.world_to_screen(world_pos);
    let dpiece_pos = self.world_to_dpiece(world_pos);
    
    println!("World: {:?} -> Screen: {:?} -> dPiece: {:?}", 
        world_pos, screen_pos, dpiece_pos);
}
```

## 7. 断言和验证

### 7.1 数据范围断言

**原理**: 验证数据在合理范围内

**实现**:
```rust
fn render_tile(&self, x: i32, y: i32) {
    assert!(x >= 0 && x < viewport_width, "x out of bounds: {}", x);
    assert!(y >= 0 && y < viewport_height, "y out of bounds: {}", y);
    // ...
}
```

### 7.2 不变量检查

**原理**: 检查系统不变量

**实现**:
```rust
fn validate_tile_data(&self, tile: &Tile) {
    assert_eq!(tile.pixels.len(), TILE_WIDTH * TILE_HEIGHT);
    assert!(tile.frame_index < MAX_FRAMES);
    // ...
}
```

### 7.3 一致性检查

**原理**: 检查不同表示之间的一致性

**实现**:
```rust
fn validate_coordinate_consistency(&self, world_pos: Point) {
    let screen1 = self.world_to_screen(world_pos);
    let screen2 = self.world_to_screen_v2(world_pos);
    assert_eq!(screen1, screen2, "Coordinate conversion mismatch");
}
```

## 8. 性能分析工具

### 8.1 性能计数器

**原理**: 统计各种操作的次数和耗时

**实现**:
```rust
struct PerformanceCounter {
    render_calls: AtomicUsize,
    texture_uploads: AtomicUsize,
    total_render_time: AtomicU64, // nanoseconds
}

impl PerformanceCounter {
    fn report(&self) {
        let calls = self.render_calls.load(Ordering::Relaxed);
        let time = self.total_render_time.load(Ordering::Relaxed);
        let avg_time = time / calls as u64;
        println!("Render: {} calls, avg: {}ns", calls, avg_time);
    }
}
```

### 8.2 内存使用监控

**原理**: 监控内存分配和释放

**实现**:
```rust
// 使用自定义分配器或 hook
#[global_allocator]
static ALLOC: TrackingAllocator = TrackingAllocator;

struct TrackingAllocator;

unsafe impl GlobalAllocator for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        println!("Alloc: {} bytes", layout.size());
        ptr
    }
    // ...
}
```

## 9. 回放/录制系统

### 9.1 输入录制

**原理**: 录制用户输入，可以回放

**实现**:
```rust
struct InputRecorder {
    inputs: Vec<InputEvent>,
}

struct InputEvent {
    frame: u64,
    key: Key,
    pressed: bool,
}

impl InputRecorder {
    fn record(&mut self, frame: u64, key: Key, pressed: bool) {
        self.inputs.push(InputEvent { frame, key, pressed });
    }
    
    fn replay(&self, frame: u64) -> Option<&InputEvent> {
        self.inputs.iter().find(|e| e.frame == frame)
    }
}
```

### 9.2 状态快照

**原理**: 保存游戏状态的快照

**实现**:
```rust
#[derive(Serialize, Deserialize)]
struct GameSnapshot {
    frame: u64,
    player_pos: Point,
    tiles: Vec<TileSnapshot>,
    // ...
}

impl World {
    fn save_snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            frame: self.frame_count,
            player_pos: self.player.position,
            tiles: self.tiles.iter().map(|t| t.snapshot()).collect(),
        }
    }
    
    fn load_snapshot(&mut self, snapshot: GameSnapshot) {
        // 恢复状态
    }
}
```

## 10. 自动化对比工具

### 10.1 图像对比

**原理**: 自动对比 C++ 和 Rust 的渲染结果

**实现**:
```rust
fn compare_renders(cpp_image: &Image, rust_image: &Image) -> ComparisonResult {
    let mut diff_count = 0;
    let mut total_diff = 0;
    
    for (cpp_pixel, rust_pixel) in cpp_image.pixels().zip(rust_image.pixels()) {
        if cpp_pixel != rust_pixel {
            diff_count += 1;
            total_diff += pixel_diff(cpp_pixel, rust_pixel);
        }
    }
    
    ComparisonResult {
        diff_pixels: diff_count,
        total_diff: total_diff,
        similarity: 1.0 - (total_diff as f32 / (cpp_image.width() * cpp_image.height() * 255 * 3) as f32),
    }
}
```

### 10.2 数据对比脚本

**原理**: 用脚本自动对比输出文件

**实现** (Python):
```python
def compare_pixel_files(cpp_file, rust_file):
    cpp_data = parse_pixel_file(cpp_file)
    rust_data = parse_pixel_file(rust_file)
    
    differences = []
    for row in range(len(cpp_data)):
        for col in range(len(cpp_data[row])):
            if cpp_data[row][col] != rust_data[row][col]:
                differences.append((row, col, cpp_data[row][col], rust_data[row][col]))
    
    return differences
```

## 11. 调试 UI 面板

### 11.1 实时信息显示

**原理**: 在游戏画面上叠加调试信息

**实现**:
```rust
fn draw_debug_overlay(&self, engine: &mut Engine) {
    let info = format!(
        "FPS: {}\nTiles: {}\nDraw Calls: {}\nMemory: {}MB",
        self.fps,
        self.tiles_rendered,
        self.draw_calls,
        self.memory_usage / 1024 / 1024
    );
    engine.draw_text(10, 10, &info, Color::WHITE);
}
```

### 11.2 交互式调试面板

**原理**: 提供可交互的调试界面

**实现**:
```rust
enum DebugCommand {
    ToggleLayer(RenderLayer),
    SetRenderMode(RenderMode),
    JumpToPosition(Point),
    // ...
}

fn handle_debug_input(&mut self, key: Key) {
    match key {
        Key::F1 => self.toggle_debug_overlay(),
        Key::F2 => self.toggle_wireframe(),
        Key::F3 => self.toggle_coordinate_axes(),
        // ...
    }
}
```

## 12. 日志系统

### 12.1 分级日志

**原理**: 不同级别的日志用于不同场景

**实现**:
```rust
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        if should_log($level) {
            println!("[{:?}] {}", $level, format!($($arg)*));
        }
    };
}

// 使用
log!(LogLevel::Debug, "Rendering tile {} at ({}, {})", tile_id, x, y);
```

### 12.2 结构化日志

**原理**: 使用结构化格式，便于分析

**实现**:
```rust
#[derive(Serialize)]
struct RenderLog {
    frame: u64,
    tile_id: usize,
    screen_pos: (i32, i32),
    world_pos: (i32, i32),
    render_time_ns: u64,
}

fn log_render(log: RenderLog) {
    let json = serde_json::to_string(&log).unwrap();
    println!("{}", json);
}
```

## 13. 测试工具

### 13.1 可视化测试

**原理**: 生成测试图像，人工或自动验证

**实现**:
```rust
fn generate_test_image(&self, test_case: &TestCase) -> Image {
    // 渲染测试场景
    let mut image = Image::new(800, 600);
    self.render_test_scene(&mut image, test_case);
    image
}

fn save_test_image(&self, image: &Image, name: &str) {
    image.save(format!("tests/output/{}.png", name)).unwrap();
}
```

### 13.2 回归测试

**原理**: 对比当前输出和已知正确的输出

**实现**:
```rust
fn regression_test(&self, test_name: &str) -> bool {
    let current = self.generate_test_image(test_name);
    let expected = Image::load(format!("tests/expected/{}.png", test_name)).unwrap();
    
    compare_images(&current, &expected) < THRESHOLD
}
```

## 14. 实用技巧总结

### 14.1 调试工作流

1. **缩小范围**: 先定位问题的大致区域
2. **数值化**: 将问题转化为数据对比
3. **可视化**: 用图形方式展示问题
4. **隔离**: 排除无关因素
5. **验证**: 用断言确保修复正确

### 14.2 常见问题模式

| 问题类型 | 调试方法 | 工具 |
|---------|---------|------|
| 像素错乱 | 像素值对比、颜色编码 | 文件输出、热力图 |
| 坐标错误 | 坐标轴绘制、转换日志 | 线框、网格 |
| 性能问题 | 时间戳、计数器 | 性能分析器 |
| 状态错误 | 状态可视化、转换日志 | 调试面板 |
| 渲染缺失 | 单层渲染、区域裁剪 | 隔离渲染 |

### 14.3 工具选择建议

- **快速定位**: 颜色编码、单对象渲染
- **精确分析**: 数值化对比、断言
- **性能分析**: 时间戳、计数器
- **重现问题**: 录制回放、快照
- **自动化**: 对比脚本、回归测试

## 15. 在 Rust Diablo 项目中的应用

### 15.1 已应用的技巧

1. **像素值对比**: 三角形布局调试 ✅
2. **单对象渲染**: piece 311/frame 706 调试 ✅
3. **区域裁剪**: 中心区域渲染 ✅
4. **文件输出**: 像素数据导出 ✅

### 15.2 未来可应用的技巧

1. **颜色编码**: Tile 类型可视化
2. **边界框**: 碰撞区域可视化
3. **性能计数器**: 渲染性能分析
4. **调试面板**: 实时信息显示
5. **自动化对比**: C++/Rust 渲染对比

## 相关文档

- [三角形像素布局调试](triangle-pixel-layout-debugging.md)
- [渲染架构分析](../cpp_rendering_architecture.md)
- [测试指南](../TESTING_GUIDE.md)

---

**创建日期**: 2025-01-XX  
**最后更新**: 2025-01-XX  
**维护者**: Rust Diablo 开发团队


