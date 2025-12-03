# 解码器验证指南

## 🎯 目标

确保Rust实现的6种TileType解码器与原版C++输出**完全一致**。

## 📋 验证策略

### 策略1：特征验证（必须）
验证解码后的数据符合TileType的几何特征（行宽、对齐、总大小等）

### 策略2：视觉验证（推荐）
将解码后的数据渲染成图像，目视检查是否正确

### 策略3：原版对比（可选）
如果能获取原版C++的解码输出，进行字节级对比

---

## 🔬 策略1：特征验证（核心方法）

### 原理

每种TileType都有**独特的几何特征**，可以用代码验证：

| TileType | 尺寸 | 特征 |
|---------|------|------|
| Square | 32×32 | 所有行宽度=32，无透明边缘 |
| LeftTriangle | 32×31 | 行宽递增→递减，左对齐 |
| RightTriangle | 32×31 | 行宽递增→递减，右对齐 |
| LeftTrapezoid | 32×32 | 下半部变宽（左对齐），上半部固定32 |
| RightTrapezoid | 32×32 | 下半部变宽（右对齐），上半部固定32 |
| TransparentSquare | 32×32 | 包含透明像素，总像素≤1024 |

### 实施步骤

#### 步骤1：创建特征验证工具

**文件：** `tests/decoder_validator.rs`

