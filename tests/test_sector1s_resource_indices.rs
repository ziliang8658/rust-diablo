/// Test case for loading sector1s.dun and printing detailed resource index information
/// 
/// This test loads the town sector1s.dun file and prints:
/// 1. DUN file MegaTile indices
/// 2. TIL MegaTile to MicroTile mappings (4 MicroTiles per MegaTile)
/// 3. MIN MicroTile to CEL frame mappings (16 blocks per MicroTile for Town)
/// 4. Final CEL frame indices and tile types
/// 
/// # Reference
/// - Original code: `Source/levels/town.cpp` FillSector() Line 26-58
/// - Original code: `Source/levels/gendung.cpp` DRLG_LPass3() Line 768-801

use rust_diablo::resources::MpqManager;
use rust_diablo::tiles::{MinData, TilData, DungeonType};
use rust_diablo::world::dungeon_map::{load_dun_to_dpiece, MAXDUNX, MAXDUNY};

/// Helper function to load MPQ and tiles data
fn load_town_data() -> Option<(MpqManager, MinData, TilData)> {
    let mut mpq = MpqManager::new();
    
    // Try loading the MPQ file from assets directory
    if mpq.load_mpq("assets/Diabdat.mpq", 1000).is_err() {
        println!("⚠ MPQ file not found at assets/Diabdat.mpq, skipping test");
        return None;
    }
    
    // Load Town tiles data
    let min_data = match MinData::load_for_dungeon(&mut mpq, DungeonType::Town) {
        Ok(data) => data,
        Err(e) => {
            println!("⚠ Failed to load MIN data: {}", e);
            return None;
        }
    };
    
    let til_data = match TilData::load_for_dungeon(&mut mpq, DungeonType::Town) {
        Ok(data) => data,
        Err(e) => {
            println!("⚠ Failed to load TIL data: {}", e);
            return None;
        }
    };
    
    Some((mpq, min_data, til_data))
}

