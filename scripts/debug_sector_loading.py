#!/usr/bin/env python3
"""调试sector加载，检查特定位置的索引计算"""

def analyze_position(x, y, sector_name, offset_x, offset_y):
    """分析特定位置在sector中的索引"""
    print(f"\n分析位置 ({x}, {y}) 在 {sector_name}:")
    print(f"  Sector偏移: ({offset_x}, {offset_y})")
    
    # 计算在sector内的相对位置
    rel_x = x - offset_x
    rel_y = y - offset_y
    
    print(f"  Sector内相对位置: ({rel_x}, {rel_y})")
    
    # 计算MegaTile索引（每个MegaTile是2x2）
    mega_i = rel_x // 2
    mega_j = rel_y // 2
    
    print(f"  MegaTile索引: ({mega_i}, {mega_j})")
    
    # 计算在.dun文件中的索引
    # .dun文件格式: [width(2B), height(2B), tile0(2B), tile1(2B), ...]
    # tile索引 = j * width + i
    tile_index = mega_j * 1000 + mega_i  # 假设width未知，用1000占位
    print(f"  .dun文件中的tile索引: {tile_index} (假设width未知)")
    
    # 计算在dPiece中的MicroTile位置
    micro_x_in_mega = rel_x % 2
    micro_y_in_mega = rel_y % 2
    
    print(f"  MegaTile内的MicroTile位置: ({micro_x_in_mega}, {micro_y_in_mega})")
    print(f"    0,0 -> v1 (top-left)")
    print(f"    1,0 -> v2 (top-right)")
    print(f"    0,1 -> v3 (bottom-left)")
    print(f"    1,1 -> v4 (bottom-right)")

# 分析有问题的位置
print("=" * 80)
print("Sector加载索引分析")
print("=" * 80)

# Sector3s的问题位置
print("\n" + "=" * 80)
print("Sector3s (offset: 0, 46) - 问题位置 (36-45, 78-79)")
print("=" * 80)
for x in range(36, 46):
    for y in [78, 79]:
        analyze_position(x, y, "sector3s", 0, 46)

# Sector1s的问题位置
print("\n" + "=" * 80)
print("Sector1s (offset: 46, 46) - 问题位置 (46, 78-79)")
print("=" * 80)
for y in [78, 79]:
    analyze_position(46, y, "sector1s", 46, 46)

# Sector2s的问题位置
print("\n" + "=" * 80)
print("Sector2s (offset: 46, 0) - 问题位置 (48-49, 20-21)")
print("=" * 80)
for x in range(48, 50):
    for y in range(20, 22):
        analyze_position(x, y, "sector2s", 46, 0)





