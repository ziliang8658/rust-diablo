/// 测试块表解密
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use rust_diablo::resources::mpq_format::{decrypt_block, hash_string, HashType, MPQ_SIGNATURE};

#[test]
fn test_block_table_decryption() {
    let mpq_path = "assets/Diabdat.mpq";

    let mut file = match File::open(mpq_path) {
        Ok(f) => f,
        Err(_) => {
            println!("⚠ DIABDAT.MPQ not found, skipping test");
            return;
        }
    };

    // 读取头部
    let mut header_bytes = vec![0u8; 32];
    file.read_exact(&mut header_bytes).unwrap();

    let block_table_offset = u32::from_le_bytes([
        header_bytes[20],
        header_bytes[21],
        header_bytes[22],
        header_bytes[23],
    ]);
    let block_table_entries = u32::from_le_bytes([
        header_bytes[28],
        header_bytes[29],
        header_bytes[30],
        header_bytes[31],
    ]);

    println!("\n=== MPQ Header ===");
    println!("Block Table Offset: 0x{:08X}", block_table_offset);
    println!("Block Table Entries: {}", block_table_entries);

    // 读取块表（加密）
    file.seek(SeekFrom::Start(block_table_offset as u64))
        .unwrap();
    let mut block_bytes = vec![0u8; 64]; // 4 entries = 64 bytes
    file.read_exact(&mut block_bytes).unwrap();

    println!("\n=== Encrypted Block Table (first entry, 16 bytes) ===");
    for (i, byte) in block_bytes[0..16].iter().enumerate() {
        if i % 4 == 0 && i != 0 {
            print!(" | ");
        }
        print!("{:02X} ", byte);
    }
    println!();

    // 转换为 u32 并解密
    let mut block_u32: Vec<u32> = block_bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    println!("\n=== Before Decryption ===");
    println!(
        "u32[0-3]: 0x{:08X} 0x{:08X} 0x{:08X} 0x{:08X}",
        block_u32[0], block_u32[1], block_u32[2], block_u32[3]
    );

    let key = hash_string("(block table)", HashType::FileKey);
    println!("\n=== Decryption Key ===");
    println!("Key: 0x{:08X}", key);

    decrypt_block(&mut block_u32, key);

    println!("\n=== After Decryption ===");
    println!(
        "u32[0-3]: 0x{:08X} 0x{:08X} 0x{:08X} 0x{:08X}",
        block_u32[0], block_u32[1], block_u32[2], block_u32[3]
    );

    // 解析为结构
    let file_offset = block_u32[0];
    let packed_size = block_u32[1];
    let unpacked_size = block_u32[2];
    let flags = block_u32[3];

    println!("\n=== First Block Entry ===");
    println!("file_offset: {} (0x{:08X})", file_offset, file_offset);
    println!("packed_size: {} bytes", packed_size);
    println!("unpacked_size: {} bytes", unpacked_size);
    println!("flags: 0x{:08X}", flags);

    // 检查标志
    if flags & 0x80000000 != 0 {
        println!("  ✓ EXISTS flag set");
    }
    if flags & 0x00000100 != 0 {
        println!("  ✓ COMPRESS_PKZIP flag set");
    }
    if flags & 0x00010000 != 0 {
        println!("  ✓ ENCRYPTED flag set");
    }

    // 检查合理性
    if file_offset < 100000000 && packed_size < 10000000 && unpacked_size < 10000000 {
        println!("\n  ✓ Block entry looks valid");
    } else {
        println!("\n  ✗ Block entry looks INVALID!");
    }
}
