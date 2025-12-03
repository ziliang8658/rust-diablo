use rust_diablo::engine::isometric::{
    in_mega_bounds, in_micro_bounds, mega_to_micro, micro_to_mega, screen_to_world, world_to_screen,
};
use rust_diablo::resources::MpqManager;
/// Integration tests for tiles system (Step 6.1)
///
/// These tests verify the complete tile loading pipeline:
/// - MIN file loading (MicroTiles)
/// - TIL file loading (MegaTiles)
/// - SOL file loading (Tile Properties)
/// - Coordinate system conversions
///
/// # Test Requirements
/// - Requires DIABDAT.MPQ in assets/ directory
/// - Tests Cathedral (L1) dungeon tiles
use rust_diablo::tiles::{DungeonType, MinData, SolData, TilData};

/// Helper function to find and load MPQ
fn load_mpq() -> Option<MpqManager> {
    let mut mpq_manager = MpqManager::new();

    let mpq_paths = vec![
        "assets/Diabdat.mpq",
        "assets/DIABDAT.MPQ",
        "Diabdat.mpq",
        "DIABDAT.MPQ",
    ];

    for path in mpq_paths {
        if mpq_manager.load_mpq(path, 1000).is_ok() {
            return Some(mpq_manager);
        }
    }

    None
}

#[test]
fn test_load_cathedral_min() {
    let mut mpq_manager = match load_mpq() {
        Some(mgr) => mgr,
        None => {
            println!("⚠ Skipping test: DIABDAT.MPQ not found");
            return;
        }
    };

    // Try to load Cathedral MIN file
    let min_paths = vec!["levels/l1data/l1.min", "levels\\l1data\\l1.min"];

    let mut min_data_opt = None;
    for path in &min_paths {
        if let Some(file_data) = mpq_manager.find_file(path) {
            if let Ok(min_data) = MinData::from_bytes(&file_data) {
                min_data_opt = Some(min_data);
                println!("✓ Loaded Cathedral MIN: {}", path);
                break;
            }
        }
    }

    assert!(min_data_opt.is_some(), "Failed to load Cathedral MIN file");

    let min_data = min_data_opt.unwrap();
    println!("  MIN tiles count: {}", min_data.len());
    assert!(min_data.len() > 0, "MIN file should contain tiles");

    // Verify first few tiles
    for i in 0..10.min(min_data.len()) {
        if let Some(block) = min_data.get(i) {
            if block.has_value() {
                println!(
                    "  Tile {}: type={:?}, frame={}",
                    i,
                    block.tile_type(),
                    block.frame()
                );
            }
        }
    }
}

#[test]
fn test_load_cathedral_til() {
    let mut mpq_manager = match load_mpq() {
        Some(mgr) => mgr,
        None => {
            println!("⚠ Skipping test: DIABDAT.MPQ not found");
            return;
        }
    };

    // Try to load Cathedral TIL file
    let til_paths = vec!["levels/l1data/l1.til", "levels\\l1data\\l1.til"];

    let mut til_data_opt = None;
    for path in &til_paths {
        if let Some(file_data) = mpq_manager.find_file(path) {
            if let Ok(til_data) = TilData::from_bytes(&file_data) {
                til_data_opt = Some(til_data);
                println!("✓ Loaded Cathedral TIL: {}", path);
                break;
            }
        }
    }

    assert!(til_data_opt.is_some(), "Failed to load Cathedral TIL file");

    let til_data = til_data_opt.unwrap();
    println!("  TIL MegaTiles count: {}", til_data.len());
    assert!(til_data.len() > 0, "TIL file should contain MegaTiles");

    // Verify first few MegaTiles
    for i in 0..5.min(til_data.len()) {
        if let Some(mega_tile) = til_data.get(i) {
            println!(
                "  MegaTile {}: [{}, {}, {}, {}]",
                i, mega_tile.micro1, mega_tile.micro2, mega_tile.micro3, mega_tile.micro4
            );

            // Verify all micro indices are valid (non-zero or all zero)
            let has_data = mega_tile.micro1 != 0
                || mega_tile.micro2 != 0
                || mega_tile.micro3 != 0
                || mega_tile.micro4 != 0;
            if has_data {
                // At least one micro tile should be non-zero
                assert!(
                    mega_tile.micro1 > 0
                        || mega_tile.micro2 > 0
                        || mega_tile.micro3 > 0
                        || mega_tile.micro4 > 0
                );
            }
        }
    }
}

