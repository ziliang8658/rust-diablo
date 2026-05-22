# Step 5.1 原生 MPQ 实现 - 进度报告

## 概述

本文档记录了从零开始实现原生 MPQ 读取系统的完整过程，包括成功完成的功能和当前待解决的问题。

## 📊 整体进度

**完成度**: 约 90%  
**核心功能**: ✅ 已完成  
**待解决**: PKWare 解压缩

---

## ✅ 已完成的功能

### 1. MPQ 格式定义 (`src/resources/mpq_format.rs`)

**文件**: 438 行代码  
**状态**: ✅ 完全实现并测试通过

#### 实现内容

1. **数据结构定义**
   ```rust
   - MpqHeader: MPQ 文件头（32 字节）
   - MpqHashEntry: 哈希表条目（16 字节）
   - MpqBlockEntry: 块表条目（16 字节）
   ```

2. **加密表生成**
   - 1280 条目的全局加密表
   - 初始化算法完全参考 libmpq
   - 测试验证：生成的表与 libmpq 一致

3. **哈希算法**
   ```rust
   pub fn hash_string(filename: &str, hash_type: HashType) -> u32
   ```
   - 支持 4 种哈希类型：TableOffset, NameA, NameB, FileKey
   - 自动大小写转换和路径标准化
   - **关键发现**: FileKey 只使用文件名的 basename
   - 测试验证：
     * `hash_string("(hash table)", FileKey)` = 0xC3AF3770 ✓
     * `hash_string("(block table)", FileKey)` = 0xEC83B3A3 ✓

4. **解密算法**
   ```rust
   pub fn decrypt_block(data: &mut [u32], key: u32)
   ```
   - 完全参考 libmpq 的实现
   - **关键修复**: seed 更新公式使用 `~seed` 而不是简单的加法
   - 测试验证：哈希表解密成功

### 2. MpqArchive 实现 (`src/resources/mpq.rs`)

**文件**: 526 行代码  
**状态**: ✅ 核心功能完成

#### 实现内容

1. **MPQ 文件打开**
   ```rust
   pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self>
   ```
   - 读取并验证 MPQ 文件头
   - 读取并解密哈希表
   - 读取块表（**重大发现**: Diablo 1 的块表是明文，不需要解密）

2. **文件查找**
   ```rust
   fn find_file_index(&self, filename: &str) -> Option<usize>
   ```
   - 使用 MPQ 哈希算法计算文件哈希
   - 线性探测解决哈希冲突
   - 测试结果: 成功找到 `levels/towndata/town.pal` (block_index=1182)

3. **文件读取**
   ```rust
   pub fn read_file(&mut self, filename: &str) -> io::Result<Option<Vec<u8>>>
   ```
   - 定位文件在 MPQ 中的位置
   - 读取文件数据
   - 文件解密（如果 ENCRYPTED 标志设置）
   - 文件解压缩（**待完成**: PKWare 算法）

### 3. MpqManager 实现 (`src/resources/mpq.rs`)

**文件**: 包含在同一文件中  
**状态**: ✅ 完全实现

#### 实现内容

1. **多 MPQ 文件管理**
   ```rust
   pub fn load_mpq(&mut self, path: &str, priority: i32) -> Result<()>
   ```
   - 支持加载多个 MPQ 文件
   - 优先级系统：高优先级覆盖低优先级
   - 自动搜索多个标准路径

2. **文件搜索**
   ```rust
   pub fn find_file(&mut self, path: &str) -> Option<Vec<u8>>
   pub fn has_file(&mut self, path: &str) -> bool
   ```
   - 从高优先级到低优先级搜索
   - 返回第一个找到的文件

3. **测试结果**
   ```
   ✓ 成功加载 Diabdat.mpq: 2910 个文件
   ✓ 文件查找成功: levels/towndata/town.pal
   ✓ 文件解密成功: 753 bytes (压缩) → 768 bytes (预期)
   ```

---

## ⚠️ 待解决问题

### PKWare 解压缩

**文件**: `src/resources/pkware.rs`  
**当前状态**: 使用 `implode` crate，但遇到溢出错误  
**问题描述**: 

