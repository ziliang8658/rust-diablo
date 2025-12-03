/// 测试 MPQ 中的不同压缩类型
use rust_diablo::resources::MpqManager;

#[test]
fn test_scan_compression_types() {
    let mut manager = MpqManager::new();

    if manager.load_mpq("Diabdat.mpq", 1000).is_err() {
        println!("⚠ DIABDAT.MPQ not found, skipping test");
        return;
    }

    println!("\n=== Scanning Compression Types ===");

    // 测试几个文件，看看它们的压缩类型
    let test_files = vec![
        "levels/towndata/town.pal",
        "ui_art/title.pcx",
        "ctrlpan/golddrop.cel",
    ];

    for filename in test_files {
        print!("Testing '{}': ", filename);
        match manager.find_file(filename) {
            Some(data) => {
                println!("✓ Read successfully ({} bytes)", data.len());
            }
            None => {
                println!("✗ Failed to read");
            }
        }
    }
}
