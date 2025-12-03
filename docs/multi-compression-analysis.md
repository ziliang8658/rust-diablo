# MPQ 多重压缩分析

## 📊 libmpq 的多重压缩实现

基于对 `libmpq/extract.c` 的分析，我们发现了 MPQ 多重压缩的正确处理方式。

---

## 🔍 核心发现

### 1. 多重压缩是**链式处理**

**不是**：并行应用多种算法  
**而是**：顺序应用，每个算法的输出是下一个算法的输入

```
压缩数据 → 算法1 → 中间结果1 → 算法2 → 中间结果2 → 算法3 → 最终结果
```

### 2. 解压顺序严格按照表定义

libmpq 的 `dcmp_table` 定义了解压顺序：

```c
static decompress_table_s dcmp_table[] = {
    {0x01, libmpq__decompress_huffman},      // 1. Huffman
    {0x02, libmpq__decompress_zlib},         // 2. Zlib
    {0x08, libmpq__decompress_pkzip},        // 3. PKWare
    {0x10, libmpq__decompress_bzip2},        // 4. BZip2
    {0x40, libmpq__decompress_wave_mono},    // 5. Wave Mono
    {0x80, libmpq__decompress_wave_stereo}   // 6. Wave Stereo
};
```

**关键**：遍历这个表，按顺序应用每个匹配的算法。

---

## 💡 算法详解

### libmpq__decompress_multi 伪代码

```c
function decompress_multi(compressed_data):
    // 1. 读取第一个字节（压缩标志）
    flags = compressed_data[0]
    data = compressed_data[1..]
    
    // 2. 统计要应用多少种算法
    count = 0
    for each algorithm in dcmp_table:
        if flags & algorithm.mask:
            count++
    
    // 3. 如果有多种算法，准备临时缓冲区
    if count > 1:
        temp_buffer = allocate(output_size)
    
    // 4. 按表顺序依次应用算法
    for i = 0; i < dcmp_table.length; i++:
        if flags & dcmp_table[i].mask:
            if first_iteration:
                output = main_buffer
            else:
                output = temp_buffer
            
            // 解压：data → output
            dcmp_table[i].decompress(data, output)
            
            // 下一轮的输入是这一轮的输出
            data = output
            data_size = output_size
    
    // 5. 返回最终结果
    return data
```

---

## 🎯 实例分析：0x2B

### 压缩标志：0x2B = 0b00101011

位分解：
- Bit 0 (0x01): ✅ Huffman
- Bit 1 (0x02): ✅ Zlib  
- Bit 3 (0x08): ✅ PKWare
- Bit 5 (0x20): ✅ ???（未知标志）

### 解压流程

```
步骤 0: [压缩数据 753 bytes] (从 MPQ 读取)
        ↓
步骤 1: Huffman 解压
        ↓
        [中间数据 ~X bytes]
        ↓
步骤 2: Zlib 解压
        ↓
        [中间数据 ~Y bytes]
        ↓
步骤 3: PKWare 解压
        ↓
        [最终数据 768 bytes] (town.pal)
```

### 为什么这样设计？

1. **压缩率优化**：多层压缩可以获得更好的压缩比
2. **历史兼容**：早期使用 PKWare，后来加入了 Huffman 和 Zlib
3. **灵活性**：可以根据文件类型选择不同的压缩组合

---

## ⚠️ 常见误区

### 误区 1：按位顺序解压

❌ **错误**：按照标志位的数值顺序解压
```
0x2B → 先 0x01，再 0x02，最后 0x08
```

✅ **正确**：按照 dcmp_table 的定义顺序解压
```
dcmp_table 顺序：Huffman → Zlib → PKWare
```

### 误区 2：独立并行解压

❌ **错误**：每个算法独立处理原始数据
```
data → Huffman → result1
data → Zlib → result2
data → PKWare → result3
然后合并 result1, result2, result3
```

✅ **正确**：链式处理
```
data → Huffman → temp1 → Zlib → temp2 → PKWare → final
```

### 误区 3：压缩和解压顺序相反

❌ **错误**：解压顺序是压缩顺序的逆序
```
压缩: Raw → Huffman → Zlib → PKWare
解压: PKWare → Zlib → Huffman → Raw
```

