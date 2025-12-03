# 原版C++代码MPQ读取流程分析

>本文档分析原版DevilutionX C++代码如何读取和解析Diablo 1的MPQ归档文件

## 关键发现

### 1. **Hash Table和Block Table是加密存储的！**

这是最重要的发现。在 `Source/mpq/mpq_writer.cpp` 中，代码明确显示：

```cpp
// 读取Block Table后，立即解密
libmpq__decrypt_block(reinterpret_cast<uint32_t *>(blockTable_.get()), 
                      fhdr.blockEntriesCount * sizeof(MpqBlockEntry), 
                      LIBMPQ_BLOCK_TABLE_HASH_KEY);

// 读取Hash Table后，立即解密
libmpq__decrypt_block(reinterpret_cast<uint32_t *>(hashTable_.get()), 
                      fhdr.hashEntriesCount * sizeof(MpqHashEntry), 
                      LIBMPQ_HASH_TABLE_HASH_KEY);
```

### 2. MPQ文件读取的完整流程

```
1. 打开MPQ文件
   ↓
2. 读取文件头 (MpqFileHeader, 32 bytes)
   - signature: 0x1A51504D ('MPQ\x1A')
   - header_size: 32
   - file_size: MPQ文件总大小
   - version: 0 (Diablo 1)
   - block_size_factor: 通常是3 (block size = 512 << 3 = 4096 bytes)
   - hash_entries_offset: Hash Table在文件中的偏移
   - block_entries_offset: Block Table在文件中的偏移
   - hash_entries_count: Hash Table条目数
   - block_entries_count: Block Table条目数
   ↓
3. 读取Block Table (加密的)
   - seek to block_entries_offset
   - read block_entries_count * 16 bytes (每个BlockEntry是16字节)
   - **关键**: 使用LIBMPQ_BLOCK_TABLE_HASH_KEY解密
   ↓
4. 读取Hash Table (加密的)
   - seek to hash_entries_offset
   - read hash_entries_count * 16 bytes (每个HashEntry是16字节)
   - **关键**: 使用LIBMPQ_HASH_TABLE_HASH_KEY解密
   ↓
5. 查找文件
   - 计算文件名的MPQ哈希 (hash[0], hash[1], hash[2])
   - 使用hash[0]作为Hash Table的起始索引
   - 在Hash Table中查找匹配的entry (hash[1] == hashA && hash[2] == hashB)
   - 获取block索引
   ↓
6. 读取文件数据
   - 从Block Table获取block entry
   - seek to block.offset
   - read block.packed_size bytes
   - 如果compressed, 解压缩 (PKWare, etc.)
   - 如果encrypted, 解密
   - 返回block.unpacked_size bytes
```

### 3. 加密算法关键点

根据代码注释：
```cpp
// The decryption algorithm treats them as a stream of 32-bit uints, so the
// sizes must be exact as there cannot be any padding.
static_assert(CheckSize<MpqHashEntry, static_cast<size_t>(4 * 4)>::value);
static_assert(CheckSize<MpqBlockEntry, static_cast<size_t>(4 * 4)>::value);
```

- **加密/解密是按32位整数为单位进行的**
- Hash Entry和Block Entry都是16字节 (4个uint32)
- 必须没有padding

### 4. 加密Key

根据原版代码使用的常量：
- `LIBMPQ_BLOCK_TABLE_HASH_KEY` - 用于Block Table加密/解密
- `LIBMPQ_HASH_TABLE_HASH_KEY` - 用于Hash Table加密/解密

这些是libmpq库定义的常量，需要在我们的实现中找出这些值。

### 5. 哈希算法

文件名哈希算法已在 `rust-diablo/src/resources/mpq_format.rs` 中实现：

```rust
pub fn calculate_mpq_file_hash(filename: &str) -> MpqFileHash {
    // 1. 转换为大写并规范化路径分隔符
    let normalized = filename.to_uppercase().replace('\\', "/");
    
    // 2. 使用MPQ加密表计算三个哈希值
    // hash[0] - 用于Hash Table索引
    // hash[1] - 存储为hashA
    // hash[2] - 存储为hashB
    
    [hash_index, hash_a, hash_b]
}
```

### 6. 压缩算法

Diablo 1的MPQ文件使用PKWare压缩：

```cpp
// Flag indicating PKWare compression
MpqBlockEntry::CompressPkZip = 0x00000100
```

PKWare压缩/解压缩实现在 `3rdParty/PKWare/explode.cpp` 和 `implode.cpp`

## 需要实现的关键功能

为了在Rust中完整实现MPQ读取，我们需要：

1. ✅ MPQ文件格式定义 (`mpq_format.rs`)
   - MpqFileHeader
   - MpqHashEntry
   - MpqBlockEntry
   
2. ✅ MPQ哈希算法 (`calculate_mpq_file_hash`)
   - 生成加密表 (MPQ_CRYPT_TABLE)
   - 计算文件名哈希
   
3. ❌ **MPQ表解密算法** (CRITICAL!)
   - `decrypt_block(data, size, hash_key)` 函数
   - 需要找到LIBMPQ_BLOCK_TABLE_HASH_KEY和LIBMPQ_HASH_TABLE_HASH_KEY的值
   - 实现解密算法
   
4. ❌ **PKWare解压缩**
   - 移植 `3rdParty/PKWare/explode.cpp`
   - 或使用FFI绑定原C++代码
   - 或使用已有的Rust PKWare库
   
5. ✅ 文件查找逻辑
   - Hash Table查找
   - 线性探测处理冲突
   
6. ❌ 文件数据读取
   - 处理压缩
   - 处理加密
   - 按block读取

## 下一步行动

1. **优先级1**: 实现MPQ表解密算法
   - 研究libmpq源代码或MPQ格式文档
   - 找到加密key的值
   - 实现`decrypt_block`函数

2. **优先级2**: 实现PKWare解压缩
   - 移植C代码或使用FFI
   - 测试解压缩功能

3. **优先级3**: 完整测试
   - 测试读取真实的Diabdat.mpq文件
   - 验证town.pal等已知文件的内容
   - 确保数据正确性

## 参考资料

- DevilutionX源代码: `Source/mpq/`
- MPQ格式文档: [MPQ Format Specification](https://www.zezula.net/en/mpq/mpqformat.html)
- libmpq库: [libmpq GitHub](https://github.com/ge0rg/libmpq)
- PKWare算法: `3rdParty/PKWare/`


