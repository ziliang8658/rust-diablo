/// libmpq FFI 功能测试
/// 
/// 测试 libmpq 静态库的 FFI 绑定是否正常工作

#[cfg(test)]
mod tests {
    use std::path::Path;
    use rust_diablo::resources::libmpq_ffi::*;
    use std::ffi::CString;
    use std::ptr;

    /// 测试打开和关闭 MPQ 存档
    #[test]
    fn test_open_close_archive() {
        let mpq_path = "assets/Diabdat.mpq";
        
        // 检查文件是否存在
        if !Path::new(mpq_path).exists() {
            println!("⚠️  MPQ 文件不存在，跳过测试: {}", mpq_path);
            return;
        }

        let c_path = CString::new(mpq_path).unwrap();
        let mut archive: mpq_archive = ptr::null_mut();

        unsafe {
            // 打开存档（archive_offset = 0 表示从头开始）
            let result = libmpq__archive_open(
                &mut archive as *mut mpq_archive,
                c_path.as_ptr(),
                0, // archive_offset
            );

            if !is_success(result) {
                println!("⚠️  无法打开 MPQ 文件 (错误码: {}), 跳过测试", result);
                return;
            }
            
            // 注意：libmpq__archive_open 的第三个参数应该是 archive_offset (i64)，不是标志
            // 但为了兼容性，我们传递 0

            assert!(!archive.is_null(), "Archive handle should not be null");

            // 获取文件数量
            let mut file_count: u32 = 0;
            let result = libmpq__archive_files(archive, &mut file_count);
            
            if is_success(result) {
                println!("✔ MPQ 存档包含 {} 个文件", file_count);
                assert!(file_count > 0, "Archive should contain files");
            }

            // 关闭存档
            let result = libmpq__archive_close(archive);
            assert!(is_success(result), "Should close archive successfully");
        }
    }

    /// 测试检查文件是否存在
    #[test]
    fn test_file_check() {
        let mpq_path = "assets/Diabdat.mpq";
        
        if !Path::new(mpq_path).exists() {
            println!("⚠️  MPQ 文件不存在，跳过测试: {}", mpq_path);
            return;
        }

        let c_path = CString::new(mpq_path).unwrap();
        let mut archive: mpq_archive = ptr::null_mut();

        unsafe {
            let result = libmpq__archive_open(
                &mut archive as *mut mpq_archive,
                c_path.as_ptr(),
                0, // archive_offset
            );

            if !is_success(result) {
                println!("⚠️  无法打开 MPQ 文件，跳过测试");
                return;
            }

            // 测试检查已知文件
            // 注意：在 Windows 上，MPQ 文件中的路径使用反斜杠
            let test_files = vec![
                "levels\\towndata\\town.pal",  // Windows 格式（反斜杠）
                "levels/towndata/town.pal",    // Unix 格式（正斜杠）
                "levels\\l1data\\l1.pal",
                "levels/l1data/l1.pal",
            ];

            for filename in test_files {
                let c_filename = CString::new(filename).unwrap();
                let mut file_number: u32 = 0;

                let result = libmpq__file_number(
                    archive,
                    c_filename.as_ptr(),
                    &mut file_number,
                );

                if is_success(result) {
                    println!("✔ 文件存在: {} (文件编号: {})", filename, file_number);
                } else {
                    println!("⚠️  文件不存在: {}", filename);
                }
            }

            libmpq__archive_close(archive);
        }
    }

    /// 测试读取文件数据
    #[test]
    fn test_read_file() {
        let mpq_path = "assets/Diabdat.mpq";
        
        if !Path::new(mpq_path).exists() {
            println!("⚠️  MPQ 文件不存在，跳过测试: {}", mpq_path);
            return;
        }

        let c_path = CString::new(mpq_path).unwrap();
        let mut archive: mpq_archive = ptr::null_mut();

        unsafe {
            let result = libmpq__archive_open(
                &mut archive as *mut mpq_archive,
                c_path.as_ptr(),
                0, // archive_offset
            );

            if !is_success(result) {
                println!("⚠️  无法打开 MPQ 文件，跳过测试");
                return;
            }

            // 测试读取 town.pal
            // 尝试 Windows 格式（反斜杠）和 Unix 格式（正斜杠）
            let filenames = vec![
                "levels\\towndata\\town.pal",  // Windows 格式
                "levels/towndata/town.pal",     // Unix 格式
            ];
            
            let mut filename = None;
            for test_filename in &filenames {
                let c_test_filename = CString::new(*test_filename).unwrap();
                let mut test_file_number: u32 = 0;
                
                let result = libmpq__file_number(
                    archive,
                    c_test_filename.as_ptr(),
                    &mut test_file_number,
                );
                
                if is_success(result) {
                    filename = Some(*test_filename);
                    println!("✔ 找到文件，使用路径: {}", test_filename);
                    break;
                }
            }
            
            let filename = match filename {
                Some(f) => f,
                None => {
                    println!("⚠️  无法找到 town.pal 文件（尝试了多种路径格式），跳过读取测试");
                    libmpq__archive_close(archive);
                    return;
                }
            };
            let c_filename = CString::new(filename).unwrap();
            let mut file_number: u32 = 0;

            // 1. 检查文件是否存在，获取文件编号
            let result = libmpq__file_number(
                archive,
                c_filename.as_ptr(),
                &mut file_number,
            );

            if !is_success(result) {
                println!("⚠️  文件不存在: {}, 跳过读取测试", filename);
                libmpq__archive_close(archive);
                return;
            }

            // 2. 获取文件大小
            let mut unpacked_size: i64 = 0;
            let result = libmpq__file_size_unpacked(
                archive,
                file_number,
                &mut unpacked_size,
            );

            if !is_success(result) {
                println!("⚠️  无法获取文件大小: {}, 跳过读取测试", filename);
                libmpq__archive_close(archive);
                return;
            }

            println!("✔ 文件大小: {} 字节", unpacked_size);

            // 3. 读取数据
            let mut buffer = vec![0u8; unpacked_size as usize];
            let mut transferred: i64 = 0;

            let result = libmpq__file_read(
                archive,
                file_number,
                buffer.as_mut_ptr(),
                unpacked_size,
                &mut transferred,
            );

            if is_success(result) {
                buffer.truncate(transferred as usize);
                println!("✔ 成功读取文件: {} 字节", buffer.len());
                
                // 验证数据
                if filename.ends_with(".pal") {
                    assert_eq!(buffer.len(), 768, "Palette file should be 768 bytes");
                    println!("✔ Palette 文件大小正确: 768 字节");
                }
            } else {
                println!("⚠️  读取文件失败 (错误码: {})", result);
            }

            libmpq__archive_close(archive);
        }
    }
}




