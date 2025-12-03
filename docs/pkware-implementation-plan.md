# PKWare 解压缩实现计划

## 📋 概述

基于 libmpq 的 explode.c 实现，从零开始用 Rust 实现 PKWare Data Compression Library 的解压缩功能。

**源文件**: `G:\DevilutionX\3rdParty\libmpq\source\libmpq\libmpq\explode.c` (606 行)  
**目标文件**: `rust-diablo/src/resources/pkware.rs`

---

## 🏗️ 核心数据结构

### 1. PkzipDecompressor（对应 pkzip_cmp_s）

这是解压缩器的主要工作结构体，约 12KB 大小。

```rust
pub struct PkzipDecompressor {
    // === 基本配置 ===
    cmp_type: u8,           // 压缩类型：0=binary, 1=ascii
    dsize_bits: u32,        // 字典大小位数：4/5/6
    dsize_mask: u32,        // 字典大小掩码：0x0F/0x1F/0x3F
    
    // === 位缓冲区 ===
    bit_buf: u32,           // 16位位缓冲区
    extra_bits: u32,        // 超过8位的额外位数
    
    // === 输入/输出位置 ===
    in_pos: usize,          // 输入缓冲区位置
    in_bytes: usize,        // 输入缓冲区字节数
    out_pos: usize,         // 输出缓冲区位置
    
    // === 缓冲区 ===
    in_buf: [u8; 0x800],    // 输入缓冲区 (2KB)
    out_buf: [u8; 0x2000],  // 输出循环缓冲区 (8KB)
    
    // === 解码表 ===
    pos1: [u8; 0x100],      // 距离解码表
    pos2: [u8; 0x100],      // 长度解码表
    
    // === ASCII 相关 ===
    offs_2c34: [u8; 0x100], // ASCII解码表1
    offs_2d34: [u8; 0x100], // ASCII解码表2
    offs_2e34: [u8; 0x80],  // ASCII解码表3
    offs_2eb4: [u8; 0x100], // ASCII解码表4
    bits_asc: [u8; 0x100],  // ASCII位数表
    
    // === 其他解码表 ===
    dist_bits: [u8; 0x40],  // 距离位数表
    slen_bits: [u8; 0x10],  // 长度位数表
    clen_bits: [u8; 0x10],  // 复制长度位数表
    len_base: [u16; 0x10],  // 长度基础表
}
```

### 2. DecompressionContext（对应 pkzip_data_s）

用于在解压缩过程中传递输入输出数据。

```rust
pub struct DecompressionContext<'a> {
    in_buf: &'a [u8],       // 输入数据
    in_pos: usize,          // 输入位置
    out_buf: &'a mut Vec<u8>, // 输出数据
}
```

---

## 📊 常量查找表

需要实现以下7个静态查找表（直接从 libmpq 复制）：

1. **PKZIP_DIST_BITS** [64] - 距离编码位数
2. **PKZIP_DIST_CODE** [64] - 距离编码
3. **PKZIP_CLEN_BITS** [16] - 复制长度位数
4. **PKZIP_LEN_BASE** [16] - 长度基础值
5. **PKZIP_SLEN_BITS** [16] - 短长度位数
6. **PKZIP_LEN_CODE** [16] - 长度编码
7. **PKZIP_BITS_ASC** [256] - ASCII 位数表
8. **PKZIP_CODE_ASC** [256] - ASCII 编码表

---

## 🔧 核心函数实现

### Phase 1: 基础位操作

#### 1.1 `skip_bit()` - 跳过指定位数
```rust
fn skip_bit(&mut self, bits: u32) -> Result<()>
```
- 从位缓冲区跳过指定位数
- 如果位缓冲区不足，从输入读取更多数据
- **关键**: 所有其他函数的基础

#### 1.2 数据读取回调
```rust
fn read_input(&mut self, ctx: &mut DecompressionContext) -> usize
```
- 从输入缓冲区读取数据到内部缓冲区
- 返回读取的字节数

### Phase 2: 解码表生成

#### 2.1 `generate_tables_decode()` - 生成解码表
```rust
fn generate_tables_decode(
    count: usize,
    bits: &[u8],
    code: &[u8],
    output: &mut [u8]
)
```
- 根据位数和编码表生成快速查找表
- 用于 pos1 和 pos2 表

#### 2.2 `generate_tables_ascii()` - 生成 ASCII 解码表
```rust
fn generate_tables_ascii(&mut self)
```
- 生成 ASCII 压缩模式的解码表
- 填充 offs_2c34, offs_2d34, offs_2e34, offs_2eb4

### Phase 3: 解码操作

#### 3.1 `decode_literal()` - 解码字面值或重复长度
```rust
fn decode_literal(&mut self) -> Result<u32>
```
- 返回值：
  * `0x000-0x0FF`: 一个字节的字面值
  * `0x100-0x305`: 重复块长度（0x100 = 重复1字节）
  * `0x306`: 缓冲区错误

**逻辑流程**:
1. 检查位缓冲区第一位
2. 如果是 1: 解码重复长度
3. 如果是 0: 
   - Binary 模式: 直接读取 8 位
   - ASCII 模式: 使用 ASCII 解码表

#### 3.2 `decode_distance()` - 解码回溯距离
```rust
fn decode_distance(&mut self, length: u32) -> Result<u32>
```
- 根据重复长度，解码需要回溯的距离
- 使用 pos1 表和 dist_bits 表

