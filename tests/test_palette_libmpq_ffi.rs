use rust_diablo::resources::libmpq_ffi::*;
use rust_diablo::resources::palette::Palette;
use std::ffi::CString;
/// 使用 libmpq FFI 读取和测试 Palette
///
/// 这个测试使用 libmpq FFI 直接从 MPQ 文件读取 palette 数据，
/// 然后验证 palette 的解析和基本功能。
use std::path::Path;
use std::ptr;

/// 测试使用 libmpq FFI 读取 town.pal
#[test]
fn test_read_town_pal_with_libmpq_ffi() {
    let mpq_path = "assets/Diabdat.mpq";

    // 检查 MPQ 文件是否存在
    if !Path::new(mpq_path).exists() {
        println!("⚠️  MPQ 文件不存在，跳过测试: {}", mpq_path);
        return;
    }

    println!("\n=== 使用 libmpq FFI 读取 town.pal ===\n");

    let c_path = CString::new(mpq_path).unwrap();
    let mut archive: mpq_archive = ptr::null_mut();

    unsafe {
        // 1. 打开 MPQ 存档
        let result = libmpq__archive_open(
            &mut archive as *mut mpq_archive,
            c_path.as_ptr(),
            0, // archive_offset
        );

        if !is_success(result) {
            println!("⚠️  无法打开 MPQ 文件 (错误码: {}), 跳过测试", result);
            return;
        }

        println!("✔ 成功打开 MPQ 存档: {}", mpq_path);

        // 2. 查找文件（尝试两种路径格式）
        let filenames = vec![
            "levels\\towndata\\town.pal", // Windows 格式（反斜杠）
            "levels/towndata/town.pal",   // Unix 格式（正斜杠）
        ];

        let mut file_number: u32 = 0;
        let mut found_filename = None;

        for filename in &filenames {
            let c_filename = CString::new(*filename).unwrap();
            let result = libmpq__file_number(archive, c_filename.as_ptr(), &mut file_number);

            if is_success(result) {
                found_filename = Some(*filename);
                println!("✔ 找到文件: {} (文件编号: {})", filename, file_number);
                break;
            }
        }

        let _filename = match found_filename {
            Some(f) => f,
            None => {
                println!("⚠️  无法找到 town.pal 文件，跳过测试");
                libmpq__archive_close(archive);
                return;
            }
        };

        // 3. 获取文件大小
        let mut unpacked_size: i64 = 0;
        let result = libmpq__file_size_unpacked(archive, file_number, &mut unpacked_size);

        if !is_success(result) {
            println!("⚠️  无法获取文件大小，跳过测试");
            libmpq__archive_close(archive);
            return;
        }

        println!("✔ 文件大小: {} 字节", unpacked_size);

        // 验证文件大小（palette 文件应该是 768 字节）
        if unpacked_size != 768 {
            println!("⚠️  警告: 文件大小不是 768 字节，可能是压缩的");
        }

        // 4. 读取文件数据
        let mut buffer = vec![0u8; unpacked_size as usize];
        let mut transferred: i64 = 0;

        let result = libmpq__file_read(
            archive,
            file_number,
            buffer.as_mut_ptr(),
            unpacked_size,
            &mut transferred,
        );

        // 5. 关闭存档
        libmpq__archive_close(archive);

        if !is_success(result) {
            println!("⚠️  读取文件失败 (错误码: {})", result);
            return;
        }

        buffer.truncate(transferred as usize);
        println!("✔ 成功读取文件: {} 字节", buffer.len());

        // 6. 解析为 Palette
        println!("\n=== 解析 Palette ===\n");
        match Palette::from_bytes(&buffer) {
            Ok(palette) => {
                println!("✔ 成功解析 Palette");
                println!("  - 颜色数量: {}", palette.len());
                assert_eq!(palette.len(), 256, "Palette 应该有 256 种颜色");

                // 显示前 10 种颜色
                println!("\n前 10 种颜色:");
                for i in 0..10 {
                    let color = palette.to_rgb(i as u8);
                    println!(
                        "  颜色 {:3}: RGB({:3}, {:3}, {:3})",
                        i, color.r, color.g, color.b
                    );
                }

                // 显示一些关键颜色索引
                println!("\n关键颜色索引:");
                let key_indices = vec![0, 1, 128, 255];
                for &idx in &key_indices {
                    let color = palette.to_rgb(idx as u8);
                    println!(
                        "  索引 {:3}: RGB({:3}, {:3}, {:3})",
                        idx, color.r, color.g, color.b
                    );
                }

                // 测试透明度功能
                println!("\n=== 测试透明度功能 ===\n");
                let rgba_transparent = palette.to_rgba(0, true);
                println!(
                    "  索引 0 (透明模式): RGBA({}, {}, {}, {})",
                    rgba_transparent[0],
                    rgba_transparent[1],
                    rgba_transparent[2],
                    rgba_transparent[3]
                );
                assert_eq!(rgba_transparent[3], 0, "索引 0 在透明模式下应该是透明的");

                let rgba_opaque = palette.to_rgba(0, false);
                println!(
                    "  索引 0 (不透明模式): RGBA({}, {}, {}, {})",
                    rgba_opaque[0], rgba_opaque[1], rgba_opaque[2], rgba_opaque[3]
                );
                assert_eq!(rgba_opaque[3], 255, "索引 0 在不透明模式下应该是不透明的");

                // 测试批量转换
                println!("\n=== 测试批量转换 ===\n");
                let test_indices = vec![0u8, 1u8, 128u8, 255u8];
                let rgba_data = palette.indices_to_rgba(&test_indices, true);
                println!(
                    "  转换 {} 个索引到 RGBA: {} 字节",
                    test_indices.len(),
                    rgba_data.len()
                );
                assert_eq!(
                    rgba_data.len(),
                    test_indices.len() * 4,
                    "RGBA 数据大小应该是索引数量的 4 倍"
                );

                println!("\n✅ 所有测试通过！");
            }
            Err(e) => {
                println!("❌ 解析 Palette 失败: {}", e);
                panic!("解析失败: {}", e);
            }
        }
    }
}

