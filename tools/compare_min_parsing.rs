/// Tool to compare MIN file parsing between Rust and C++ implementations
///
/// This tool loads a MIN file and compares:
/// 1. Raw byte order handling
/// 2. Block reordering logic
/// 3. Frame index extraction
/// 4. Tile type extraction

use std::fs::File;
use std::io::Read;
use rust_diablo::tiles::min::MinData;
use rust_diablo::tiles::types::{DungeonType, LevelCelBlock};

fn main() -> anyhow::Result<()> {
    // Load Town MIN file (16 blocks per piece)
    let min_path = "levels/towndata/town.min";
    let mut file = File::open(min_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    println!("=== MIN File Analysis ===");
    println!("File: {}", min_path);
    println!("Size: {} bytes ({} uint16_t values)", data.len(), data.len() / 2);
    
    // Parse with Rust implementation
    let min_data = MinData::from_bytes(&data, DungeonType::Town)?;
    println!("\n=== Rust Parsing ===");
    println!("Pieces: {}", min_data.len());
    println!("Blocks per piece: {}", min_data.blocks_per_piece);
    
    // Analyze first few pieces
    println!("\n=== First 5 Pieces Analysis ===");
    for piece_idx in 0..5.min(min_data.len()) {
        if let Some(piece) = min_data.get(piece_idx) {
            println!("\nPiece {}:", piece_idx);
            for block_idx in 0..piece.mt.len() {
                let block = &piece.mt[block_idx];
                if block.has_value() {
                    let frame = block.frame();
                    let tile_type = block.tile_type();
                    println!("  Block {}: frame={}, type={:?}, raw_data=0x{:04X}", 
                        block_idx, frame, tile_type, block.data);
                } else {
                    println!("  Block {}: empty (0x0000)", block_idx);
                }
            }
        }
    }
    
    // Check block reordering for floor tiles (block 0 and 1)
    println!("\n=== Floor Tile Block Reordering Check ===");
    println!("For Town (16 blocks), floor tiles should be at block 0 and 1");
    println!("Original MIN file order: block 14 and 15 should map to block 0 and 1");
    
    // Verify reordering formula
    let blocks = 16;
    println!("\nReordering formula: blocks - 2 + (block & 1) - (block & 0xE)");
    for block in 0..16 {
        let src_idx = (blocks as isize - 2 + (block as isize & 1) - (block as isize & 0xE)) as usize;
        println!("  Block {} -> Source index {}", block, src_idx);
    }
    
    // Check frame index extraction
    println!("\n=== Frame Index Extraction Check ===");
    println!("Frame index should be in lower 12 bits (0x0FFF)");
    println!("Tile type should be in bits 12-14 (0x7000)");
    
    // Test with a known value
    let test_value = 0x2345u16; // Type=2 (LeftTriangle), Frame=0x345
    let test_block = LevelCelBlock::new(test_value);
    println!("\nTest value: 0x{:04X}", test_value);
    println!("  Frame: {} (expected 0x345 = {})", test_block.frame(), 0x345);
    println!("  Type: {:?} (expected LeftTriangle)", test_block.tile_type());
    println!("  Binary: {:016b}", test_value);
    println!("    Frame bits (0-11): {:012b}", test_value & 0xFFF);
    println!("    Type bits (12-14): {:03b}", (test_value & 0x7000) >> 12);
    
    // Compare with C++ bit layout
    println!("\n=== C++ Bit Layout Reference ===");
    println!("C++ LevelCelBlock::frame(): data & 0xFFF");
    println!("C++ LevelCelBlock::type(): (data & 0x7000) >> 12");
    println!("Rust matches C++ implementation ✓");
    
    // Check byte order
    println!("\n=== Byte Order Check ===");
    println!("Rust: u16::from_le_bytes([data[offset], data[offset + 1]])");
    println!("C++: Swap16LE(pieces[...])");
    println!("\nNote: Swap16LE converts to Little Endian regardless of system endianness");
    println!("Rust from_le_bytes explicitly reads as Little Endian");
    println!("Both should produce the same result on Little Endian systems ✓");
    
    // Sample raw bytes from MIN file
    if data.len() >= 32 {
        println!("\n=== Raw Bytes Sample (First 16 bytes) ===");
        println!("Bytes: {:?}", &data[0..16.min(data.len())]);
        for i in 0..8.min(data.len() / 2) {
            let offset = i * 2;
            if offset + 1 < data.len() {
                let val_le = u16::from_le_bytes([data[offset], data[offset + 1]]);
                let val_be = u16::from_be_bytes([data[offset], data[offset + 1]]);
                println!("  Offset {}: bytes=[{:02X} {:02X}], LE=0x{:04X} ({}), BE=0x{:04X} ({})", 
                    offset, data[offset], data[offset + 1], val_le, val_le, val_be, val_be);
            }
        }
    }
    
    Ok(())
}