#[test]
fn test_sector1s_resource_indices() {
    let (mut mpq, min_data, til_data) = match load_town_data() {
        Some(data) => data,
        None => {
            println!("⚠ Skipping test - MPQ or tiles data not available");
            return;
        }
    };
    
    println!("\n=== Loading sector1s.dun and analyzing resource indices ===\n");
    
    // Load sector1s.dun file
    let dun_path = "levels\\towndata\\sector1s.dun";
    let default_piece = 218u16; // Town default piece (from C++ FillSector)
    
    // Load DUN file to dPiece (like C++ FillSector)
    let dungeon_map = match load_dun_to_dpiece(
        &mut mpq,
        dun_path,
        &til_data,
        0,   // offset_x (for testing, load at 0,0)
        0,   // offset_y
        default_piece,
    ) {
        Ok(map) => map,
        Err(e) => {
            println!("⚠ Failed to load DUN file: {}", e);
            return;
        }
    };
    
    // Load raw DUN file data to get MegaTile indices
    let dun_data = match mpq.find_file(dun_path) {
        Some(data) => data,
        None => {
            println!("⚠ Failed to find DUN file in MPQ");
            return;
        }
    };
    
    if dun_data.len() < 4 {
        println!("⚠ DUN file too small");
        return;
    }
    
    let width = u16::from_le_bytes([dun_data[0], dun_data[1]]) as usize;
    let height = u16::from_le_bytes([dun_data[2], dun_data[3]]) as usize;
    
    println!("DUN File: {} ({}x{} MegaTiles)", dun_path, width, height);
    println!("MIN Data: {} pieces, {} blocks per piece", min_data.len(), min_data.blocks_per_piece);
    println!("TIL Data: {} MegaTiles\n", til_data.len());
    
    // Parse DUN tile data
    let tile_layer = &dun_data[4..];
    
    println!("=== Resource Index Analysis ===\n");
    
    // Analyze each MegaTile position in the DUN file
    for j in 0..height {
        for i in 0..width {
            let idx = (j * width + i) * 2;
            if idx + 1 >= tile_layer.len() {
                continue;
            }
            
            let mega_tile_id = u16::from_le_bytes([tile_layer[idx], tile_layer[idx + 1]]);
            
            // Skip empty tiles (0)
            if mega_tile_id == 0 {
                continue;
            }
            
            println!("--- Position ({}, {}) ---", i, j);
            println!("  DUN MegaTile ID: {} (1-based)", mega_tile_id);
            
            // Get MegaTile from TIL data (convert 1-based to 0-based)
            let mega_tile_idx = (mega_tile_id - 1) as usize;
            if let Some(mega_tile) = til_data.get(mega_tile_idx) {
                println!("  TIL MegaTile[{}]:", mega_tile_idx);
                println!("    micro1 (top-left):   {} (levelPieceId, 1-based)", mega_tile.micro1);
                println!("    micro2 (top-right):  {} (levelPieceId, 1-based)", mega_tile.micro2);
                println!("    micro3 (bottom-left): {} (levelPieceId, 1-based)", mega_tile.micro3);
                println!("    micro4 (bottom-right): {} (levelPieceId, 1-based)", mega_tile.micro4);
                
                // Analyze each MicroTile
                let micro_tiles = [
                    ("micro1 (top-left)", mega_tile.micro1),
                    ("micro2 (top-right)", mega_tile.micro2),
                    ("micro3 (bottom-left)", mega_tile.micro3),
                    ("micro4 (bottom-right)", mega_tile.micro4),
                ];
                
                for (name, level_piece_id) in micro_tiles.iter() {
                    if *level_piece_id == 0 {
                        println!("    {}: empty (0)", name);
                        continue;
                    }
                    
                    // Convert 1-based levelPieceId to 0-based index
                    let piece_idx = (*level_piece_id - 1) as usize;
                    if let Some(piece) = min_data.get(piece_idx) {
                        println!("    {}: levelPieceId={} -> MIN Piece[{}] ({} blocks)", 
                            name, level_piece_id, piece_idx, piece.mt.len());
                        
                        // Print all blocks in this piece
                        for (block_idx, block) in piece.mt.iter().enumerate() {
                            if block.has_value() {
                                let frame = block.frame();
                                let tile_type = block.tile_type();
                                println!("      Block[{}]: frame={} (1-based CEL index), type={:?}", 
                                    block_idx, frame, tile_type);
                            } else {
                                println!("      Block[{}]: empty (0)", block_idx);
                            }
                        }
                    } else {
                        println!("    {}: levelPieceId={} -> MIN Piece[{}] NOT FOUND", 
                            name, level_piece_id, piece_idx);
                    }
                }
            } else {
                println!("  TIL MegaTile[{}]: NOT FOUND", mega_tile_idx);
            }
            
            println!();
        }
    }
    
    // Also print dPiece values for the loaded area
    println!("\n=== dPiece Values (first 20x20 area) ===\n");
    let print_width = width.min(20);
    let print_height = height.min(20);
    
    for j in 0..print_height {
        let row: Vec<String> = (0..print_width)
            .map(|i| {
                let xx = i * 2;
                let yy = j * 2;
                if xx < MAXDUNX && yy < MAXDUNY {
                    format!("{:4}", dungeon_map.d_piece[xx][yy])
                } else {
                    "  - ".to_string()
                }
            })
            .collect();
        println!("dPiece row {}: [{}]", j, row.join(", "));
    }
    
    println!("\n=== Summary ===");
    println!("Total MegaTiles in DUN: {}x{} = {}", width, height, width * height);
    println!("Non-empty MegaTiles: {}", 
        (0..height).map(|j| {
            (0..width).filter(|i| {
                let idx = (j * width + i) * 2;
                if idx + 1 < tile_layer.len() {
                    let mega_tile_id = u16::from_le_bytes([tile_layer[idx], tile_layer[idx + 1]]);
                    mega_tile_id != 0
                } else {
                    false
                }
            }).count()
        }).sum::<usize>());
    println!("\n✓ Resource index analysis complete");
}

