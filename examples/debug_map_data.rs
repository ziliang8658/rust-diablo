use anyhow::Result;
/// Debug utility to inspect map rendering data
///
/// This example loads the tileset and map data, then prints out detailed information
/// to help diagnose rendering issues.
use rust_diablo::resources::MpqManager;
use rust_diablo::tiles::types::DungeonType;
use rust_diablo::tiles::{texture_manager::TileTextureManager, MinData, TilData};
use rust_diablo::world::DungeonMap;

fn main() -> Result<()> {
    println!("=== Map Data Debug Utility ===\n");

    // Initialize MPQ
    let data_path = "../diabdat.mpq";
    println!("Loading MPQ from: {}", data_path);
    let mut mpq_manager = MpqManager::new();
    mpq_manager.load_mpq(data_path, 0)?;
    println!("✓ MPQ loaded\n");

    // Test with Town
    let dungeon_type = DungeonType::Town;
    println!("=== Testing Dungeon Type: {:?} ===\n", dungeon_type);

    // Load TIL data
    println!("Loading TIL data...");
    let til_data = TilData::load_for_dungeon(&mut mpq_manager, dungeon_type)?;
    println!("✓ TIL loaded: {} mega tiles\n", til_data.len());

    // Print first few MegaTiles
    println!("First 5 MegaTiles:");
    for i in 0..5.min(til_data.len()) {
        if let Some(mega) = til_data.get(i) {
            println!(
                "  MegaTile[{}]: micro1={}, micro2={}, micro3={}, micro4={}",
                i, mega.micro1, mega.micro2, mega.micro3, mega.micro4
            );
        }
    }
    println!();

    // Load MIN data
    println!("Loading MIN data...");
    let min_data = MinData::from_mpq(&mut mpq_manager, "levels/towndata/town.min", dungeon_type)?;
    println!(
        "✓ MIN loaded: {} pieces, {} blocks per piece\n",
        min_data.len(),
        min_data.blocks_per_piece
    );

    // Print first few pieces
    println!("First 5 Pieces:");
    for piece_idx in 0..5.min(min_data.len()) {
        println!("  Piece[{}]:", piece_idx);
        if let Some(piece) = min_data.get(piece_idx) {
            for (block_idx, block) in piece.mt.iter().take(piece.mt.len().min(4)).enumerate() {
                println!(
                    "    Block[{}]: data={:#06x}, has_value={}, type={:?}, frame={} (1-based)",
                    block_idx,
                    block.data,
                    block.has_value(),
                    block.tile_type(),
                    block.frame()
                );
            }
            if piece.mt.len() > 4 {
                println!("    ... ({} more blocks)", piece.mt.len() - 4);
            }
        }
    }
    println!();

    // Load texture manager
    println!("Loading Texture Manager...");
    let mut texture_mgr = TileTextureManager::load_for_dungeon(dungeon_type, &mut mpq_manager)?;
    println!("✓ Texture Manager loaded\n");

    // Test decoding a few tiles
    println!("Testing tile decoding:");
    let test_pieces = [0, 1, 2];
    for &piece_idx in &test_pieces {
        if piece_idx >= min_data.len() {
            continue;
        }

        println!("  Piece[{}]:", piece_idx);
        for block_idx in 0..4.min(min_data.blocks_per_piece) {
            match texture_mgr.get_decoded_tile(piece_idx, block_idx) {
                Ok(rgba) => {
                    let pixel_count = rgba.len() / 4;
                    let non_transparent = rgba.chunks(4).filter(|p| p[3] != 0).count();
                    let all_black = rgba
                        .chunks(4)
                        .filter(|p| p[0] == 0 && p[1] == 0 && p[2] == 0 && p[3] == 255)
                        .count();
                    println!(
                        "    Block[{}]: {} pixels, {} non-transparent, {} black",
                        block_idx, pixel_count, non_transparent, all_black
                    );

                    // Sample a few pixels
                    if pixel_count > 0 {
                        let sample_indices = [0, pixel_count / 2, pixel_count - 1];
                        print!("      Samples: ");
                        for &idx in &sample_indices {
                            if idx < pixel_count {
                                let offset = idx * 4;
                                print!(
                                    "[{},{},{},{}] ",
                                    rgba[offset],
                                    rgba[offset + 1],
                                    rgba[offset + 2],
                                    rgba[offset + 3]
                                );
                            }
                        }
                        println!();
                    }
                }
                Err(e) => {
                    println!("    Block[{}]: Error - {}", block_idx, e);
                }
            }
        }
        println!();
    }

    // Create and inspect dungeon map
    println!("Creating DungeonMap...");
    let mut dungeon_map = DungeonMap::new();

    // Fill with test pattern
    println!("Filling with test pattern...");
    for j in 0..40 {
        for i in 0..40 {
            if i == 0 || i == 39 || j == 0 || j == 39 {
                dungeon_map.set_dungeon_tile(i, j, 3); // Wall tile
            } else {
                dungeon_map.set_dungeon_tile(i, j, 2); // Floor tile
            }
        }
    }

    // Expand to dPiece
    println!("Expanding to dPiece...");
    dungeon_map.expand_to_d_piece(&til_data, 0);
    println!("✓ DungeonMap created\n");

    // Print a small section of dungeon array
    println!("Dungeon array (top-left 10x10):");
    for j in 0..10 {
        print!("  ");
        for i in 0..10 {
            print!("{:3} ", dungeon_map.dungeon[i][j]);
        }
        println!();
    }
    println!();

    // Print a small section of dPiece array
    println!("dPiece array (16-26, 16-26) - levelPieceId values:");
    for j in 16..26 {
        print!("  ");
        for i in 16..26 {
            print!("{:4} ", dungeon_map.get_piece(i as i32, j as i32));
        }
        println!();
    }
    println!();

    // Verify levelPieceId values are in valid range
    println!("Verifying dPiece values...");
    let mut min_val = u16::MAX;
    let mut max_val = 0u16;
    let mut zero_count = 0;
    let mut valid_count = 0;

    for j in 16..96 {
        for i in 16..96 {
            let val = dungeon_map.get_piece(i, j);
            if val == 0 {
                zero_count += 1;
            } else {
                valid_count += 1;
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }
    }

    println!("  Zero values: {}", zero_count);
    println!("  Valid values: {}", valid_count);
    println!(
        "  Range: {} to {} (should be 0 to {})",
        min_val,
        max_val,
        min_data.len() - 1
    );

    if max_val as usize >= min_data.len() {
        println!("  ⚠️ WARNING: Some dPiece values exceed MIN data size!");
    } else {
        println!("  ✓ All dPiece values are in valid range");
    }
    println!();

    // Test rendering logic
    println!("Testing rendering logic:");
    let test_positions = [(20, 20), (21, 20), (20, 21), (21, 21)];
    for &(x, y) in &test_positions {
        let level_piece_id = dungeon_map.get_piece(x, y) as usize;
        println!("  dPiece[{}, {}] = {} (levelPieceId)", x, y, level_piece_id);

        if level_piece_id > 0 && level_piece_id < min_data.len() {
            if let Some(piece) = min_data.get(level_piece_id) {
                println!("    This piece has {} blocks", piece.mt.len());

                // Test floor tiles (blocks 0 and 1)
                for block_idx in 0..2 {
                    if let Some(block) = piece.mt.get(block_idx) {
                        println!(
                            "      Block[{}]: frame={}, type={:?}",
                            block_idx,
                            block.frame(),
                            block.tile_type()
                        );
                    }
                }
            }
        } else {
            println!("    Invalid levelPieceId (out of range or zero)");
        }
    }

    println!("\n=== Debug Complete ===");
    println!("\nIf you see issues above, they might explain the rendering problems.");
    println!("Look for:");
    println!("  - Invalid frame indices (should be 1-based, not 0)");
    println!("  - levelPieceId values out of range");
    println!("  - All-transparent tiles");
    println!("  - All-black tiles");

    Ok(())
}
