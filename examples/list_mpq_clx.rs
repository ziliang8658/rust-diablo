/// List all CLX files in the MPQ archive

use rust_diablo::resources::MpqManager;

fn main() -> anyhow::Result<()> {
    println!("=== Listing CLX files in MPQ ===");
    
    let mut mpq = MpqManager::new();
    
    // Try to load DIABDAT.MPQ
    let mpq_paths = vec![
        "assets/Diabdat.mpq",
        "assets/DIABDAT.MPQ",
        "Diabdat.mpq",
        "DIABDAT.MPQ",
    ];
    
    let mut loaded = false;
    for path in &mpq_paths {
        match mpq.load_mpq(path, 1000) {
            Ok(_) => {
                println!("✓ Loaded MPQ: {}", path);
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
    
    // Get list of files
    let files = mpq.list_files();
    
    // Filter and display CLX files
    let clx_files: Vec<_> = files.iter()
        .filter(|f| f.to_lowercase().ends_with(".clx"))
        .collect();
    
    println!("\nFound {} CLX files:", clx_files.len());
    for (i, file) in clx_files.iter().enumerate() {
        println!("  {}: {}", i + 1, file);
        
        // Show warrior files
        if file.to_lowercase().contains("warrior") || file.to_lowercase().contains("plrgfx") {
            println!("    ^ Potential warrior sprite");
        }
    }
    
    Ok(())
}
