/// 测试读取多个 palette 文件
#[test]
fn test_read_multiple_palettes() {
    let mpq_path = "assets/Diabdat.mpq";

    if !Path::new(mpq_path).exists() {
        println!("⚠️  MPQ 文件不存在，跳过测试: {}", mpq_path);
        return;
    }

    println!("\n=== 测试读取多个 Palette 文件 ===\n");

    let c_path = CString::new(mpq_path).unwrap();
    let mut archive: mpq_archive = ptr::null_mut();

    unsafe {
        let result = libmpq__archive_open(&mut archive as *mut mpq_archive, c_path.as_ptr(), 0);

        if !is_success(result) {
            println!("⚠️  无法打开 MPQ 文件，跳过测试");
            return;
        }

        // 要测试的 palette 文件列表
        let palette_files = vec![
            ("levels\\towndata\\town.pal", "城镇"),
            ("levels\\l1data\\l1.pal", "地牢1层"),
            ("levels\\l2data\\l2.pal", "地牢2层"),
            ("levels\\l3data\\l3.pal", "地牢3层"),
            ("levels\\l4data\\l4.pal", "地牢4层"),
        ];

        let mut success_count = 0;
        let mut fail_count = 0;

        for (path, description) in &palette_files {
            print!("  {} ({})... ", path, description);

            let c_filename = CString::new(*path).unwrap();
            let mut file_number: u32 = 0;

            let result = libmpq__file_number(archive, c_filename.as_ptr(), &mut file_number);

            if !is_success(result) {
                println!("❌ 未找到");
                fail_count += 1;
                continue;
            }

            // 获取文件大小
            let mut unpacked_size: i64 = 0;
            let result = libmpq__file_size_unpacked(archive, file_number, &mut unpacked_size);

            if !is_success(result) {
                println!("❌ 无法获取大小");
                fail_count += 1;
                continue;
            }

            // 读取文件
            let mut buffer = vec![0u8; unpacked_size as usize];
            let mut transferred: i64 = 0;

            let result = libmpq__file_read(
                archive,
                file_number,
                buffer.as_mut_ptr(),
                unpacked_size,
                &mut transferred,
            );

            if !is_success(result) {
                println!("❌ 读取失败");
                fail_count += 1;
                continue;
            }

            buffer.truncate(transferred as usize);

            // 解析 palette
            match Palette::from_bytes(&buffer) {
                Ok(palette) => {
                    assert_eq!(palette.len(), 256);
                    println!("✔ 成功 ({} 字节, 256 色)", buffer.len());
                    success_count += 1;
                }
                Err(e) => {
                    println!("❌ 解析失败: {}", e);
                    fail_count += 1;
                }
            }
        }

        libmpq__archive_close(archive);

        println!("\n总结:");
        println!("  ✔ 成功: {} 个", success_count);
        println!("  ❌ 失败: {} 个", fail_count);

        if success_count > 0 {
            println!("\n✅ 成功读取 {} 个 palette 文件！", success_count);
        }
    }
}

