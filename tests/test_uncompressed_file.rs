/// 测试未压缩文件的读取
use rust_diablo::resources::MpqManager;

#[test]
fn test_read_uncompressed_file() {
    let mut manager = MpqManager::new();

    if manager.load_mpq("Diabdat.mpq", 1000).is_err() {
        println!("⚠ DIABDAT.MPQ not found, skipping test");
        return;
    }

    println!("\n=== Testing Uncompressed Files ===");

    // 尝试一些可能未压缩的文件
    let test_files = vec!["(listfile)", "(attributes)", "diabdat.txt"];

    for filename in test_files {
        match manager.find_file(filename) {
            Some(data) => {
                println!("✓ Successfully read '{}': {} bytes", filename, data.len());
                if data.len() < 100 {
                    println!("  First bytes: {:02X?}", &data[..data.len().min(32)]);
                }
            }
            None => {
                println!("✗ Could not read '{}'", filename);
            }
        }
    }
}
