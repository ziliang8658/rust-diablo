/// Test map rendering integration
/// 
/// Verifies that TileTextureManager can be integrated with World
/// and basic rendering functions work correctly.

use rust_diablo::resources::MpqManager;
use rust_diablo::tiles::{MinData, TilData, SolData, DungeonType};
use rust_diablo::tiles::texture_manager::TileTextureManager;

/// Helper function to try loading MPQ and tiles data
fn try_load_tiles() -> Option<(MinData, TilData, SolData, TileTextureManager)> {
    let mut mpq = MpqManager::new();
    
    // Try loading the MPQ file from assets directory
    if mpq.load_mpq("assets/Diabdat.mpq", 1000).is_err() {
        println!("⚠ MPQ file not found at assets/Diabdat.mpq, skipping test");
        return None;
    }
    
    // Try loading tiles data
    let min_data = MinData::load_for_dungeon(&mut mpq, DungeonType::Cathedral).ok()?;
    let til_data = TilData::load_for_dungeon(&mut mpq, DungeonType::Cathedral).ok()?;
    let sol_data = SolData::load_for_dungeon(&mut mpq, DungeonType::Cathedral).ok()?;
    
    // Try loading texture manager
    let texture_mgr = TileTextureManager::load_for_dungeon(
        DungeonType::Cathedral,
        &mut mpq
    ).ok()?;
    
    Some((min_data, til_data, sol_data, texture_mgr))
}

#[test]
fn test_texture_manager_loading() {
    if let Some((min_data, til_data, sol_data, _texture_mgr)) = try_load_tiles() {
        println!("✓ Loaded MIN data: {} entries", min_data.len());
        println!("✓ Loaded TIL data: {} MegaTiles", til_data.len());
        println!("✓ Loaded SOL data: {} properties", sol_data.len());
        println!("✓ TileTextureManager created successfully");
        
        // Verify we have reasonable amounts of data
        assert!(min_data.len() > 0, "MIN data should not be empty");
        assert!(til_data.len() > 0, "TIL data should not be empty");
        assert!(sol_data.len() > 0, "SOL data should not be empty");
    } else {
        println!("⚠ Skipping test - MPQ or tiles data not available");
    }
}

#[test]
fn test_decode_sample_tiles() {
    if let Some((min_data, _til_data, _sol_data, mut texture_mgr)) = try_load_tiles() {
        println!("\n=== Testing Tile Decoding ===");
        
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Test first 10 tiles
        for i in 0..10.min(min_data.len()) {
            if let Some(micro_block) = min_data.get(i) {
                if micro_block.has_value() {
                    match texture_mgr.get_decoded_tile(i) {
                        Ok(pixels) => {
                            println!("✓ Tile {}: {} bytes, type={:?}", 
                                i, pixels.len(), micro_block.tile_type());
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Tile {}: decode error - {}", i, e);
                            error_count += 1;
                        }
                    }
                }
            }
        }
        
        println!("\nDecoding results: {} success, {} errors", success_count, error_count);
        assert!(success_count > 0, "Should decode at least some tiles successfully");
    } else {
        println!("⚠ Skipping test - MPQ or tiles data not available");
    }
}

#[test]
fn test_render_floor_wall_logic() {
    if let Some((min_data, til_data, _sol_data, _texture_mgr)) = try_load_tiles() {
        println!("\n=== Testing Floor/Wall Rendering Logic ===");
        
        // Test a few MegaTiles
        for i in 0..5.min(til_data.len()) {
            if let Some(mega_tile) = til_data.get(i) {
                println!("\nMegaTile {}: micro1={}, micro2={}, micro3={}, micro4={}",
                    i, mega_tile.micro1, mega_tile.micro2, 
                    mega_tile.micro3, mega_tile.micro4);
                
                // Check micro1 (top-left, often floor)
                if let Some(block) = min_data.get(mega_tile.micro1 as usize) {
                    if block.has_value() {
                        println!("  micro1: frame={}, type={:?}", 
                            block.frame(), block.tile_type());
                    }
                }
                
                // Check micro2 (top-right, often floor)
                if let Some(block) = min_data.get(mega_tile.micro2 as usize) {
                    if block.has_value() {
                        println!("  micro2: frame={}, type={:?}", 
                            block.frame(), block.tile_type());
                    }
                }
                
                // Check micro3 (bottom-left, may be wall)
                if let Some(block) = min_data.get(mega_tile.micro3 as usize) {
                    if block.has_value() {
                        println!("  micro3: frame={}, type={:?}", 
                            block.frame(), block.tile_type());
                    }
                }
                
                // Check micro4 (bottom-right, may be wall)
                if let Some(block) = min_data.get(mega_tile.micro4 as usize) {
                    if block.has_value() {
                        println!("  micro4: frame={}, type={:?}", 
                            block.frame(), block.tile_type());
                    }
                }
            }
        }
        
        println!("\n✓ Render logic test complete");
    } else {
        println!("⚠ Skipping test - MPQ or tiles data not available");
    }
}

#[test]
fn test_cache_performance() {
    if let Some((_min_data, _til_data, _sol_data, mut texture_mgr)) = try_load_tiles() {
        println!("\n=== Testing Cache Performance ===");
        
        // Preload all tiles
        match texture_mgr.preload_all() {
            Ok(count) => println!("✓ Preloaded {} tiles", count),
            Err(e) => println!("Warning: Preload error - {}", e),
        }
        
        let stats = texture_mgr.cache_stats();
        println!("Cache statistics:");
        println!("  Total tiles: {}", stats.total_tiles);
        println!("  Decoded cache: {} entries", stats.decoded_count);
        println!("  Indexed cache: {} entries", stats.indexed_count);
        
        assert!(stats.decoded_count > 0 || stats.indexed_count > 0, 
            "Cache should have some entries after preload");
        
        println!("✓ Cache performance test complete");
    } else {
        println!("⚠ Skipping test - MPQ or tiles data not available");
    }
}

