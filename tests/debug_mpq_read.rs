/// Debug test to compare MPQ reading behavior
use rust_diablo::resources::{MpqManager, Palette};

#[test]
fn debug_mpq_palette_read() {
    println!("\n========================================");
    println!("DEBUG: Testing MPQ Palette Reading");
    println!("========================================\n");
    
    let mut mpq = MpqManager::new();
    
    // Load MPQ
    let mpq_paths = vec![
        "assets/Diabdat.mpq",
        "rust-diablo/assets/Diabdat.mpq",
    ];
    
    let mut loaded = false;
    for path in mpq_paths {
        println!("[STEP 1] Trying to load MPQ: {}", path);
        match mpq.load_mpq(path, 1000) {
            Ok(_) => {
                println!("[STEP 1] ✓ MPQ loaded successfully\n");
                loaded = true;
                break;
            }
            Err(e) => {
                println!("[STEP 1] ✗ Failed: {}\n", e);
            }
        }
    }
    
    if !loaded {
        println!("⚠ Skipping test: MPQ not found");
        return;
    }
    
    let test_file = "levels/towndata/town.pal";
    
    // Test 1: has_file
    println!("\n[STEP 2] Testing has_file('{}')...", test_file);
    let exists = mpq.has_file(test_file);
    println!("[STEP 2] Result: {}\n", if exists { "✓ EXISTS" } else { "✗ NOT FOUND" });
    
    // Test 2: find_file (raw data)
    println!("\n[STEP 3] Testing find_file('{}')...", test_file);
    match mpq.find_file(test_file) {
        Some(data) => {
            println!("[STEP 3] ✓ Data retrieved!");
            println!("[STEP 3]   Size: {} bytes", data.len());
            println!("[STEP 3]   First 3 bytes: {:?}", &data[0..3.min(data.len())]);
        }
        None => {
            println!("[STEP 3] ✗ find_file returned None");
        }
    }
    
    // Test 3: Palette::from_mpq
    println!("\n[STEP 4] Testing Palette::from_mpq('{}')...", test_file);
    match Palette::from_mpq(&mut mpq, test_file) {
        Ok(palette) => {
            println!("[STEP 4] ✓ Palette loaded!");
            println!("[STEP 4]   Colors: {}", palette.len());
            let color0 = palette.to_rgb(0);
            println!("[STEP 4]   Color 0: RGB({}, {}, {})", color0.r, color0.g, color0.b);
        }
        Err(e) => {
            println!("[STEP 4] ✗ Palette::from_mpq failed");
            println!("[STEP 4]   Error: {}", e);
            println!("[STEP 4]   Error chain:");
            let mut source = e.source();
            let mut level = 1;
            while let Some(err) = source {
                println!("[STEP 4]     Level {}: {}", level, err);
                source = err.source();
                level += 1;
            }
        }
    }
    
    println!("\n========================================");
    println!("DEBUG TEST COMPLETE");
    println!("========================================\n");
}

