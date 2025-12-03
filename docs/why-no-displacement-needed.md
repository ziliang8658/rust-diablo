# 为什么Rust实现不需要Displacement返回到行首

## 问题描述

用户发现添加了"返回到行首"的代码后，只能渲染第一行。移除后正常。用户想知道为什么要用Displacement。

## 关键差异：变量作用域和修改方式

### C++实现方式

```cpp
void DrawFloor(..., Point tilePosition, ...) {
    for (int i = 0; i < rows; i++) {
        // tilePosition是外部变量，在循环中直接被修改
        for (int j = 0; j < columns; j++, 
             tilePosition += Direction::East,  // ← 直接修改外部变量！
             targetBufferPosition.x += TILE_WIDTH) {
            // 使用tilePosition渲染
        }
        // 循环结束后，tilePosition已经移动到行尾！
        // 例如：从(10,10)开始，5列后变成(14,6)
        
        // 必须返回到行首
        tilePosition += Displacement(Direction::West) * columns;
        // (14,6) + (-1,1)*5 = (14-5, 6+5) = (9,11) ❌ 等等，这个计算不对
        
        // 实际上应该是：
        // 从(10,10)开始，向东移动5次：(10,10) -> (11,9) -> (12,8) -> (13,7) -> (14,6)
        // Direction::East = (1, -1)，所以移动5次后是(10+5, 10-5) = (15, 5)
        // 返回到行首：(15,5) + (-1,1)*5 = (15-5, 5+5) = (10,10) ✅
        
        // 移动到下一行
        if ((i & 1) != 0) {
            tilePosition.x++;  // 从行首(10,10)移动到(11,10)
        } else {
            tilePosition.y++;  // 从行首(10,10)移动到(10,11)
        }
    }
}
```

**关键点：**
- `tilePosition`是外部变量，在循环的更新部分被直接修改
- 循环结束后，`tilePosition`已经移动到行尾
- **必须**返回到行首，才能正确移动到下一行

### Rust实现方式

```rust
let mut tile_x = start_tile_x;
let mut tile_y = start_tile_y;

for row in 0..rows {
    // 关键：使用局部变量，从外部变量复制
    let mut tx = tile_x;  // ← 复制，不是引用！
    let mut ty = tile_y;  // ← 复制，不是引用！
    let mut sx = screen_x;
    
    for _col in 0..current_columns {
        // 使用局部变量tx, ty渲染
        // ...
        tx += 1;  // ← 只修改局部变量
        ty -= 1;  // ← 只修改局部变量
    }
    // 循环结束后，局部变量tx, ty被丢弃
    // tile_x, tile_y根本没有改变！仍然是行首位置！
    
    // ❌ 如果添加这段代码，会导致错误：
    // tile_x -= current_columns;  // tile_x根本没有移动，却被减去了columns！
    // tile_y += current_columns;  // tile_y根本没有移动，却被加上了columns！
    
    // 直接移动到下一行即可
    if (row & 1) != 0 {
        tile_x += 1;  // 从行首移动到下一行的行首
    } else {
        tile_y += 1;  // 从行首移动到下一行的行首
    }
}
```

**关键点：**
- `tile_x, tile_y`是外部变量，在行开始时复制到局部变量`tx, ty`
- 循环中只修改局部变量`tx, ty`
- 循环结束后，局部变量被丢弃，`tile_x, tile_y`**保持不变**（仍然是行首位置）
- **不需要**返回到行首，因为根本没有移动！

## 为什么添加Displacement代码会导致只渲染第一行？

### 错误的执行流程

假设从(10,10)开始，第一行5列：

```
Row 0:
  - tile_x = 10, tile_y = 10 (行首)
  - tx = 10, ty = 10 (复制)
  - 渲染：(10,10), (11,9), (12,8), (13,7), (14,6)
  - 循环结束，tx, ty被丢弃
  - tile_x, tile_y仍然是(10,10) ✅
  
  - ❌ 错误代码执行：
    tile_x -= 5;  // 10 - 5 = 5
    tile_y += 5;  // 10 + 5 = 15
    // 现在tile_x=5, tile_y=15（错误的位置！）
  
  - 移动到下一行：
    tile_y += 1;  // 15 + 1 = 16
    // 现在tile_x=5, tile_y=16

Row 1:
  - tile_x = 5, tile_y = 16（错误的位置！应该是(10,11)或(11,10)）
  - 渲染位置完全错误，可能超出边界或渲染到错误区域
```

### 正确的执行流程（不添加Displacement代码）

```
Row 0:
  - tile_x = 10, tile_y = 10 (行首)
  - tx = 10, ty = 10 (复制)
  - 渲染：(10,10), (11,9), (12,8), (13,7), (14,6)
  - 循环结束，tx, ty被丢弃
  - tile_x, tile_y仍然是(10,10) ✅
  
  - 移动到下一行：
    tile_y += 1;  // 10 + 1 = 11
    // 现在tile_x=10, tile_y=11 ✅

Row 1:
  - tile_x = 10, tile_y = 11（正确的位置！）
  - tx = 10, ty = 11 (复制)
  - 渲染：(10,11), (11,10), (12,9), (13,8), (14,7)
  - ...
```

## 总结

### C++需要Displacement的原因

1. **直接修改外部变量**：`tilePosition`在循环中被直接修改
2. **循环结束后位置改变**：`tilePosition`已经移动到行尾
3. **必须返回到行首**：才能正确移动到下一行

### Rust不需要Displacement的原因

1. **使用局部变量**：`tx, ty`是局部变量，从`tile_x, tile_y`复制
2. **外部变量不变**：循环结束后，`tile_x, tile_y`仍然是行首位置
3. **直接移动到下一行**：不需要返回到行首，因为根本没有移动

### 设计模式差异

- **C++模式**：直接修改外部状态，需要手动恢复
- **Rust模式**：使用局部变量，外部状态保持不变，更安全

## 参考代码

- C++原版：`Source/engine/render/scrollrt.cpp:927-955`
- Rust实现：`rust-diablo/src/world/mod.rs:366-427`

## 结论

**Rust实现不需要Displacement返回到行首的代码**，因为：
1. Rust使用了不同的实现模式（局部变量 vs 直接修改）
2. `tile_x, tile_y`在循环中根本没有改变
3. 添加Displacement代码会导致错误的位置计算，从而只渲染第一行

正确的做法是：**保持现有代码，不要添加Displacement返回到行首的逻辑**。





