# dPiece 查询工具使用说明

## 文件说明

- `query_dpiece.py`: 主要的查询工具，支持按值查询和按位置查询
- `compare_dpiece_detail.py`: 详细对比sector内部差异

## query_dpiece.py 使用方法

### 1. 按值查询

查找所有包含指定 dPiece 值的位置。

**语法**:
```bash
python query_dpiece.py <value> [cpp_file] [rust_file]
```

**示例**:
```bash
# 查找所有 dPiece = 290 的位置
python query_dpiece.py 290

# 使用自定义文件路径
python query_dpiece.py 290 ../docs/dpiece_cpp.txt docs/dpiece_rust.txt
```

**输出内容**:
- C++中找到的所有位置（按sector分组）
- Rust中找到的所有位置（按sector分组）
- 对比结果（仅在C++中、仅在Rust中、两者都有）

### 2. 按位置查询

查询指定位置及其周围位置的 dPiece 值。

**语法**:
```bash
python query_dpiece.py pos <x> <y> [radius] [cpp_file] [rust_file]
```

**参数**:
- `x`: X坐标 (0-111)
- `y`: Y坐标 (0-111)
- `radius`: 查询半径（默认2，即显示周围5x5的区域）

**示例**:
```bash
# 查询位置 (13, 40) 及其周围（默认半径2）
python query_dpiece.py pos 13 40

# 查询位置 (13, 40) 及其周围（半径3）
python query_dpiece.py pos 13 40 3

# 使用自定义文件路径
python query_dpiece.py pos 13 40 2 ../docs/dpiece_cpp.txt docs/dpiece_rust.txt
```

**输出内容**:
- 中心位置的C++和Rust值及匹配状态
- 周围所有位置的C++和Rust值对比表
- 每个位置的sector归属
- 中心位置用 `*` 标记

## 默认文件路径

- C++文件: `../docs/dpiece_cpp.txt`
- Rust文件: `docs/dpiece_rust.txt`

## 使用场景

### 场景1: 查找特定 tile 的所有位置
```bash
# 查找所有 levelPieceId = 290 的位置
python query_dpiece.py 290
```

### 场景2: 检查特定位置的 tile 值
```bash
# 检查位置 (13, 40) 的 tile 值
python query_dpiece.py pos 13 40
```

### 场景3: 查看周围 tile 分布
```bash
# 查看位置 (71, 34) 周围3x3区域的 tile 分布
python query_dpiece.py pos 71 34 1
```

### 场景4: 调试差异位置
```bash
# 查看已知差异位置的详细信息
python query_dpiece.py pos 36 78 3
```

## 输出说明

### 按值查询输出

- **✓**: C++和Rust值一致
- **✗(Rust=xxx)**: Rust值不同
- **✗(C++=xxx)**: C++值不同

### 按位置查询输出

- **✓**: C++和Rust值一致
- **✗**: C++和Rust值不一致
- **\***: 标记中心位置
- **Sector**: 显示位置所属的sector（sector4s/sector2s/sector3s/sector1s/border）

## 注意事项

1. 确保 dPiece 文件已生成（运行程序后会生成）
2. 文件路径是相对于脚本所在目录的
3. 坐标范围：0-111（MAXDUNX=112, MAXDUNY=112）
4. 查询半径建议不超过5，否则输出会很长





