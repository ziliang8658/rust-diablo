# PNG纹理导出功能使用说明

## 功能说明

已添加PNG导出功能，可以将指定piece的所有block导出为PNG图片，方便直接查看纹理。

## 已添加的代码

### 1. 导出模块
**文件**: `rust-diablo/src/tiles/debug_export.rs`

**功能**:
- `export_rgba_to_png()` - 将RGBA数据导出为PNG
- `export_piece_blocks_to_png()` - 导出单个piece的所有block
- `export_multiple_pieces_to_png()` - 批量导出多个piece

### 2. 自动导出
**文件**: `rust-diablo/src/game.rs` (Line 806-843)

**导出的piece列表**:
- **C++桥上的piece**: [38, 39, 195, 198, 1125, 1126]
- **Rust桥上的piece**: [6, 7, 12, 13, 20, 22, 23, 30, 81, 83, 99, 187, 201]

## 编译和运行

### 步骤1: 关闭正在运行的程序

如果遇到编译错误 "拒绝访问"，说明rust-diablo.exe正在运行。

**请先关闭Rust程序窗口**。

### 步骤2: 编译

```bash
cd rust-diablo
cargo build
```

### 步骤3: 运行

```bash
cargo run
```

**程序启动时会自动导出纹理到**：
```
debug_textures/cpp_pieces/piece_XXXX_block_X.png
debug_textures/rust_pieces/piece_XXXX_block_X.png
```

### 步骤4: 查看导出的PNG

导出的文件结构：
```
debug_textures/
├── cpp_pieces/
│   ├── piece_0038_block_0.png
│   ├── piece_0038_block_1.png
│   ├── piece_0039_block_0.png
│   ├── ...
│   ├── piece_1125_block_0.png
│   └── piece_1126_block_1.png
└── rust_pieces/
    ├── piece_0006_block_0.png
    ├── piece_0006_block_1.png
    ├── piece_0007_block_0.png
    ├── ...
    ├── piece_0201_block_0.png
    └── piece_0201_block_1.png
```

## 输出示例

运行时会看到：

```
=== Exporting Piece Textures for Diagnosis ===

=== Exporting Piece 38 to PNG ===
  Block 0: type=LeftTriangle frame=457
  Block 1: type=RightTriangle frame=458
✓ Exported PNG: debug_textures/cpp_pieces/piece_0038_block_0.png
✓ Exported PNG: debug_textures/cpp_pieces/piece_0038_block_1.png
=== Exported 2 blocks from piece 38 ===

=== Exporting Piece 39 to PNG ===
  Block 0: type=LeftTriangle frame=459
  Block 1: type=RightTriangle frame=460
✓ Exported PNG: debug_textures/cpp_pieces/piece_0039_block_0.png
✓ Exported PNG: debug_textures/cpp_pieces/piece_0039_block_1.png
=== Exported 2 blocks from piece 39 ===

...

=== Texture Export Complete ===
```

## 查看和分析

### 1. 打开PNG文件

用任何图片查看器打开PNG：
- Windows照片查看器
- Paint.NET
- GIMP
- 或者在Cursor中直接打开

### 2. 识别桥的piece

**视觉特征**:
- 桥：木板纹理、横条纹、棕色
- 水：深蓝色、波纹
- 草地：绿色、杂草纹理

### 3. 对比C++和Rust相同piece

如果C++的piece=38是桥，查看：
- `debug_textures/cpp_pieces/piece_0038_block_0.png` (应该是桥纹理)
- Rust的piece是否也有相同的纹理？

### 4. 检查Rust的piece

在`debug_textures/rust_pieces/`中查看所有piece，找出：
- 哪个piece看起来像桥？
- 哪个piece显示为深蓝色或黑色（解码失败）？
- 哪个piece纹理不对？

## 预期结果

### 正常情况
- 所有PNG都应该有清晰的纹理
- LeftTriangle和RightTriangle应该是三角形
- Square应该是正方形
- 颜色应该是Town的色调（绿色/棕色/灰色）

### 异常情况
- **全黑/全透明** - 解码失败或数据全是0
- **深蓝色** - palette问题或颜色索引错误
- **噪点/乱码** - 解码算法错误
- **尺寸不对** - width/height计算错误

## 分析建议

1. **对比C++和Rust相同piece_id的纹理**
   - 如果不同，说明解码或palette有问题

2. **找出深蓝色的piece**
   - 查看这些piece的frame_idx
   - 检查解码日志是否有异常

3. **确认桥的piece_id**
   - 从PNG中识别出桥的纹理
   - 记录桥的piece_id列表

---

**创建日期**: 2024-12-04  
**目的**: PNG纹理导出功能使用说明




