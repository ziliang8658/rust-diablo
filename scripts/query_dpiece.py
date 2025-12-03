#!/usr/bin/env python3
"""查询dPiece数组的工具

功能：
1. 输入dPiece值，查找所有包含该值的位置
2. 输入位置(x, y)，显示该位置及其周围位置的dPiece值
"""

import sys
import os

def load_dpiece_file(filename):
    """加载dPiece文件，返回二维数组"""
    if not os.path.exists(filename):
        print(f"错误: 文件不存在: {filename}")
        return None
    
    with open(filename, 'r') as f:
        lines = f.readlines()
        # 跳过第一行（标题）
        data = []
        for line in lines[1:]:
            line = line.strip()
            if line:
                values = [int(x) for x in line.split(',')]
                data.append(values)
        return data

def find_all_positions_with_value(data, value):
    """查找所有包含指定值的位置"""
    positions = []
    for y in range(len(data)):
        for x in range(len(data[y])):
            if data[y][x] == value:
                positions.append((x, y))
    return positions

def get_sector_name(x, y):
    """根据坐标确定所属sector"""
    if 0 <= x < 46 and 0 <= y < 46:
        return "sector4s"
    elif 46 <= x < 92 and 0 <= y < 46:
        return "sector2s"
    elif 0 <= x < 46 and 46 <= y < 92:
        return "sector3s"
    elif 46 <= x < 92 and 46 <= y < 92:
        return "sector1s"
    else:
        return "border"

def query_by_value(cpp_file, rust_file, value):
    """根据dPiece值查询所有位置"""
    print("=" * 80)
    print(f"查询 dPiece 值 = {value} 的所有位置")
    print("=" * 80)
    
    cpp_data = load_dpiece_file(cpp_file)
    rust_data = load_dpiece_file(rust_file)
    
    if cpp_data is None or rust_data is None:
        return
    
    # 查找C++中的位置
    cpp_positions = find_all_positions_with_value(cpp_data, value)
    print(f"\nC++中找到 {len(cpp_positions)} 个位置:")
    if cpp_positions:
        # 按sector分组
        sectors = {}
        for x, y in cpp_positions:
            sector = get_sector_name(x, y)
            if sector not in sectors:
                sectors[sector] = []
            sectors[sector].append((x, y))
        
        for sector in ["sector4s", "sector2s", "sector3s", "sector1s", "border"]:
            if sector in sectors:
                print(f"\n  {sector}:")
                for x, y in sorted(sectors[sector]):
                    rust_val = rust_data[y][x] if y < len(rust_data) and x < len(rust_data[y]) else -1
                    match = "✓" if rust_val == value else f"✗(Rust={rust_val})"
                    print(f"    ({x:3}, {y:3}) {match}")
    
    # 查找Rust中的位置
    rust_positions = find_all_positions_with_value(rust_data, value)
    print(f"\nRust中找到 {len(rust_positions)} 个位置:")
    if rust_positions:
        # 按sector分组
        sectors = {}
        for x, y in rust_positions:
            sector = get_sector_name(x, y)
            if sector not in sectors:
                sectors[sector] = []
            sectors[sector].append((x, y))
        
        for sector in ["sector4s", "sector2s", "sector3s", "sector1s", "border"]:
            if sector in sectors:
                print(f"\n  {sector}:")
                for x, y in sorted(sectors[sector]):
                    cpp_val = cpp_data[y][x] if y < len(cpp_data) and x < len(cpp_data[y]) else -1
                    match = "✓" if cpp_val == value else f"✗(C++={cpp_val})"
                    print(f"    ({x:3}, {y:3}) {match}")
    
    # 找出差异
    cpp_set = set(cpp_positions)
    rust_set = set(rust_positions)
    only_cpp = cpp_set - rust_set
    only_rust = rust_set - cpp_set
    both = cpp_set & rust_set
    
    print(f"\n对比结果:")
    print(f"  仅在C++中: {len(only_cpp)} 个位置")
    print(f"  仅在Rust中: {len(only_rust)} 个位置")
    print(f"  两者都有: {len(both)} 个位置")

