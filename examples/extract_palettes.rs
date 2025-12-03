/// 使用 mpq-rust-patched 提取调色板文件
/// 
/// 用法: cargo run --example extract_palettes

use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MPQ 调色板提取工具 ===\n");
    
    // 打开 MPQ
    let mut archive = mpq::Archive::open("assets/Diabdat.mpq")?;
    println!("✔ 成功打开: assets/Diabdat.mpq\n");
    
    // 要提取的文件
    let files = vec![
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
    fs::create_dir_all(output_dir)?;
    
    println!("提取文件:\n");
    
    let mut success = 0;
    let mut failed = 0;
    
    for filename in &files {
        print!("  {} ... ", filename);
        
        match archive.open_file(filename) {
            Ok(file) => {
                let mut buffer = vec![0u8; file.size() as usize];
                
                match file.read(&mut archive, &mut buffer) {
                    Ok(bytes_read) => {
                        println!("✔ 成功 ({} 字节)", bytes_read);
                        
                        let output_path = Path::new(output_dir).join(
                            Path::new(filename).file_name().unwrap()
                        );
                        
                        fs::write(&output_path, &buffer[..bytes_read])?;
                        println!("    → 保存到: {}", output_path.display());
                        success += 1;
                    }
                    Err(e) => {
                        println!("❌ 读取失败: {}", e);
                        failed += 1;
                    }
                }
            }
            Err(e) => {
                println!("❌ 打开失败: {}", e);
                failed += 1;
            }
        }
    }
    
    println!("\n=== 总结 ===");
    println!("✔ 成功: {} 个", success);
    println!("❌ 失败: {} 个", failed);
    println!("\n文件保存在: {}/", output_dir);
    
    Ok(())
}


























