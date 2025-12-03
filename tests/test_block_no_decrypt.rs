/// 测试块表是否真的需要解密
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[test]
fn test_block_table_raw() {
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
    
    let block_table_offset = u32::from_le_bytes([header_bytes[20], header_bytes[21], header_bytes[22], header_bytes[23]]);
    
    println!("\n=== Reading Raw Block Table (No Decryption) ===");
    println!("Block Table Offset: 0x{:08X}", block_table_offset);
    
    // 读取前3个块表条目
    file.seek(SeekFrom::Start(block_table_offset as u64)).unwrap();
    
    for i in 0..3 {
        let mut block_bytes = vec![0u8; 16];
        file.read_exact(&mut block_bytes).unwrap();
        
        let file_offset = u32::from_le_bytes([block_bytes[0], block_bytes[1], block_bytes[2], block_bytes[3]]);
        let packed_size = u32::from_le_bytes([block_bytes[4], block_bytes[5], block_bytes[6], block_bytes[7]]);
        let unpacked_size = u32::from_le_bytes([block_bytes[8], block_bytes[9], block_bytes[10], block_bytes[11]]);
        let flags = u32::from_le_bytes([block_bytes[12], block_bytes[13], block_bytes[14], block_bytes[15]]);
        
        println!("\n=== Block Entry {} (RAW - NO DECRYPTION) ===", i);
        println!("file_offset:   {} (0x{:08X})", file_offset, file_offset);
        println!("packed_size:   {} bytes", packed_size);
        println!("unpacked_size: {} bytes", unpacked_size);
        println!("flags:         0x{:08X}", flags);
        
        // 检查标志
        if flags & 0x80000000 != 0 {
            println!("  ✓ EXISTS flag (0x80000000)");
        }
        if flags & 0x00000100 != 0 {
            println!("  ✓ COMPRESS_PKZIP flag (0x00000100)");
        }
        if flags & 0x00010000 != 0 {
            println!("  ✓ ENCRYPTED flag (0x00010000)");
        }
        
        // 检查合理性
        let is_valid = file_offset < 100000000 
            && packed_size < 10000000 
            && unpacked_size < 10000000
            && (flags & 0x80000000 != 0); // EXISTS flag必须设置
        
        if is_valid {
            println!("  ✓ Entry looks VALID (reasonable values)");
        } else {
            println!("  ✗ Entry looks INVALID");
        }
    }
}


























