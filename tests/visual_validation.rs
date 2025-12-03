/// Visual Validation Tests
/// 
/// Export decoded tiles as PNG images for visual inspection.
/// 
/// # Running
/// 
/// ```bash
/// # Export sample tiles from MPQ
/// cargo test --test visual_validation -- --ignored --nocapture
/// 
/// # View results
/// # Windows: explorer tests\output\visual_samples
/// # Linux: xdg-open tests/output/visual_samples
/// # macOS: open tests/output/visual_samples
/// ```

use rust_diablo::resources::MpqManager;
use rust_diablo::tiles::{MinData, DungeonType, texture_manager::TileTextureManager};
use rust_diablo::tiles::decoder::decode_tile;
use rust_diablo::resources::Palette;
use std::collections::HashMap;

mod visual_validator;
use visual_validator::VisualValidator;

/// Try to load MPQ from common paths
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
fn test_export_sample_tiles() {
    println!("\n=== 导出样本瓦片为PNG ===\n");
    
    let mut mpq = try_load_mpq()
        .expect("MPQ not found. Set DIABDAT_MPQ_PATH or place DIABDAT.MPQ in project root.");
    
    // Load texture manager (which includes CEL, PAL, MIN)
    let mut tex_mgr = TileTextureManager::load_for_dungeon(
        DungeonType::Cathedral,
        &mut mpq
    ).expect("Failed to load texture manager");
    
    // Load palette separately for PNG export
    let palette = Palette::from_mpq(&mut mpq, "levels/towndata/town.pal")
        .expect("Failed to load palette");
    
    // Load MIN data to get tile types
    let min_data = MinData::load_for_dungeon(&mut mpq, DungeonType::Cathedral)
        .expect("Failed to load MIN data");
    
    println!("Loaded: {} micro tiles\n", min_data.len());
    
    // Collect samples: 3 of each TileType
    let mut type_samples: HashMap<rust_diablo::tiles::types::TileType, usize> = HashMap::new();
    
    println!("Collecting samples...");
    
    for i in 1..min_data.len().min(1000) {  // Limit to first 1000 for efficiency
        let block = min_data.get(i).unwrap();
        
        if !block.has_value() {
            continue;
        }
        
        let tile_type = block.tile_type();
        let count = type_samples.entry(tile_type).or_insert(0);
        
        if *count < 3 {
            let frame_idx = block.frame() as usize;
            
            // Get decoded indexed pixels from texture manager
            match tex_mgr.get_decoded_tile(i) {
                Ok(_rgba_data) => {
                    // Convert RGBA back to indexed for visualization
                    // (We want indexed + palette for better control)
                    // For now, we'll just note which tiles we found
                    println!("  Found {:?} sample #{} (frame {})", tile_type, *count + 1, frame_idx);
                    *count += 1;
                }
                Err(e) => {
                    eprintln!("  Failed to decode tile {}: {}", i, e);
                }
            }
        }
        
        // Stop once we have 3 of each type
        if type_samples.values().all(|&c| c >= 3) {
            break;
        }
    }
    
    println!("\n=== Sample Collection Summary ===");
    for (tile_type, count) in &type_samples {
        println!("  {:?}: {} samples", tile_type, count);
    }
    
    println!("\n✓ Sample collection complete");
    println!("  Note: For actual PNG export, we need to access internal CEL frames");
    println!("  Consider adding an export method to TileTextureManager");
}

