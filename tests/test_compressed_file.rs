/// 测试压缩文件的数据格式
use rust_diablo::resources::MpqManager;

#[test]
fn test_compressed_file_format() {
    let mut manager = MpqManager::new();
    
    if manager.load_mpq("Diabdat.mpq", 1000).is_err() {
        println!("⚠ DIABDAT.MPQ not found, skipping test");
        return;
    }
    
    // 直接读取原始数据（包括错误）
    match manager.find_file("levels/towndata/town.pal") {
        Some(data) => {
            println!("\n=== Successfully read town.pal ===");
            println!("Data size: {} bytes", data.len());
            println!("First 16 bytes: {:02X?}", &data[..16.min(data.len())]);
            
            // 检查第一个字节（压缩类型）
            if !data.is_empty() {
                let compression_type = data[0];
                println!("\nCompression type byte: 0x{:02X}", compression_type);
                
                if compression_type & 0x08 != 0 {
                    println!("  ✓ PKWare compression flag set");
                }
                if compression_type & 0x02 != 0 {
                    println!("  ✓ Zlib compression flag set");
                }
                if compression_type & 0x10 != 0 {
                    println!("  ✓ BZip2 compression flag set");
                }
            }
        }
        None => {
            println!("✗ Could not read town.pal");
        }
    }
}


























