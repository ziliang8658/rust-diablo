/// MPQ Archive Manager - 使用 libmpq FFI
/// 
/// 完全使用 libmpq C 库的 FFI 实现
/// 支持多 MPQ 文件优先级系统
/// 
/// 参考: Source/mpq/mpq_reader.cpp, Source/engine/assets.cpp

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::ffi::CString;
use std::ptr;

use super::libmpq_ffi::*;

// ============================================================================
// MpqArchive - 单个 MPQ 归档文件（FFI 实现）
// ============================================================================

/// MPQ 归档文件（使用 libmpq FFI）
/// 
/// 表示一个打开的 MPQ 文件
pub struct MpqArchive {
    /// libmpq 存档句柄
    handle: mpq_archive,
    /// 文件路径（用于调试）
    path: PathBuf,
}

impl MpqArchive {
    /// 打开 MPQ 归档文件
    /// 
    /// # 参数
    /// * `path` - MPQ 文件路径
    /// 
    /// # 返回
    /// `Ok(MpqArchive)` 如果成功打开，`Err` 如果失败
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();
        let c_path = CString::new(path_str.as_ref())?;
        let mut handle: mpq_archive = ptr::null_mut();
        
        unsafe {
            let result = libmpq__archive_open(
                &mut handle as *mut mpq_archive,
                c_path.as_ptr(),
                0, // archive_offset
            );
            
            if !is_success(result) {
                return Err(anyhow::anyhow!(
                    "Failed to open MPQ archive: {} (error code: {})",
                    path.display(),
                    result
                ));
            }
        }
        
        Ok(Self {
            handle,
            path: path.to_path_buf(),
        })
    }
    
    /// 查找文件并获取文件编号
    /// 
    /// # 参数
    /// * `filename` - 文件名（支持 Windows 和 Unix 路径格式）
    /// 
    /// # 返回
    /// 文件编号，如果未找到返回 None
    fn find_file_number(&self, filename: &str) -> Option<u32> {
        // 尝试两种路径格式（Windows 和 Unix）
        let filenames = vec![
            filename.replace('/', "\\"),  // Windows 格式（反斜杠）
            filename.to_string(),          // 原始格式
        ];
        
        for test_filename in &filenames {
            let c_filename = match CString::new(test_filename.as_str()) {
                Ok(s) => s,
                Err(_) => continue,
            };
            
            let mut file_number: u32 = 0;
            
            unsafe {
                let result = libmpq__file_number(
                    self.handle,
                    c_filename.as_ptr(),
                    &mut file_number,
                );
                
                if is_success(result) {
                    return Some(file_number);
                }
            }
        }
        
        None
    }
    
    /// 读取文件
    /// 
    /// # 参数
    /// * `filename` - 文件名
    /// 
    /// # 返回
    /// 文件内容（已解压），如果未找到返回 None
    pub fn read_file(&self, filename: &str) -> Result<Option<Vec<u8>>> {
        let file_number = match self.find_file_number(filename) {
            Some(num) => num,
            None => return Ok(None),
        };
        
        unsafe {
            // 获取文件大小
            let mut unpacked_size: i64 = 0;
            let result = libmpq__file_size_unpacked(
                self.handle,
                file_number,
                &mut unpacked_size,
            );
            
            if !is_success(result) {
                return Err(anyhow::anyhow!(
                    "Failed to get file size: {} (error code: {})",
                    filename,
                    result
                ));
            }
            
            // 读取数据
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
                return Err(anyhow::anyhow!(
                    "Failed to read file: {} (error code: {})",
                    filename,
                    result
                ));
            }
            
            buffer.truncate(transferred as usize);
            Ok(Some(buffer))
        }
    }
    
    /// 检查文件是否存在
    pub fn has_file(&self, filename: &str) -> bool {
        self.find_file_number(filename).is_some()
    }
    
    /// 获取归档路径
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// 获取文件数量
    pub fn file_count(&self) -> Result<u32> {
        unsafe {
            let mut count: u32 = 0;
            let result = libmpq__archive_files(self.handle, &mut count);
            
            if !is_success(result) {
                return Err(anyhow::anyhow!(
                    "Failed to get file count (error code: {})",
                    result
                ));
            }
            
            Ok(count)
        }
    }
}

