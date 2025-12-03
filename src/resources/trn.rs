/// TRN颜色转换模块
///
/// TRN (Translation) 文件用于实现精灵的颜色映射变换。
/// 这是90年代游戏常用的内存优化技术：只存储一个精灵，
/// 通过256字节的颜色映射表生成不同颜色的变体。
///
/// 文件格式：
/// - 大小：256字节
/// - 内容：调色板索引映射表
/// - 映射规则：new_index = trn[old_index]
///
/// 参考代码：
/// - Source/engine/load_file.hpp - 文件加载
/// - Source/monster.cpp - 怪物生成时应用TRN
/// - Source/player.cpp - 玩家装备变色
use anyhow::Result;

/// TRN颜色转换表
///
/// 存储256个字节的调色板索引映射关系。
/// 通过将原始索引映射到新索引，实现颜色变换。
#[derive(Debug, Clone)]
pub struct ColorTransform {
    /// 颜色索引映射表 [256]
    /// map[old_index] = new_index
    pub map: [u8; 256],
}

impl ColorTransform {
    /// 从字节数据加载TRN
    ///
    /// # 参数
    /// - `data`: TRN文件数据，必须是256字节
    ///
    /// # 返回
    /// - `Ok(ColorTransform)`: 成功加载
    /// - `Err`: 数据长度不正确
    ///
    /// # 示例
    /// ```
    /// let trn_data = vec![0u8; 256]; // 256字节数据
    /// let trn = ColorTransform::from_bytes(&trn_data)?;
    /// ```
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() != 256 {
            anyhow::bail!(
                "TRN file must be exactly 256 bytes, got {} bytes",
                data.len()
            );
        }

        let mut map = [0u8; 256];
        map.copy_from_slice(data);

