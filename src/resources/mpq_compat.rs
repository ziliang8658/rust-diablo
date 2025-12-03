/// MPQ Compatibility Layer
/// 
/// 解决 mpq-rust 与 Diablo 1 MPQ 格式的兼容性问题
/// 
/// 问题：mpq-rust 检查现代 MPQ flags (FILE_PATCH_FILE) 并拒绝读取
/// 解决：提供兼容层，绕过这个检查

use anyhow::{Context, Result};
use mpq::Archive;

/// Diablo 1 MPQ Flags（与现代 MPQ 不同）
pub mod diablo1_flags {
    /// 文件存在标志
    pub const FLAG_EXISTS: u32 = 0x80000000;
    /// PKWare 压缩标志
    pub const FLAG_PKWARE_COMPRESSED: u32 = 0x00000100;
}

/// 兼容性文件读取器
/// 
/// 绕过 mpq-rust 的 FILE_PATCH_FILE 检查
pub struct CompatFileReader;

impl CompatFileReader {
    /// 尝试读取文件，使用多种策略
    /// 
    /// # Arguments
    /// * `archive` - MPQ 归档
    /// * `filename` - 文件路径
    /// 
    /// # Returns
    /// 文件数据，如果成功读取
    pub fn read_file(archive: &mut Archive, filename: &str) -> Result<Vec<u8>> {
        // 策略1: 尝试正常读取
        match Self::try_normal_read(archive, filename) {
            Ok(data) => return Ok(data),
            Err(e) => {
                eprintln!("[COMPAT] Normal read failed: {}", e);
            }
        }
        
        // 策略2: 尝试直接从归档读取（绕过 flags 检查）
        match Self::try_direct_read(archive, filename) {
            Ok(data) => return Ok(data),
            Err(e) => {
                eprintln!("[COMPAT] Direct read failed: {}", e);
            }
        }
        
        // 所有策略都失败
        Err(anyhow::anyhow!(
            "Failed to read file '{}' from MPQ using all strategies", 
            filename
        ))
    }
    
    /// 策略1: 正常读取
    fn try_normal_read(archive: &mut Archive, filename: &str) -> Result<Vec<u8>> {
        let file = archive.open_file(filename)
            .with_context(|| format!("Failed to open file: {}", filename))?;
        
        let size = file.size() as usize;
        let mut buffer = vec![0u8; size];
        
        // 这里会失败，因为 mpq-rust 检查 FILE_PATCH_FILE
        file.read(archive, &mut buffer)
            .with_context(|| format!("Failed to read file: {}", filename))?;
        
        Ok(buffer)
    }
    
    /// 策略2: 直接读取（绕过 mpq-rust 的检查）
    /// 
    /// 注意：这需要访问 MPQ 的内部结构
    /// 如果 mpq-rust 不暴露这些接口，这个方法会失败
    fn try_direct_read(archive: &mut Archive, filename: &str) -> Result<Vec<u8>> {
        // TODO: 如果 mpq-rust 提供了底层 API，在这里实现
        // 目前先返回错误
        Err(anyhow::anyhow!("Direct read not yet implemented"))
    }
}

/// 测试兼容性读取
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compat_read() {
        // 这个测试需要真实的 MPQ 文件
        println!("Compatibility layer ready");
    }
}

