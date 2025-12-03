# MPQ文件完整读取流程

> 本文档详细说明如何读取、列出并解压MPQ归档文件中的所有文件

## 完整流程图

```
┌─────────────────────────────────────────────────────────────────────┐
│                    1. 打开MPQ文件                                    │
├─────────────────────────────────────────────────────────────────────┤
│ • 打开文件句柄                                                       │
│ • 记录文件大小                                                       │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    2. 读取并验证文件头                               │
├─────────────────────────────────────────────────────────────────────┤
│ • 读取32字节MPQ Header                                              │
│ • 验证签名: 0x1A51504D ('MPQ\x1A')                                 │
│ • 验证版本: 0 (Diablo 1)                                           │
│ • 验证头大小: 32字节                                                │
│ • 获取关键信息:                                                     │
│   - hash_entries_offset: Hash Table位置                            │
│   - hash_entries_count: Hash Table大小                             │
│   - block_entries_offset: Block Table位置                          │
│   - block_entries_count: Block Table大小                           │
│   - block_size_factor: 块大小因子 (通常是3, 即4096字节)            │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              3. 读取并解密Block Table（关键！）                      │
├─────────────────────────────────────────────────────────────────────┤
│ • seek到block_entries_offset位置                                    │
│ • 读取: block_entries_count * 16 字节                               │
│ • **解密**: decrypt_block(data, BLOCK_TABLE_KEY = 0xEC83B3A3)      │
│ • 解析为BlockEntry数组，每个entry包含:                             │
│   - offset: 文件数据在MPQ中的偏移                                  │
│   - packed_size: 压缩后的大小                                      │
│   - unpacked_size: 解压后的大小                                    │
│   - flags: 标志位                                                  │
│     * 0x80000000: 文件存在                                         │
│     * 0x00000100: PKWare压缩                                       │
│     * 0x00010000: 加密                                             │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              4. 读取并解密Hash Table（关键！）                       │
├─────────────────────────────────────────────────────────────────────┤
│ • seek到hash_entries_offset位置                                     │
│ • 读取: hash_entries_count * 16 字节                                │
│ • **解密**: decrypt_block(data, HASH_TABLE_KEY = 0xC3AF3770)       │
│ • 解析为HashEntry数组，每个entry包含:                              │
│   - hash_a: 文件名哈希值A (用于冲突解决)                           │
│   - hash_b: 文件名哈希值B (用于冲突解决)                           │
│   - locale: 语言标识 (Diablo 1中总是0)                            │
│   - platform: 平台标识 (Diablo 1中总是0)                          │
│   - block: Block Table中的索引                                     │
│     * 0xFFFFFFFF: 未使用的槽位                                     │
│     * 0xFFFFFFFE: 已删除的条目                                     │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    5. 列出所有文件                                   │
├─────────────────────────────────────────────────────────────────────┤
│ 方法1: 读取listfile (推荐)                                          │
│ ├─ 尝试查找文件 "(listfile)"                                        │
│ ├─ 这是一个特殊文件，包含所有文件名列表                             │
│ ├─ 每行一个文件路径                                                 │
│ └─ 解析后得到完整的文件列表                                         │
│                                                                      │
│ 方法2: 遍历Hash Table (备选)                                        │
│ ├─ 遍历所有Hash Entry                                              │
│ ├─ 找到所有有效的entry (block != 0xFFFFFFFF && != 0xFFFFFFFE)     │
│ ├─ 但无法得到文件名，只知道有多少个文件                             │
│ └─ 需要尝试常见文件名或暴力破解                                     │
│                                                                      │
│ 方法3: 尝试常见文件路径 (调试用)                                    │
│ └─ 测试已知的Diablo文件路径，看哪些存在                            │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                6. 对每个文件执行读取流程                             │
├─────────────────────────────────────────────────────────────────────┤
│ 对于listfile中的每个文件路径:                                       │
│                                                                      │
│ 6.1 计算文件名哈希                                                  │
│     • hash = calculate_mpq_file_hash(filename)                      │
│     • 得到 [hash_index, hash_a, hash_b]                            │
│                                                                      │
│ 6.2 在Hash Table中查找                                              │
│     • start_index = hash_index & (hash_table_size - 1)             │
│     • 从start_index开始线性探测                                     │
│     • 查找匹配的entry:                                              │
│       - entry.hash_a == hash_a                                      │
│       - entry.hash_b == hash_b                                      │
│       - entry.locale == 0                                           │
│       - entry.platform == 0                                         │
│     • 如果找到，得到block_index = entry.block                       │
│     • 如果遇到空槽位(0xFFFFFFFF)，文件不存在                        │
│                                                                      │
│ 6.3 从Block Table获取文件信息                                       │
│     • block = block_table[block_index]                              │
│     • 检查block.flags & 0x80000000 (FlagExists)                    │
│                                                                      │
│ 6.4 读取文件数据                                                     │
│     • seek到block.offset                                            │
│     • read block.packed_size 字节                                   │
│                                                                      │
│ 6.5 解密（如果需要）                                                │
│     • if block.flags & 0x00010000:                                  │
│         - 计算文件加密key                                            │
│         - decrypt_block(data, file_key)                             │
│                                                                      │
│ 6.6 解压缩（如果需要）                                              │
│     • if block.flags & 0x00000100: (PKWare压缩)                    │
│         - pkware_decompress(data, block.unpacked_size)              │
│     • if block.flags & 0x00000200: (其他压缩)                      │
│         - 使用相应的解压算法                                         │
│     • if packed_size == unpacked_size:                              │
│         - 无压缩，直接使用                                           │
│                                                                      │
│ 6.7 验证数据                                                         │
│     • 检查解压后大小是否等于block.unpacked_size                     │
│     • 可选: 验证CRC/校验和                                          │
│                                                                      │
│ 6.8 保存或使用文件数据                                              │
│     • 写入到输出目录                                                 │
│     • 或存储在内存中供游戏使用                                       │
└─────────────────────────────────────────────────────────────────────┘
```

