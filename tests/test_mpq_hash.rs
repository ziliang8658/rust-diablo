/// 测试 MPQ 哈希算法和文件查找
/// 
/// 这个测试用于调试哈希算法和哈希表查找

use rust_diablo::resources::mpq_format::{initialize_crypt_table, hash_string, HashType};
use rust_diablo::resources::MpqManager;

#[test]
fn test_hash_various_formats() {
    initialize_crypt_table();
    
    let filenames = vec![
        "levels/towndata/town.pal",
        "LEVELS/TOWNDATA/TOWN.PAL",
        "levels\\towndata\\town.pal",
        "LEVELS\\TOWNDATA\\TOWN.PAL",
        "town.pal",
        "TOWN.PAL",
    ];
    
    println!("\n=== Hash Test ===");
    for filename in &filenames {
        let hash_a = hash_string(filename, HashType::NameA);
        let hash_b = hash_string(filename, HashType::NameB);
        let hash_offset = hash_string(filename, HashType::TableOffset);
        
        println!("File: '{}'", filename);
        println!("  hash_a    = 0x{:08X}", hash_a);
        println!("  hash_b    = 0x{:08X}", hash_b);
        println!("  offset    = 0x{:08X}", hash_offset);
        println!();
    }
}

#[test]
fn test_inspect_hash_table() {
    let mut manager = MpqManager::new();
    
    if manager.load_mpq("Diabdat.mpq", 1000).is_err() {
        println!("⚠ DIABDAT.MPQ not found, skipping test");
        return;
    }
    
    println!("\n=== Trying different file path formats ===");
    
    let test_paths = vec![
        "levels/towndata/town.pal",
        "LEVELS/TOWNDATA/TOWN.PAL",
        "levels\\towndata\\town.pal",
        "LEVELS\\TOWNDATA\\TOWN.PAL",
        "town.pal",
        "TOWN.PAL",
        // Try some other known files
        "(listfile)",
        "(attributes)",
        "ui_art\\title.pcx",
        "UI_ART\\TITLE.PCX",
    ];
    
    for path in &test_paths {
        let found = manager.has_file(path);
        println!("{}: {}", if found { "✓" } else { "✗" }, path);
    }
}