```rust
//! 解码器特征验证工具

use rust_diablo::tiles::types::TileType;

/// 验证解码后的像素数据是否符合TileType的几何特征
pub struct DecoderValidator;

impl DecoderValidator {
    /// 统计每一行的非透明像素数量
    fn count_row_pixels(pixels: &[u8], row: usize, width: usize) -> usize {
        let start = row * width;
        let end = start + width;
        pixels[start..end].iter().filter(|&&p| p != 0).count()
    }
    
    /// 统计每一行从左边开始的连续透明像素数量
    fn count_left_transparent(pixels: &[u8], row: usize, width: usize) -> usize {
        let start = row * width;
        let end = start + width;
        pixels[start..end].iter().take_while(|&&p| p == 0).count()
    }
    
    /// 验证Square：所有行都是满的
    pub fn validate_square(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 32 {
            return Err(format!("Square size wrong: {}, expected 1024", pixels.len()));
        }
        
        for row in 0..32 {
            let count = Self::count_row_pixels(pixels, row, 32);
            if count != 32 {
                return Err(format!("Square row {} not full: {} pixels (expected 32)", 
                    row, count));
            }
        }
        
        Ok(())
    }
    
    /// 验证LeftTriangle：变宽行，左对齐
    pub fn validate_left_triangle(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 31 {
            return Err(format!("LeftTriangle size wrong: {}, expected 992", pixels.len()));
        }
        
        // 下半部分（行0-15）：宽度递增
        let expected_widths_lower = vec![
            2, 4,   // i=0
            6, 8,   // i=1
            10, 12, // i=2
            14, 16, // i=3
            18, 20, // i=4
            22, 24, // i=5
            26, 28, // i=6
            30, 32, // i=7
        ];
        
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "LeftTriangle row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
            
            // 验证左对齐（透明像素应该在右边）
            let left_trans = Self::count_left_transparent(pixels, row, 32);
            if left_trans != 0 {
                return Err(format!(
                    "LeftTriangle row {} not left-aligned: {} left transparent pixels",
                    row, left_trans
                ));
            }
        }
        
        // 上半部分（行16-30）：宽度递减
        let expected_widths_upper = vec![
            30, 28, // row 16-17
            26, 24, // row 18-19
            22, 20, // row 20-21
            18, 16, // row 22-23
            14, 12, // row 24-25
            10, 8,  // row 26-27
            6, 4,   // row 28-29
            2,      // row 30
        ];
        
        for (i, &expected_width) in expected_widths_upper.iter().enumerate() {
            let row = 16 + i;
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "LeftTriangle row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
        }
        
        Ok(())
    }
    
    /// 验证RightTriangle：变宽行，右对齐
    pub fn validate_right_triangle(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 31 {
            return Err(format!("RightTriangle size wrong: {}, expected 992", pixels.len()));
        }
        
        // 下半部分（行0-15）：宽度递增，右对齐
        let expected_widths_lower = vec![
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ];
        
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "RightTriangle row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
            
            // 验证右对齐（透明像素应该在左边）
            let left_trans = Self::count_left_transparent(pixels, row, 32);
            let expected_left_trans = 32 - expected_width;
            if left_trans != expected_left_trans {
                return Err(format!(
                    "RightTriangle row {} not right-aligned: {} left transparent (expected {})",
                    row, left_trans, expected_left_trans
                ));
            }
        }
        
        Ok(())
    }
    
    /// 验证LeftTrapezoid：下三角（变宽）+ 上矩形（固定32）
    pub fn validate_left_trapezoid(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 32 {
            return Err(format!("LeftTrapezoid size wrong: {}, expected 1024", pixels.len()));
        }
        
        // 下半部分（行0-15）：LeftTriangle的下半部
        let expected_widths_lower = vec![
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ];
        
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "LeftTrapezoid row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
        }
        
        // 上半部分（行16-31）：32x16纯矩形
        for row in 16..32 {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != 32 {
                return Err(format!(
                    "LeftTrapezoid row {} not full: {} pixels (expected 32)",
                    row, actual
                ));
            }
        }
        
        Ok(())
    }
    
    /// 验证RightTrapezoid：下三角（变宽）+ 上矩形（固定32）
    pub fn validate_right_trapezoid(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 32 {
            return Err(format!("RightTrapezoid size wrong: {}, expected 1024", pixels.len()));
        }
        
        // 下半部分（行0-15）：RightTriangle的下半部（右对齐）
        let expected_widths_lower = vec![
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ];
        
        for (row, &expected_width) in expected_widths_lower.iter().enumerate() {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != expected_width {
                return Err(format!(
                    "RightTrapezoid row {} width wrong: {} (expected {})",
                    row, actual, expected_width
                ));
            }
            
            // 验证右对齐
            let left_trans = Self::count_left_transparent(pixels, row, 32);
            let expected_left_trans = 32 - expected_width;
            if left_trans != expected_left_trans {
                return Err(format!(
                    "RightTrapezoid row {} not right-aligned",
                    row
                ));
            }
        }
        
        // 上半部分（行16-31）：32x16纯矩形
        for row in 16..32 {
            let actual = Self::count_row_pixels(pixels, row, 32);
            if actual != 32 {
                return Err(format!(
                    "RightTrapezoid row {} not full: {} pixels",
                    row, actual
                ));
            }
        }
        
        Ok(())
    }
    
    /// 验证TransparentSquare：32x32，可能包含透明像素
    pub fn validate_transparent_square(pixels: &[u8]) -> Result<(), String> {
        if pixels.len() != 32 * 32 {
            return Err(format!("TransparentSquare size wrong: {}, expected 1024", 
                pixels.len()));
        }
        
        // TransparentSquare应该至少有一些透明像素
        let transparent_count = pixels.iter().filter(|&&p| p == 0).count();
        if transparent_count == 0 {
            return Err("TransparentSquare has no transparent pixels".to_string());
        }
        
        println!("  TransparentSquare: {} transparent, {} opaque pixels",
            transparent_count, 1024 - transparent_count);
        
        Ok(())
    }
    
    /// 统一验证接口
    pub fn validate(tile_type: TileType, pixels: &[u8]) -> Result<(), String> {
        match tile_type {
            TileType::Square => Self::validate_square(pixels),
            TileType::LeftTriangle => Self::validate_left_triangle(pixels),
            TileType::RightTriangle => Self::validate_right_triangle(pixels),
            TileType::LeftTrapezoid => Self::validate_left_trapezoid(pixels),
            TileType::RightTrapezoid => Self::validate_right_trapezoid(pixels),
            TileType::TransparentSquare => Self::validate_transparent_square(pixels),
        }
    }
}
```

#### 步骤2：创建MPQ验证测试

**文件：** `tests/mpq_decoder_validation.rs`