### Phase 4: 主解压缩循环

#### 4.1 `expand()` - 主解压缩函数
```rust
fn expand(&mut self, ctx: &mut DecompressionContext) -> Result<()>
```

**算法流程**:
```
1. 初始化输出位置 = 0x1000
2. 循环:
   a. 调用 decode_literal() 获取值
   b. 如果值 >= 0x100:
      - 减去 0xFE 得到重复长度
      - 调用 decode_distance() 获取回溯距离
      - 从 (当前位置 - 距离) 复制数据
   c. 如果值 < 0x100:
      - 直接写入输出缓冲区
   d. 如果输出位置 >= 0x2000:
      - 刷新输出缓冲区（写入最终输出）
      - 保留后半部分数据
3. 复制剩余数据
```

### Phase 5: 公共接口

#### 5.1 `decompress()` - 公共解压缩接口
```rust
pub fn decompress(
    compressed: &[u8],
    uncompressed_size: usize
) -> io::Result<Vec<u8>>
```

**步骤**:
1. 读取压缩头（3字节）:
   - Byte 0: 压缩类型 (0=binary, 1=ascii)
   - Byte 1: 字典大小 (4/5/6)
   - Byte 2: 初始位缓冲区
2. 验证字典大小 (必须是 4, 5, 或 6)
3. 初始化解压缩器
4. 生成解码表
5. 调用 expand() 解压
6. 返回解压数据

---

## ⚠️ 关键实现细节

### 1. 位缓冲区管理
```rust
// 位缓冲区是 16 位，从低位开始读取
// 每次读取新字节时，放在高 8 位
bit_buf |= (new_byte << 8);
bit_buf >>= bits_to_skip;
extra_bits = (extra_bits - bits_to_skip) + 8;
```

### 2. 输出循环缓冲区
- 大小: 8KB (0x2000)
- 起始位置: 0x1000
- 当位置 >= 0x2000 时，刷新前 4KB，保留后 4KB

### 3. 复制操作
```rust
// 从 (out_pos - distance) 复制 length 字节到 out_pos
// 注意：source 和 target 可能重叠！
let source_pos = out_pos - distance;
for i in 0..length {
    out_buf[out_pos + i] = out_buf[source_pos + i];
}
```

### 4. 错误处理
- 字典大小无效 (不是 4, 5, 6)
- 压缩类型无效 (不是 0 或 1)
- 数据不足
- 解码过程中的错误 (返回 0x306)

---

## 📝 实现步骤

### Step 1: 定义常量和数据结构 (50 行)
- [ ] 定义所有查找表常量
- [ ] 定义 PkzipDecompressor 结构
- [ ] 定义 DecompressionContext 结构

### Step 2: 实现位操作 (80 行)
- [ ] `skip_bit()`
- [ ] `read_input()`
- [ ] 位缓冲区初始化

### Step 3: 实现解码表生成 (120 行)
- [ ] `generate_tables_decode()`
- [ ] `generate_tables_ascii()`
- [ ] 初始化函数（复制静态表到结构体）

### Step 4: 实现解码器 (200 行)
- [ ] `decode_literal()`
  - [ ] Binary 模式分支
  - [ ] ASCII 模式分支
- [ ] `decode_distance()`

### Step 5: 实现主循环 (150 行)
- [ ] `expand()`
  - [ ] 字面值处理
  - [ ] 重复块处理
  - [ ] 缓冲区刷新
- [ ] 输出数据管理

### Step 6: 公共接口和测试 (100 行)
- [ ] `decompress()` 公共函数
- [ ] 错误处理
- [ ] 单元测试

**总计**: 约 700 行代码

---

## 🧪 测试策略

### 测试 1: town.pal
```rust
#[test]
fn test_decompress_town_pal() {
    let mut manager = MpqManager::new();
    manager.load_mpq("Diabdat.mpq", 1000).unwrap();
    
    let data = manager.find_file("levels/towndata/town.pal").unwrap();
    assert_eq!(data.len(), 768); // 256 colors * 3 bytes
}
```

### 测试 2: 其他资源文件
- 测试 PCX 图像
- 测试 CEL 动画
- 测试其他压缩文件

### 测试 3: 边界情况
- 最小压缩文件
- 大文件
- 不同压缩模式

---

## 🎯 预期结果

成功实现后：
- ✅ 能够解压 Diablo 1 的所有 MPQ 文件
- ✅ 通过 town.pal 测试
- ✅ 调色板系统可以正常工作
- ✅ Step 5.1 完整完成

---

## 📚 参考资料

1. **libmpq explode.c**
   - 路径: `G:\DevilutionX\3rdParty\libmpq\source\libmpq\libmpq\explode.c`
   - 作者: Maik Broemme, Ladislav Zezula
   - 基于: PKWARE Data Compression Library

2. **PKWare 专利**
   - Patent No. 5,051,745
   - "PKWARE Data Compression Library for Win32"

3. **算法说明**
   - LZ77 变体 + Shannon-Fano 编码
   - 字典大小: 1KB/2KB/4KB
   - 两种模式: Binary / ASCII

---

## ⏱️ 预计时间

- **实现**: 4-6 小时
- **测试**: 1-2 小时
- **调试**: 2-4 小时
- **文档**: 1 小时

**总计**: 8-13 小时

---

**计划版本**: v1.0  
**创建时间**: 2025-11-24  
**状态**: 准备开始实现


























