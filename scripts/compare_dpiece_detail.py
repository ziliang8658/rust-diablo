#!/usr/bin/env python3
"""详细对比sector内部的差异"""

def load_dpiece_file(filename):
    """加载dPiece文件，返回二维数组"""
    with open(filename, 'r') as f:
        lines = f.readlines()
        data = []
        for line in lines[1:]:
            line = line.strip()
            if line:
                values = [int(x) for x in line.split(',')]
                data.append(values)
        return data

cpp_data = load_dpiece_file("../docs/dpiece_cpp.txt")
rust_data = load_dpiece_file("../dpiece_rust.txt")

sectors = {
    'sector4s': (0, 0, 46, 46),
    'sector2s': (46, 0, 92, 46),
    'sector3s': (0, 46, 46, 92),
    'sector1s': (46, 46, 92, 92),
}

print("=" * 80)
print("Sector内部差异详情")
print("=" * 80)

for sector_name, (x1, y1, x2, y2) in sectors.items():
    print(f"\n{sector_name} (x:{x1}-{x2}, y:{y1}-{y2}):")
    print("-" * 80)
    sector_diffs = []
    for y in range(y1, min(y2, len(cpp_data), len(rust_data))):
        for x in range(x1, min(x2, len(cpp_data[y]), len(rust_data[y]))):
            cpp_val = cpp_data[y][x]
            rust_val = rust_data[y][x]
            if cpp_val != rust_val:
                sector_diffs.append((x, y, cpp_val, rust_val))
    
    if sector_diffs:
        print(f"  差异数: {len(sector_diffs)}")
        for x, y, cpp_val, rust_val in sector_diffs:
            print(f"    ({x:3}, {y:3}): C++={cpp_val:4}, Rust={rust_val:4}, 差值={cpp_val-rust_val:+4}")
    else:
        print("  ✓ 无差异")