```rust
//! 使用真实MPQ数据验证所有解码器

use rust_diablo::resources::ResourceManager;
use rust_diablo::tiles::{MinData, DungeonType};
use rust_diablo::tiles::decoder::decode_tile; // 你实现的解码器

mod decoder_validator;
use decoder_validator::DecoderValidator;

/// 创建ResourceManager（尝试多个MPQ路径）
fn create_rm_with_mpq() -> Option<ResourceManager> {
    let paths = vec![
        "DIABDAT.MPQ",
        "../DIABDAT.MPQ",
        "../../DIABDAT.MPQ",
        std::env::var("DIABDAT_MPQ_PATH").ok()?,
    ];
    
    for path in paths {
        if let Ok(rm) = ResourceManager::new(vec![(&path, 1000)]) {
            println!("✓ Found MPQ at: {}", path);
            return Some(rm);
        }
    }
    None
}

#[test]
#[ignore]
fn test_validate_all_cathedral_tiles() {
    println!("\n=== 验证Cathedral所有瓦片解码器 ===\n");
    
    // 1. 加载MPQ
    let mut rm = create_rm_with_mpq()
        .expect("MPQ not found. Set DIABDAT_MPQ_PATH or place DIABDAT.MPQ in root.");
    
    // 2. 加载CEL数据
    let cel_frames = rm.load_dungeon_cel("levels/l1data/l1.cel")
        .expect("Failed to load l1.cel");
    
    println!("✓ Loaded {} CEL frames", cel_frames.len());
    
    // 3. 加载MIN数据（包含TileType映射）
    let min_data = MinData::from_mpq(
        rm.mpq_manager_mut(),
        "levels/l1data/l1.min"
    ).expect("Failed to load l1.min");
    
    println!("✓ Loaded {} MIN entries\n", min_data.len());
    
    // 4. 统计各类型的瓦片数量
    let mut type_counts = std::collections::HashMap::new();
    let mut validated_counts = std::collections::HashMap::new();
    let mut error_counts = std::collections::HashMap::new();
    
    // 5. 遍历所有有效的frame
    for i in 1..min_data.len() {
        let block = min_data.get(i).unwrap();
        
        if !block.has_value() {
            continue;
        }
        
        let tile_type = block.tile_type();
        let frame_idx = block.frame();
        
        *type_counts.entry(tile_type).or_insert(0) += 1;
        
        // 获取原始CEL数据
        if frame_idx >= cel_frames.len() {
            eprintln!("⚠ Frame {} out of range (max={})", frame_idx, cel_frames.len());
            continue;
        }
        
        let cel_frame = &cel_frames[frame_idx];
        
        // 解码
        match decode_tile(tile_type, &cel_frame.raw_data) {
            Ok(decoded_pixels) => {
                // 验证特征
                match DecoderValidator::validate(tile_type, &decoded_pixels) {
                    Ok(_) => {
                        *validated_counts.entry(tile_type).or_insert(0) += 1;
                    }
                    Err(e) => {
                        eprintln!("✗ Validation failed for frame {} ({:?}): {}", 
                            frame_idx, tile_type, e);
                        *error_counts.entry(tile_type).or_insert(0) += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ Decode failed for frame {} ({:?}): {}", 
                    frame_idx, tile_type, e);
                *error_counts.entry(tile_type).or_insert(0) += 1;
            }
        }
    }
    
    // 6. 打印统计结果
    println!("\n=== 验证结果统计 ===\n");
    
    use rust_diablo::tiles::types::TileType;
    let all_types = vec![
        TileType::Square,
        TileType::LeftTriangle,
        TileType::RightTriangle,
        TileType::LeftTrapezoid,
        TileType::RightTrapezoid,
        TileType::TransparentSquare,
    ];
    
    let mut total_tiles = 0;
    let mut total_validated = 0;
    let mut total_errors = 0;
    
    for tile_type in all_types {
        let count = type_counts.get(&tile_type).unwrap_or(&0);
        let validated = validated_counts.get(&tile_type).unwrap_or(&0);
        let errors = error_counts.get(&tile_type).unwrap_or(&0);
        
        total_tiles += count;
        total_validated += validated;
        total_errors += errors;
        
        let status = if errors == &0 { "✓" } else { "✗" };
        
        println!("{} {:20} - Total: {:4}, Validated: {:4}, Errors: {:4}", 
            status, format!("{:?}", tile_type), count, validated, errors);
    }
    
    println!("\n{}", "=".repeat(70));
    println!("总计: {} 瓦片, {} 验证成功, {} 错误", 
        total_tiles, total_validated, total_errors);
    println!("{}\n", "=".repeat(70));
    
    // 7. 断言：所有瓦片都应该验证成功
    assert_eq!(total_errors, 0, 
        "解码器验证失败！有 {} 个瓦片未通过几何特征验证", total_errors);
    
    println!("🎉 所有解码器验证通过！");
}

#[test]
#[ignore]
fn test_validate_specific_frames() {
    println!("\n=== 验证特定Frame ===\n");
    
    let mut rm = create_rm_with_mpq()
        .expect("MPQ not found");
    
    let cel_frames = rm.load_dungeon_cel("levels/l1data/l1.cel").unwrap();
    let min_data = MinData::from_mpq(
        rm.mpq_manager_mut(),
        "levels/l1data/l1.min"
    ).unwrap();
    
    // 选择几个有代表性的frame进行详细验证
    let test_cases = vec![
        (1, "第1个有效frame"),
        (10, "第10个frame"),
        (50, "第50个frame"),
        (100, "第100个frame"),
    ];
    
    for (micro_idx, desc) in test_cases {
        if micro_idx >= min_data.len() {
            continue;
        }
        
        let block = min_data.get(micro_idx).unwrap();
        if !block.has_value() {
            continue;
        }
        
        let tile_type = block.tile_type();
        let frame_idx = block.frame();
        
        println!("测试 {} (Micro={}, Frame={}, Type={:?})", 
            desc, micro_idx, frame_idx, tile_type);
        
        let cel_frame = &cel_frames[frame_idx];
        let decoded = decode_tile(tile_type, &cel_frame.raw_data).unwrap();
        
        match DecoderValidator::validate(tile_type, &decoded) {
            Ok(_) => println!("  ✓ 验证通过"),
            Err(e) => {
                println!("  ✗ 验证失败: {}", e);
                panic!("Frame {} validation failed", frame_idx);
            }
        }
    }
    
    println!("\n✓ 所有测试Frame验证通过\n");
}
```