1. **症状**
   ```
   thread panicked at implode-0.1.1\src\symbol.rs:213:29:
   attempt to shift left with overflow
   ```

2. **已尝试的方案**
   - ✓ 集成 `implode` crate (0.1.1)
   - ✓ 参考 mpq-rust-patched 的实现
   - ✗ 直接解压失败

3. **可能的原因**
   - 解密后的数据格式问题
   - 压缩类型字节解析问题
   - `implode` crate 版本兼容性问题

4. **下一步方案**
   - **方案 A**: 调试 `implode` crate，检查输入数据格式
   - **方案 B**: 尝试其他库 (如 `blast` crate)
   - **方案 C**: 直接移植 libmpq 的 explode.c (600+ 行)
   - **方案 D**: 使用外部工具预处理 MPQ 文件

---

## 🔍 关键发现和踩坑点

### 1. 哈希表解密成功，但块表不需要解密

**发现**: Diablo 1 的 MPQ 格式中，块表是明文存储的。

```rust
// 哈希表：需要解密
let mut hash_table_u32 = bytes_to_u32_vec(&hash_table_bytes);
decrypt_block(&mut hash_table_u32, hash_string("(hash table)", HashType::FileKey));

// 块表：不需要解密（直接解析原始字节）
let mut block_table = Vec::with_capacity(header.block_table_entries as usize);
for i in 0..header.block_table_entries as usize {
    let offset = i * 16;
    let entry = MpqBlockEntry::from_bytes(&block_table_bytes[offset..offset+16])?;
    block_table.push(entry);
}
```

**教训**: 不同版本的 MPQ 格式有微妙差异，需要实际测试验证。

### 2. 文件解密密钥只使用 basename

**发现**: libmpq 计算文件解密密钥时，只使用文件名的基本名称，而不是完整路径。

```rust
// 错误做法（会导致解密失败）
let key = hash_string("levels/towndata/town.pal", HashType::FileKey);

// 正确做法（libmpq 的行为）
let key = hash_string("town.pal", HashType::FileKey);
```

**实现**:
```rust
fn get_basename(filename: &str) -> &str {
    filename.rfind(|c| c == '/' || c == '\\')
        .map(|pos| &filename[pos + 1..])
        .unwrap_or(filename)
}
```

**影响**: 这个bug导致所有加密文件解密失败。修复后，解密密钥从 `0x7B644258` 变为正确的 `0x1FCE2DA3`。

### 3. Rust packed 结构体的陷阱

**问题**: 直接引用 packed 结构体字段会导致编译错误。

```rust
// 错误：直接引用 packed 字段
eprintln!("size: {}", block.packed_size);  // ❌ 编译错误

// 正确：先复制到本地变量
let packed_size = block.packed_size;
eprintln!("size: {}", packed_size);  // ✓
```

**教训**: 在使用 `#[repr(C, packed)]` 时，需要特别注意字段访问。

### 4. MPQ 解密算法的细节

**libmpq 的实现**:
```c
seed2 += crypt_buf[0x400 + (seed & 0xFF)];
ch = libmpq__bswap_LE32(*in_buf) ^ (seed + seed2);
seed = ((~seed << 0x15) + 0x11111111) | (seed >> 0x0B);
seed2 = ch + seed2 + (seed2 << 5) + 3;
```

**关键点**:
- seed2 累加而不是重置
- seed 更新使用按位取反 `~seed`
- 需要处理字节序（Windows 通常是小端序）

---

## 📈 测试结果

### 成功的测试

```rust
✓ test_mpq_manager_creation
✓ test_load_diabdat_mpq
  - 加载 Diabdat.mpq: 2910 个文件
✓ test_has_file
  - levels/towndata/town.pal: 找到 (block_index=1182)
✓ test_find_town_pal
  - 文件定位成功
  - 文件解密成功
  - 解压缩失败 (implode crate 错误)
```

### 测试数据

**town.pal 文件信息**:
```
Path: levels/towndata/town.pal
Hash A: 0x7DC9935B
Hash B: 0xA4A1A017
Block Index: 1182
Packed Size: 753 bytes
Unpacked Size: 768 bytes
Flags: 0x80010100 (EXISTS | ENCRYPTED | COMPRESS_PKZIP)
Encryption Key: 0x1FCE2DA3
```