def query_by_position(cpp_file, rust_file, x, y, radius=2):
    """根据位置查询dPiece值及其周围位置"""
    print("=" * 80)
    print(f"查询位置 ({x}, {y}) 及其周围 (半径={radius}) 的 dPiece 值")
    print("=" * 80)
    
    cpp_data = load_dpiece_file(cpp_file)
    rust_data = load_dpiece_file(rust_file)
    
    if cpp_data is None or rust_data is None:
        return
    
    # 检查坐标是否有效
    if y >= len(cpp_data) or x >= len(cpp_data[y]):
        print(f"错误: 坐标 ({x}, {y}) 超出C++数组范围 (最大: {len(cpp_data[0])-1}, {len(cpp_data)-1})")
        return
    
    if y >= len(rust_data) or x >= len(rust_data[y]):
        print(f"错误: 坐标 ({x}, {y}) 超出Rust数组范围 (最大: {len(rust_data[0])-1}, {len(rust_data)-1})")
        return
    
    # 显示中心位置
    cpp_val = cpp_data[y][x]
    rust_val = rust_data[y][x]
    sector = get_sector_name(x, y)
    match = "✓" if cpp_val == rust_val else "✗"
    
    print(f"\n中心位置 ({x}, {y}) [{sector}]:")
    print(f"  C++: {cpp_val}")
    print(f"  Rust: {rust_val}")
    print(f"  匹配: {match}")
    if cpp_val != rust_val:
        print(f"  差值: {cpp_val - rust_val:+d}")
    
    # 显示周围位置（8方向 + 中心）
    print(f"\n周围位置 (半径={radius}):")
    print("-" * 80)
    
    # 计算显示范围
    min_x = max(0, x - radius)
    max_x = min(len(cpp_data[0]) - 1, x + radius)
    min_y = max(0, y - radius)
    max_y = min(len(cpp_data) - 1, y + radius)
    
    # 打印表头
    print(f"{'位置':<12} {'C++':<8} {'Rust':<8} {'匹配':<6} {'Sector':<10}")
    print("-" * 80)
    
    for py in range(min_y, max_y + 1):
        for px in range(min_x, max_x + 1):
            if py < len(cpp_data) and px < len(cpp_data[py]):
                cpp_v = cpp_data[py][px]
                rust_v = rust_data[py][px] if py < len(rust_data) and px < len(rust_data[py]) else -1
                match_str = "✓" if cpp_v == rust_v else "✗"
                sector_name = get_sector_name(px, py)
                
                # 标记中心位置
                marker = " *" if px == x and py == y else "  "
                print(f"({px:3},{py:3}){marker} {cpp_v:6}   {rust_v:6}   {match_str:4}   {sector_name}")

def print_usage():
    """打印使用说明"""
    print("=" * 80)
    print("dPiece 查询工具")
    print("=" * 80)
    print("\n用法:")
    print("  1. 按值查询: python query_dpiece.py value [cpp_file] [rust_file]")
    print("     示例: python query_dpiece.py 290")
    print("            python query_dpiece.py 290 ../docs/dpiece_cpp.txt ../dpiece_rust.txt")
    print()
    print("  2. 按位置查询: python query_dpiece.py pos x y [radius] [cpp_file] [rust_file]")
    print("     示例: python query_dpiece.py pos 13 40")
    print("            python query_dpiece.py pos 13 40 3")
    print("            python query_dpiece.py pos 13 40 2 ../docs/dpiece_cpp.txt ../dpiece_rust.txt")
    print()
    print("默认文件路径:")
    print("  C++: ../docs/dpiece_cpp.txt")
    print("  Rust: docs/dpiece_rust.txt")

if __name__ == "__main__":
    # 默认文件路径
    default_cpp_file = "../docs/dpiece_cpp.txt"
    default_rust_file = "docs/dpiece_rust.txt"
    
    if len(sys.argv) < 2:
        print_usage()
        sys.exit(1)
    
    command = sys.argv[1].lower()
    
    if command == "value" or command.isdigit():
        # 按值查询
        if command.isdigit():
            value = int(command)
            cpp_file = sys.argv[2] if len(sys.argv) > 2 else default_cpp_file
            rust_file = sys.argv[3] if len(sys.argv) > 3 else default_rust_file
        else:
            if len(sys.argv) < 3:
                print("错误: 需要指定dPiece值")
                print_usage()
                sys.exit(1)
            value = int(sys.argv[2])
            cpp_file = sys.argv[3] if len(sys.argv) > 3 else default_cpp_file
            rust_file = sys.argv[4] if len(sys.argv) > 4 else default_rust_file
        
        query_by_value(cpp_file, rust_file, value)
    
    elif command == "pos" or command == "position":
        # 按位置查询
        if len(sys.argv) < 4:
            print("错误: 需要指定位置坐标 (x, y)")
            print_usage()
            sys.exit(1)
        
        x = int(sys.argv[2])
        y = int(sys.argv[3])
        radius = int(sys.argv[4]) if len(sys.argv) > 4 and sys.argv[4].isdigit() else 2
        
        # 检查是否有文件路径参数
        arg_idx = 4 if sys.argv[4].isdigit() else 3
        cpp_file = sys.argv[arg_idx + 1] if len(sys.argv) > arg_idx + 1 else default_cpp_file
        rust_file = sys.argv[arg_idx + 2] if len(sys.argv) > arg_idx + 2 else default_rust_file
        
        query_by_position(cpp_file, rust_file, x, y, radius)
    
    else:
        print(f"错误: 未知命令 '{command}'")
        print_usage()
        sys.exit(1)

