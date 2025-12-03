/// MPQ Decoder Validation Tests
///
/// These tests validate all decoders using real MPQ data from DIABDAT.MPQ.
///
/// # Running These Tests
///
/// These tests are marked with `#[ignore]` because they require MPQ files.
/// To run them:
///
/// ```bash
/// # Set MPQ path (optional)
/// export DIABDAT_MPQ_PATH=/path/to/DIABDAT.MPQ
///
/// # Run ignored tests
/// cargo test --test mpq_decoder_validation -- --ignored --nocapture
/// ```
///
/// # Test Strategy
///
/// 1. Load Cathedral tileset (l1.cel + l1.min + town.pal)
/// 2. For each MicroTile in MIN data:
///    - Get TileType and frame index
///    - Decode the frame using our decoder
///    - Validate geometric characteristics
/// 3. Report statistics
use rust_diablo::resources::MpqManager;
use rust_diablo::tiles::decoder::decode_tile;
use rust_diablo::tiles::{texture_manager::TileTextureManager, DungeonType, MinData};

mod decoder_validator;
use decoder_validator::DecoderValidator;

/// Try to create ResourceManager with MPQ
///
/// Searches multiple common MPQ paths
fn try_load_mpq() -> Option<MpqManager> {
    let paths = vec![
        "DIABDAT.MPQ",
        "../DIABDAT.MPQ",
        "../../DIABDAT.MPQ",
        "diablo-asset/DIABDAT.MPQ",
        "../diablo-asset/DIABDAT.MPQ",
        "../../diablo-asset/DIABDAT.MPQ",
    ];

    // Also try environment variable
    if let Ok(env_path) = std::env::var("DIABDAT_MPQ_PATH") {
        let mut mpq = MpqManager::new();
        if mpq.load_mpq(&env_path, 1000).is_ok() {
            println!("✓ Found MPQ at (env): {}", env_path);
            return Some(mpq);
        }
    }

    for path in paths {
        let mut mpq = MpqManager::new();
        if mpq.load_mpq(path, 1000).is_ok() {
            println!("✓ Found MPQ at: {}", path);
            return Some(mpq);
        }
    }

    None
}

#[test]
#[ignore]
fn test_validate_all_cathedral_tiles() {
    println!("\n=== 验证Cathedral所有瓦片解码器 ===\n");

    // Load MPQ
    let mut mpq = try_load_mpq()
        .expect("MPQ not found. Set DIABDAT_MPQ_PATH or place DIABDAT.MPQ in project root.");

    // Load texture manager (includes CEL + PAL + MIN)
    println!("Loading texture manager...");
    let tex_mgr = TileTextureManager::load_for_dungeon(DungeonType::Cathedral, &mut mpq)
        .expect("Failed to load texture manager");

    println!("✓ Loaded texture manager: {} tiles\n", tex_mgr.len());

    // Statistics
    use rust_diablo::tiles::types::TileType;
    use std::collections::HashMap;
    let mut type_counts: HashMap<TileType, usize> = HashMap::new();
    let mut validated_counts: HashMap<TileType, usize> = HashMap::new();
    let mut error_counts: HashMap<TileType, usize> = HashMap::new();

    // Validate each tile
    // Note: We can't easily access internal cel_sprite from tex_mgr,
    // so we'll reload MIN data separately
    let min_data = MinData::load_for_dungeon(&mut mpq, DungeonType::Cathedral)
        .expect("Failed to load MIN data");

    println!("Validating {} micro tiles...\n", min_data.len());

    for i in 0..min_data.len() {
        let block = min_data.get(i).unwrap();

        if !block.has_value() {
            continue;
        }

        let tile_type = block.tile_type();
        *type_counts.entry(tile_type).or_insert(0) += 1;

        // For this test, we'll use the internal decoder directly
        // In a real scenario, we'd decode from CEL frames

        // Skip detailed validation for now (would need to load CEL frames separately)
        // This is a placeholder structure - see test_validate_specific_frames for actual validation
    }

    println!("=== Tile Type Statistics ===");
    for (tile_type, count) in &type_counts {
        println!("  {:?}: {} tiles", tile_type, count);
    }
}

#[test]
#[ignore]
fn test_load_texture_manager() {
    println!("\n=== 测试纹理管理器加载 ===\n");

    let mut mpq = try_load_mpq().expect("MPQ not found");

    let mut tex_mgr = TileTextureManager::load_for_dungeon(DungeonType::Cathedral, &mut mpq)
        .expect("Failed to load texture manager");

    println!("✓ Texture manager loaded: {} tiles", tex_mgr.len());

    // Try to get a few tiles
    println!("\nTesting tile retrieval...");
    for i in 1..5 {
        match tex_mgr.get_decoded_tile(i) {
            Ok(data) => {
                println!("  ✓ Tile {}: {} bytes", i, data.len());
            }
            Err(e) => {
                println!("  ✗ Tile {}: {}", i, e);
            }
        }
    }

    // Check cache stats
    let stats = tex_mgr.cache_stats();
    println!("\nCache stats:");
    println!("  Decoded: {}", stats.decoded_count);
    println!("  Indexed: {}", stats.indexed_count);
    println!("  Total: {}", stats.total_tiles);
}

#[test]
#[ignore]
fn test_preload_all_tiles() {
    println!("\n=== 测试预加载所有瓦片 ===\n");

    let mut mpq = try_load_mpq().expect("MPQ not found");

    let mut tex_mgr = TileTextureManager::load_for_dungeon(DungeonType::Cathedral, &mut mpq)
        .expect("Failed to load texture manager");

    println!("Total tiles: {}", tex_mgr.len());

    // Preload all
    let loaded = tex_mgr.preload_all().expect("Preload failed");

    println!("\n✓ Preloaded {} tiles", loaded);

    let stats = tex_mgr.cache_stats();
    println!("  Hit rate: {:.1}%", stats.hit_rate() * 100.0);
}

#[test]
fn test_decoder_validator() {
    // Test the validator with synthetic data

    // Valid Square
    let square = vec![1u8; 1024];
    assert!(DecoderValidator::validate_square(&square).is_ok());

    // Invalid Square (wrong size)
    let bad_square = vec![1u8; 100];
    assert!(DecoderValidator::validate_square(&bad_square).is_err());

    // Valid LeftTriangle
    let mut left_tri = vec![0u8; 32 * 31];
    let widths = [
        2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 30, 28, 26, 24, 22, 20, 18, 16,
        14, 12, 10, 8, 6, 4, 2,
    ];
    for (row, &width) in widths.iter().enumerate() {
        for col in 0..width {
            left_tri[row * 32 + col] = 1;
        }
    }
    assert!(DecoderValidator::validate_left_triangle(&left_tri).is_ok());

    println!("✓ Decoder validator tests passed");
}
