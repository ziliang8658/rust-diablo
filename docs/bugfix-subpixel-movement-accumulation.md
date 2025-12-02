# Bug修复：子像素移动累积丢失问题

## 📅 修复日期
2025年11月20日

## 🐛 问题描述

**症状**：
- 玩家在可行走的瓦片上按 WASD 键，但玩家位置没有任何变化
- 即使瓦片类型是 `Floor` 或 `Grass`（walkable），玩家仍然无法移动

**影响范围**：
- 所有实体移动系统
- 高帧率或低速度时特别明显

## 🔍 问题分析

### 根本原因

在 `src/entity/mod.rs` 的 `Entity::update` 方法中，存在一个**子像素移动累积丢失**的bug：

```rust
// 问题代码（修复前）
if let Some((map, tile_size)) = collision_info {
    if new_pos != self.position {
        // 只有当整数位置改变时才更新
        // ... 碰撞检测和位置更新 ...
    }
    // ❌ 问题：如果 new_pos == self.position，position_f 不会被更新！
    // 导致子像素移动被丢弃
}
```

**问题详解**：

1. **浮点数累积机制**：
   - `Entity` 使用 `position_f: (f32, f32)` 来累积子像素移动
   - 每帧计算：`position_f += velocity * dt`
   - 然后转换为整数：`position = round(position_f)`

2. **高帧率场景**：
   - 假设 FPS = 1000，dt = 0.001 秒
   - 玩家速度 = 200 像素/秒
   - 每帧移动 = 200 * 0.001 = 0.2 像素
   - `round(0.2)` = 0，所以 `new_pos == self.position`

3. **累积丢失**：
   - 当 `new_pos == self.position` 时，代码跳过了 `position_f` 的更新
   - 下一帧又从旧的 `position_f` 开始计算
   - 结果：永远无法累积到 1 像素，玩家被"卡住"

### 代码流程分析

**修复前的错误流程**：
```
Frame 1: position_f = (100.0, 100.0), position = (100, 100)
         velocity = (200, 0), dt = 0.001
         new_pos_f = (100.2, 100.0)
         new_pos = (100, 100)  // round(100.2) = 100
         new_pos == position? YES → 跳过更新
         position_f 仍然是 (100.0, 100.0) ❌

Frame 2: position_f = (100.0, 100.0)  // 从旧值开始！
         new_pos_f = (100.2, 100.0)    // 又计算一次
         new_pos = (100, 100)
         再次跳过更新 ❌

... 永远无法累积到 1 像素
```

**修复后的正确流程**：
```
Frame 1: position_f = (100.0, 100.0), position = (100, 100)
         velocity = (200, 0), dt = 0.001
         new_pos_f = (100.2, 100.0)
         new_pos = (100, 100)
         new_pos == position? YES → 但更新 position_f = (100.2, 100.0) ✅

Frame 2: position_f = (100.2, 100.0)  // 从累积值开始
         new_pos_f = (100.4, 100.0)
         new_pos = (100, 100)
         更新 position_f = (100.4, 100.0) ✅

Frame 3: position_f = (100.4, 100.0)
         new_pos_f = (100.6, 100.0)
         ...

Frame 5: position_f = (101.0, 100.0)
         new_pos = (101, 100)  // round(101.0) = 101
         new_pos != position? YES → 触发碰撞检测和位置更新 ✅
```

## ✅ 修复方案

### 修改内容

**文件**：`rust-diablo/src/entity/mod.rs`

**修改**：在 `Entity::update` 方法中添加 `else` 分支，确保即使整数位置未改变，也要更新浮点位置。

```rust
// 修复后的代码
if let Some((map, tile_size)) = collision_info {
    if new_pos != self.position {
        // 整数位置改变，需要碰撞检测
        let validated_pos = map.validate_move(self.position, new_pos, self.size, tile_size);
        
        if validated_pos != new_pos {
            // 碰撞！重置浮点位置到验证后的整数位置
            self.position_f.0 = validated_pos.x as f32;
            self.position_f.1 = validated_pos.y as f32;
            self.position = validated_pos;
        } else {
            // 允许移动
            self.position_f = new_pos_f;
            self.position = new_pos;
        }
    } else {
        // ✅ 新增：整数位置未改变，但仍需更新浮点位置以累积子像素移动
        // 由于整数位置未变，我们仍在同一个碰撞单元格内，所以是安全的
        self.position_f = new_pos_f;
    }
} else {
    // 无碰撞检测，直接更新
    self.position_f = new_pos_f;
    self.position = new_pos;
}
```

### 修复原理

1. **子像素累积**：
   - 即使整数位置未改变，也要更新 `position_f`
   - 允许浮点值累积，直到跨越整数边界

2. **安全性保证**：
   - 如果整数位置未改变，实体仍在同一个瓦片内
   - 因此不需要重新进行碰撞检测
   - 直接更新浮点位置是安全的

3. **性能影响**：
   - 无额外性能开销
   - 只是添加了一个简单的赋值操作

## 🧪 测试验证

### 测试场景

1. **高帧率测试**：
   - 设置 FPS 限制为 1000+
   - 验证玩家仍能正常移动

2. **低速度测试**：
   - 设置玩家速度为 50 像素/秒
   - 验证子像素移动能正确累积

3. **正常速度测试**：
   - 默认速度 200 像素/秒
   - 验证移动流畅无卡顿

### 验证方法

运行游戏并观察：
- ✅ 玩家可以正常移动
- ✅ 移动速度在不同帧率下保持一致
- ✅ 碰撞检测正常工作（不会穿过墙壁）

## 📚 技术要点

### 为什么需要浮点数累积？

1. **帧率独立性**：
   - 游戏逻辑应该与帧率无关
   - 使用 `dt`（delta time）确保速度恒定

2. **高帧率支持**：
   - 现代显示器支持 120Hz、144Hz、甚至 240Hz
   - 每帧移动可能小于 1 像素
   - 必须累积才能产生可见移动

3. **平滑移动**：
   - 浮点数累积提供更平滑的移动
   - 避免"抖动"或"跳跃"

### 相关概念

- **Delta Time (dt)**：帧间时间差，用于帧率独立计算
- **Sub-pixel Movement**：小于 1 像素的移动，需要累积
- **Position Accumulation**：位置累积机制，避免精度丢失

## 🔗 相关文件

- `rust-diablo/src/entity/mod.rs` - 实体更新逻辑
- `rust-diablo/src/world/collision.rs` - 碰撞检测
- `rust-diablo/src/game.rs` - 游戏循环

## 📝 经验教训

1. **浮点数累积的重要性**：
   - 在基于时间的移动系统中，必须正确处理浮点数累积
   - 不能因为整数位置未改变就跳过浮点更新

2. **边界情况测试**：
   - 高帧率和低速度是常见的边界情况
   - 应该在开发早期就考虑这些场景

3. **代码审查要点**：
   - 检查所有涉及位置更新的代码
   - 确保浮点累积逻辑完整

## 🎯 后续改进

1. **性能优化**：
   - 考虑使用定点数（fixed-point）代替浮点数
   - 减少浮点运算的开销

2. **代码重构**：
   - 将位置更新逻辑提取为独立方法
   - 提高代码可读性和可测试性

3. **单元测试**：
   - 添加针对子像素移动的单元测试
   - 确保类似问题不会再次出现

---

**修复状态**：✅ 已完成  
**测试状态**：✅ 已验证  
**文档状态**：✅ 已记录








