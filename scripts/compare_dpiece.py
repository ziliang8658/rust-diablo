#!/usr/bin/env python3
"""对比C++和Rust的dPiece数组，找出所有差异"""

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

def compare_dpiece(cpp_file, rust_file):
    """对比两个dPiece文件"""
    print("=" * 80)
    print("dPiece数组对比分析")
    print("=" * 80)
    
    cpp_data = load_dpiece_file(cpp_file)
    rust_data = load_dpiece_file(rust_file)
    
    if cpp_data is None or rust_data is None:
        return
    
    print(f"\nC++数组大小: {len(cpp_data)}行 x {len(cpp_data[0]) if cpp_data else 0}列")
    print(f"Rust数组大小: {len(rust_data)}行 x {len(rust_data[0]) if rust_data else 0}列")
    
    # 检查尺寸
    if len(cpp_data) != len(rust_data):
        print(f"⚠️ 警告: 行数不一致 (C++: {len(cpp_data)}, Rust: {len(rust_data)})")
    
    if cpp_data and rust_data and len(cpp_data[0]) != len(rust_data[0]):
        print(f"⚠️ 警告: 列数不一致 (C++: {len(cpp_data[0])}, Rust: {len(rust_data[0])})")
    
    # 找出所有差异
    differences = []
    max_y = min(len(cpp_data), len(rust_data))
    
    for y in range(max_y):
        max_x = min(len(cpp_data[y]), len(rust_data[y]))
        for x in range(max_x):
            cpp_val = cpp_data[y][x]
            rust_val = rust_data[y][x]
            if cpp_val != rust_val:
                differences.append((x, y, cpp_val, rust_val))
    
    print(f"\n总差异数: {len(differences)}")
    
    if len(differences) == 0:
        print("✓ 两个数组完全一致！")
        return
    
    # 按sector分组统计差异
    sectors = {
        'sector4s': (0, 0, 46, 46),      # Top-left: (0,0) to (46,46)
        'sector2s': (46, 0, 92, 46),     # Top-right: (46,0) to (92,46)
        'sector3s': (0, 46, 46, 92),     # Bottom-left: (0,46) to (46,92)
        'sector1s': (46, 46, 92, 92),    # Bottom-right: (46,46) to (92,92)
        'border': None,  # 其他区域
    }
    
    print("\n按sector统计差异:")
    sector_stats = {}
    for sector_name, bounds in sectors.items():
        if bounds is None:
            # border区域：不在任何sector内
            sector_diffs = [(x, y, cpp_val, rust_val) 
                          for x, y, cpp_val, rust_val in differences
                          if not any(x1 <= x < x2 and y1 <= y < y2 
                                    for x1, y1, x2, y2 in [sectors[s] for s in sectors if s != 'border' and sectors[s] is not None])]
        else:
            x1, y1, x2, y2 = bounds
            sector_diffs = [(x, y, cpp_val, rust_val) 
                          for x, y, cpp_val, rust_val in differences
                          if x1 <= x < x2 and y1 <= y < y2]
        sector_stats[sector_name] = sector_diffs
        print(f"  {sector_name}: {len(sector_diffs)} 个差异")
    
    # 输出前100个差异的详细信息
    print(f"\n前100个差异详情 (x, y, C++, Rust):")
    print("-" * 80)
    for i, (x, y, cpp_val, rust_val) in enumerate(differences[:100]):
        # 确定属于哪个sector
        sector = "border"
        for s_name, bounds in sectors.items():
            if bounds is not None:
                x1, y1, x2, y2 = bounds
                if x1 <= x < x2 and y1 <= y < y2:
                    sector = s_name
                    break
        
        print(f"  ({x:3}, {y:3}) [{sector:8}]: C++={cpp_val:4}, Rust={rust_val:4}, 差值={cpp_val-rust_val:+4}")
    
    if len(differences) > 100:
        print(f"\n  ... 还有 {len(differences) - 100} 个差异未显示")
    
    # 统计差异值的分布
    print("\n差异值统计:")
    diff_values = {}
    for x, y, cpp_val, rust_val in differences:
        diff = cpp_val - rust_val
        if diff not in diff_values:
            diff_values[diff] = 0
        diff_values[diff] += 1
    
    print("  最常见的差值:")
    for diff, count in sorted(diff_values.items(), key=lambda x: x[1], reverse=True)[:10]:
        print(f"    差值 {diff:+4}: {count} 次")
    
    # 检查特定位置
    print("\n检查特定位置:")
    test_positions = [
        (13, 40, "C++调试输出中的levelPieceId=290位置"),
        (15, 44, "Rust调试输出中的levelPieceId=290位置"),
        (71, 34, "之前Rust找到的levelPieceId=290位置"),
    ]
    
    for x, y, desc in test_positions:
        if y < len(cpp_data) and x < len(cpp_data[y]) and y < len(rust_data) and x < len(rust_data[y]):
            cpp_val = cpp_data[y][x]
            rust_val = rust_data[y][x]
            match = "✓" if cpp_val == rust_val else "✗"
            print(f"  {match} ({x:3}, {y:3}) [{desc}]: C++={cpp_val:4}, Rust={rust_val:4}")
        else:
            print(f"  ? ({x:3}, {y:3}) [{desc}]: 坐标超出范围")
    
    # 检查边界区域（最后16行和列）
    print("\n边界区域检查 (最后16行/列):")
    border_diffs = [(x, y, cpp_val, rust_val) 
                   for x, y, cpp_val, rust_val in differences
                   if x >= 96 or y >= 96]
    print(f"  边界区域差异数: {len(border_diffs)}")
    if border_diffs:
        print("  前10个边界差异:")
        for x, y, cpp_val, rust_val in border_diffs[:10]:
            print(f"    ({x:3}, {y:3}): C++={cpp_val:4}, Rust={rust_val:4}")

if __name__ == "__main__":
    # 默认文件路径
    cpp_file = "../../docs/dpiece_cpp.txt"
    rust_file = "docs/dpiece_rust.txt"
    
    # 如果提供了命令行参数，使用它们
    if len(sys.argv) >= 3:
        cpp_file = sys.argv[1]
        rust_file = sys.argv[2]
    elif len(sys.argv) == 2:
        print("用法: python compare_dpiece.py [cpp_file] [rust_file]")
        print(f"使用默认路径: C++={cpp_file}, Rust={rust_file}")
    
    compare_dpiece(cpp_file, rust_file)





