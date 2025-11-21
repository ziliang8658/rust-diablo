# Assets Directory

This directory contains game assets like sprites, textures, and sounds.

## Creating Player Sprite

### Option 1: Use Python script (requires Pillow)

```bash
cd assets
pip install pillow
python create_player_sprite.py
```

### Option 2: Manual creation

Create a 32x32 PNG image named `player.png` in the `sprites/` directory.

The game will work without sprites - entities will be rendered as colored rectangles if sprites are missing.

## Sprite Requirements

- Format: PNG with transparency
- Size: 32×32 pixels (recommended)
- Location: `assets/sprites/player.png`