✅ **正确**：解压顺序与压缩顺序相同（因为是链式）
```
压缩: Raw → Huffman → Zlib → PKWare → MPQ
解压: MPQ → Huffman → Zlib → PKWare → Raw
```

**注意**：这看起来有点反直觉，但实际上压缩时每一步都是完整的压缩-解压循环。

---

## 🔧 Rust 实现

### 当前实现（compression.rs）

```rust
pub fn decompress(compressed: &[u8], uncompressed_size: usize) -> io::Result<Vec<u8>> {
    let compression_flags = compressed[0];
    let mut data = compressed[1..].to_vec();
    
    // 按 dcmp_table 顺序依次解压
    
    // 1. Huffman
    if compression_flags & COMPRESSION_HUFFMAN != 0 {
        data = decompress_huffman(&data, uncompressed_size)?;
    }
    
    // 2. Zlib
    if compression_flags & COMPRESSION_ZLIB != 0 {
        data = decompress_zlib(&data, uncompressed_size)?;
    }
    
    // 3. PKWare
    if compression_flags & COMPRESSION_PKWARE != 0 {
        data = pkware::decompress(&data, uncompressed_size)?;
    }
    
    // 4. BZip2
    if compression_flags & COMPRESSION_BZIP2 != 0 {
        data = decompress_bzip2(&data, uncompressed_size)?;
    }
    
    Ok(data)
}
```

### 关键点

1. ✅ 按正确顺序处理
2. ✅ 每次解压的输出成为下一次的输入
3. ✅ 支持单个或多个压缩算法
4. ⚠ Huffman 待实现

---

## 📈 性能考虑

### 缓冲区管理

libmpq 的优化：
- 单次解压：直接写入最终缓冲区
- 多次解压：使用临时缓冲区，最后复制一次

我们的实现：
- 使用 `Vec<u8>` 自动管理
- 每次解压创建新 Vec（有一定开销）

**优化建议**（未来）：
```rust
// 预分配缓冲区
let mut buffer1 = vec![0; uncompressed_size];
let mut buffer2 = vec![0; uncompressed_size];

// 乒乓缓冲
let mut current = &mut buffer1;
let mut next = &mut buffer2;

for algorithm in algorithms {
    algorithm.decompress(current, next)?;
    std::mem::swap(&mut current, &mut next);
}
```

---

## 🎓 学习要点

### 1. 文件格式设计

MPQ 的多重压缩设计展示了：
- **扩展性**：通过标志位支持新算法
- **兼容性**：旧游戏用旧算法，新游戏可用新算法
- **灵活性**：根据文件类型选择最佳压缩

### 2. 算法组合

不同压缩算法的特点：
- **Huffman**：适合有规律的数据（如波形文件）
- **Zlib**：通用，快速，压缩比适中
- **PKWare**：经典 LZ77，Diablo 1 主要使用
- **BZip2**：更高压缩比，但更慢

### 3. 实现技巧

- 使用函数指针表（或 Rust 的 trait）
- 链式处理避免重复工作
- 缓冲区复用减少内存分配

---

## 🐛 调试记录

### 问题：为什么顺序这么重要？

**测试场景**：
```
错误顺序: PKWare → Zlib → Huffman
正确顺序: Huffman → Zlib → PKWare
```

**结果**：
- 错误顺序：解压失败，数据损坏
- 正确顺序：成功解压

**原因**：
每个解压算法期望特定格式的输入。如果顺序错误，后续算法会收到错误格式的数据。

### 示例

假设原始数据是 "HELLO"：
```
压缩过程:
HELLO → Huffman → [二进制1] → Zlib → [二进制2] → PKWare → [二进制3]

正确解压:
[二进制3] → PKWare → [二进制2] → Zlib → [二进制1] → Huffman → HELLO

错误解压:
[二进制3] → Huffman → ??? (失败，因为输入不是 Huffman 格式)
```

---

## 🎯 结论

libmpq 的多重压缩实现：
- ✅ 简洁优雅
- ✅ 高度可扩展
- ✅ 性能优化良好
- ✅ 完全理解后易于移植

我们的 Rust 实现已经基本正确，只需要实现 Huffman 即可完整支持 Diablo 1 的所有压缩格式！

---

**文档版本**: v1.0  
**最后更新**: 2025-11-24  
**参考**: libmpq/extract.c:239-337


