#### 步骤3：运行验证

```bash
# 设置MPQ路径
export DIABDAT_MPQ_PATH=/path/to/DIABDAT.MPQ

# 运行完整验证（所有Cathedral瓦片）
cargo test --test mpq_decoder_validation test_validate_all_cathedral_tiles -- --ignored --nocapture

# 运行特定Frame验证
cargo test --test mpq_decoder_validation test_validate_specific_frames -- --ignored --nocapture
```

---

## 🖼️ 策略2：视觉验证（推荐）

### 原理

将解码后的数据渲染成PNG图像，用肉眼检查是否正确。

### 实施步骤

#### 步骤1：创建图像导出工具

**文件：** `tests/visual_validator.rs`

```rust
//! 视觉验证工具 - 将解码后的瓦片保存为PNG

use image::{ImageBuffer, Rgba};
use rust_diablo::resources::Palette;

pub struct VisualValidator;

impl VisualValidator {
    /// 将解码后的索引色瓦片转换为PNG
    /// 
    /// # Arguments
    /// * `pixels` - 解码后的像素（索引色，0=透明）
    /// * `width` - 瓦片宽度
    /// * `height` - 瓦片高度
    /// * `palette` - 调色板
    /// * `output_path` - 输出PNG路径
    pub fn save_as_png(
        pixels: &[u8],
        width: u32,
        height: u32,
        palette: &Palette,
        output_path: &str,
    ) -> Result<(), String> {
        if pixels.len() != (width * height) as usize {
            return Err(format!(
                "Pixel count mismatch: {} != {}x{}",
                pixels.len(), width, height
            ));
        }
        
        let mut img = ImageBuffer::new(width, height);
        
        for (i, &index) in pixels.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            
            let rgba = if index == 0 {
                // 透明像素 -> 粉色（便于识别）
                [255, 0, 255, 100]
            } else {
                palette.to_rgba(index, false)
            };
            
            img.put_pixel(x, y, Rgba(rgba));
        }
        
        img.save(output_path)
            .map_err(|e| format!("Failed to save PNG: {}", e))?;
        
        Ok(())
    }
    
    /// 批量导出瓦片为PNG
    pub fn export_tiles(
        frames: &[(usize, TileType, Vec<u8>)],
        palette: &Palette,
        output_dir: &str,
    ) {
        std::fs::create_dir_all(output_dir).unwrap();
        
        for (i, (frame_idx, tile_type, pixels)) in frames.iter().enumerate() {
            let (width, height) = match tile_type {
                TileType::LeftTriangle | TileType::RightTriangle => (32, 31),
                _ => (32, 32),
            };
            
            let filename = format!(
                "{}/frame_{:04}_type_{:?}.png",
                output_dir, frame_idx, tile_type
            );
            
            if let Err(e) = Self::save_as_png(pixels, width, height, palette, &filename) {
                eprintln!("Failed to export frame {}: {}", frame_idx, e);
            } else {
                println!("  ✓ Exported: {}", filename);
            }
        }
    }
}
```

#### 步骤2：创建视觉验证测试

**文件：** `tests/visual_validation.rs`