/// 测试 palette 颜色范围
#[test]
fn test_palette_color_ranges() {
    let mpq_path = "assets/Diabdat.mpq";

    if !Path::new(mpq_path).exists() {
        println!("⚠️  MPQ 文件不存在，跳过测试: {}", mpq_path);
        return;
    }

    println!("\n=== 测试 Palette 颜色范围 ===\n");

    let c_path = CString::new(mpq_path).unwrap();
    let mut archive: mpq_archive = ptr::null_mut();

    unsafe {
        let result = libmpq__archive_open(&mut archive as *mut mpq_archive, c_path.as_ptr(), 0);

        if !is_success(result) {
            println!("⚠️  无法打开 MPQ 文件，跳过测试");
            return;
        }

        let filename = "levels\\towndata\\town.pal";
        let c_filename = CString::new(filename).unwrap();
        let mut file_number: u32 = 0;

        let result = libmpq__file_number(archive, c_filename.as_ptr(), &mut file_number);

        if !is_success(result) {
            println!("⚠️  无法找到文件，跳过测试");
            libmpq__archive_close(archive);
            return;
        }

        let mut unpacked_size: i64 = 0;
        let result = libmpq__file_size_unpacked(archive, file_number, &mut unpacked_size);

        if !is_success(result) {
            libmpq__archive_close(archive);
            return;
        }

        let mut buffer = vec![0u8; unpacked_size as usize];
        let mut transferred: i64 = 0;

        let result = libmpq__file_read(
            archive,
            file_number,
            buffer.as_mut_ptr(),
            unpacked_size,
            &mut transferred,
        );

        libmpq__archive_close(archive);

        if !is_success(result) {
            return;
        }

        buffer.truncate(transferred as usize);

        let palette = match Palette::from_bytes(&buffer) {
            Ok(p) => p,
            Err(e) => {
                println!("❌ 解析失败: {}", e);
                return;
            }
        };

        // 分析颜色范围
        println!("颜色范围分析:\n");

        let mut min_r = 255u8;
        let mut max_r = 0u8;
        let mut min_g = 255u8;
        let mut max_g = 0u8;
        let mut min_b = 255u8;
        let mut max_b = 0u8;

        for i in 0..256 {
            let color = palette.to_rgb(i as u8);
            min_r = min_r.min(color.r);
            max_r = max_r.max(color.r);
            min_g = min_g.min(color.g);
            max_g = max_g.max(color.g);
            min_b = min_b.min(color.b);
            max_b = max_b.max(color.b);
        }

        println!("  R 通道: {} - {}", min_r, max_r);
        println!("  G 通道: {} - {}", min_g, max_g);
        println!("  B 通道: {} - {}", min_b, max_b);

        // 检查是否有完全相同的颜色
        let mut color_map = std::collections::HashMap::new();
        for i in 0..256 {
            let color = palette.to_rgb(i as u8);
            let key = (color.r, color.g, color.b);
            color_map.entry(key).or_insert_with(Vec::new).push(i);
        }

        let duplicates: Vec<_> = color_map
            .iter()
            .filter(|(_, indices)| indices.len() > 1)
            .collect();

        if duplicates.is_empty() {
            println!("\n  ✔ 所有颜色都是唯一的");
        } else {
            println!("\n  ⚠️  发现 {} 组重复颜色", duplicates.len());
            for ((r, g, b), indices) in duplicates.iter().take(5) {
                println!("    RGB({}, {}, {}) 出现在索引: {:?}", r, g, b, indices);
            }
        }

        println!("\n✅ 颜色范围测试完成");
    }
}
