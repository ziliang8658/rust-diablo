//! FFI bindings for libmpq
//! 
//! 这个模块提供了 libmpq C 库的 Rust 绑定

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int, c_uint, c_void};

// ============================================================================
// 类型定义
// ============================================================================

pub type mpq_archive_s = c_void;
pub type mpq_archive = *mut mpq_archive_s;

// mpq_file 类型不再需要，使用文件编号直接读取

// ============================================================================
// 常量
// ============================================================================

/// 打开方式：只读
pub const LIBMPQ_FILE_TYPE_INT: c_uint = 0;

/// 打开现有存档
pub const LIBMPQ_OPEN_EXISTING: c_int = 0;

// ============================================================================
// 函数声明
// ============================================================================

#[link(name = "mpq", kind = "static")]
extern "C" {
    /// 打开 MPQ 存档
    /// 
    /// # 参数
    /// * `mpq_archive` - 输出的存档句柄指针
    /// * `mpq_filename` - MPQ 文件路径
    /// * `mpq_flags` - 打开标志
    /// 
    /// # 返回
    /// 成功返回 0，失败返回错误码
    pub fn libmpq__archive_open(
        mpq_archive: *mut mpq_archive,
        mpq_filename: *const c_char,
        archive_offset: i64,
    ) -> c_int;

    /// 关闭 MPQ 存档
    pub fn libmpq__archive_close(mpq_archive: mpq_archive) -> c_int;

    /// 获取存档中的文件数量
    pub fn libmpq__archive_files(
        mpq_archive: mpq_archive,
        number_of_files: *mut c_uint,
    ) -> c_int;

    /// 获取文件编号（检查文件是否存在）
    pub fn libmpq__file_number(
        mpq_archive: mpq_archive,
        filename: *const c_char,
        file_number: *mut c_uint,
    ) -> c_int;

    /// 获取文件的解压后大小
    pub fn libmpq__file_size_unpacked(
        mpq_archive: mpq_archive,
        file_number: c_uint,
        unpacked_size: *mut i64,
    ) -> c_int;

    /// 读取文件数据（使用文件编号）
    pub fn libmpq__file_read(
        mpq_archive: mpq_archive,
        file_number: c_uint,
        out_buf: *mut u8,
        out_size: i64,
        transferred: *mut i64,
    ) -> c_int;
}

// ============================================================================
// 错误码
// ============================================================================

pub const LIBMPQ_SUCCESS: c_int = 0;
pub const LIBMPQ_ERROR_OPEN: c_int = -1;
pub const LIBMPQ_ERROR_READ: c_int = -2;
pub const LIBMPQ_ERROR_NOT_FOUND: c_int = -3;

// ============================================================================
// 辅助函数
// ============================================================================

/// 检查返回值是否成功
#[inline]
pub fn is_success(result: c_int) -> bool {
    result == LIBMPQ_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types() {
        // 确保类型大小合理
        assert_eq!(std::mem::size_of::<mpq_archive>(), std::mem::size_of::<*mut c_void>());
    }
}