impl Drop for MpqArchive {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                libmpq__archive_close(self.handle);
            }
        }
    }
}

// ============================================================================
// MpqManager - 多 MPQ 文件管理器
// ============================================================================

/// MPQ 管理器 - 管理多个 MPQ 归档文件
/// 
/// 支持多个 MPQ 文件的优先级系统
/// 查找文件时从高优先级到低优先级搜索
pub struct MpqManager {
    /// MPQ 归档列表，按优先级排序（低优先级在前）
    archives: Vec<MpqArchiveWrapper>,
}

struct MpqArchiveWrapper {
    archive: MpqArchive,
    priority: i32,
    path: PathBuf,
}

impl MpqManager {
    /// 创建新的 MPQ 管理器
    pub fn new() -> Self {
        Self {
            archives: Vec::new(),
        }
    }
    
    /// 加载 MPQ 文件
    /// 
    /// # 参数
    /// * `path` - MPQ 文件路径（如 "DIABDAT.MPQ" 或 "assets/Diabdat.mpq"）
    /// * `priority` - 优先级（数字越大优先级越高）
    /// 
    /// # 示例
    /// ```rust
    /// let mut mpq = MpqManager::new();
    /// mpq.load_mpq("Diabdat.mpq", 1000)?;
    /// ```
    pub fn load_mpq(&mut self, path: &str, priority: i32) -> Result<()> {
        // 搜索 MPQ 文件
        let full_path = Self::find_mpq_path(path)
            .ok_or_else(|| anyhow::anyhow!("MPQ file not found: {}", path))?;
        
        // 打开 MPQ 归档
        let archive = MpqArchive::open(&full_path)
            .with_context(|| format!("Failed to open MPQ archive: {}", full_path.display()))?;
        
        let file_count = archive.file_count().unwrap_or(0);
        println!("✓ Loaded MPQ: {} (priority: {}, files: {})", 
            full_path.display(), priority, file_count);
        
        // 添加到列表并排序
        self.archives.push(MpqArchiveWrapper {
            archive,
            priority,
            path: full_path,
        });
        
        // 按优先级排序（低优先级在前）
        self.archives.sort_by_key(|a| a.priority);
        
        Ok(())
    }
    
    /// 在所有加载的 MPQ 归档中查找文件
    /// 
    /// 从高优先级到低优先级搜索，返回第一个找到的文件
    /// 
    /// # 参数
    /// * `path` - 文件路径（如 "levels/towndata/town.pal" 或 "levels\\towndata\\town.pal"）
    /// 
    /// # 返回
    /// 文件内容，如果未找到返回 None
    pub fn find_file(&mut self, path: &str) -> Option<Vec<u8>> {
        // 从高优先级到低优先级搜索（反向迭代）
        for wrapper in self.archives.iter().rev() {
            match wrapper.archive.read_file(path) {
                Ok(Some(data)) => return Some(data),
                Ok(None) => continue,
                Err(e) => {
                    eprintln!("WARNING: Error reading {} from {}: {}", 
                        path, wrapper.path.display(), e);
                    continue;
                }
            }
        }
        None
    }
    
    /// 检查文件是否存在
    pub fn has_file(&mut self, path: &str) -> bool {
        for wrapper in self.archives.iter().rev() {
            if wrapper.archive.has_file(path) {
                return true;
            }
        }
        false
    }
    
    /// 查找 MPQ 文件路径
    /// 
    /// 在多个标准位置搜索 MPQ 文件
    fn find_mpq_path(filename: &str) -> Option<PathBuf> {
        let search_paths = Self::get_search_paths();
        
        for base_path in &search_paths {
            let mut full_path = base_path.join(filename);
            
            // 尝试原始文件名
            if full_path.exists() {
                return Some(full_path);
            }
            
            // 尝试小写
            full_path = base_path.join(filename.to_lowercase());
            if full_path.exists() {
                return Some(full_path);
            }
            
            // 尝试大写
            full_path = base_path.join(filename.to_uppercase());
            if full_path.exists() {
                return Some(full_path);
            }
        }
        
        None
    }
    
