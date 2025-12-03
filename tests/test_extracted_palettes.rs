/// 测试提取的调色板文件
/// 
/// 前提：需要先手动提取 palette 文件到 assets/extracted/

use rust_diablo::resources::palette::Palette;

#[test]
fn test_town_pal_extracted() {
    let path = "assets/extracted/town.pal";
    
    println!("\n=== 测试提取的 town.pal ===");
    println!("文件路径: {}", path);
    
    // 检查文件是否存在
    if !std::path::Path::new(path).exists() {
        println!("\n⚠️  文件不存在！");
        println!("请先使用 MPQ Editor 提取文件到 assets/extracted/");
        println!("参考: docs/QUICK-EXTRACT-GUIDE.md");
        return;
    }
    
    // 读取并解析
    match Palette::from_file(path) {
        Ok(palette) => {
            println!("✔ 成功读取 Palette");
            println!("  - 颜色数量: {}", palette.len());
            assert_eq!(palette.len(), 256, "应该有 256 种颜色");
            
            println!("\n前 10 种颜色:");
            for i in 0..10 {
                let color = palette.to_rgb(i);
                println!("  颜色 {:3}: R={:3}, G={:3}, B={:3}", 
                    i, color.r, color.g, color.b);
            }
            
            println!("\n✔ town.pal 测试通过！");
        }
        Err(e) => {
            panic!("❌ 读取失败: {}", e);
        }
    }
}

#[test]
fn test_all_extracted_palettes() {
    let palettes = vec![
        ("town.pal", "城镇"),
        ("l1.pal", "地牢1层"),
        ("l2.pal", "地牢2层"),
        ("l3.pal", "地牢3层"),
        ("l4.pal", "地牢4层"),
    ];
    
    println!("\n=== 测试所有提取的调色板 ===\n");
    
    let mut success = 0;
    let mut missing = 0;
    let mut failed = 0;
    
    for (filename, desc) in &palettes {
        let path = format!("assets/extracted/{}", filename);
        print!("  {} ({})... ", filename, desc);
        
        if !std::path::Path::new(&path).exists() {
            println!("⚠️  未提取");
            missing += 1;
            continue;
        }
        
        match Palette::from_file(&path) {
            Ok(palette) => {
                assert_eq!(palette.len(), 256);
                println!("✔ 通过 (256 色)");
                success += 1;
            }
            Err(e) => {
                println!("❌ 失败: {}", e);
                failed += 1;
            }
        }
    }
    
    println!("\n总结:");
    println!("  ✔ 成功: {} 个", success);
    println!("  ⚠️  未提取: {} 个", missing);
    println!("  ❌ 失败: {} 个", failed);
    
    if missing > 0 {
        println!("\n请使用 MPQ Editor 提取缺失的文件");
        println!("参考: docs/QUICK-EXTRACT-GUIDE.md");
    }
    
    if success > 0 {
        println!("\n✔ 至少有 {} 个 palette 可用，可以继续开发！", success);
    }
}


