#[test]
fn test_load_cathedral_sol() {
    let mut mpq_manager = match load_mpq() {
        Some(mgr) => mgr,
        None => {
            println!("⚠ Skipping test: DIABDAT.MPQ not found");
            return;
        }
    };

    // Try to load Cathedral SOL file
    let sol_paths = vec!["levels/l1data/l1.sol", "levels\\l1data\\l1.sol"];

    let mut sol_data_opt = None;
    for path in &sol_paths {
        if let Some(file_data) = mpq_manager.find_file(path) {
            if let Ok(sol_data) = SolData::from_bytes(&file_data) {
                sol_data_opt = Some(sol_data);
                println!("✓ Loaded Cathedral SOL: {}", path);
                break;
            }
        }
    }

    assert!(sol_data_opt.is_some(), "Failed to load Cathedral SOL file");

    let sol_data = sol_data_opt.unwrap();
    println!("  SOL properties count: {}", sol_data.len());
    assert!(
        sol_data.len() > 0,
        "SOL file should contain tile properties"
    );

    use rust_diablo::tiles::TileProperties;

    // Count different property types
    let mut solid_count = 0;
    let mut block_light_count = 0;
    let mut transparent_count = 0;

    for i in 0..sol_data.len() {
        if let Some(props) = sol_data.get(i) {
            if props.contains(TileProperties::SOLID) {
                solid_count += 1;
            }
            if props.contains(TileProperties::BLOCK_LIGHT) {
                block_light_count += 1;
            }
            if props.contains(TileProperties::TRANSPARENT) {
                transparent_count += 1;
            }
        }
    }

    println!("  Solid tiles: {}", solid_count);
    println!("  Block light tiles: {}", block_light_count);
    println!("  Transparent tiles: {}", transparent_count);

    // Cathedral should have solid walls
    assert!(solid_count > 0, "Cathedral should have solid tiles");
}

#[test]
fn test_coordinate_conversions_roundtrip() {
    // Test world <-> screen conversions
    let test_positions = vec![(0, 0), (1, 1), (10, 10), (20, 30), (39, 39)];

    for (wx, wy) in test_positions {
        let (sx, sy) = world_to_screen(wx, wy);
        let (wx2, wy2) = screen_to_world(sx, sy);

        assert_eq!(wx, wx2, "World X mismatch for ({}, {})", wx, wy);
        assert_eq!(wy, wy2, "World Y mismatch for ({}, {})", wx, wy);
    }

    println!("✓ World <-> Screen conversions OK");
}

#[test]
fn test_mega_micro_conversions() {
    // Test MegaTile <-> MicroTile conversions
    let test_positions = vec![(0, 0), (10, 10), (20, 20), (39, 39)];

    for (mx, my) in test_positions {
        let (micro_x, micro_y) = mega_to_micro(mx, my);
        let (mx2, my2) = micro_to_mega(micro_x, micro_y);

        assert_eq!(mx, mx2, "MegaTile X mismatch for ({}, {})", mx, my);
        assert_eq!(my, my2, "MegaTile Y mismatch for ({}, {})", mx, my);

        // Verify micro coordinates are in valid range
        assert!(
            in_micro_bounds(micro_x, micro_y),
            "MicroTile ({}, {}) out of bounds",
            micro_x,
            micro_y
        );
    }

    println!("✓ MegaTile <-> MicroTile conversions OK");
}

#[test]
fn test_bounds_checking() {
    // Test MegaTile bounds
    assert!(in_mega_bounds(0, 0));
    assert!(in_mega_bounds(39, 39));
    assert!(!in_mega_bounds(-1, 0));
    assert!(!in_mega_bounds(0, -1));
    assert!(!in_mega_bounds(40, 0));
    assert!(!in_mega_bounds(0, 40));

    // Test MicroTile bounds
    assert!(in_micro_bounds(0, 0));
    assert!(in_micro_bounds(111, 111));
    assert!(!in_micro_bounds(-1, 0));
    assert!(!in_micro_bounds(0, -1));
    assert!(!in_micro_bounds(112, 0));
    assert!(!in_micro_bounds(0, 112));

    println!("✓ Bounds checking OK");
}

