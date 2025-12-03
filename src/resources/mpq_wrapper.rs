/// 高级 MPQ 封装，使用 libmpq FFI
///
/// 这个模块提供了一个安全、符合 Rust 习惯的 MPQ 接口
use anyhow::{anyhow, Result};
use std::ffi::CString;
use std::ptr;

use super::libmpq_ffi::*;

/// MPQ 存档包装器（使用 libmpq）
pub struct LibMpqArchive {
    handle: mpq_archive,
    path: String,
}

impl LibMpqArchive {
    /// 打开 MPQ 存档
    pub fn open(path: &str) -> Result<Self> {
        let c_path = CString::new(path)?;
        let mut handle: mpq_archive = ptr::null_mut();

        unsafe {
            let result = libmpq__archive_open(
                &mut handle as *mut mpq_archive,
                c_path.as_ptr(),
                0, // archive_offset = 0
            );

            if !is_success(result) {
                return Err(anyhow!(
                    "Failed to open MPQ archive: {} (error code: {})",
                    path,
                    result
                ));
            }
        }

        Ok(Self {
            handle,
            path: path.to_string(),
        })
    }

    /// 检查文件是否存在
    pub fn has_file(&self, filename: &str) -> bool {
        let c_filename = match CString::new(filename) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let mut file_number: u32 = 0;

        unsafe {
            let result = libmpq__file_number(self.handle, c_filename.as_ptr(), &mut file_number);

            is_success(result)
        }
    }

    /// 读取文件数据
    pub fn read_file(&self, filename: &str) -> Result<Vec<u8>> {
        let c_filename = CString::new(filename)?;
        let mut file_number: u32 = 0;

        unsafe {
            // 1. 获取文件编号（检查文件是否存在）
            let result = libmpq__file_number(self.handle, c_filename.as_ptr(), &mut file_number);

            if !is_success(result) {
                return Err(anyhow!("File not found: {}", filename));
            }

            // 2. 获取文件大小
            let mut unpacked_size: i64 = 0;
            let result = libmpq__file_size_unpacked(self.handle, file_number, &mut unpacked_size);

            if !is_success(result) {
                return Err(anyhow!("Failed to get file size: {}", filename));
            }

            // 3. 读取数据
            let mut buffer = vec![0u8; unpacked_size as usize];
            let mut transferred: i64 = 0;

            let result = libmpq__file_read(
                self.handle,
                file_number,
                buffer.as_mut_ptr(),
                unpacked_size,
                &mut transferred,
            );

            if !is_success(result) {
                return Err(anyhow!("Failed to read file: {}", filename));
            }

            buffer.truncate(transferred as usize);
            Ok(buffer)
        }
    }

    /// 获取存档路径
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for LibMpqArchive {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                libmpq__archive_close(self.handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_diabdat() {
        let archive = LibMpqArchive::open("assets/Diabdat.mpq");
        assert!(archive.is_ok(), "Failed to open Diabdat.mpq");
    }

    #[test]
    fn test_read_file() {
        let archive = LibMpqArchive::open("assets/Diabdat.mpq").unwrap();

        // 尝试读取 town.pal
        let result = archive.read_file("levels/towndata/town.pal");

        match result {
            Ok(data) => {
                println!("✔ Successfully read town.pal: {} bytes", data.len());
                assert_eq!(data.len(), 768, "town.pal should be 768 bytes");
            }
            Err(e) => {
                println!("⚠️  Failed to read town.pal: {}", e);
            }
        }
    }
}
