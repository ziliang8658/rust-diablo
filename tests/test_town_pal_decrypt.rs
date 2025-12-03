/// 测试 town.pal 的解密
///
/// 目的: 验证 town.pal 文件的解密是否正确
use rust_diablo::resources::mpq::MpqManager;

#[test]
fn test_town_pal_decryption() {
    let mut manager = MpqManager::new();

    println!("\n=== 测试 town.pal 解密 ===\n");

    // 加载 MPQ
    manager
        .load_mpq("Diabdat.mpq", 1000)
        .expect("Failed to load Diabdat.mpq");

    // 尝试读取 town.pal
    match manager.find_file("levels/towndata/town.pal") {
        Some(data) => {
            println!("✔ 成功读取 town.pal，共 {} 字节", data.len());
            println!("\n前 32 字节:");
            for i in 0..32.min(data.len()) {
                if i % 16 == 0 {
                    print!("{:04X}: ", i);
                }
                print!("{:02X} ", data[i]);
                if (i + 1) % 16 == 0 {
                    println!();
                }
            }
            println!();

            // 检查第一个字节（应该是压缩标志）
            if !data.is_empty() {
                let compression_flags = data[0];
                println!("\n压缩标志: 0x{:02X}", compression_flags);
                println!(
                    "  Huffman (0x01): {}",
                    if compression_flags & 0x01 != 0 {
                        "是"
                    } else {
                        "否"
                    }
                );
                println!(
                    "  Zlib    (0x02): {}",
                    if compression_flags & 0x02 != 0 {
                        "是"
                    } else {
                        "否"
                    }
                );
                println!(
                    "  PKWare  (0x08): {}",
                    if compression_flags & 0x08 != 0 {
                        "是"
                    } else {
                        "否"
                    }
                );
                println!(
                    "  BZip2   (0x10): {}",
                    if compression_flags & 0x10 != 0 {
                        "是"
                    } else {
                        "否"
                    }
                );
                println!(
                    "  未知     (0x20): {}",
                    if compression_flags & 0x20 != 0 {
                        "⚠️  是 (异常！)"
                    } else {
                        "否"
                    }
                );
                println!(
                    "  WaveMono (0x40): {}",
                    if compression_flags & 0x40 != 0 {
                        "是"
                    } else {
                        "否"
                    }
                );
                println!(
                    "  WaveStereo (0x80): {}",
                    if compression_flags & 0x80 != 0 {
                        "是"
                    } else {
                        "否"
                    }
                );
            }
        }
        None => {
            panic!("❌ 无法读取 town.pal");
        }
    }
}