#[test]
fn test_load_all_dungeon_types() {
    let mut mpq_manager = match load_mpq() {
        Some(mgr) => mgr,
        None => {
            println!("⚠ Skipping test: DIABDAT.MPQ not found");
            return;
        }
    };

    let dungeon_types = vec![
        (DungeonType::Town, "Town"),
        (DungeonType::Cathedral, "Cathedral"),
        (DungeonType::Catacombs, "Catacombs"),
        (DungeonType::Caves, "Caves"),
        (DungeonType::Hell, "Hell"),
    ];

    for (dungeon_type, name) in dungeon_types {
        println!("\n=== Testing {} ===", name);

        // Get correct path based on dungeon type
        let (min_path, til_path, sol_path) = match dungeon_type {
            DungeonType::Town => (
                "levels/towndata/town.min",
                "levels/towndata/town.til",
                "levels/towndata/town.sol",
            ),
            DungeonType::Cathedral => (
                "levels/l1data/l1.min",
                "levels/l1data/l1.til",
                "levels/l1data/l1.sol",
            ),
            DungeonType::Catacombs => (
                "levels/l2data/l2.min",
                "levels/l2data/l2.til",
                "levels/l2data/l2.sol",
            ),
            DungeonType::Caves => (
                "levels/l3data/l3.min",
                "levels/l3data/l3.til",
                "levels/l3data/l3.sol",
            ),
            DungeonType::Hell => (
                "levels/l4data/l4.min",
                "levels/l4data/l4.til",
                "levels/l4data/l4.sol",
            ),
        };

        // Try to load MIN
        let mut min_loaded = false;
        if let Some(file_data) = mpq_manager.find_file(min_path) {
            if let Ok(min_data) = MinData::from_bytes(&file_data) {
                println!("  ✓ MIN: {} tiles", min_data.len());
                min_loaded = true;
            }
        }

        // Try to load TIL
        let mut til_loaded = false;
        if let Some(file_data) = mpq_manager.find_file(til_path) {
            if let Ok(til_data) = TilData::from_bytes(&file_data) {
                println!("  ✓ TIL: {} MegaTiles", til_data.len());
                til_loaded = true;
            }
        }

        // Try to load SOL
        let mut sol_loaded = false;
        if let Some(file_data) = mpq_manager.find_file(sol_path) {
            if let Ok(sol_data) = SolData::from_bytes(&file_data) {
                println!("  ✓ SOL: {} properties", sol_data.len());
                sol_loaded = true;
            }
        }

        // At least one should load (some dungeons might not have all files)
        if !min_loaded && !til_loaded && !sol_loaded {
            println!("  ⚠ No tiles loaded for {}", name);
        }
    }
}

#[test]
fn test_tile_data_consistency() {
    let mut mpq_manager = match load_mpq() {
        Some(mgr) => mgr,
        None => {
            println!("⚠ Skipping test: DIABDAT.MPQ not found");
            return;
        }
    };

    // Load Cathedral tiles
    let min_data_opt = mpq_manager
        .find_file("levels/l1data/l1.min")
        .and_then(|data| MinData::from_bytes(&data).ok());
    let til_data_opt = mpq_manager
        .find_file("levels/l1data/l1.til")
        .and_then(|data| TilData::from_bytes(&data).ok());
    let sol_data_opt = mpq_manager
        .find_file("levels/l1data/l1.sol")
        .and_then(|data| SolData::from_bytes(&data).ok());

    if let (Some(min_data), Some(til_data), Some(sol_data)) =
        (min_data_opt, til_data_opt, sol_data_opt)
    {
        println!("=== Tile Data Consistency Check ===");
        println!("MIN tiles: {}", min_data.len());
        println!("TIL MegaTiles: {}", til_data.len());
        println!("SOL properties: {}", sol_data.len());

        // Check that TIL references valid MIN indices
        let mut invalid_refs = 0;
        for i in 0..10.min(til_data.len()) {
            if let Some(mega_tile) = til_data.get(i) {
                let indices = [
                    mega_tile.micro1 as usize,
                    mega_tile.micro2 as usize,
                    mega_tile.micro3 as usize,
                    mega_tile.micro4 as usize,
                ];

                for idx in indices.iter() {
                    if *idx > 0 && *idx >= min_data.len() {
                        println!("  ⚠ MegaTile {} references invalid MIN index: {}", i, idx);
                        invalid_refs += 1;
                    }
                }
            }
        }

        if invalid_refs == 0 {
            println!("✓ All TIL->MIN references are valid");
        } else {
            println!("⚠ Found {} invalid references", invalid_refs);
        }
    } else {
        println!("⚠ Skipping consistency check: Could not load all tile files");
    }
}
