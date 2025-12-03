# dPiece数组对比调试指南

## 概述

本文档说明如何使用添加的调试代码来输出和对比C++和Rust的dPiece数组，以找出数据不一致的根本原因。

## 输出文件

运行程序后，会在项目根目录生成两个文件：

- **`dpiece_cpp.txt`**: C++版本的dPiece数组（112x112）
- **`dpiece_rust.txt`**: Rust版本的d_piece数组（112x112）

## 文件格式

两个文件都使用相同的格式：
- 每行代表一个y坐标（0到111）
- 每行的值用逗号分隔，代表x坐标（0到111）
- 格式：`value1,value2,value3,...`

示例：
```
=== C++ dPiece Array (MAXDUNX=112, MAXDUNY=112) ===
218,218,218,218,218,218,...
218,218,218,218,218,218,...
...
```

## 对比方法

### 方法1：使用文本对比工具

1. **使用diff工具**：
```bash
diff dpiece_cpp.txt dpiece_rust.txt
```

2. **使用Beyond Compare或其他对比工具**：
   - 打开两个文件
   - 查看差异高亮

### 方法2：使用Python脚本对比

创建一个Python脚本来详细对比：

```python
#!/usr/bin/env python3
"""对比C++和Rust的dPiece数组"""

def load_dpiece_file(filename):
    """加载dPiece文件，返回二维数组"""
    with open(filename, 'r') as f:
        lines = f.readlines()
        # 跳过第一行（标题）
        data = []
        for line in lines[1:]:
            if line.strip():
                values = [int(x) for x in line.strip().split(',')]
                data.append(values)
        return data

cpp_data = load_dpiece_file('dpiece_cpp.txt')
rust_data = load_dpiece_file('dpiece_rust.txt')

print(f"C++ array size: {len(cpp_data)}x{len(cpp_data[0]) if cpp_data else 0}")
print(f"Rust array size: {len(rust_data)}x{len(rust_data[0]) if rust_data else 0}")

# 找出所有差异
differences = []
for y in range(min(len(cpp_data), len(rust_data))):
    for x in range(min(len(cpp_data[y]), len(rust_data[y]))):
        if cpp_data[y][x] != rust_data[y][x]:
            differences.append((x, y, cpp_data[y][x], rust_data[y][x]))

print(f"\nTotal differences: {len(differences)}")

# 输出前100个差异
print("\nFirst 100 differences (x, y, C++, Rust):")
for i, (x, y, cpp_val, rust_val) in enumerate(differences[:100]):
    print(f"  ({x:3}, {y:3}): C++={cpp_val:4}, Rust={rust_val:4}")

# 按sector分组统计差异
sectors = {
    'sector4s': (0, 0, 46, 46),      # Top-left: (0,0) to (46,46)
    'sector2s': (46, 0, 92, 46),     # Top-right: (46,0) to (92,46)
    'sector3s': (0, 46, 46, 92),     # Bottom-left: (0,46) to (46,92)
    'sector1s': (46, 46, 92, 92),    # Bottom-right: (46,46) to (92,92)
}

print("\nDifferences by sector:")
for sector_name, (x1, y1, x2, y2) in sectors.items():
    sector_diffs = [(x, y, cpp_val, rust_val) 
                    for x, y, cpp_val, rust_val in differences
                    if x1 <= x < x2 and y1 <= y < y2]
    print(f"  {sector_name}: {len(sector_diffs)} differences")
```

### 方法3：使用Excel或Google Sheets

1. 将两个文件导入Excel
2. 使用条件格式高亮差异
3. 使用公式对比对应单元格

## 关键检查点

### 1. 检查sector边界

确认每个sector的边界是否正确：

- **sector4s**: (0,0) 到 (46,46)
- **sector2s**: (46,0) 到 (92,46)
- **sector3s**: (0,46) 到 (46,92)
- **sector1s**: (46,46) 到 (92,92)

### 2. 检查默认值

Town的默认piece是218，检查：
- 未加载的区域是否都是218？
- 边界区域（16-tile border）是否都是218？

### 3. 检查特定位置

对于已知的levelPieceId=290的位置：
- C++: dPiece(13,40)
- Rust: dPiece(15,44)

检查这些位置的值是否一致。

### 4. 检查坐标系统

确认数组索引顺序：
- C++: `dPiece[x][y]` (列优先？)
- Rust: `d_piece[x][y]` (列优先？)

## 常见问题排查

### 问题1：整个数组都是218（默认值）

**可能原因**：
- Sector加载失败
- 偏移量计算错误
- 文件路径错误

**检查**：
- 查看控制台输出，确认sector是否成功加载
- 检查MPQ文件路径
- 验证offset参数

### 问题2：部分sector数据缺失

**可能原因**：
- 某个sector加载失败
- 数组边界检查错误

**检查**：
- 查看每个sector的加载日志
- 检查数组边界条件

### 问题3：坐标偏移

**可能原因**：
- offset计算错误
- 数组索引顺序不同

**检查**：
- 对比sector边界位置
- 验证offset参数是否正确

### 问题4：值不匹配但模式相似

**可能原因**：
- TIL数据不同
- MegaTile到MicroTile的转换不同

**检查**：
- 对比TIL数据加载
- 验证MegaTile扩展逻辑

## 调试技巧

1. **缩小范围**：如果差异太多，先对比一个小的区域（如10x10）
2. **按sector对比**：分别对比每个sector，找出问题sector
3. **检查边界**：特别关注sector边界和数组边界
4. **验证加载顺序**：确认sector加载顺序是否一致

## 输出文件位置

- C++: 项目根目录下的 `dpiece_cpp.txt`
- Rust: 项目根目录下的 `dpiece_rust.txt`

## 注意事项

1. **文件大小**：每个文件约125KB（112x112个值）
2. **输出时机**：只在第一次渲染时输出一次
3. **文件覆盖**：每次运行会覆盖之前的文件
4. **性能影响**：文件写入只在第一次渲染时执行，对性能影响很小

## 下一步

对比完成后：
1. 记录所有差异位置
2. 分析差异模式（是否集中在某个sector？）
3. 检查相关的地图加载代码
4. 修复差异并重新测试