## 详细代码示例（伪代码）

```rust
// 完整的MPQ读取流程
fn extract_all_files_from_mpq(mpq_path: &Path, output_dir: &Path) -> Result<()> {
    // ===== 步骤1: 打开文件 =====
    let mut file = File::open(mpq_path)?;
    
    // ===== 步骤2: 读取并验证文件头 =====
    let header = MpqFileHeader::read(&mut file)?;
    if !header.is_valid() {
        return Err(anyhow!("Invalid MPQ file"));
    }
    
    println!("MPQ Header:");
    println!("  File size: {}", header.file_size);
    println!("  Block size: {}", header.block_size());
    println!("  Hash entries: {}", header.hash_entries_count);
    println!("  Block entries: {}", header.block_entries_count);
    
    // ===== 步骤3: 读取并解密Block Table =====
    file.seek(SeekFrom::Start(header.block_entries_offset as u64))?;
    
    let block_table_size = (header.block_entries_count as usize) * 16;
    let mut block_table_data = vec![0u8; block_table_size];
    file.read_exact(&mut block_table_data)?;
    
    // **关键**: 解密Block Table
    decrypt_block(&mut block_table_data, BLOCK_TABLE_KEY);
    
    // 解析Block Table
    let mut block_table = Vec::new();
    for i in 0..header.block_entries_count as usize {
        let offset = i * 16;
        let entry = MpqBlockEntry::from_bytes(&block_table_data[offset..offset+16])?;
        block_table.push(entry);
    }
    
    // ===== 步骤4: 读取并解密Hash Table =====
    file.seek(SeekFrom::Start(header.hash_entries_offset as u64))?;
    
    let hash_table_size = (header.hash_entries_count as usize) * 16;
    let mut hash_table_data = vec![0u8; hash_table_size];
    file.read_exact(&mut hash_table_data)?;
    
    // **关键**: 解密Hash Table
    decrypt_block(&mut hash_table_data, HASH_TABLE_KEY);
    
    // 解析Hash Table
    let mut hash_table = Vec::new();
    for i in 0..header.hash_entries_count as usize {
        let offset = i * 16;
        let entry = MpqHashEntry::from_bytes(&hash_table_data[offset..offset+16])?;
        hash_table.push(entry);
    }
    
    // ===== 步骤5: 列出所有文件 =====
    let mut file_list = Vec::new();
    
    // 尝试读取listfile
    if let Some(listfile_data) = find_and_read_file(
        &mut file, 
        "(listfile)", 
        &hash_table, 
        &block_table, 
        &header
    )? {
        let listfile_str = String::from_utf8_lossy(&listfile_data);
        for line in listfile_str.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                file_list.push(trimmed.to_string());
            }
        }
        println!("Found {} files in listfile", file_list.len());
    } else {
        // 如果没有listfile，使用常见文件路径
        println!("No listfile found, using common file paths");
        file_list = get_common_diablo_file_paths();
    }
    
    // ===== 步骤6: 提取每个文件 =====
    let mut extracted_count = 0;
    let mut failed_count = 0;
    
    for file_path in &file_list {
        match extract_single_file(
            &mut file,
            file_path,
            &hash_table,
            &block_table,
            &header,
            output_dir
        ) {
            Ok(_) => {
                println!("✓ Extracted: {}", file_path);
                extracted_count += 1;
            }
            Err(e) => {
                eprintln!("✗ Failed to extract {}: {}", file_path, e);
                failed_count += 1;
            }
        }
    }
    
    println!("\n=== Extraction Complete ===");
    println!("Successfully extracted: {}", extracted_count);
    println!("Failed: {}", failed_count);
    println!("Total: {}", file_list.len());
    
    Ok(())
}

// 查找并读取单个文件
fn find_and_read_file(
    file: &mut File,
    filename: &str,
    hash_table: &[MpqHashEntry],
    block_table: &[MpqBlockEntry],
    header: &MpqFileHeader,
) -> Result<Option<Vec<u8>>> {
    // 6.1 计算文件名哈希
    let hash = calculate_mpq_file_hash(filename);
    let hash_index = hash[0];
    let hash_a = hash[1];
    let hash_b = hash[2];
    
    // 6.2 在Hash Table中查找
    let hash_table_size = hash_table.len() as u32;
    let mut current_index = (hash_index & (hash_table_size - 1)) as usize;
    let start_index = current_index;
    
    loop {
        let entry = &hash_table[current_index];
        
        // 检查是否匹配
        if entry.is_valid() 
            && entry.hash_a == hash_a 
            && entry.hash_b == hash_b 
        {
            // 找到了！
            let block_index = entry.block as usize;
            
            // 6.3 从Block Table获取文件信息
            if block_index >= block_table.len() {
                return Err(anyhow!("Invalid block index"));
            }
            
            let block = &block_table[block_index];
            
            if !block.exists() {
                return Err(anyhow!("Block does not exist"));
            }
            
            // 6.4 读取文件数据
            file.seek(SeekFrom::Start(block.offset as u64))?;
            let mut data = vec![0u8; block.packed_size as usize];
            file.read_exact(&mut data)?;
            
            // 6.5 解密（如果需要）
            if block.is_encrypted() {
                // 注意: 文件内容加密需要基于文件名和位置计算key
                // 这里简化处理
                let file_key = calculate_file_encryption_key(filename, block_index);
                decrypt_block(&mut data, file_key);
            }
            
            // 6.6 解压缩（如果需要）
            if block.is_pkware_compressed() {
                data = pkware_decompress(&data, block.unpacked_size as usize)?;
            }
            
            // 6.7 验证数据
            if data.len() != block.unpacked_size as usize {
                return Err(anyhow!(
                    "Unpacked size mismatch: expected {}, got {}",
                    block.unpacked_size,
                    data.len()
                ));
            }
            
            return Ok(Some(data));
        }
        
        // 如果是空槽位，文件不存在
        if entry.block == 0xFFFFFFFF {
            return Ok(None);
        }
        
        // 线性探测下一个位置
        current_index = (current_index + 1) % hash_table.len();
        
        // 如果回到起点，说明找遍了整个表
        if current_index == start_index {
            return Ok(None);
        }
    }
}

// 提取单个文件到磁盘
fn extract_single_file(
    file: &mut File,
    filename: &str,
    hash_table: &[MpqHashEntry],
    block_table: &[MpqBlockEntry],
    header: &MpqFileHeader,
    output_dir: &Path,
) -> Result<()> {
    if let Some(data) = find_and_read_file(file, filename, hash_table, block_table, header)? {
        // 6.8 保存文件
        let output_path = output_dir.join(filename);
        
        // 创建必要的目录
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // 写入文件
        std::fs::write(&output_path, &data)?;
        
        Ok(())
    } else {
        Err(anyhow!("File not found in MPQ: {}", filename))
    }
}
```

