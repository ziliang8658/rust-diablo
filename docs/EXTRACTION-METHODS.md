# 调色板提取方法汇总

## 方法 1: MPQ Editor GUI（最简单）⭐⭐⭐⭐⭐

### 下载地址
- **Softpedia**: https://www.softpedia.com/get/Programming/File-Editors/Ladik-s-MPQ-Editor.shtml
- **备用**: https://www.wowmodding.net/files/file/149-ladiks-mpq-editor/

### 使用步骤
1. 下载并安装 MPQ Editor
2. 运行程序
3. File → Open → 选择 `G:\DevilutionX\rust-diablo\assets\Diabdat.mpq`
4. 在文件列表中找到需要的 `.pal` 文件
5. 选中文件 → 右键 → Extract
6. 保存到 `G:\DevilutionX\rust-diablo\assets\extracted\`

### 需要提取的文件
- `levels/towndata/town.pal` (768 bytes)
- `levels/l1data/l1.pal` (768 bytes)
- `levels/l2data/l2.pal` (768 bytes)
- `levels/l3data/l3.pal` (768 bytes)
- `levels/l4data/l4.pal` (768 bytes)

---

## 方法 2: mpqcli 命令行工具

### 下载
https://github.com/TheGrayDot/mpqcli/releases

### 使用
1. 下载 `mpqcli.exe` 到 `rust-diablo` 目录
2. 运行 `extract_palettes.bat`

或手动执行：
```bash
mpqcli extract assets\Diabdat.mpq levels/towndata/town.pal assets\extracted\town.pal
mpqcli extract assets\Diabdat.mpq levels/l1data/l1.pal assets\extracted\l1.pal
# ... 等等
```

---

## 方法 3: 在线 MPQ 查看器

如果无法下载工具，可以尝试：
1. 将 `Diabdat.mpq` 上传到在线 MPQ 查看器
2. 下载需要的 `.pal` 文件

**注意**: 谨慎使用在线工具，注意隐私安全。

---

## 方法 4: 从原版游戏复制

如果你有 Diablo 1 安装：
1. 找到游戏安装目录
2. palette 文件可能在某个缓存目录
3. 直接复制 `.pal` 文件

---

## 验证提取结果

提取完成后，运行：

```bash
cd rust-diablo
cargo test test_extracted_palettes -- --nocapture
```

应该看到：
```
✔ town.pal 测试通过！
✔ 成功: 5 个
```

每个 `.pal` 文件应该正好是 **768 字节** (256 色 × 3 字节 RGB)。

---

## 故障排除

### 文件大小不对
- 正确大小: 768 bytes
- 如果太小: 提取失败，重试
- 如果太大: 可能包含额外数据，需要手动裁剪

### 测试失败
```bash
# 查看文件大小
dir assets\extracted\*.pal

# 查看十六进制（前几个字节应该是 RGB 值）
# 使用文本编辑器或十六进制查看器
```

---

## 推荐顺序

1. **首选**: 方法 1 (MPQ Editor GUI) - 最直观
2. **备选**: 方法 2 (mpqcli) - 适合命令行用户
3. **最后**: 方法 4 (从游戏复制) - 如果有原版游戏

完成提取后，即可继续 Rust Diablo 的开发！🎮


























