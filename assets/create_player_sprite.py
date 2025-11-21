#!/usr/bin/env python3
"""
Create a simple player sprite for testing
Requires PIL/Pillow: pip install pillow
"""

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("Please install Pillow: pip install pillow")
    exit(1)

# Create a 32x32 image with transparency
img = Image.new('RGBA', (32, 32), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Draw a simple player character (cyan square with details)
# Body (cyan)
draw.rectangle([8, 8, 24, 28], fill=(0, 255, 255, 255))

# Head (lighter cyan)
draw.ellipse([10, 4, 22, 12], fill=(100, 255, 255, 255))

# Eyes (black dots)
draw.ellipse([13, 7, 15, 9], fill=(0, 0, 0, 255))
draw.ellipse([18, 7, 20, 9], fill=(0, 0, 0, 255))

# Arms (cyan)
draw.rectangle([6, 12, 8, 20], fill=(0, 200, 200, 255))  # Left arm
draw.rectangle([24, 12, 26, 20], fill=(0, 200, 200, 255))  # Right arm

# Legs (darker cyan)
draw.rectangle([10, 24, 14, 30], fill=(0, 150, 150, 255))  # Left leg
draw.rectangle([18, 24, 22, 30], fill=(0, 150, 150, 255))  # Right leg

# Save the image
img.save('sprites/player.png')
print("Player sprite created: sprites/player.png")

