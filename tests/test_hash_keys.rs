/// 测试哈希密钥计算
use rust_diablo::resources::mpq_format::{hash_string, initialize_crypt_table, HashType};

#[test]
fn test_hash_table_keys() {
    initialize_crypt_table();

    // 根据 libmpq 的定义：
    // LIBMPQ_HASH_TABLE_HASH_KEY = libmpq__hash_string("(hash table)", 0x300) = 3283040112u
    // LIBMPQ_BLOCK_TABLE_HASH_KEY = libmpq__hash_string("(block table)", 0x300) = 3968054179u

    let hash_table_key = hash_string("(hash table)", HashType::FileKey);
    let block_table_key = hash_string("(block table)", HashType::FileKey);

    println!("\n=== Hash Keys ===");
    println!(
        "hash_table_key:  {} (0x{:08X})",
        hash_table_key, hash_table_key
    );
    println!(
        "Expected:        {} (0x{:08X})",
        3283040112u32, 3283040112u32
    );
    println!();
    println!(
        "block_table_key: {} (0x{:08X})",
        block_table_key, block_table_key
    );
    println!(
        "Expected:        {} (0x{:08X})",
        3968054179u32, 3968054179u32
    );

    assert_eq!(hash_table_key, 3283040112u32, "Hash table key mismatch!");
    assert_eq!(block_table_key, 3968054179u32, "Block table key mismatch!");
}
