# Frame Data Comparison Guide

This guide explains how to compare frame data between Rust and C++ implementations to verify consistency.

## Overview

The key difference is:
- **C++**: After `ReencodeDungeonCels()`, frame data is optimized (padding removed, foliage extracted)
- **Rust**: Stores raw frame data, decodes on-demand

To compare, we need to:
1. Extract decoded frame data from Rust
2. Extract re-encoded frame data from C++ (after `ReencodeDungeonCels`)
3. Compare the decoded/processed data

## Rust Tool: `compare_frame_data`

### Usage

```bash
# Extract frame 1 as LeftTriangle (decoded)
cargo run --bin compare_frame_data -- 1 LeftTriangle frame_1_left.bin

# Extract frame 1 as RightTriangle (decoded)
cargo run --bin compare_frame_data -- 1 RightTriangle frame_1_right.bin

# Extract frame 100 as TransparentSquare (decoded)
cargo run --bin compare_frame_data -- 100 TransparentSquare frame_100_trans.bin
```

### Output

The tool will:
1. Load the frame from `TileTextureManager`
2. Decode it according to the specified `TileType`
3. Save the decoded data to a binary file
4. Print statistics and hex dump of first bytes

### Example Output

```
=== Frame Data Comparison Tool ===
Frame index: 1 (1-based)
Tile type: LeftTriangle

=== Frame Information ===
Frame 1 (1-based) / 0 (0-based):
  Raw data size: 544 bytes
  Is decoded: false
  First 32 bytes (hex):
    0000: 00 00 12 34 56 78 9A BC DE F0 12 34 56 78 9A BC
    0010: DE F0 12 34 56 78 9A BC DE F0 12 34 56 78 9A BC

=== Decoded Data ===
Decoded data size: 512 bytes
  First 32 bytes (hex):
    0000: 12 34 56 78 9A BC DE F0 12 34 56 78 9A BC DE F0
    0010: 12 34 56 78 9A BC DE F0 12 34 56 78 9A BC DE F0

=== Data Statistics ===
Raw data:
  Size: 544 bytes
  Expected for LeftTriangle:
    Original (with padding): 544 bytes
    Re-encoded (compact): 512 bytes
  Actual: 544 bytes

Decoded data:
  Size: 512 bytes
  Expected: 512 bytes (compact, no padding)
  ✅ Size matches expected!

Saved decoded data to: frame_1_left.bin
```

## C++ Tool: Export Frame Data

### Method 1: Add Debug Code to C++

Add this function to `Source/diablo.cpp` or create a debug tool:

```cpp
#include <fstream>
#include "engine/render/dun_render.hpp"

void ExportFrameData(uint32_t frame, const char* output_file) {
    if (!pDungeonCels) {
        Log("pDungeonCels is null!");
        return;
    }
    
    const uint8_t* frame_data = GetDunFrame(pDungeonCels.get(), frame);
    
    // Determine frame size based on type
    // For triangles: ReencodedTriangleFrameSize (512)
    // For squares: 1024
    // For TransparentSquare: variable (RLE)
    // For now, we'll use a reasonable max size
    size_t max_size = 2048; // Adjust based on your needs
    
    std::ofstream out(output_file, std::ios::binary);
    if (!out) {
        Log("Failed to open output file: %s", output_file);
        return;
    }
    
    // For triangles, we know the size is 512
    // For other types, we need to determine the size
    // This is a simplified version - you may need to adjust
    
    // Get frame offset table
    const auto* frameTable = reinterpret_cast<const uint32_t*>(pDungeonCels.get());
    uint32_t frame_start = Swap32LE(frameTable[frame]);
    uint32_t frame_end = Swap32LE(frameTable[frame + 1]);
    size_t frame_size = frame_end - frame_start;
    
    out.write(reinterpret_cast<const char*>(frame_data), frame_size);
    out.close();
    
    Log("Exported frame %u to %s (size: %zu bytes)", frame, output_file, frame_size);
}
```

### Method 2: Use Debugger

In Visual Studio or GDB:

1. Set breakpoint in `RenderTileFrame` or `GetDunFrame`
2. When frame is accessed, inspect the data:
   ```cpp
   const uint8_t* frame_data = GetDunFrame(pDungeonCels.get(), frame_idx);
   // In debugger: print memory at frame_data
   ```

3. Export memory to file using debugger commands

### Method 3: Create Standalone C++ Tool

Create `tools/export_frame_data.cpp`:

