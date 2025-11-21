# Bug Fix: Sub-pixel Movement Accumulation

## 🐛 问题描述
在实现 Step 4.2 的碰撞检测后，出现了一个严重的游戏性 Bug：**玩家无法移动**。
具体表现为：
1. 地图数据正确，瓦片标记为可行走（Walkable）。
2. 按下 WASD 键时，速度向量（Velocity）计算正确。
3. 但是玩家位置（Position）始终保持不变。

## 🔍 原因分析
经过代码审查和调试，发现问题出在**浮点数位置累积的丢失**。

### 1. 移动机制
*   游戏以 60 FPS 运行（或更高）。
*   玩家速度设为 200 像素/秒。
*   每帧移动距离 = `200 * dt`。
*   如果 `dt` 很小（例如 0.005s），移动距离 = 1.0 像素。
*   但如果帧率很高或速度稍慢，移动距离可能小于 1.0 像素（例如 0.5 像素）。

### 2. 错误的代码逻辑
原有的更新逻辑如下：

```rust
// 计算新位置（浮点数）
let mut new_pos_f = self.position_f;
new_pos_f.0 += move_x;
new_pos_f.1 += move_y;

// 转换为整数坐标用于碰撞检测
let new_pos = Point::new(
    new_pos_f.0.round() as i32,
    new_pos_f.1.round() as i32
);

if let Some((map, tile_size)) = collision_info {
    // ❌ 错误点：只有当整数坐标发生变化时，才更新 float 坐标
    if new_pos != self.position {
        // ... 验证碰撞 ...
        self.position = new_pos;
        self.position_f = new_pos_f; // 更新累积值
    }
    // 如果整数坐标没变（new_pos == self.position），这里什么都没做！
    // 导致 new_pos_f 中的 0.5 像素移动被直接丢弃。
    // 下一帧又是从旧的 position_f 开始计算，永远无法累积超过 1.0。
}
```

## 🛠️ 修复方案
无论整数坐标是否变化，都必须更新浮点数坐标 `position_f`，以便累积微小的移动量。

**修复后的代码：**

```rust
if let Some((map, tile_size)) = collision_info {
    if new_pos != self.position {
        // 整数坐标改变，需要检测碰撞
        let validated_pos = map.validate_move(self.position, new_pos, self.size, tile_size);
        
        if validated_pos != new_pos {
            // 发生碰撞：被墙挡住
            // 将 float 坐标重置为合法的整数坐标，防止"卡"进墙里
            self.position_f.0 = validated_pos.x as f32;
            self.position_f.1 = validated_pos.y as f32;
            self.position = validated_pos;
        } else {
            // 移动有效：更新坐标
            self.position_f = new_pos_f;
            self.position = new_pos;
        }
    } else {
        // ✅ 关键修复：即使整数坐标没变，也要更新 float 坐标
        // 累积子像素移动（Sub-pixel movement）
        self.position_f = new_pos_f;
    }
}
```

## 📚 经验总结
在游戏开发中处理移动时：
1.  **始终保留浮点数累积值**：不要每帧重置为整数坐标。
2.  **碰撞检测与移动分离**：碰撞检测通常基于整数网格或包围盒，但移动物理应基于浮点数。
3.  **Sub-pixel 精度**：对于慢速移动或高帧率情况，子像素精度至关重要。