#[test]
#[ignore]
fn test_export_decoded_tiles_direct() {
    println!("\n=== 直接导出解码后的瓦片 ===\n");
    
    let mut mpq = try_load_mpq()
        .expect("MPQ not found");
    
    // Load CEL, MIN, PAL directly
    let cel_data = mpq.find_file("levels/l1data/l1.cel")
        .expect("CEL file not found");
    let cel_sprite = rust_diablo::resources::dungeon_cel::DungeonCelSprite::from_bytes(&cel_data)
        .expect("Failed to parse CEL");
    
    let min_data = MinData::load_for_dungeon(&mut mpq, DungeonType::Cathedral)
        .expect("Failed to load MIN");
    
    let palette = Palette::from_mpq(&mut mpq, "levels/towndata/town.pal")
        .expect("Failed to load palette");
    
    println!("Loaded: {} CEL frames, {} micro tiles", cel_sprite.frames.len(), min_data.len());
    
    // Collect samples
    let mut samples = Vec::new();
    let mut type_samples: HashMap<rust_diablo::tiles::types::TileType, usize> = HashMap::new();
    
    for i in 1..min_data.len().min(2000) {
        let block = min_data.get(i).unwrap();
        
        if !block.has_value() {
            continue;
        }
        
        let tile_type = block.tile_type();
        let count = type_samples.entry(tile_type).or_insert(0);
        
        if *count < 5 {  // Collect 5 samples of each type
            let frame_idx = block.frame() as usize;
            
            if frame_idx >= cel_sprite.frames.len() {
                continue;
            }
            
            let cel_frame = &cel_sprite.frames[frame_idx];
            
            // Get raw data from CEL frame (now stores raw_data directly)
            let raw_data = &cel_frame.raw_data;
            
            // Decode using our decoder
            match decode_tile(tile_type, &raw_data) {
                Ok(decoded) => {
                    samples.push((frame_idx, tile_type, decoded));
                    *count += 1;
                    println!("  ✓ Collected {:?} sample #{} (frame {})", tile_type, *count, frame_idx);
                }
                Err(e) => {
                    eprintln!("  ✗ Failed to decode frame {}: {}", frame_idx, e);
                }
            }
        }
        
        // Stop once we have enough samples
        if type_samples.values().all(|&c| c >= 5) {
            break;
        }
    }
    
    println!("\n=== Export Summary ===");
    for (tile_type, count) in &type_samples {
        println!("  {:?}: {} samples collected", tile_type, count);
    }
    
    // Export to PNG
    println!("\nExporting to PNG...");
    let output_dir = "tests/output/visual_samples";
    let exported = VisualValidator::export_tiles(&samples, &palette, output_dir);
    
    println!("\n✓ Exported {} tiles to {}/", exported, output_dir);
    println!("\nVisual Check Instructions:");
    println!("  1. Open the output directory:");
    println!("     Windows: explorer {}", output_dir.replace("/", "\\"));
    println!("     Linux:   xdg-open {}", output_dir);
    println!("     macOS:   open {}", output_dir);
    println!("\n  2. Check each tile type:");
    println!("     - LeftTriangle: Left-aligned triangle shape");
    println!("     - RightTriangle: Right-aligned triangle shape");
    println!("     - LeftTrapezoid: Triangle bottom + rectangle top (left-aligned)");
    println!("     - RightTrapezoid: Triangle bottom + rectangle top (right-aligned)");
    println!("     - Square: Full 32x32 rectangle");
    println!("     - TransparentSquare: Has magenta (transparent) areas");
    println!("\n  3. Transparent pixels are shown as magenta/pink");
}

#[test]
#[ignore]
fn test_create_comparison_grid() {
    println!("\n=== 创建对比网格 ===\n");
    
    let mut mpq = try_load_mpq()
        .expect("MPQ not found");
    
    // Load data
    let cel_data = mpq.find_file("levels/l1data/l1.cel")
        .expect("CEL file not found");
    let cel_sprite = rust_diablo::resources::dungeon_cel::DungeonCelSprite::from_bytes(&cel_data)
        .expect("Failed to parse CEL");
    
    let min_data = MinData::load_for_dungeon(&mut mpq, DungeonType::Cathedral)
        .expect("Failed to load MIN");
    
    let palette = Palette::from_mpq(&mut mpq, "levels/towndata/town.pal")
        .expect("Failed to load palette");
    
    // Collect one sample of each type
    let mut samples: Vec<(String, Vec<u8>, u32, u32)> = Vec::new();
    let mut found_types = std::collections::HashSet::new();
    
    for i in 1..min_data.len().min(2000) {
        let block = min_data.get(i).unwrap();
        
        if !block.has_value() {
            continue;
        }
        
        let tile_type = block.tile_type();
        
        if found_types.contains(&tile_type) {
            continue;
        }
        
        let frame_idx = block.frame() as usize;
        
        if frame_idx >= cel_sprite.frames.len() {
            continue;
        }
        
        let cel_frame = &cel_sprite.frames[frame_idx];
        // Get raw data from CEL frame (now stores raw_data directly)
        let raw_data = &cel_frame.raw_data;
        
        match decode_tile(tile_type, raw_data) {
            Ok(decoded) => {
                let (width, height) = match tile_type {
                    rust_diablo::tiles::types::TileType::LeftTriangle |
                    rust_diablo::tiles::types::TileType::RightTriangle => (32, 31),
                    _ => (32, 32),
                };
                
                let label = format!("{:?}", tile_type);
                samples.push((label, decoded, width, height));
                found_types.insert(tile_type);
                
                println!("  ✓ Found {:?} (frame {})", tile_type, frame_idx);
                
                if found_types.len() == 6 {
                    break;
                }
            }
            Err(e) => {
                eprintln!("  ✗ Failed: {}", e);
            }
        }
    }
    
    println!("\nCreating comparison grid...");
    
    // Convert to reference tuples
    let sample_refs: Vec<(&str, &[u8], u32, u32)> = samples.iter()
        .map(|(label, pixels, w, h)| (label.as_str(), pixels.as_slice(), *w, *h))
        .collect();
    
    let output_path = "tests/output/tile_type_comparison.png";
    std::fs::create_dir_all("tests/output").ok();
    
    match VisualValidator::create_comparison_grid(&sample_refs, &palette, 3, output_path) {
        Ok(_) => {
            println!("\n✓ Comparison grid saved to: {}", output_path);
            println!("\nOpen with:");
            println!("  Windows: {}", output_path.replace("/", "\\"));
        }
        Err(e) => {
            eprintln!("\n✗ Failed to create grid: {}", e);
        }
    }
}