```cpp
#include <fstream>
#include <iostream>
#include "engine/render/dun_render.hpp"
#include "levels/gendung.h"

int main(int argc, char* argv[]) {
    if (argc < 3) {
        std::cerr << "Usage: " << argv[0] << " <frame_index> <output_file>" << std::endl;
        return 1;
    }
    
    uint32_t frame = std::stoul(argv[1]);
    const char* output_file = argv[2];
    
    // Load game level first (this calls ReencodeDungeonCels)
    // You'll need to initialize the game state
    
    if (!pDungeonCels) {
        std::cerr << "pDungeonCels is null! Load level first." << std::endl;
        return 1;
    }
    
    const uint8_t* frame_data = GetDunFrame(pDungeonCels.get(), frame);
    
    // Get frame size from offset table
    const auto* frameTable = reinterpret_cast<const uint32_t*>(pDungeonCels.get());
    uint32_t frame_start = Swap32LE(frameTable[frame]);
    uint32_t frame_end = Swap32LE(frameTable[frame + 1]);
    size_t frame_size = frame_end - frame_start;
    
    std::ofstream out(output_file, std::ios::binary);
    if (!out) {
        std::cerr << "Failed to open output file: " << output_file << std::endl;
        return 1;
    }
    
    out.write(reinterpret_cast<const char*>(frame_data), frame_size);
    out.close();
    
    std::cout << "Exported frame " << frame << " to " << output_file 
              << " (size: " << frame_size << " bytes)" << std::endl;
    
    return 0;
}
```

## Comparison Process

### Step 1: Extract from Rust

```bash
# Extract frame 1 as LeftTriangle
cargo run --bin compare_frame_data -- 1 LeftTriangle rust_frame_1.bin
```

### Step 2: Extract from C++

Use one of the methods above to export the same frame from C++:

```cpp
// In C++ code or debugger
ExportFrameData(1, "cpp_frame_1.bin");
```

### Step 3: Compare Files

**Windows:**
```cmd
fc /b rust_frame_1.bin cpp_frame_1.bin
```

**Linux/Mac:**
```bash
diff rust_frame_1.bin cpp_frame_1.bin
# Or use hexdump for visual comparison
hexdump -C rust_frame_1.bin > rust_hex.txt
hexdump -C cpp_frame_1.bin > cpp_hex.txt
diff rust_hex.txt cpp_hex.txt
```

**Python (detailed comparison):**
```python
with open('rust_frame_1.bin', 'rb') as f:
    rust_data = f.read()
with open('cpp_frame_1.bin', 'rb') as f:
    cpp_data = f.read()

if rust_data == cpp_data:
    print("✅ Files are identical!")
else:
    print(f"❌ Files differ!")
    print(f"Rust size: {len(rust_data)} bytes")
    print(f"C++ size: {len(cpp_data)} bytes")
    
    # Find first difference
    min_len = min(len(rust_data), len(cpp_data))
    for i in range(min_len):
        if rust_data[i] != cpp_data[i]:
            print(f"First difference at byte {i}: Rust=0x{rust_data[i]:02X}, C++=0x{cpp_data[i]:02X}")
            break
```

## Important Notes

### Frame Index

- **Frame indices are 1-based** in both Rust and C++
- Frame 0 is invalid (empty/no tile)
- Frame 1 is the first actual frame

### Data Format

- **Raw data** (before decoding): Contains padding for triangles (544 bytes)
- **Decoded data** (after decoding): Compact format (512 bytes for triangles)
- **C++ re-encoded data**: Already processed, no padding

### Tile Type Matters

The same frame can be decoded differently based on `TileType`:
- `LeftTriangle`: Decodes as left triangle (removes padding from left)
- `RightTriangle`: Decodes as right triangle (removes padding from right)
- `Square`: Direct copy (1024 bytes)
- `TransparentSquare`: RLE decode

**Always specify the correct `TileType` when comparing!**

### Re-encoding in C++

C++ performs `ReencodeDungeonCels()` during level loading:
- Removes padding from triangles
- Extracts foliage from floor tiles
- Removes unused frames

Rust does **not** perform this optimization:
- Stores raw data
- Decodes on-demand
- Keeps all frames

**Therefore, compare decoded Rust data with re-encoded C++ data.**

## Troubleshooting

### Size Mismatch

If sizes don't match:
1. Check if you're comparing raw vs decoded data
2. Verify the `TileType` is correct
3. Check if C++ has re-encoded the data (after `ReencodeDungeonCels`)

### Content Mismatch

If content differs:
1. Verify frame index is correct (1-based)
2. Check if frame is from main CEL or special CEL
3. Ensure both are using the same CEL file (town.cel, l1.cel, etc.)
4. Check if C++ has applied any transformations

### Frame Not Found

If frame doesn't exist:
1. Check frame index is within valid range
2. Verify CEL file is loaded
3. Check if frame was removed by `ReencodeDungeonCels` (C++ only)

## Example: Complete Comparison Workflow

```bash
# 1. Extract frame 1 from Rust (as LeftTriangle)
cargo run --bin compare_frame_data -- 1 LeftTriangle rust_frame_1_left.bin

# 2. Extract frame 1 from C++ (after ReencodeDungeonCels)
# (Use C++ tool or debugger)
# Saves to: cpp_frame_1_left.bin

# 3. Compare
fc /b rust_frame_1_left.bin cpp_frame_1_left.bin

# Expected: Files should be identical (both 512 bytes, same content)
```

## Next Steps

If frames don't match:
1. Check the decoding logic in `rust-diablo/src/tiles/decoder/`
2. Verify padding removal matches C++ `ReencodeDungeonCels*` functions
3. Check frame offset calculation
4. Verify byte order (little-endian)