```rust
#[test]
#[ignore]
fn test_export_sample_tiles() {
    println!("\n=== 导出样本瓦片为PNG ===\n");
    
    let mut rm = create_rm_with_mpq().expect("MPQ not found");
    
    // 加载数据
    let cel_frames = rm.load_dungeon_cel("levels/l1data/l1.cel").unwrap();
    let min_data = MinData::from_mpq(
        rm.mpq_manager_mut(),
        "levels/l1data/l1.min"
    ).unwrap();
    let palette = Palette::from_mpq(
        rm.mpq_manager_mut(),
        "levels/towndata/town.pal"
    ).unwrap();
    
    // 选择每种TileType的前3个sample
    let mut samples = Vec::new();
    let mut type_samples = std::collections::HashMap::new();
    
    for i in 1..min_data.len() {
        let block = min_data.get(i).unwrap();
        if !block.has_value() {
            continue;
        }
        
        let tile_type = block.tile_type();
        let count = type_samples.entry(tile_type).or_insert(0);
        
        if *count < 3 {
            let frame_idx = block.frame();
            let cel_frame = &cel_frames[frame_idx];
            let decoded = decode_tile(tile_type, &cel_frame.raw_data).unwrap();
            
            samples.push((frame_idx, tile_type, decoded));
            *count += 1;
        }
        
        // 所有类型都收集够3个sample后停止
        if type_samples.values().all(|&c| c >= 3) {
            break;
        }
    }
    
    // 导出PNG
    VisualValidator::export_tiles(&samples, &palette, "tests/output/visual_samples");
    
    println!("\n✓ 导出完成！查看 tests/output/visual_samples/ 目录");
    println!("  用图像查看器检查瓦片是否正确渲染\n");
}
```

#### 步骤3：检查输出

```bash
# 运行导出
cargo test --test visual_validation test_export_sample_tiles -- --ignored --nocapture

# 查看输出
ls tests/output/visual_samples/
# frame_0001_type_LeftTriangle.png
# frame_0002_type_RightTriangle.png
# frame_0010_type_Square.png
# ...

# 用图像查看器打开
# Windows: explorer tests\output\visual_samples
# Linux: xdg-open tests/output/visual_samples
# macOS: open tests/output/visual_samples
```

**检查要点：**
- LeftTriangle: 应该是左对齐的三角形
- RightTriangle: 应该是右对齐的三角形
- LeftTrapezoid: 下半部是三角形（左对齐），上半部是矩形
- RightTrapezoid: 下半部是三角形（右对齐），上半部是矩形
- Square: 完整的32x32矩形
- TransparentSquare: 有透明区域（粉色显示）

---

## 🔬 策略3：原版对比（可选，最严格）

### 原理

如果能获取原版C++解码后的输出，进行字节级对比。

### 方法A：使用DevilutionX导出

修改DevilutionX源码，添加导出功能：

```cpp
// 在Source/levels/gendung.cpp中添加
void ExportDecodedFrames() {
    for (int i = 0; i < nummicros; i++) {
        uint16_t blockValue = pMicroTiles[i];
        TileType tileType = GetTileType(blockValue);
        uint16_t frameIdx = GetFrame(blockValue);
        
        const uint8_t *frameData = GetDunFrame(pDungeonCels, frameIdx);
        
        // 根据tileType确定尺寸
        int width = 32;
        int height = (tileType == TileType::LeftTriangle || 
                      tileType == TileType::RightTriangle) ? 31 : 32;
        
        // 导出为二进制文件
        char filename[256];
        sprintf(filename, "frame_%04d_type_%d.bin", frameIdx, (int)tileType);
        FILE *f = fopen(filename, "wb");
        fwrite(frameData, 1, width * height, f);
        fclose(f);
    }
}
```

### 方法B：在Rust中对比

