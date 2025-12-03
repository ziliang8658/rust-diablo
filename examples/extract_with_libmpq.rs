/// 使用 libmpq FFI 提取文件
///
/// 用法: cargo run --example extract_with_libmpq

fn main() {
    use rust_diablo::resources::LibMpqArchive;
    use std::fs;
    use std::path::Path;

    println!("=== libmpq 文件提取工具 ===\n");

    // 打开 MPQ
    let archive = match LibMpqArchive::open("assets/Diabdat.mpq") {
        Ok(a) => {
            println!("✔ 成功打开: {}\n", a.path());
            a
        }
        Err(e) => {
            eprintln!("❌ 打开失败: {}", e);
            return;
        }
    };

    // 要提取的文件列表
    let files = vec![
        "levels/towndata/town.pal",
        "levels/l1data/l1.pal",
        "levels/l2data/l2.pal",
        "levels/l3data/l3.pal",
        "levels/l4data/l4.pal",
        "ui_art/title.pal",
        "plrgfx/warrior/wl/wl.pal",
        "plrgfx/rogue/rl/rl.pal",
        "plrgfx/sorceror/sl/sl.pal",
    ];

    let output_dir = "assets/extracted";
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    println!("提取文件:\n");

    let mut success_count = 0;
    let mut fail_count = 0;

    for filename in &files {
        print!("  {} ... ", filename);

        match archive.read_file(filename) {
            Ok(data) => {
                println!("✔ 成功 ({} 字节)", data.len());

                let output_path =
                    Path::new(output_dir).join(Path::new(filename).file_name().unwrap());

                if let Err(e) = fs::write(&output_path, &data) {
                    eprintln!("    保存失败: {}", e);
                    fail_count += 1;
                } else {
                    println!("    → 保存到: {}", output_path.display());
                    success_count += 1;
                }
            }
            Err(e) => {
                println!("❌ 失败: {}", e);
                fail_count += 1;
            }
        }
    }

    println!("\n=== 总结 ===");
    println!("✔ 成功: {} 个", success_count);
    println!("❌ 失败: {} 个", fail_count);
    println!("\n文件保存在: {}/", output_dir);
}
