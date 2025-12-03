/// Find player CL2 files in MPQ
use rust_diablo::resources::MpqManager;

fn main() -> anyhow::Result<()> {
    println!("=== Searching for player CL2 files in MPQ ===\n");

    let mut mpq = MpqManager::new();

    // Load DIABDAT.MPQ
    match mpq.load_mpq("assets/Diabdat.mpq", 1000) {
        Ok(_) => println!("✓ Loaded MPQ\n"),
        Err(e) => {
            eprintln!("Failed to load MPQ: {}", e);
            return Ok(());
        }
    }

    // Try to find CL2 files (original format before CLX conversion)
    let test_paths = vec![
        // CL2 versions (original format)
        "plrgfx/warrior/wmn/wmnas.cl2",
        "plrgfx/warrior/wmd/wmdas.cl2",
        "plrgfx\\warrior\\wmn\\wmnas.cl2",
        "plrgfx\\warrior\\wmd\\wmdas.cl2",
        "plrgfx/warrior/wmu/wmuas.cl2",
        "plrgfx\\warrior\\wmu\\wmuas.cl2",
    ];

    println!("Testing CL2 paths:");
    for path in &test_paths {
        if let Some(data) = mpq.find_file(path) {
            println!("  ✓ Found: {} ({} bytes)", path, data.len());
        } else {
            println!("  ✗ Not found: {}", path);
        }
    }

    Ok(())
}
