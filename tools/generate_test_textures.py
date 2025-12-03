"""
生成测试用的地板和墙体纹理
用于验证渲染逻辑是否正确

地板纹理: 32x31 三角形 (LeftTriangle/RightTriangle)
墙体纹理: 32x32 方形/梯形
"""

from PIL import Image, ImageDraw
import os

OUTPUT_DIR = "../assets/test_textures"

def create_left_triangle(width=32, height=31, color=(100, 150, 200), bg=(0, 0, 0, 0)):
    """创建左三角形纹理 (地板左半部分)"""
    img = Image.new('RGBA', (width, height), bg)
    draw = ImageDraw.Draw(img)
    
    # 左三角形: 从左下角开始，顶点在中间
    # 下半部分: 宽度从2增加到32
    # 上半部分: 宽度从32减少到2
    for y in range(height):
        if y < height // 2:
            # 下半部分，宽度增加
            w = 2 + (y * 2)
        else:
            # 上半部分，宽度减少
            w = 2 + ((height - 1 - y) * 2)
        w = min(w, width)
        # 左对齐
        for x in range(w):
            # 添加一些纹理变化
            shade = 20 if (x + y) % 4 == 0 else 0
            r = min(255, color[0] + shade)
            g = min(255, color[1] + shade)
            b = min(255, color[2] + shade)
            img.putpixel((x, y), (r, g, b, 255))
    
    return img

def create_right_triangle(width=32, height=31, color=(100, 150, 200), bg=(0, 0, 0, 0)):
    """创建右三角形纹理 (地板右半部分)"""
    img = Image.new('RGBA', (width, height), bg)
    draw = ImageDraw.Draw(img)
    
    for y in range(height):
        if y < height // 2:
            w = 2 + (y * 2)
        else:
            w = 2 + ((height - 1 - y) * 2)
        w = min(w, width)
        # 右对齐
        start_x = width - w
        for x in range(w):
            shade = 20 if (x + y) % 4 == 0 else 0
            r = min(255, color[0] + shade)
            g = min(255, color[1] + shade)
            b = min(255, color[2] + shade)
            img.putpixel((start_x + x, y), (r, g, b, 255))
    
    return img

def create_square(width=32, height=32, color=(150, 100, 80), bg=(0, 0, 0, 0)):
    """创建方形纹理 (墙体)"""
    img = Image.new('RGBA', (width, height), bg)
    
    for y in range(height):
        for x in range(width):
            # 砖块纹理效果
            brick_x = x // 8
            brick_y = y // 4
            is_mortar = (x % 8 == 0) or (y % 4 == 0)
            
            if is_mortar:
                r, g, b = 80, 70, 60  # 灰泥颜色
            else:
                shade = ((brick_x + brick_y) % 2) * 20
                r = min(255, color[0] + shade)
                g = min(255, color[1] + shade)
                b = min(255, color[2] + shade)
            
            img.putpixel((x, y), (r, g, b, 255))
    
    return img

def create_left_trapezoid(width=32, height=32, color=(150, 100, 80), bg=(0, 0, 0, 0)):
    """创建左梯形纹理"""
    img = Image.new('RGBA', (width, height), bg)
    
    for y in range(height):
        if y < height // 2:
            # 下半部分是三角形
            w = 2 + (y * 2)
        else:
            # 上半部分是矩形
            w = width
        w = min(w, width)
        
        for x in range(w):
            shade = 15 if (x + y) % 3 == 0 else 0
            r = min(255, color[0] + shade)
            g = min(255, color[1] + shade)
            b = min(255, color[2] + shade)
            img.putpixel((x, y), (r, g, b, 255))
    
    return img

def create_right_trapezoid(width=32, height=32, color=(150, 100, 80), bg=(0, 0, 0, 0)):
    """创建右梯形纹理"""
    img = Image.new('RGBA', (width, height), bg)
    
    for y in range(height):
        if y < height // 2:
            w = 2 + (y * 2)
            start_x = width - w
        else:
            w = width
            start_x = 0
        w = min(w, width)
        
        for x in range(w):
            shade = 15 if (x + y) % 3 == 0 else 0
            r = min(255, color[0] + shade)
            g = min(255, color[1] + shade)
            b = min(255, color[2] + shade)
            img.putpixel((start_x + x, y), (r, g, b, 255))
    
    return img

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    # 地板颜色 (蓝灰色，类似Cathedral)
    floor_color = (80, 90, 110)
    
    # 墙体颜色 (棕色)
    wall_color = (120, 90, 70)
    
    # 生成地板纹理
    left_tri = create_left_triangle(color=floor_color)
    left_tri.save(os.path.join(OUTPUT_DIR, "floor_left.png"))
    print(f"Created floor_left.png (32x31)")
    
    right_tri = create_right_triangle(color=floor_color)
    right_tri.save(os.path.join(OUTPUT_DIR, "floor_right.png"))
    print(f"Created floor_right.png (32x31)")
    
    # 生成墙体纹理
    square = create_square(color=wall_color)
    square.save(os.path.join(OUTPUT_DIR, "wall_square.png"))
    print(f"Created wall_square.png (32x32)")
    
    left_trap = create_left_trapezoid(color=wall_color)
    left_trap.save(os.path.join(OUTPUT_DIR, "wall_left_trap.png"))
    print(f"Created wall_left_trap.png (32x32)")
    
    right_trap = create_right_trapezoid(color=wall_color)
    right_trap.save(os.path.join(OUTPUT_DIR, "wall_right_trap.png"))
    print(f"Created wall_right_trap.png (32x32)")
    
    # 生成一个完整的菱形地板 (64x31)
    diamond = Image.new('RGBA', (64, 31), (0, 0, 0, 0))
    diamond.paste(left_tri, (0, 0))
    diamond.paste(right_tri, (32, 0))
    diamond.save(os.path.join(OUTPUT_DIR, "floor_diamond.png"))
    print(f"Created floor_diamond.png (64x31)")
    
    # 生成不同颜色的测试纹理用于区分不同piece
    colors = [
        (200, 100, 100),  # 红
        (100, 200, 100),  # 绿
        (100, 100, 200),  # 蓝
        (200, 200, 100),  # 黄
        (200, 100, 200),  # 紫
        (100, 200, 200),  # 青
    ]
    
    for i, color in enumerate(colors):
        left = create_left_triangle(color=color)
        left.save(os.path.join(OUTPUT_DIR, f"test_floor_left_{i}.png"))
        
        right = create_right_triangle(color=color)
        right.save(os.path.join(OUTPUT_DIR, f"test_floor_right_{i}.png"))
    
    print(f"\nAll textures saved to {OUTPUT_DIR}")
    print("\nTo use these in Rust, load them and use as fallback textures when piece/block loading fails.")

if __name__ == "__main__":
    main()













