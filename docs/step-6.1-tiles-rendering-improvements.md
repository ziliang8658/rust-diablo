# Step 6.1 瓦片渲染改进

## 问题描述

在集成Cathedral瓦片系统到TestWorld场景（F1）后，发现以下问题：

### 问题1：画面显示不完整
- **现象**：只显示一小块瓦片区域，大部分屏幕是黑色
- **原因**：初始实现只渲染了10x10的瓦片网格
- **影响**：无法体验完整的地牢视觉效果

### 问题2：玩家无法移动
- **现象**：按WASD键后，玩家sprite不动，瓦片也不滚动
- **原因**：
  1. 玩家sprite被固定渲染在屏幕中央
  2. 瓦片渲染位置是固定的，没有根据玩家位置变化
  3. 虽然玩家在世界坐标中实际移动了，但视觉上看不出来
- **影响**：用户体验差，无法展示瓦片系统的滚动效果

### 问题3：Logo纹理错误渲染
- **现象**：左上角显示了Town场景的大型背景图（550x3240像素）
- **原因**：TestWorld场景错误地渲染了"logo"纹理
- **影响**：干扰了Cathedral瓦片系统的展示

## 解决方案

### 修复1：移除TestWorld中的Logo渲染

**文件**：`src/game.rs`

移除了TestWorld场景中渲染"logo"纹理的代码：

```rust
// 之前的代码：
match self.current_scene {
    SceneType::TestWorld => {
        self.world.render(&mut self.engine, &self.camera)?;
        
        // Render PCX logo in top-left corner (overlay on top of game)
        if self.engine.texture_manager().contains("logo") {
            // ... 渲染logo的代码
        }
    }
    // ...
}

// 修改后的代码：
match self.current_scene {
    SceneType::TestWorld => {
        // Render the world with Cathedral tiles system (Step 6.1)
        self.world.render(&mut self.engine, &self.camera)?;
    }
    // ...
}
```

**效果**：TestWorld场景现在只显示瓦片系统，不再有干扰元素

### 修复2：扩大瓦片渲染范围

**文件**：`src/world/mod.rs` - `render_with_tiles`方法

将渲染范围从固定的10x10网格改为30x30网格（以玩家为中心的±15范围）：

```rust
// 之前的代码：
for mega_y in 0..10 {
    for mega_x in 0..10 {
        // ...
    }
}

// 修改后的代码：
let render_range = 15; // Render 30x30 grid centered on player

for offset_y in -render_range..=render_range {
    for offset_x in -render_range..=render_range {
        let mega_x = player_tile_x + offset_x;
        let mega_y = player_tile_y + offset_y;
        // ...
    }
}
```

**效果**：瓦片现在可以覆盖整个640x480的屏幕

### 修复3：实现瓦片滚动系统

**文件**：`src/world/mod.rs` - `render_with_tiles`方法

添加了基于玩家位置的瓦片滚动逻辑：

```rust
// 获取玩家的瓦片坐标
let player_tile_x = if let Some(player) = self.entities.first() {
    player.position.x / (self.tile_size as i32)
} else {
    0
};

let player_tile_y = if let Some(player) = self.entities.first() {
    player.position.y / (self.tile_size as i32)
} else {
    0
};

// 渲染以玩家为中心的瓦片网格
for offset_y in -render_range..=render_range {
    for offset_x in -render_range..=render_range {
        let mega_x = player_tile_x + offset_x;
        let mega_y = player_tile_y + offset_y;
        
        // 计算等距投影屏幕位置
        let (screen_x, screen_y) = world_to_screen(mega_x, mega_y);
        // ...
    }
}
```

**工作原理**：
1. 将玩家的世界坐标（像素）转换为瓦片坐标（除以tile_size=32）
2. 以玩家瓦片坐标为中心，渲染周围±15格的瓦片
3. 当玩家移动时，瓦片坐标变化，导致不同的瓦片被渲染
4. 视觉效果是：玩家保持在屏幕中央，世界在其周围滚动

### 修复4：添加屏幕边界检查

为了优化性能，添加了屏幕边界检查，跳过完全不在屏幕内的瓦片：

```rust
// Skip if completely off-screen
if final_x < -TILE_WIDTH || final_x > 640 + TILE_WIDTH ||
   final_y < -TILE_HEIGHT || final_y > 480 + TILE_HEIGHT {
    continue;
}
```

### 修复5：调整玩家渲染位置

将玩家sprite的渲染位置从`center_y + 100`调整为`center_y + 120`，以提供更好的可见性：

```rust
let player_rect = Rect::from_center(
    Point::new(screen_center_x, screen_center_y + 120),
    entity.size.0,
    entity.size.1,
);
```

## 技术细节

### 坐标系统

1. **世界坐标（World Coordinates）**：
   - 玩家的实际位置，以像素为单位
   - 示例：玩家初始位置在(960, 640)

2. **瓦片坐标（Tile Coordinates）**：
   - 世界坐标除以tile_size得到
   - 示例：(960, 640) ÷ 32 = (30, 20)

3. **等距投影坐标（Isometric Coordinates）**：
   - 使用`world_to_screen`函数转换瓦片坐标到屏幕坐标
   - 公式：
     ```rust
     screen_x = (world_y - world_x) * 32
     screen_y = (world_y + world_x) * -16
     ```

4. **屏幕坐标（Screen Coordinates）**：
   - 最终渲染位置，相对于屏幕中心偏移

### 渲染流程