    /// 获取 MPQ 文件搜索路径列表
    fn get_search_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from("."),           // 当前目录
            PathBuf::from("assets"),      // assets 目录
            PathBuf::from(".."),          // 上级目录
            PathBuf::from("../assets"),   // 上级的 assets 目录
        ]
    }
    
    /// 获取已加载的 MPQ 数量
    pub fn archive_count(&self) -> usize {
        self.archives.len()
    }
    
    /// 获取已加载的归档列表（用于调试）
    pub fn get_loaded_archives(&self) -> Vec<String> {
        self.archives
            .iter()
            .map(|a| format!("{} (priority: {})", a.path.display(), a.priority))
            .collect()
    }
    
    /// 列出所有文件
    /// 
    /// 通过读取 `(listfile)` 文件来获取 MPQ 归档中的所有文件列表
    /// 从高优先级到低优先级搜索，合并所有找到的文件列表
    /// 
    /// # 返回
    /// 文件路径列表（已去重和排序）
    pub fn list_files(&mut self) -> Vec<String> {
        use std::collections::HashSet;
        
        let mut all_files = HashSet::new();
        
        // 从高优先级到低优先级搜索 (listfile)
        for wrapper in self.archives.iter().rev() {
            match wrapper.archive.read_file("(listfile)") {
                Ok(Some(listfile_data)) => {
                    // 解析 listfile（文本格式，每行一个文件路径）
                    if let Ok(listfile_text) = std::str::from_utf8(&listfile_data) {
                        for line in listfile_text.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                all_files.insert(trimmed.to_string());
                            }
                        }
                    }
                    // 找到第一个有效的 listfile 后，可以继续搜索其他归档以合并文件列表
                    // 或者在这里 break 只使用第一个找到的
                    // 这里选择合并所有归档的 listfile
                }
                Ok(None) => continue,
                Err(_) => continue,
            }
        }
        
        // 转换为排序的 Vec
        let mut files: Vec<String> = all_files.into_iter().collect();
        files.sort();
        files
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mpq_manager_creation() {
        let manager = MpqManager::new();
        assert_eq!(manager.archive_count(), 0);
    }
    
    #[test]
    fn test_load_diabdat_mpq() {
        let mut manager = MpqManager::new();
        
        // 尝试加载 DIABDAT.MPQ
        match manager.load_mpq("Diabdat.mpq", 1000) {
            Ok(_) => {
                assert_eq!(manager.archive_count(), 1);
                println!("✓ Successfully loaded DIABDAT.MPQ");
            }
            Err(e) => {
                println!("⚠ DIABDAT.MPQ not found (this is OK if file doesn't exist): {}", e);
            }
        }
    }
    
    #[test]
    #[test]
    fn test_find_town_pal() {
        let mut manager = MpqManager::new();
        
        if manager.load_mpq("Diabdat.mpq", 1000).is_ok() {
            // 尝试读取 town.pal（尝试两种路径格式）
            let paths = vec![
                "levels\\towndata\\town.pal",  // Windows 格式
                "levels/towndata/town.pal",    // Unix 格式
            ];
            
            for path in paths {
                if let Some(pal_data) = manager.find_file(path) {
                    println!("✓ Successfully read town.pal from {}: {} bytes", path, pal_data.len());
                    assert_eq!(pal_data.len(), 768, "Palette should be 768 bytes");
                    return;
                }
            }
            
            println!("✗ town.pal not found in DIABDAT.MPQ");
        }
    }
    
    #[test]
    #[test]
    fn test_has_file() {
        let mut manager = MpqManager::new();
        
        if manager.load_mpq("Diabdat.mpq", 1000).is_ok() {
            // 测试文件存在性检查（尝试两种路径格式）
            let exists_windows = manager.has_file("levels\\towndata\\town.pal");
            let exists_unix = manager.has_file("levels/towndata/town.pal");
            
            assert!(exists_windows || exists_unix, "town.pal should exist");
            assert!(!manager.has_file("nonexistent/file.txt"));
        }
    }
}

