#!/usr/bin/env python3
"""
Create monkey sprite sheet for player
- Frame 1: Monkey idle (standing monkey)
- Frames 2-4: Monkey on cloud (walking animation)

Requires PIL/Pillow: pip install pillow
"""

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("Please install Pillow: pip install pillow")
    exit(1)

# Sprite sheet dimensions: 4 frames horizontally, each 64x64
FRAME_WIDTH = 64
FRAME_HEIGHT = 64
NUM_FRAMES = 4
SHEET_WIDTH = FRAME_WIDTH * NUM_FRAMES
SHEET_HEIGHT = FRAME_HEIGHT

# Create sprite sheet with transparency
img = Image.new('RGBA', (SHEET_WIDTH, SHEET_HEIGHT), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Colors
MONKEY_BROWN = (139, 90, 43, 255)      # Brown for monkey body
MONKEY_DARK_BROWN = (101, 67, 33, 255) # Dark brown for details
MONKEY_FACE = (160, 120, 80, 255)      # Lighter brown for face
CLOUD_WHITE = (240, 240, 255, 255)     # White for cloud
CLOUD_GRAY = (200, 200, 220, 255)      # Gray for cloud shadow

def draw_monkey_idle(x_offset, y_offset):
    """Draw idle monkey (standing)"""
    # Body (brown oval)
    draw.ellipse([x_offset + 20, y_offset + 25, x_offset + 44, y_offset + 55], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=2)
    
    # Head (brown circle)
    draw.ellipse([x_offset + 18, y_offset + 8, x_offset + 46, y_offset + 28], 
                 fill=MONKEY_FACE, outline=MONKEY_DARK_BROWN, width=2)
    
    # Ears
    draw.ellipse([x_offset + 12, y_offset + 10, x_offset + 22, y_offset + 20], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)
    draw.ellipse([x_offset + 42, y_offset + 10, x_offset + 52, y_offset + 20], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)
    
    # Eyes (black dots)
    draw.ellipse([x_offset + 24, y_offset + 16, x_offset + 28, y_offset + 20], 
                 fill=(0, 0, 0, 255))
    draw.ellipse([x_offset + 36, y_offset + 16, x_offset + 40, y_offset + 20], 
                 fill=(0, 0, 0, 255))
    
    # Nose (triangle)
    draw.polygon([
        (x_offset + 30, y_offset + 22),
        (x_offset + 34, y_offset + 22),
        (x_offset + 32, y_offset + 26)
    ], fill=(0, 0, 0, 255))
    
    # Mouth (smile)
    draw.arc([x_offset + 26, y_offset + 24, x_offset + 38, y_offset + 32], 
             start=0, end=180, fill=(0, 0, 0, 255), width=2)
    
    # Arms
    draw.ellipse([x_offset + 10, y_offset + 30, x_offset + 18, y_offset + 45], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)
    draw.ellipse([x_offset + 46, y_offset + 30, x_offset + 54, y_offset + 45], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)
    
    # Legs
    draw.ellipse([x_offset + 22, y_offset + 50, x_offset + 30, y_offset + 60], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)
    draw.ellipse([x_offset + 34, y_offset + 50, x_offset + 42, y_offset + 60], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)

def draw_monkey_on_cloud(x_offset, y_offset, frame_offset):
    """Draw monkey on cloud (walking animation)
    
    Args:
        frame_offset: 0, 1, or 2 for different animation frames
    """
    # Cloud base (white fluffy cloud)
    cloud_y = y_offset + 45
    
    # Cloud shape varies by frame for animation effect
    cloud_offsets = [
        (0, 0),   # Frame 1: centered
        (-2, 1),  # Frame 2: slight left and up
        (2, 1),   # Frame 3: slight right and up
    ]
    
    offset_x, offset_y = cloud_offsets[frame_offset % 3]
    
    # Cloud puffs (multiple overlapping circles)
    cloud_puffs = [
        (x_offset + 15 + offset_x, cloud_y + offset_y),
        (x_offset + 25 + offset_x, cloud_y - 3 + offset_y),
        (x_offset + 35 + offset_x, cloud_y + offset_y),
        (x_offset + 45 + offset_x, cloud_y - 2 + offset_y),
    ]
    
    for cx, cy in cloud_puffs:
        # Shadow layer
        draw.ellipse([cx - 8, cy - 3, cx + 8, cy + 5], 
                     fill=CLOUD_GRAY)
        # Main cloud
        draw.ellipse([cx - 8, cy - 5, cx + 8, cy + 3], 
                     fill=CLOUD_WHITE, outline=(180, 180, 200, 255), width=1)
    
    # Monkey body (slightly smaller, sitting on cloud)
    body_y = y_offset + 20
    draw.ellipse([x_offset + 22, body_y + 5, x_offset + 42, body_y + 25], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=2)
    
    # Head
    draw.ellipse([x_offset + 20, body_y - 8, x_offset + 44, body_y + 12], 
                 fill=MONKEY_FACE, outline=MONKEY_DARK_BROWN, width=2)
    
    # Ears
    draw.ellipse([x_offset + 14, body_y - 6, x_offset + 22, body_y + 2], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)
    draw.ellipse([x_offset + 42, body_y - 6, x_offset + 50, body_y + 2], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)
    
    # Eyes (excited, slightly larger)
    eye_y = body_y + 2
    draw.ellipse([x_offset + 26, eye_y, x_offset + 30, eye_y + 4], 
                 fill=(255, 255, 255, 255))  # White highlight
    draw.ellipse([x_offset + 28, eye_y + 1, x_offset + 30, eye_y + 3], 
                 fill=(0, 0, 0, 255))  # Black pupil
    draw.ellipse([x_offset + 34, eye_y, x_offset + 38, eye_y + 4], 
                 fill=(255, 255, 255, 255))
    draw.ellipse([x_offset + 36, eye_y + 1, x_offset + 38, eye_y + 3], 
                 fill=(0, 0, 0, 255))
    
    # Nose
    draw.polygon([
        (x_offset + 30, eye_y + 6),
        (x_offset + 34, eye_y + 6),
        (x_offset + 32, eye_y + 10)
    ], fill=(0, 0, 0, 255))
    
    # Mouth (bigger smile, excited)
    draw.arc([x_offset + 28, eye_y + 8, x_offset + 36, eye_y + 16], 
             start=0, end=180, fill=(0, 0, 0, 255), width=2)
    
    # Arms (waving, varies by frame)
    arm_angles = [0, -3, 3]  # Slight rotation for animation
    arm_offset = arm_angles[frame_offset % 3]
    
    # Left arm
    draw.ellipse([x_offset + 12 + arm_offset, body_y + 8, 
                  x_offset + 18 + arm_offset, body_y + 20], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)
    # Right arm
    draw.ellipse([x_offset + 46 - arm_offset, body_y + 8, 
                  x_offset + 52 - arm_offset, body_y + 20], 
                 fill=MONKEY_BROWN, outline=MONKEY_DARK_BROWN, width=1)

# Draw Frame 1: Idle monkey
draw_monkey_idle(0, 0)

# Draw Frames 2-4: Monkey on cloud (walking animation)
for i in range(3):
    frame_num = i + 1  # Frame 2, 3, 4
    x_offset = frame_num * FRAME_WIDTH
    draw_monkey_on_cloud(x_offset, 0, i)

# Save the image
output_path = 'sprites/player.png'
img.save(output_path)
print(f"Monkey sprite sheet created: {output_path}")
print(f"Dimensions: {SHEET_WIDTH}x{SHEET_HEIGHT}")
print(f"Frame size: {FRAME_WIDTH}x{FRAME_HEIGHT}")
print(f"Frames:")
print(f"  - Frame 1 (0-63): Idle monkey")
print(f"  - Frame 2 (64-127): Monkey on cloud (walk 1)")
print(f"  - Frame 3 (128-191): Monkey on cloud (walk 2)")
print(f"  - Frame 4 (192-255): Monkey on cloud (walk 3)")


