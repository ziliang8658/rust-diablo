# MPQ Block Flags 分析

## 问题根源

`mpq-rust` 库报错：
```
if self.block.flags & FILE_PATCH_FILE != 0 {
    Err(Error::new(ErrorKind::Other, "Patch file not supported"))
}
```

## Diablo 1 MPQ Flags (原版定义)

根据 `Source/mpq/mpq_common.hpp`：

```cpp
struct MpqBlockEntry {
    static constexpr uint32_t FlagExists = 0x80000000;
    static constexpr uint32_t CompressPkZip = 0x00000100;
    
    uint32_t offset;
    uint32_t packedSize;
    uint32_t unpackedSize;
    uint32_t flags;  // ⬅️ 这里！
};
```

**Diablo 1 只使用两个 flag**：
- `0x80000000` - 文件存在
- `0x00000100` - PKWare 压缩

## 现代 MPQ Flags (mpq-rust 期望的)

现代 MPQ 格式（WoW, SC2 等）有更多 flags：
- `0x00000001` - FILE_IMPLODE (PKWare implode)
- `0x00000002` - FILE_COMPRESS (多种压缩)
- `0x00000100` - FILE_ENCRYPTED
- `0x00000200` - FILE_FIX_KEY
- `0x00010000` - FILE_PATCH_FILE ⬅️ **就是这个！**
- `0x00020000` - FILE_SINGLE_UNIT
- `0x80000000` - FILE_EXISTS

## 关键问题

**Diablo 1 的 flags 值可能与现代格式冲突！**

比如 Diablo 1 某个文件的 flags 可能是 `0x80000100`：
- Diablo 1 解释：存在 + PKWare 压缩
- mpq-rust 解释：可能误判为包含 FILE_PATCH_FILE 或其他现代 flags

## 解决方案

### 方案 1: Fork mpq-rust 并修改 ✂️
```rust
// 移除 patch file 检查
// if self.block.flags & FILE_PATCH_FILE != 0 {
//     return Err(...);
// }

// 或者只检查 Diablo 1 支持的 flags
const DIABLO1_FLAG_EXISTS: u32 = 0x80000000;
const DIABLO1_FLAG_COMPRESSED: u32 = 0x00000100;

if self.block.flags & DIABLO1_FLAG_EXISTS == 0 {
    return Err("File does not exist");
}
```

### 方案 2: 完全原生实现 🏗️
不依赖 `mpq-rust`，自己实现所有逻辑：
1. 读取 Header/Hash/Block Table（已有代码）
2. 解密表格（已实现 `decrypt_block`）
3. 读取文件数据
4. 实现 PKWare 解压

这样可以完全控制 flags 的解释。

### 方案 3: 使用 libmpq 的 Rust 绑定 🔗
找或创建 libmpq 的 FFI 绑定，因为 libmpq 支持 Diablo 1。

## 推荐方案

**我强烈推荐方案 2**：完全原生实现

**原因**：
1. ✅ **完全控制**：可以精确按照 Diablo 1 的格式实现
2. ✅ **学习价值**：这是学习项目，自己实现更有意义
3. ✅ **无依赖问题**：不受第三方库限制
4. ✅ **已有基础**：我们已经实现了大部分结构（`mpq_format.rs`）
5. ✅ **PKWare 可后置**：先实现无压缩文件读取，压缩的后续再处理

## 实现难度评估

### 已完成 ✅ (60%)
- MPQ 文件格式定义
- 加密表生成
- 哈希算法
- 解密算法
- Block/Hash Table 结构

### 需要完成 📝 (30%)
- 文件查找逻辑（100行代码）
- 文件读取逻辑（50行代码）
- 集成到 MpqManager（50行代码）

### 可选完成 ⏳ (10%)
- PKWare 解压（复杂，可以先跳过）
- 其他压缩格式

## 估计工作量

- **基础实现**（无压缩文件）：1-2小时
- **PKWare 解压**：3-5小时（或使用现有库）
- **测试和优化**：1-2小时

**总计**：大约半天工作量即可实现基础功能。

## 下一步

如果选择原生实现，立即开始的步骤：
1. 重新创建 `mpq_format.rs`（已有代码）
2. 在 `mpq.rs` 中实现 `MpqArchive` 结构体
3. 实现文件查找和读取
4. 测试读取 `town.pal`（768字节，无压缩）

