/// 简单的 MPQ 测试 - 不使用 packed 结构体
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use rust_diablo::resources::mpq_format::{decrypt_block, hash_string, HashType, MPQ_SIGNATURE};

#[test]
fn test_simple_mpq_read() {
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

    let signature = u32::from_le_bytes([
        header_bytes[0],
        header_bytes[1],
        header_bytes[2],
        header_bytes[3],
    ]);
    let hash_table_offset = u32::from_le_bytes([
        header_bytes[16],
        header_bytes[17],
        header_bytes[18],
        header_bytes[19],
    ]);
    let hash_table_entries = u32::from_le_bytes([
        header_bytes[24],
        header_bytes[25],
        header_bytes[26],
        header_bytes[27],
    ]);

    println!("\n=== MPQ Header ===");
    println!(
        "Signature: 0x{:08X} (expected: 0x{:08X})",
        signature, MPQ_SIGNATURE
    );
    println!("Hash Table Offset: 0x{:08X}", hash_table_offset);
    println!("Hash Table Entries: {}", hash_table_entries);

    assert_eq!(signature, MPQ_SIGNATURE);

    // 读取哈希表（加密）
    file.seek(SeekFrom::Start(hash_table_offset as u64))
        .unwrap();
    let mut hash_bytes = vec![0u8; 64]; // 4 entries = 64 bytes
    file.read_exact(&mut hash_bytes).unwrap();

    println!("\n=== Encrypted Hash Table (first entry, 16 bytes) ===");
    for (i, byte) in hash_bytes[0..16].iter().enumerate() {
        if i % 4 == 0 && i != 0 {
            print!(" | ");
        }
        print!("{:02X} ", byte);
    }
    println!();

    // 转换为 u32 并解密
    let mut hash_u32: Vec<u32> = hash_bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    println!("\n=== Before Decryption ===");
    println!(
        "u32[0-3]: 0x{:08X} 0x{:08X} 0x{:08X} 0x{:08X}",
        hash_u32[0], hash_u32[1], hash_u32[2], hash_u32[3]
    );

    let key = hash_string("(hash table)", HashType::FileKey);
    println!("\n=== Decryption Key ===");
    println!("Key: 0x{:08X}", key);

    decrypt_block(&mut hash_u32, key);

    println!("\n=== After Decryption ===");
    println!(
        "u32[0-3]: 0x{:08X} 0x{:08X} 0x{:08X} 0x{:08X}",
        hash_u32[0], hash_u32[1], hash_u32[2], hash_u32[3]
    );

    // 解析为结构
    let name_a = hash_u32[0];
    let name_b = hash_u32[1];
    let locale_platform = hash_u32[2];
    let block_index = hash_u32[3];

    println!("\n=== First Hash Entry ===");
    println!("name_a: 0x{:08X}", name_a);
    println!("name_b: 0x{:08X}", name_b);
    println!("locale+platform: 0x{:08X}", locale_platform);
    println!("block_index: {} (0x{:08X})", block_index, block_index);

    if block_index < hash_table_entries {
        println!("  ✓ Valid block index (< {})", hash_table_entries);
    } else if block_index == 0xFFFFFFFF {
        println!("  ✓ Empty entry");
    } else if block_index == 0xFFFFFFFE {
        println!("  ✓ Deleted entry");
    } else {
        println!(
            "  ✗ INVALID block index! (should be < {} or 0xFFFFFFFF/0xFFFFFFFE)",
            hash_table_entries
        );
    }
}
