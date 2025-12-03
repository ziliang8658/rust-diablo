# 快速提取调色板文件指南

由于 Diablo 1 的 MPQ 使用了扇区压缩，需要特殊处理。最快的方法是使用现成工具。

## 方案 A: MPQ Editor（推荐）⭐⭐⭐⭐⭐

### 1. 下载工具

**MPQ Editor by Ladislav Zezula**
- 下载地址：http://www.zezula.net/en/mpq/download.html
- 或者：http://www.zezula.net/download/MPQEditor_en.zip

### 2. 使用步骤

1. 解压并运行 `MPQEditor.exe`
2. File → Open MPQ → 选择 `G:\DevilutionX\rust-diablo\assets\Diabdat.mpq`
3. 在文件列表中找到需要的 `.pal` 文件：
   - `levels/towndata/town.pal`
   - `levels/l1data/l1.pal`
   - `levels/l2data/l2.pal`
   - `levels/l3data/l3.pal`
   - `levels/l4data/l4.pal`
4. 右键 → Extract → 选择保存路径：`G:\DevilutionX\rust-diablo\assets\extracted\`

### 3. 验证

提取后应该有：
```
assets/extracted/
  ├── town.pal (768 bytes)
  ├── l1.pal (768 bytes)
  ├── l2.pal (768 bytes)
  ├── l3.pal (768 bytes)
  └── l4.pal (768 bytes)
```

每个文件都应该是 **768 字节**（256 色 × 3 RGB）。

## 方案 B: StormLib 命令行工具

如果你更喜欢命令行：

### 1. 下载 StormLib

从 https://github.com/ladislav-zezula/StormLib/releases 下载预编译版本。

### 2. 提取文件

```bash
StormLib.exe extract Diabdat.mpq levels/towndata/town.pal extracted/town.pal
StormLib.exe extract Diabdat.mpq levels/l1data/l1.pal extracted/l1.pal
# ... 等等
```

## 方案 C: 从 DevilutionX 编译版本复制

如果你已经编译了 DevilutionX：

1. 运行 DevilutionX 一次
2. 它会自动解压需要的文件到缓存目录
3. 从缓存复制到 rust-diablo 项目

## 验证提取结果

运行这个测试：

```rust
cargo test test_extracted_palettes -- --nocapture
```

## 下一步

提取完成后，可以：

1. 测试 Palette 系统读取这些文件
2. 继续开发游戏逻辑
3. 后续再完善扇区读取功能


