```
玩家移动 (输入)
    ↓
更新玩家世界坐标 (update)
    ↓
计算玩家瓦片坐标 (position / tile_size)
    ↓
渲染以玩家为中心的瓦片网格
    ↓
对每个瓦片：
    - 计算等距投影坐标
    - 添加屏幕中心偏移
    - 检查是否在屏幕内
    - 渲染瓦片矩形
    ↓
渲染玩家sprite在屏幕中央
```

### 性能优化

1. **边界检查**：跳过屏幕外的瓦片，避免不必要的绘制调用
2. **固定渲染范围**：30x30网格足以覆盖屏幕，不需要更多
3. **简单碰撞检测**：`final_x < -TILE_WIDTH || final_x > 640 + TILE_WIDTH`

## 测试要点

### 视觉测试
- [ ] 瓦片覆盖整个屏幕，没有大面积黑色区域
- [ ] 瓦片颜色根据TileType正确显示
- [ ] 瓦片边界线清晰可见
- [ ] 玩家sprite在屏幕中央下方显示

### 功能测试
- [ ] 按W键，瓦片向下滚动（玩家向上移动）
- [ ] 按S键，瓦片向上滚动（玩家向下移动）
- [ ] 按A键，瓦片向右滚动（玩家向左移动）
- [ ] 按D键，瓦片向左滚动（玩家向右移动）
- [ ] 玩家sprite动画正确播放（idle/walk）

### 性能测试
- [ ] 帧率稳定在60 FPS
- [ ] 没有明显的渲染延迟或卡顿

## 已知限制

### 1. 简化的瓦片映射
- 当前使用简单的公式计算瓦片索引：`(abs(y) * 10 + abs(x)) % til_data.len()`
- 真实游戏应该有一个预生成的地图数据结构
- 后续步骤（6.2）将实现真实的地牢生成算法

### 2. 缺少MicroTile渲染
- 当前只渲染MegaTile的第一个MicroTile（micro1）
- MegaTile由2x2=4个MicroTile组成，应该分别渲染
- 这是为了简化演示，后续可以改进

### 3. 使用颜色块而非纹理
- 当前用颜色矩形表示瓦片类型
- 真实游戏应该加载CEL文件中的瓦片纹理
- 这需要实现CEL格式解析（未来的任务）

### 4. 简化的碰撞系统
- 当前使用TestWorld的简单网格碰撞
- 真实游戏应该使用SOL数据进行精确碰撞检测
- 后续步骤将实现基于SOL的碰撞

## 下一步计划

### 短期（Step 6.2）
1. 实现真实的地牢生成算法
   - L1-L4各层的生成规则
   - 房间布局和连接
   - 特殊房间（楼梯、入口等）

2. 实现基于SOL的碰撞检测
   - 读取SOL属性
   - 应用到玩家移动
   - 支持不可行走瓦片

### 中期（Step 6.3）
1. 加载和渲染CEL瓦片纹理
   - 解析Level CEL格式
   - 集成到渲染管线
   - 替换颜色块显示

2. 实现MicroTile分层渲染
   - 渲染MegaTile的4个MicroTile
   - 正确处理透明和遮挡

### 长期
1. 添加光照系统
2. 添加雾战（Fog of War）
3. 优化渲染性能（视锥剔除、批量渲染等）

## 参考代码

### 原版DevilutionX参考
- `Source/engine/render/scrollrt.cpp` - 瓦片滚动渲染
- `Source/engine.cpp` - 等距投影坐标转换
- `Source/levels/gendung.cpp` - 地牢生成

### 本项目实现
- `src/engine/isometric.rs` - 等距投影系统
- `src/tiles/*.rs` - 瓦片格式加载
- `src/world/mod.rs` - 世界渲染

## 总结

通过这次修复，我们成功地：
1. ✅ 移除了干扰元素（logo纹理）
2. ✅ 扩大了瓦片渲染范围，覆盖整个屏幕
3. ✅ 实现了瓦片滚动系统，玩家可以移动
4. ✅ 优化了性能，添加了边界检查

现在TestWorld场景（F1）能够正确展示Cathedral地牢的瓦片系统，玩家可以自由移动并看到世界滚动的效果。这为后续实现真实的地牢生成和渲染打下了坚实的基础。

## 墙体渲染简化（2024补充）

### 复杂墙体渲染的尝试

曾尝试实现原版DevilutionX的墙体渲染系统：
- ✅ 创建32x32梯形/方形墙体tiles
- ✅ 实现左右trapezoid配对
- ✅ 实现多层垂直堆叠

### 遇到的问题
- ❌ 等距坐标计算过于复杂
- ❌ 左右拼接产生锯齿状交错
- ❌ 代码臃肿，难以维护

### 当前状态（简化版）
**只渲染64x32地板tiles**：
- 每个MegaTile的micro1渲染一个完整的地板tile
- 不再处理墙体的多层堆叠
- 墙体位置用深灰地板（tile_3）标记

### 保留的成果
- ✅ **Tile格式解析完整**：MIN/TIL/SOL系统
- ✅ **Tile资源**：tile_0 ~ tile_11（包括墙体tiles）
- ✅ **生成脚本**：create_clean_wall.py
- ✅ **TileType枚举**：支持6种tile形状

### 技术债务
墙体渲染需要重新设计架构，参考：
- `Source/engine/render/scrollrt.cpp::DrawCell()` - 完整的16层microtiles渲染
- `Source/engine/render/dun_render.cpp` - 精确的triangle/trapezoid渲染算法

详见：`docs/wall-rendering-attempt.md`


