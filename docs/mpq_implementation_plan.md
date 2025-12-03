# MPQ 原生实现方案

## 问题确认

**现状**：
- ✅ `stormlib-rs` 无法编译/使用
- ✅ `mpq-rust` 可以打开 MPQ，但 `file.read()` 失败，报 `std::io::Error`
- ✅ 两个第三方库都无法正确处理 Diablo 1 的早期 MPQ 格式

**结论**：必须自己实现完整的 MPQ 读取功能

## 实现方案

### Phase 1: 核心数据结构 ✅
- [x] `mpq_format.rs` - MPQ 文件格式定义
  - MpqFileHeader
  - MpqHashEntry  
  - MpqBlockEntry
  - 加密表生成
  - 哈希算法
  - 解密算法

### Phase 2: MPQ Archive 读取 🔄
- [ ] `MpqArchive::open()` - 打开并读取 MPQ
  1. 读取 Header
  2. 读取并解密 Block Table
  3. 读取并解密 Hash Table
  
- [ ] `MpqArchive::find_file()` - 查找文件
  1. 计算文件名哈希
  2. 在 Hash Table 中查找（线性探测）
  3. 返回 Block Table 索引

- [ ] `MpqArchive::read_file()` - 读取文件数据
  1. 从 Block Table 获取文件信息
  2. 读取压缩数据
  3. 解密（如果需要）
  4. 解压（PKWare - 暂时返回错误）
  5. 返回数据

### Phase 3: PKWare 解压 ⏳
两个选择：
1. **使用 `libflate` crate** (推荐)
   - Diablo MPQ 的 PKWare 实际上可能是 DEFLATE 变种
   - 尝试用 `flate2` 或 `libflate` 解压
   
2. **移植 C 代码**
   - 从 `3rdParty/PKWare/explode.cpp` 移植
   - 复杂但保证兼容性

### Phase 4: 集成测试 ⏳
- [ ] 测试读取 `town.pal` (768字节，无压缩)
- [ ] 测试读取其他文件类型
- [ ] 性能测试

## 当前紧急任务

### 任务 1: 重新创建 `mpq_format.rs` ✅
已完成，包含：
- 完整的数据结构
- 加密表生成
- 哈希算法
- 解密算法

### 任务 2: 实现 `MpqArchive` 📝
需要在 `mpq.rs` 中实现原生的 `MpqArchive` 结构体。

### 任务 3: 移除 `mpq-rust` 依赖 📝
从 `Cargo.toml` 移除 `mpq` 依赖。

## 代码结构

```
rust-diablo/src/resources/
├── mod.rs              # 导出模块
├── mpq_format.rs       # MPQ 格式定义（已完成）
├── mpq.rs              # MPQ 管理器（需要重写）
└── palette.rs          # 调色板系统（已完成）
```

## 实现优先级

1. **P0 - 立即**：实现基本的文件读取（无压缩）
   - 这样至少可以读取 palette 文件（768字节，通常无压缩）
   
2. **P1 - 重要**：实现 PKWare 解压
   - 需要读取压缩的资源文件
   
3. **P2 - 可选**：优化性能
   - 缓存、并发等

## 测试策略

1. **单元测试**：测试每个函数
2. **集成测试**：测试完整流程
3. **真实数据**：使用 `Diabdat.mpq` 测试

## 预期结果

完成后应该能够：
- ✅ 打开 `Diabdat.mpq`
- ✅ 列出文件（如果有 listfile）
- ✅ 读取无压缩文件（如 `.pal`）
- ✅ 读取压缩文件（需要 PKWare）

