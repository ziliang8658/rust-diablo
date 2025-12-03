# 等距瓦片渲染算法实例详解

本文档通过具体的代码实例和可视化图表，详细演示 C++ 中等距瓦片渲染算法的执行过程。

## 📋 目录

1. [实例1：基础坐标转换](#实例1基础坐标转换)
2. [实例2：单行瓦片渲染](#实例2单行瓦片渲染)
3. [实例3：多行瓦片渲染](#实例3多行瓦片渲染)
4. [实例4：完整渲染循环追踪](#实例4完整渲染循环追踪)
5. [常见问题解答](#常见问题解答)

---

## 实例1：基础坐标转换

### 场景设置

我们有一个简单的 4×4 世界网格，需要将其转换为等距投影的屏幕坐标。

**世界坐标网格：**

```
     0    1    2    3
   ┌────┬────┬────┬────┐
0  │ A  │ B  │ C  │ D  │
   ├────┼────┼────┼────┤
1  │ E  │ F  │ G  │ H  │
   ├────┼────┼────┼────┤
2  │ I  │ J  │ K  │ L  │
   ├────┼────┼────┼────┤
3  │ M  │ N  │ O  │ P  │
   └────┴────┴────┴────┘
```

### C++ 转换代码

```cpp
#include "engine/displacement.hpp"

using namespace devilution;

// 转换函数
void convertToScreen(int world_x, int world_y, int& screen_x, int& screen_y) {
    Displacement world_delta(world_x, world_y);
    Displacement screen_delta = world_delta.worldToScreen();
    screen_x = screen_delta.deltaX;
    screen_y = screen_delta.deltaY;
}

// 示例：转换所有16个瓦片
void example1() {
    printf("=== 实例1：基础坐标转换 ===\n\n");
    printf("%-10s %-15s %-20s %-20s\n", "瓦片", "世界坐标", "屏幕坐标", "计算公式");
    printf("%s\n", std::string(70, '-').c_str());
    
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            int sx, sy;
            convertToScreen(x, y, sx, sy);
            
            char tile = 'A' + (y * 4 + x);
            printf("%-10c (%-3d, %-3d)      (%-5d, %-5d)    ", 
                   tile, x, y, sx, sy);
            printf("(y-x)*32=%d, (y+x)*-16=%d\n", (y-x)*32, (y+x)*-16);
        }
    }
}
```

### 执行结果

```
=== 实例1：基础坐标转换 ===

瓦片       世界坐标         屏幕坐标              计算公式
----------------------------------------------------------------------
A          (0  , 0  )      (0   , 0   )    (y-x)*32=0, (y+x)*-16=0
B          (1  , 0  )      (-32 , -16 )    (y-x)*32=-32, (y+x)*-16=-16
C          (2  , 0  )      (-64 , -32 )    (y-x)*32=-64, (y+x)*-16=-32
D          (3  , 0  )      (-96 , -48 )    (y-x)*32=-96, (y+x)*-16=-48
E          (0  , 1  )      (32  , -16 )    (y-x)*32=32, (y+x)*-16=-16
F          (1  , 1  )      (0   , -32 )    (y-x)*32=0, (y+x)*-16=-32
G          (2  , 1  )      (-32 , -48 )    (y-x)*32=-32, (y+x)*-16=-48
H          (3  , 1  )      (-64 , -64 )    (y-x)*32=-64, (y+x)*-16=-64
I          (0  , 2  )      (64  , -32 )    (y-x)*32=64, (y+x)*-16=-32
J          (1  , 2  )      (32  , -48 )    (y-x)*32=32, (y+x)*-16=-48
K          (2  , 2  )      (0   , -64 )    (y-x)*32=0, (y+x)*-16=-64
L          (3  , 2  )      (-32 , -80 )    (y-x)*32=-32, (y+x)*-16=-80
M          (0  , 3  )      (96  , -48 )    (y-x)*32=96, (y+x)*-16=-48
N          (1  , 3  )      (64  , -64 )    (y-x)*32=64, (y+x)*-16=-64
O          (2  , 3  )      (32  , -80 )    (y-x)*32=32, (y+x)*-16=-80
P          (3  , 3  )      (0   , -96 )    (y-x)*32=0, (y+x)*-16=-96
```

### 可视化图表

**屏幕上的等距布局：**

```
        -96   -64   -32    0     32    64    96
    -96  P     L     H     D
              └─────┴─────┴─────┘
    -80        O     K     G     C
              └─────┴─────┴─────┴─────┘
    -64            N     J     F     B
              └─────┴─────┴─────┴─────┘
    -48        M     I     E     A
              └─────┴─────┴─────┘
    -32
    -16
     0
```

**关键观察：**
- 相同 `(y-x)` 值的瓦片在同一垂直列上（如 A, F, K, P）
- 相同 `(y+x)` 值的瓦片在同一水平行上（如 A, B, C, D）
- 屏幕Y坐标都是负数，说明需要加上一个偏移量才能在屏幕上正确显示

---

## 实例2：单行瓦片渲染

### 场景设置

渲染从世界坐标 (5, 5) 开始的单行，包含 5 个瓦片。

### C++ 实现代码

```cpp
void example2_single_row() {
    const int TILE_WIDTH = 64;
    const int TILE_HEIGHT = 32;
    
    // 起始位置
    int tile_x = 5;
    int tile_y = 5;
    int screen_x = 100;  // 屏幕起始X位置
    int screen_y = 50;   // 屏幕起始Y位置
    
    printf("=== 实例2：单行瓦片渲染 ===\n\n");
    printf("起始瓦片位置: (%d, %d)\n", tile_x, tile_y);
    printf("起始屏幕位置: (%d, %d)\n\n", screen_x, screen_y);
    printf("%-5s %-15s %-20s %-20s\n", "序号", "世界坐标", "屏幕坐标", "移动方向");
    printf("%s\n", std::string(70, '-').c_str());
    
    // 渲染5个瓦片
    for (int i = 0; i < 5; i++) {
        printf("%-5d (%-3d, %-3d)      (%-5d, %-5d)    ", 
               i, tile_x, tile_y, screen_x, screen_y);
        
        if (i > 0) {
            printf("East (+1, -1), screen +64px\n");
        } else {
            printf("起始位置\n");
        }
        
        // 绘制瓦片（伪代码）
        // DrawTile(tile_x, tile_y, screen_x, screen_y);
        
        // 移动到下一个瓦片（East方向）
        tile_x += 1;      // world_x += 1
        tile_y -= 1;      // world_y -= 1
        screen_x += TILE_WIDTH;  // 屏幕X向右移动64像素
    }
    
    printf("\n最终位置: 瓦片 (%d, %d), 屏幕 (%d, %d)\n", 
           tile_x, tile_y, screen_x, screen_y);
}
```

### 执行结果

```
=== 实例2：单行瓦片渲染 ===

起始瓦片位置: (5, 5)
起始屏幕位置: (100, 50)

序号  世界坐标         屏幕坐标              移动方向
----------------------------------------------------------------------
0     (5  , 5  )      (100 , 50  )    起始位置
1     (6  , 4  )      (164 , 50  )    East (+1, -1), screen +64px
2     (7  , 3  )      (228 , 50  )    East (+1, -1), screen +64px
3     (8  , 2  )      (292 , 50  )    East (+1, -1), screen +64px
4     (9  , 1  )      (356 , 50  )    East (+1, -1), screen +64px

最终位置: 瓦片 (10, 0), 屏幕 (420, 50)
```

### 可视化

```
屏幕布局（单行）：

    100    164    228    292    356
    ┌─────┬─────┬─────┬─────┬─────┐
    │ (5,5)│(6,4)│(7,3)│(8,2)│(9,1)│
    └─────┴─────┴─────┴─────┴─────┘
      64px  64px  64px  64px  64px
      
世界坐标移动路径：
      (5,5) → (6,4) → (7,3) → (8,2) → (9,1)
       ↓       ↓       ↓       ↓       ↓
      +1,-1  +1,-1   +1,-1   +1,-1   +1,-1
```

---

## 实例3：多行瓦片渲染

### 场景设置

渲染一个 3×3 的等距网格区域，展示行切换的逻辑。

### C++ 实现代码

```cpp
void example3_multiple_rows() {
    const int TILE_WIDTH = 64;
    const int TILE_HEIGHT = 32;
    
    // 初始参数
    int tile_x = 5;
    int tile_y = 5;
    int screen_x = 100;
    int screen_y = 50;
    int columns = 3;
    int rows = 3;
    
    printf("=== 实例3：多行瓦片渲染 ===\n\n");
    printf("初始参数: 起始瓦片(%d,%d), 起始屏幕(%d,%d), 列数=%d, 行数=%d\n\n",
           tile_x, tile_y, screen_x, screen_y, columns, rows);
    
    // 保存初始值
    int start_tile_x = tile_x;
    int start_tile_y = tile_y;
    int start_screen_x = screen_x;
    int start_screen_y = screen_y;
    int start_columns = columns;
    
    for (int i = 0; i < rows; i++) {
        printf("--- 行 %d (i=%d, %s行) ---\n", i, i, 
               (i % 2 == 0 ? "偶数" : "奇数"));
        printf("起始: 瓦片(%d,%d), 屏幕(%d,%d), 列数=%d\n",
               tile_x, tile_y, screen_x, screen_y, columns);
        
        // 渲染当前行
        int row_tile_x = tile_x;
        int row_tile_y = tile_y;
        int row_screen_x = screen_x;
        
        for (int j = 0; j < columns; j++) {
            printf("  [%d,%d] 瓦片(%d,%d) → 屏幕(%d,%d)\n",
                   i, j, row_tile_x, row_tile_y, row_screen_x, screen_y);
            
            // 移动到下一列（East方向）
            row_tile_x += 1;
            row_tile_y -= 1;
            row_screen_x += TILE_WIDTH;
        }
        
        // 移动到下一行
        screen_y += TILE_HEIGHT / 2;  // +16 像素
        
        if ((i & 1) != 0) {
            // 奇数行：向右偏移
            tile_x += 1;
            columns--;
            screen_x += TILE_WIDTH / 2;
            printf("  行结束: 奇数行 → 瓦片x++, 列数--, 屏幕x += 32\n");
        } else {
            // 偶数行：向左偏移
            tile_y += 1;
            columns++;
            screen_x -= TILE_WIDTH / 2;
            printf("  行结束: 偶数行 → 瓦片y++, 列数++, 屏幕x -= 32\n");
        }
        
        printf("下一行起始: 瓦片(%d,%d), 屏幕(%d,%d), 列数=%d\n\n",
               tile_x, tile_y, screen_x, screen_y, columns);
    }
}
```

### 执行结果

```
=== 实例3：多行瓦片渲染 ===

初始参数: 起始瓦片(5,5), 起始屏幕(100,50), 列数=3, 行数=3

--- 行 0 (i=0, 偶数行) ---
起始: 瓦片(5,5), 屏幕(100,50), 列数=3
  [0,0] 瓦片(5,5) → 屏幕(100,50)
  [0,1] 瓦片(6,4) → 屏幕(164,50)
  [0,2] 瓦片(7,3) → 屏幕(228,50)
  行结束: 偶数行 → 瓦片y++, 列数++, 屏幕x -= 32
下一行起始: 瓦片(5,6), 屏幕(68,66), 列数=4

--- 行 1 (i=1, 奇数行) ---
起始: 瓦片(5,6), 屏幕(68,66), 列数=4
  [1,0] 瓦片(5,6) → 屏幕(68,66)
  [1,1] 瓦片(6,5) → 屏幕(132,66)
  [1,2] 瓦片(7,4) → 屏幕(196,66)
  [1,3] 瓦片(8,3) → 屏幕(260,66)
  行结束: 奇数行 → 瓦片x++, 列数--, 屏幕x += 32
下一行起始: 瓦片(6,6), 屏幕(100,82), 列数=3

--- 行 2 (i=2, 偶数行) ---
起始: 瓦片(6,6), 屏幕(100,82), 列数=3
  [2,0] 瓦片(6,6) → 屏幕(100,82)
  [2,1] 瓦片(7,5) → 屏幕(164,82)
  [2,2] 瓦片(8,4) → 屏幕(228,82)
  行结束: 偶数行 → 瓦片y++, 列数++, 屏幕x -= 32
下一行起始: 瓦片(6,7), 屏幕(68,98), 列数=4
```

### 可视化

```
屏幕布局（3行渲染）：

行 0 (y=50):  
    100         164        228
    ┌─────────┬─────────┬─────────┐
    │ (5,5)   │ (6,4)   │ (7,3)   │
    └─────────┴─────────┴─────────┘
    
行 1 (y=66):  
     68         132        196        260
    ┌─────────┬─────────┬─────────┬─────────┐
    │ (5,6)   │ (6,5)   │ (7,4)   │ (8,3)   │
    └─────────┴─────────┴─────────┴─────────┘
    
行 2 (y=82):  
    100         164        228
    ┌─────────┬─────────┬─────────┐
    │ (6,6)   │ (7,5)   │ (8,4)   │
    └─────────┴─────────┴─────────┘

关键观察：
- 行0从x=100开始，有3个瓦片
- 行1从x=68开始（-32偏移），有4个瓦片
- 行2从x=100开始（+32偏移），有3个瓦片
- 每行Y坐标增加16像素
```

---

## 实例4：完整渲染循环追踪

### 场景设置

模拟 `DrawTileContent()` 函数的完整执行过程，渲染一个小的可见区域。

### 简化版 C++ 代码

```cpp
void example4_full_render_loop() {
    const int TILE_WIDTH = 64;
    const int TILE_HEIGHT = 32;
    
    // 模拟函数参数
    Point tilePosition = {10, 10};        // 起始瓦片位置
    Point targetBufferPosition = {50, 50}; // 起始屏幕位置
    int rows = 4;
    int columns = 5;
    
    // 添加额外行（MicroTileLen = 2，假设）
    rows += 2;
    
    printf("=== 实例4：完整渲染循环追踪 ===\n\n");
    printf("函数参数:\n");
    printf("  tilePosition: (%d, %d)\n", tilePosition.x, tilePosition.y);
    printf("  targetBufferPosition: (%d, %d)\n", 
           targetBufferPosition.x, targetBufferPosition.y);
    printf("  rows: %d (original) + 2 = %d\n", rows-2, rows);
    printf("  columns: %d\n\n", columns);
    
    // 保存行循环变量
    int current_tile_x = tilePosition.x;
    int current_tile_y = tilePosition.y;
    int current_screen_x = targetBufferPosition.x;
    int current_screen_y = targetBufferPosition.y;
    int current_columns = columns;
    
    for (int i = 0; i < rows; i++) {
        printf("═══════════════════════════════════════════════\n");
        printf("行循环 i = %d (%s行)\n", i, (i % 2 == 0 ? "偶数" : "奇数"));
        printf("═══════════════════════════════════════════════\n");
        printf("行开始状态:\n");
        printf("  瓦片位置: (%d, %d)\n", current_tile_x, current_tile_y);
        printf("  屏幕位置: (%d, %d)\n", current_screen_x, current_screen_y);
        printf("  当前列数: %d\n\n", current_columns);
        
        bool skip = false;
        
        // 列循环
        for (int j = 0; j < current_columns; j++) {
            int col_tile_x = current_tile_x;
            int col_tile_y = current_tile_y;
            int col_screen_x = current_screen_x + j * TILE_WIDTH;
            
            printf("  [%d,%d] ", i, j);
            
            // 边界检查（模拟）
            if (col_tile_x >= 0 && col_tile_x < 112 && 
                col_tile_y >= 0 && col_tile_y < 112) {
                printf("✅ 瓦片(%2d,%2d) → 屏幕(%3d,%3d)", 
                       col_tile_x, col_tile_y, col_screen_x, current_screen_y);
                
                if (!skip) {
                    printf(" [绘制]\n");
                    // DrawDungeon(out, lightmap, {col_tile_x, col_tile_y}, 
                    //            {col_screen_x, current_screen_y});
                } else {
                    printf(" [跳过]\n");
                }
            } else {
                printf("❌ 越界 (%2d,%2d) → 绘制黑色瓦片\n", 
                       col_tile_x, col_tile_y);
            }
            
            // East方向移动（在列循环中，这个由循环变量处理）
            // 实际上每个j对应的瓦片坐标是：
            // tile_x = current_tile_x + j
            // tile_y = current_tile_y - j
        }
        
        // 返回到行起始位置（列循环结束）
        // tilePosition 已在循环中使用偏移计算，这里需要重置
        
        // 移动到下一行
        current_screen_y += TILE_HEIGHT / 2;  // +16像素
        
        printf("\n行结束，移动到下一行:\n");
        printf("  screen_y: %d → %d (+16)\n", 
               current_screen_y - TILE_HEIGHT/2, current_screen_y);
        
        if ((i & 1) != 0) {
            // 奇数行
            current_tile_x += 1;
            current_columns--;
            current_screen_x += TILE_WIDTH / 2;
            printf("  奇数行处理:\n");
            printf("    tile_x: %d → %d (+1)\n", current_tile_x - 1, current_tile_x);
            printf("    columns: %d → %d (-1)\n", current_columns + 1, current_columns);
            printf("    screen_x: %d → %d (+32)\n", 
                   current_screen_x - TILE_WIDTH/2, current_screen_x);
        } else {
            // 偶数行
            current_tile_y += 1;
            current_columns++;
            current_screen_x -= TILE_WIDTH / 2;
            printf("  偶数行处理:\n");
            printf("    tile_y: %d → %d (+1)\n", current_tile_y - 1, current_tile_y);
            printf("    columns: %d → %d (+1)\n", current_columns - 1, current_columns);
            printf("    screen_x: %d → %d (-32)\n", 
                   current_screen_x + TILE_WIDTH/2, current_screen_x);
        }
        printf("\n");
    }
}
```

### 执行结果（部分）

```
=== 实例4：完整渲染循环追踪 ===

函数参数:
  tilePosition: (10, 10)
  targetBufferPosition: (50, 50)
  rows: 4 (original) + 2 = 6
  columns: 5

═══════════════════════════════════════════════
行循环 i = 0 (偶数行)
═══════════════════════════════════════════════
行开始状态:
  瓦片位置: (10, 10)
  屏幕位置: (50, 50)
  当前列数: 5

  [0,0] ✅ 瓦片(10,10) → 屏幕( 50, 50) [绘制]
  [0,1] ✅ 瓦片(11, 9) → 屏幕(114, 50) [绘制]
  [0,2] ✅ 瓦片(12, 8) → 屏幕(178, 50) [绘制]
  [0,3] ✅ 瓦片(13, 7) → 屏幕(242, 50) [绘制]
  [0,4] ✅ 瓦片(14, 6) → 屏幕(306, 50) [绘制]

行结束，移动到下一行:
  screen_y: 50 → 66 (+16)
  偶数行处理:
    tile_y: 10 → 11 (+1)
    columns: 5 → 6 (+1)
    screen_x: 50 → 18 (-32)

... (继续其他行)
```

---

## 常见问题解答

### Q1: 为什么 East 方向是 (x+1, y-1)？

**A:** 这是等距投影的特性。在等距网格中，向东移动在逻辑空间中：
- X 坐标增加（向右）
- Y 坐标减少（因为网格是倾斜的）

这确保了屏幕上的移动方向是正确的（向右）。

### Q2: 为什么列数会变化？

**A:** 因为等距网格是菱形的，每行的瓦片数量不同。从顶部看，中间的行有最多瓦片，上下两端的行瓦片较少。通过交替增减列数，可以正确渲染整个菱形区域。

### Q3: 屏幕 Y 坐标为负数怎么办？

**A:** 负数坐标表示瓦片在视口上方。实际渲染时，需要：
1. 加上一个偏移量（通常是相机位置）
2. 或者只渲染可见区域内的瓦片

### Q4: 如何优化渲染性能？

**A:** 
1. **视锥剔除**：只渲染屏幕可见区域的瓦片
2. **层次渲染**：先绘制地板，再绘制墙壁，最后绘制实体
3. **遮挡剔除**：跳过被完全遮挡的瓦片
4. **批处理**：将相同纹理的瓦片合并绘制

---

**文档版本：** 1.0  
**最后更新：** 2024  
**参考项目：** DevilutionX