```rust
#[test]
#[ignore]
fn test_compare_with_reference_data() {
    // 假设已经从原版导出了参考数据
    let reference_dir = "tests/reference_data/";
    
    let mut rm = create_rm_with_mpq().expect("MPQ not found");
    let cel_frames = rm.load_dungeon_cel("levels/l1data/l1.cel").unwrap();
    let min_data = MinData::from_mpq(
        rm.mpq_manager_mut(),
        "levels/l1data/l1.min"
    ).unwrap();
    
    let mut mismatches = Vec::new();
    
    for i in 1..100 {  // 测试前100个frame
        let block = min_data.get(i).unwrap();
        if !block.has_value() {
            continue;
        }
        
        let frame_idx = block.frame();
        let tile_type = block.tile_type();
        
        // 读取参考数据
        let ref_file = format!("{}/frame_{:04d}_type_{}.bin", 
            reference_dir, frame_idx, tile_type as u8);
        
        if let Ok(ref_data) = std::fs::read(&ref_file) {
            // 解码
            let cel_frame = &cel_frames[frame_idx];
            let decoded = decode_tile(tile_type, &cel_frame.raw_data).unwrap();
            
            // 字节级对比
            if decoded != ref_data {
                mismatches.push((frame_idx, tile_type));
                
                // 详细对比
                for (i, (&a, &b)) in decoded.iter().zip(ref_data.iter()).enumerate() {
                    if a != b {
                        eprintln!(
                            "Frame {} mismatch at byte {}: Rust={}, C++={}", 
                            frame_idx, i, a, b
                        );
                    }
                }
            }
        }
    }
    
    assert_eq!(mismatches.len(), 0, 
        "Found {} frames with mismatches: {:?}", 
        mismatches.len(), mismatches);
}
```

---

## 📊 推荐的验证流程

### 开发阶段（快速迭代）

1. **单元测试** - 手工构造简单数据，测试解码器基础功能
2. **特征验证** - 使用`DecoderValidator`验证几何特征
3. **修复问题** - 根据错误信息调整解码逻辑

### 集成阶段（完整验证）

1. **MPQ特征验证** - 运行`test_validate_all_cathedral_tiles`
2. **视觉验证** - 导出PNG，目视检查
3. **调试问题瓦片** - 对有问题的frame单独分析

### 发布前（严格验证）

1. **全量测试** - 测试所有dungeon类型（Cathedral, Catacombs, Caves, Hell）
2. **原版对比**（可选）- 字节级对比
3. **性能测试** - 确保解码速度可接受

---

## 🎯 成功标准

✅ **特征验证100%通过**
- 所有TileType的几何特征都正确

✅ **视觉验证通过**
- 导出的PNG图像看起来正确
- 三角形对齐正确
- 梯形上下部分过渡自然

✅ **无崩溃/panic**
- 处理所有真实CEL数据不出错

✅ **（可选）原版对比100%一致**
- 字节级完全相同

---

## 💡 调试技巧

### 问题1：某个frame验证失败

```rust
// 添加详细日志
fn debug_frame(pixels: &[u8], width: usize, height: usize) {
    println!("Frame dump:");
    for y in 0..height {
        print!("Row {:2}: ", y);
        for x in 0..width {
            let p = pixels[y * width + x];
            if p == 0 {
                print!(".");
            } else {
                print!("#");
            }
        }
        println!();
    }
}
```

### 问题2：padding处理不正确

```rust
// 验证padding跳过
#[test]
fn test_triangle_padding() {
    // 手工构造包含padding的数据
    let mut raw = Vec::new();
    
    // 第一对行
    raw.extend_from_slice(&[0xFF, 0xFF]);  // padding
    raw.extend_from_slice(&[1, 2]);        // width=2
    raw.extend_from_slice(&[3, 4, 5, 6]);  // width=4
    // ...
    
    let decoded = decode_left_triangle(&raw).unwrap();
    
    // 验证padding被正确跳过
    assert_eq!(decoded[0], 1);  // 第一个有效像素
    assert_eq!(decoded[1], 2);
}
```

### 问题3：RLE解码错误

```rust
#[test]
fn test_rle_edge_cases() {
    // 测试边界情况
    let test_cases = vec![
        (vec![32i8 as u8, /* 32个像素 */], "整行实际像素"),
        (vec![-32i8 as u8], "整行透明"),
        (vec![10i8 as u8, /* 10像素 */, -22i8 as u8], "混合"),
    ];
    
    for (rle_data, desc) in test_cases {
        println!("Testing: {}", desc);
        let decoded = decode_transparent_square(&rle_data);
        assert!(decoded.is_ok(), "Failed: {}", desc);
    }
}
```

---

## 📝 总结

**推荐验证顺序：**

1. ✅ **单元测试**（手工数据） - 快速验证基础逻辑
2. ✅ **特征验证**（MPQ数据） - 验证所有真实瓦片
3. ✅ **视觉验证**（PNG导出） - 人眼确认正确性
4. ⭐ **原版对比**（可选） - 终极验证

**关键点：**
- 特征验证是**最实用**的方法，不需要原版输出
- 视觉验证帮助**快速发现**对齐等问题
- 原版对比最严格，但**不是必须**的

按照这个流程，可以确保你的解码器与原版100%兼容！🎉














