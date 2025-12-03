/// List player graphics files in MPQ

use rust_diablo::resources::MpqManager;

fn main() -> anyhow::Result<()> {
    println!("=== Searching for player graphics in MPQ ===\n");
    
    let mut mpq = MpqManager::new();
    
    // Load DIABDAT.MPQ
    let mpq_paths = vec![
        "assets/Diabdat.mpq",
        "../assets/Diabdat.mpq",
    ];
    
    let mut loaded = false;
    for path in &mpq_paths {
        match mpq.load_mpq(path, 1000) {
            Ok(_) => {
                println!("✓ Loaded MPQ: {}\n", path);
                loaded = true;
                break;
            }
            Err(_) => continue,
        }
    }
    
    if !loaded {
        eprintln!("Failed to load MPQ archive");
        return Ok(());
    }
    
    // Try to find player graphics
    let test_paths = vec![
        // Warrior
        "plrgfx/warrior/wmn/wmnas.clx",
        "plrgfx/warrior/wmd/wmdas.clx",
        "plrgfx/warrior/wln/wlnas.clx",
        // Alternative paths
        "plrgfx\\warrior\\wmn\\wmnas.clx",
        "plrgfx\\warrior\\wmd\\wmdas.clx",
    ];
    
    println!("Testing specific paths:");
    for path in &test_paths {
        if let Some(data) = mpq.find_file(path) {
            println!("  ✓ Found: {} ({} bytes)", path, data.len());
        } else {
            println!("  ✗ Not found: {}", path);
        }
    }
    
    Ok(())
}
















