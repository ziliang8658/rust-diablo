/// MPQ 未压缩文件提取工具
/// 
/// 用法: cargo run --example extract_uncompressed

use rust_diablo::resources::mpq::MpqManager;
use std::fs;
use std::path::Path;

fn main() {
    let mut manager = MpqManager::new();
    
    println!("=== MPQ 未压缩文件提取工具 ===\n");
    
    // 加载 Diabdat.mpq
    match manager.load_mpq("Diabdat.mpq", 1000) {
        Ok(_) => println!("✔ 成功加载 Diabdat.mpq\n"),
        Err(e) => {
            eprintln!("❌ 无法加载 Diabdat.mpq: {}", e);
            return;
        }
    }
    
    // 常见的 palette 文件路径
    let palette_files = vec![
        "levels/towndata/town.pal",
        "levels/l1data/l1.pal",
        "levels/l2data/l2.pal",
        "levels/l3data/l3.pal",
        "levels/l4data/l4.pal",
        "ui_art/title.pal",
        "gendata/cut.pal",
        "plrgfx/warrior/wl/wl.pal",
        "plrgfx/rogue/rl/rl.pal",
        "plrgfx/sorceror/sl/sl.pal",
    ];
    
    let output_dir = "assets/extracted";
    fs::create_dir_all(output_dir).expect("Failed to create output directory");
    
    println!("检查文件状态:\n");
    
    let mut extracted_count = 0;
    let mut failed_count = 0;
    
    for file_path in &palette_files {
        print!("  {} ... ", file_path);
        
        // 检查文件是否存在
        if !manager.has_file(file_path) {
            println!("❌ 不存在");
            continue;
        }
        
        // 尝试读取
        match manager.find_file(file_path) {
            Some(data) => {
                println!("✔ 成功 ({} 字节)", data.len());
                
                // 提取到文件
                let output_path = Path::new(output_dir).join(
                    Path::new(file_path).file_name().unwrap()
                );
                
                match fs::write(&output_path, &data) {
                    Ok(_) => {
                        println!("    → 已保存到: {}", output_path.display());
                        extracted_count += 1;
                    }
                    Err(e) => {
                        eprintln!("    → 保存失败: {}", e);
                        failed_count += 1;
                    }
                }
            }
            None => {
                println!("❌ 读取失败（可能需要解压）");
                failed_count += 1;
            }
        }
    }
    
    println!("\n=== 总结 ===");
    println!("✔ 成功提取: {} 个文件", extracted_count);
    println!("❌ 失败: {} 个文件", failed_count);
    println!("\n提取的文件保存在: {}/", output_dir);
}


