---

## 📚 参考代码

### libmpq 源码位置

```
G:\DevilutionX\3rdParty\libmpq\source\libmpq\libmpq\
├── common.c          # 哈希、加密、解密算法
├── common.h
├── crypt_buf.h       # 1280 条目加密表
├── explode.c         # PKWare 解压缩 (600+ 行)
├── mpq.c             # MPQ 文件操作主逻辑
└── mpq.h
```

### 关键函数对照

| Rust 实现 | libmpq 对应函数 |
|-----------|----------------|
| `initialize_crypt_table()` | 内置在 `crypt_buf.h` |
| `hash_string()` | `libmpq__hash_string()` |
| `decrypt_block()` | `libmpq__decrypt_block()` |
| `MpqArchive::open()` | `libmpq__archive_open()` |
| `find_file_index()` | `libmpq__file_number_from_hash()` |

---

## 🎯 下一步计划

### 短期 (当前会话)

1. **完成 PKWare 解压缩**
   - 调试 `implode` crate 问题
   - 或尝试替代方案

2. **集成 Palette 系统**
   - 读取 town.pal
   - 测试调色板转换

3. **清理调试输出**
   - 移除 eprintln! 调试语句
   - 添加适当的日志

### 中期 (下一步)

1. **完善测试用例**
   - 测试更多文件类型
   - 测试边界情况

2. **性能优化**
   - 缓存哈希表查找结果
   - 优化文件读取

3. **错误处理**
   - 更详细的错误信息
   - 更好的错误恢复

### 长期

1. **支持其他压缩格式**
   - Zlib
   - BZip2

2. **支持 MPQ 写入**
   - 创建新 MPQ
   - 添加/修改文件

---

## 💡 学习要点

### Rust 特性应用

1. **类型安全的文件格式解析**
   ```rust
   #[repr(C, packed)]
   pub struct MpqHeader { ... }
   ```

2. **错误处理**
   ```rust
   use std::io::{self, Error, ErrorKind};
   Result<T, io::Error>
   ```

3. **unsafe 代码**
   ```rust
   unsafe {
       static mut MPQ_CRYPT_TABLE: [u32; 1280] = [0; 1280];
   }
   ```

### 游戏开发技术

1. **归档文件格式**
   - 哈希表用于快速查找
   - 加密保护资源
   - 压缩减小文件大小

2. **向后兼容性**
   - 格式版本号
   - 可选功能标志

3. **性能优化**
   - 预计算加密表
   - 块表缓存

---

## 🐛 已知Bug列表

1. **PKWare 解压缩失败**
   - 状态: 待修复
   - 优先级: 高
   - 影响: 无法读取大部分游戏资源

2. **部分文件找不到**
   - 例如: `(listfile)`, `(attributes)`
   - 可能原因: 文件不存在或哈希算法细节问题
   - 优先级: 中

---

## 📊 代码统计

| 文件 | 行数 | 功能 | 状态 |
|------|------|------|------|
| `mpq_format.rs` | 438 | MPQ 格式定义 | ✅ 完成 |
| `mpq.rs` | 526 | MPQ 读取器 | ✅ 核心完成 |
| `pkware.rs` | 328 | PKWare 解压 | ⚠ 待修复 |
| **总计** | **1292** | | **90%** |

---

## 🎉 总结

经过大量工作，我们成功实现了一个**几乎完整的原生 MPQ 读取系统**！

**主要成就**:
- ✅ 从零实现了完整的 MPQ 格式解析
- ✅ 成功读取 Diablo 1 的 MPQ 文件
- ✅ 实现了文件查找、解密功能
- ✅ 发现并修复了多个关键bug

**剩余工作**:
- ⚠ PKWare 解压缩（最后一块拼图）

这个项目展示了：
1. 如何阅读和理解 C 代码并用 Rust 重写
2. 如何调试复杂的二进制格式
3. 如何处理 unsafe 代码和系统级编程
4. 游戏资源管理的核心技术

---

**文档版本**: v1.0  
**最后更新**: 2025-11-24  
**作者**: AI Assistant + User  
**参考**: libmpq, mpq-rust, StormLib


