## 关键技术要点

### 1. 加密解密
- **Block Table** 和 **Hash Table** 必须解密才能使用
- 使用固定的加密key:
  - Block Table Key: `0xEC83B3A3`
  - Hash Table Key: `0xC3AF3770`

### 2. 文件查找
- 使用MPQ哈希算法计算文件名的3个哈希值
- 在Hash Table中使用线性探测处理冲突

### 3. 压缩支持
- **PKWare**: 最常见，flag = 0x00000100
- **Huffman**: 较少见
- **无压缩**: packed_size == unpacked_size

### 4. Listfile的重要性
- Diablo 1的MPQ通常不包含listfile
- 需要依赖已知的文件路径列表
- 或者使用工具（如StormLib）重建listfile

## 常见陷阱

1. ❌ **忘记解密表格**
   - 直接读取的Block/Hash Table是加密的，必须先解密

2. ❌ **字节序错误**
   - MPQ使用小端序(little-endian)

3. ❌ **哈希冲突处理**
   - 必须使用线性探测，不能只查找一个位置

4. ❌ **文件名大小写**
   - 哈希计算前要转换为大写
   - 路径分隔符统一为 `/` 或 `\`

5. ❌ **压缩算法选择**
   - 要检查flags正确选择解压算法
   - Diablo 1主要使用PKWare

## 测试建议

```rust
#[test]
fn test_complete_mpq_extraction() {
    let mpq_path = Path::new("assets/Diabdat.mpq");
    let output_dir = Path::new("test_output/mpq_extract");
    
    // 提取所有文件
    extract_all_files_from_mpq(mpq_path, output_dir).unwrap();
    
    // 验证已知文件
    assert!(output_dir.join("levels/towndata/town.pal").exists());
    
    // 验证文件内容
    let pal_data = std::fs::read(output_dir.join("levels/towndata/town.pal")).unwrap();
    assert_eq!(pal_data.len(), 768); // 256 colors * 3 bytes
}
```

## 参考资料

- MPQ Format: https://www.zezula.net/en/mpq/mpqformat.html
- libmpq源代码: https://github.com/diasurgical/libmpq
- DevilutionX实现: `Source/mpq/`