        Ok(Self { map })
    }

    /// 创建恒等映射（identity mapping）
    ///
    /// 返回一个不做任何变换的TRN（每个索引映射到自己）。
    /// 用于测试或作为默认值。
    pub fn identity() -> Self {
        let mut map = [0u8; 256];
        for i in 0..256 {
            map[i] = i as u8;
        }
        Self { map }
    }

    /// 应用颜色映射到单个颜色索引
    ///
    /// # 参数
    /// - `color_index`: 原始调色板索引
    ///
    /// # 返回
    /// 映射后的新索引
    ///
    /// # 示例
    /// ```
    /// let trn = ColorTransform::from_bytes(&trn_data)?;
    /// let old_index = 10;
    /// let new_index = trn.apply(old_index);
    /// ```
    pub fn apply(&self, color_index: u8) -> u8 {
        self.map[color_index as usize]
    }

    /// 应用颜色映射到像素数组（批量操作）
    ///
    /// 修改传入的像素数组，将每个非透明像素的索引进行映射。
    /// 透明像素（None）保持不变。
    ///
    /// # 参数
    /// - `pixels`: 可变的像素数组，包含 Option<u8>（None表示透明）
    ///
    /// # 示例
    /// ```
    /// let mut pixels = vec![Some(10), None, Some(20), Some(30)];
    /// trn.apply_to_pixels(&mut pixels);
    /// // pixels现在是: [Some(mapped_10), None, Some(mapped_20), Some(mapped_30)]
    /// ```
    pub fn apply_to_pixels(&self, pixels: &mut [Option<u8>]) {
        for pixel in pixels.iter_mut() {
            if let Some(index) = pixel {
                *index = self.apply(*index);
            }
        }
    }

    /// 检查TRN是否为恒等映射
    ///
    /// 如果所有索引都映射到自己，返回true。
    pub fn is_identity(&self) -> bool {
        for i in 0..256 {
            if self.map[i] != i as u8 {
                return false;
            }
        }
        true
    }

    /// 获取TRN的变化统计
    ///
    /// 返回有多少个索引被映射到了不同的值。
    ///
    /// # 返回
    /// (changed_count, total_count) - (变化的索引数, 总索引数256)
    pub fn change_stats(&self) -> (usize, usize) {
        let changed = self
            .map
            .iter()
            .enumerate()
            .filter(|(i, &mapped)| *i != mapped as usize)
            .count();
        (changed, 256)
    }

    /// 打印TRN映射表（调试用）
    ///
    /// 只打印被映射到不同值的索引。
    pub fn print_mappings(&self) {
        println!("TRN Color Mappings:");
        let mut changed = 0;
        for i in 0..256 {
            if self.map[i] != i as u8 {
                println!("  {} -> {}", i, self.map[i]);
                changed += 1;
            }
        }
        if changed == 0 {
            println!("  (Identity mapping - no changes)");
        } else {
            println!("Total changed: {} / 256", changed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes_correct_size() {
        let data = vec![0u8; 256];
        let result = ColorTransform::from_bytes(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_bytes_wrong_size() {
        let data = vec![0u8; 255]; // 少一个字节
        let result = ColorTransform::from_bytes(&data);
        assert!(result.is_err());

        let data = vec![0u8; 257]; // 多一个字节
        let result = ColorTransform::from_bytes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_identity_mapping() {
        let trn = ColorTransform::identity();

        // 检查是否为恒等映射
        assert!(trn.is_identity());

        // 检查每个索引映射到自己
        for i in 0..256 {
            assert_eq!(trn.apply(i as u8), i as u8);
        }
    }

    #[test]
    fn test_simple_mapping() {
        let mut data = vec![0u8; 256];
        // 创建一个简单的映射：所有索引+1（模256）
        for i in 0..256 {
            data[i] = ((i + 1) % 256) as u8;
        }

        let trn = ColorTransform::from_bytes(&data).unwrap();

        // 测试映射
        assert_eq!(trn.apply(0), 1);
        assert_eq!(trn.apply(10), 11);
        assert_eq!(trn.apply(255), 0); // 255+1 = 0 (mod 256)

        // 不是恒等映射
        assert!(!trn.is_identity());
    }

    #[test]
    fn test_apply_to_pixels() {
        // 创建映射：所有索引*2（模256）
        let mut data = vec![0u8; 256];
        for i in 0..256 {
            data[i] = ((i * 2) % 256) as u8;
        }
        let trn = ColorTransform::from_bytes(&data).unwrap();

        // 测试像素数组
        let mut pixels = vec![
            Some(10),
            None, // 透明像素
            Some(20),
            Some(100),
            None, // 透明像素
        ];

        trn.apply_to_pixels(&mut pixels);

        // 检查结果
        assert_eq!(pixels[0], Some(20)); // 10 * 2 = 20
        assert_eq!(pixels[1], None); // 透明保持不变
        assert_eq!(pixels[2], Some(40)); // 20 * 2 = 40
        assert_eq!(pixels[3], Some(200)); // 100 * 2 = 200
        assert_eq!(pixels[4], None); // 透明保持不变
    }

    #[test]
    fn test_change_stats() {
        // 恒等映射
        let trn_id = ColorTransform::identity();
        let (changed, total) = trn_id.change_stats();
        assert_eq!(changed, 0);
        assert_eq!(total, 256);

        // 部分变化的映射
        let mut data = vec![0u8; 256];
        for i in 0..256 {
            data[i] = i as u8; // 先设置为恒等
        }
        // 修改前10个索引
        for i in 0..10 {
            data[i] = (i + 100) as u8;
        }

        let trn = ColorTransform::from_bytes(&data).unwrap();
        let (changed, total) = trn.change_stats();
        assert_eq!(changed, 10);
        assert_eq!(total, 256);
    }

    #[test]
    fn test_apply_preserves_zero() {
        // 测试索引0是否可以被映射
        let mut data = vec![0u8; 256];
        for i in 0..256 {
            data[i] = i as u8;
        }
        data[0] = 10; // 将索引0映射到10

        let trn = ColorTransform::from_bytes(&data).unwrap();
        assert_eq!(trn.apply(0), 10);
    }

    #[test]
    fn test_round_trip_mapping() {
        // 测试映射和反映射
        let mut forward = vec![0u8; 256];
        let mut backward = vec![0u8; 256];

        // 创建一个可逆的映射（简单的反转：i -> 255-i）
        for i in 0..256 {
            forward[i] = (255 - i) as u8;
            backward[255 - i] = i as u8;
        }

        let trn_fwd = ColorTransform::from_bytes(&forward).unwrap();
        let trn_bwd = ColorTransform::from_bytes(&backward).unwrap();

        // 测试：应用forward再应用backward应该回到原值
        for i in 0..256 {
            let mapped = trn_fwd.apply(i as u8);
            let unmapped = trn_bwd.apply(mapped);
            assert_eq!(unmapped, i as u8);
        }
    }
}
