import os
from PIL import Image, ImageDraw
import random

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

def create_tile_floor(path):
    # 32x32 Dark Gray Stone Floor with noise/texture
    img = Image.new('RGB', (32, 32), color=(60, 60, 60))
    draw = ImageDraw.Draw(img)
    
    # Add random noise
    for i in range(100):
        x = random.randint(0, 31)
        y = random.randint(0, 31)
        color = random.randint(50, 70)
        draw.point((x, y), fill=(color, color, color))
        
    # Draw border/cracks
    draw.rectangle([0, 0, 31, 31], outline=(40, 40, 40))
    img.save(path)
    print(f"Created {path}")

def create_tile_wall(path):
    # 32x32 Brick Wall
    img = Image.new('RGB', (32, 32), color=(100, 80, 60))
    draw = ImageDraw.Draw(img)
    
    # Draw bricks
    brick_color = (120, 90, 70)
    outline_color = (60, 40, 30)
    
    # Row 1
    draw.rectangle([0, 0, 15, 7], fill=brick_color, outline=outline_color)
    draw.rectangle([16, 0, 31, 7], fill=brick_color, outline=outline_color)
    # Row 2 (offset)
    draw.rectangle([0, 8, 7, 15], fill=brick_color, outline=outline_color)
    draw.rectangle([8, 8, 23, 15], fill=brick_color, outline=outline_color)
    draw.rectangle([24, 8, 31, 15], fill=brick_color, outline=outline_color)
    # Row 3
    draw.rectangle([0, 16, 15, 23], fill=brick_color, outline=outline_color)
    draw.rectangle([16, 16, 31, 23], fill=brick_color, outline=outline_color)
    # Row 4 (offset)
    draw.rectangle([0, 24, 7, 31], fill=brick_color, outline=outline_color)
    draw.rectangle([8, 24, 23, 31], fill=brick_color, outline=outline_color)
    draw.rectangle([24, 24, 31, 31], fill=brick_color, outline=outline_color)
    
    img.save(path)
    print(f"Created {path}")

def create_tile_grass(path):
    # 32x32 Grass
    img = Image.new('RGB', (32, 32), color=(34, 139, 34))
    draw = ImageDraw.Draw(img)
    
    # Add grass blades
    for i in range(200):
        x = random.randint(0, 31)
        y = random.randint(0, 31)
        # Variations of green
        g = random.randint(100, 200)
        draw.point((x, y), fill=(34, g, 34))
        
    img.save(path)
    print(f"Created {path}")

def create_tile_water(path):
    # 32x32 Water
    img = Image.new('RGB', (32, 32), color=(65, 105, 225))
    draw = ImageDraw.Draw(img)
    
    # Add waves
    for i in range(50):
        x = random.randint(0, 30)
        y = random.randint(0, 31)
        draw.line([(x, y), (x+2, y)], fill=(100, 149, 237))
        
    img.save(path)
    print(f"Created {path}")

def main():
    sprites_dir = os.path.join("assets", "sprites")
    ensure_dir(sprites_dir)
    
    create_tile_floor(os.path.join(sprites_dir, "tile_floor.png"))
    create_tile_wall(os.path.join(sprites_dir, "tile_wall.png"))
    create_tile_grass(os.path.join(sprites_dir, "tile_grass.png"))
    create_tile_water(os.path.join(sprites_dir, "tile_water.png"))

if __name__ == "__main__":
    main()

