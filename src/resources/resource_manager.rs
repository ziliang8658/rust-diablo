/// 资源管理器 - 统一资源加载和缓存管理
/// 
/// ResourceManager 提供统一的接口来加载和管理各种游戏资源：
/// - MPQ归档管理
/// - 调色板缓存
/// - 精灵缓存（CL2/CLX）
/// - TRN颜色转换缓存
/// - 纹理ID映射
/// 
/// 设计模式：
/// - 单例模式：游戏全局唯一的资源管理器
/// - 工厂模式：统一的资源创建接口
/// - 缓存模式：使用Arc<T>共享所有权，避免重复加载

use anyhow::{Result, Context, bail};
use std::collections::HashMap;
use std::sync::Arc;
use crate::resources::{
    MpqManager, 
    Palette, 
    ColorTransform,
    ClxSprite,
    ClxFrame,
    Cl2Sprite,
    PcxImage,
};
use crate::engine::Engine;

/// 资源管理器主结构
pub struct ResourceManager {
    /// MPQ归档管理器
    mpq_manager: MpqManager,
    
    /// 调色板缓存 (路径 -> 调色板)
    palette_cache: HashMap<String, Arc<Palette>>,
    
    /// TRN颜色转换缓存 (路径 -> TRN)
    trn_cache: HashMap<String, Arc<ColorTransform>>,
    
    /// CLX精灵缓存 (路径 -> 精灵帧列表)
    clx_cache: HashMap<String, Arc<Vec<crate::resources::ClxFrame>>>,
    
    /// CL2精灵缓存 (路径 -> 精灵帧列表)
    cl2_cache: HashMap<String, Arc<Vec<crate::resources::ClxFrame>>>,
    
    /// 纹理ID映射 (资源标识 -> 纹理ID列表)
    /// 例如: "warrior_idle" -> ["warrior_idle_0", "warrior_idle_1", ...]
    texture_id_map: HashMap<String, Vec<String>>,
}

impl ResourceManager {
    /// 创建新的资源管理器
    /// 
    /// # 参数
    /// - `mpq_paths`: MPQ文件路径列表，格式为 (路径, 优先级)
    /// 
    /// # 返回
    /// - `Ok(ResourceManager)`: 成功创建
    /// - `Err`: 初始化失败
    /// 
    /// # 示例
    /// ```
    /// let rm = ResourceManager::new(vec![
    ///     ("assets/DIABDAT.MPQ", 1000),
    /// ])?;
    /// ```
    pub fn new(mpq_paths: Vec<(&str, i32)>) -> Result<Self> {
        let mut mpq_manager = MpqManager::new();
        
        // 加载所有MPQ文件
        for (path, priority) in mpq_paths {
            mpq_manager.load_mpq(path, priority)
                .with_context(|| format!("Failed to load MPQ: {}", path))?;
        }
        
        Ok(Self {
            mpq_manager,
            palette_cache: HashMap::new(),
            trn_cache: HashMap::new(),
            clx_cache: HashMap::new(),
            cl2_cache: HashMap::new(),
            texture_id_map: HashMap::new(),
        })
    }
    
    /// 创建空的资源管理器（仅用于测试）
    pub fn new_empty() -> Self {
        Self {
            mpq_manager: MpqManager::new(),
            palette_cache: HashMap::new(),
            trn_cache: HashMap::new(),
            clx_cache: HashMap::new(),
            cl2_cache: HashMap::new(),
            texture_id_map: HashMap::new(),
        }
    }
    
    /// 尝试加载MPQ文件（不失败继续）
    pub fn try_load_mpqs(&mut self, mpq_paths: Vec<(&str, i32)>) -> Vec<String> {
        let mut loaded = Vec::new();
        
        for (path, priority) in mpq_paths {
            match self.mpq_manager.load_mpq(path, priority) {
                Ok(_) => {
                    println!("✓ Loaded MPQ: {} (priority: {})", path, priority);
                    loaded.push(path.to_string());
                }
                Err(e) => {
                    eprintln!("⚠ Failed to load MPQ {}: {}", path, e);
                }
            }
        }
        
        loaded
    }
    
    // ========== 基础资源加载 ==========
    
    /// 加载调色板（带缓存）
    /// 
    /// 如果已经加载过，直接返回缓存的Arc指针。
    pub fn load_palette(&mut self, path: &str) -> Result<Arc<Palette>> {
        // 检查缓存
        if let Some(cached) = self.palette_cache.get(path) {
            return Ok(Arc::clone(cached));
        }
        
        // 从MPQ加载
        let data = self.mpq_manager.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("Palette file not found in MPQ: {}", path))?;
        
        let palette = Palette::from_bytes(&data)
            .with_context(|| format!("Failed to parse palette: {}", path))?;
        
        let palette_arc = Arc::new(palette);
        self.palette_cache.insert(path.to_string(), Arc::clone(&palette_arc));
        
        Ok(palette_arc)
    }
    
    /// 加载TRN颜色转换（带缓存）
    pub fn load_trn(&mut self, path: &str) -> Result<Arc<ColorTransform>> {
        // 检查缓存
        if let Some(cached) = self.trn_cache.get(path) {
            return Ok(Arc::clone(cached));
        }
        
        // 从MPQ加载
        let data = self.mpq_manager.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("TRN file not found in MPQ: {}", path))?;
        
        let trn = ColorTransform::from_bytes(&data)
            .with_context(|| format!("Failed to parse TRN: {}", path))?;
        
        let trn_arc = Arc::new(trn);
        self.trn_cache.insert(path.to_string(), Arc::clone(&trn_arc));
        
        Ok(trn_arc)
    }
    
    /// 加载CL2精灵（带缓存）
    pub fn load_cl2(&mut self, path: &str, frame_width: u16) -> Result<Arc<Vec<crate::resources::ClxFrame>>> {
        // 检查缓存
        if let Some(cached) = self.cl2_cache.get(path) {
            return Ok(Arc::clone(cached));
        }
        
        // 从MPQ加载
        let data = self.mpq_manager.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("CL2 file not found in MPQ: {}", path))?;
        
        let sprite = Cl2Sprite::from_bytes(&data, frame_width)
            .with_context(|| format!("Failed to parse CL2: {}", path))?;
        
        let frames_arc = Arc::new(sprite.frames);
        self.cl2_cache.insert(path.to_string(), Arc::clone(&frames_arc));
        
        Ok(frames_arc)
    }
    
    /// 加载CLX精灵（带缓存）
    pub fn load_clx(&mut self, path: &str) -> Result<Arc<Vec<crate::resources::ClxFrame>>> {
        // 检查缓存
        if let Some(cached) = self.clx_cache.get(path) {
            return Ok(Arc::clone(cached));
        }
        
        // 从MPQ加载
        let data = self.mpq_manager.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("CLX file not found in MPQ: {}", path))?;
        
        let sprite = ClxSprite::from_bytes(&data)
            .with_context(|| format!("Failed to parse CLX: {}", path))?;
        
        let frames_arc = Arc::new(sprite.frames);
        self.clx_cache.insert(path.to_string(), Arc::clone(&frames_arc));
        
        Ok(frames_arc)
    }
    
    /// 加载PCX图像（不缓存，因为通常只加载一次）
    pub fn load_pcx(&mut self, path: &str) -> Result<PcxImage> {
        let data = self.mpq_manager.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("PCX file not found in MPQ: {}", path))?;
        
        PcxImage::from_bytes(&data)
            .with_context(|| format!("Failed to parse PCX: {}", path))
    }
    
    /// Load CEL dungeon tileset
    /// 
    /// CEL files contain tileset textures for dungeon rendering.
    /// Format is identical to CLX, so we can reuse the CLX parser.
    /// Returns raw indexed frames that need to be converted with a palette.
    /// 
    /// # Arguments
    /// * `path` - Path to CEL file (e.g. "levels/l1data/l1.cel")
    /// 
    /// # Returns
    /// Vector of CLX frames (indexed color data)
    pub fn load_cel(&mut self, path: &str) -> Result<Vec<ClxFrame>> {
        let data = self.mpq_manager.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("CEL file not found in MPQ: {}", path))?;
        
        let sprite = ClxSprite::from_bytes(&data)
            .with_context(|| format!("Failed to parse CEL: {}", path))?;
        
        // Return frames as-is (indexed color)
        Ok(sprite.frames)
    }
    
    /// Load a Dungeon CEL file (special tile format for l1.cel, l2.cel, etc.)
    /// 
    /// Dungeon CEL files use a different format from sprite CLX files.
    /// They contain 32x32 tile frames used for dungeon rendering.
    /// 
    /// # Arguments
    /// * `path` - Path to Dungeon CEL file in MPQ (e.g. "levels/l1data/l1.cel")
    /// 
    /// # Returns
    /// Vector of DungeonCelFrame (32x32 indexed color frames)
    pub fn load_dungeon_cel(&mut self, path: &str) -> Result<Vec<crate::resources::DungeonCelFrame>> {
        let data = self.mpq_manager.find_file(path)
            .ok_or_else(|| anyhow::anyhow!("Dungeon CEL file not found in MPQ: {}", path))?;
        
        let sprite = crate::resources::DungeonCelSprite::from_bytes(&data)
            .with_context(|| format!("Failed to parse Dungeon CEL: {}", path))?;
        
        // Return frames as-is (indexed color, 32x32)
        Ok(sprite.frames)
    }
    
    /// Load a single tile from PNG file (preserves RGB colors)
    /// 
    /// Modern approach: Load tiles from PNG files instead of Dungeon CEL format.
    /// 
    /// # Arguments
    /// * `path` - Path to PNG file
    /// 
    /// # Returns
    /// ClxFrame with RGB data encoded as pseudo-palette indices
    /// 
    /// # Note
    /// This encodes RGB values in a way that can be reconstructed later,
    /// avoiding the grayscale conversion issue.
    pub fn load_tile_png(&self, path: &str) -> Result<ClxFrame> {
        use image::GenericImageView;
        
        let img = image::open(path)
            .with_context(|| format!("Failed to load PNG: {}", path))?;
        
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        
        // Store RGBA data directly (we'll handle it specially in texture creation)
        let mut pixels = Vec::with_capacity((width * height) as usize);
        
        for pixel in rgba.pixels() {
            let alpha = pixel[3];
            if alpha < 128 {
                // Transparent
                pixels.push(None);
            } else {
                // Use red channel as pseudo-index (will be handled specially)
                // This is a hack - we'll need to pass actual RGBA data
                pixels.push(Some(pixel[0]));
            }
        }
        
        Ok(ClxFrame {
            width: width as u16,
            height: height as u16,
            pixels,
        })
    }
    
    /// Load tiles from PNG folder
    /// 
    /// Loads all tiles from a folder containing tile_0.png, tile_1.png, etc.
    /// 
    /// # Arguments
    /// * `folder_path` - Path to folder containing tile PNG files
    /// 
    /// # Returns
    /// Vector of ClxFrame tiles
    pub fn load_tiles_from_folder(&self, folder_path: &str) -> Result<Vec<ClxFrame>> {
        use std::path::Path;
        
        let folder = Path::new(folder_path);
        if !folder.exists() {
            bail!("Tile folder not found: {}", folder_path);
        }
        
        let mut tiles = Vec::new();
        let mut index = 0;
        
        // Try to load tile_0.png, tile_1.png, etc. until we hit a missing one
        loop {
            let tile_path = folder.join(format!("tile_{}.png", index));
            
            if !tile_path.exists() {
                break;
            }
            
            let tile = self.load_tile_png(tile_path.to_str().unwrap())?;
            
            tiles.push(tile);
            index += 1;
        }
        
        if tiles.is_empty() {
            bail!("No tile PNG files found in folder: {}", folder_path);
        }
        
        Ok(tiles)
    }
    
    // ========== 高级接口：加载并创建纹理 ==========
    
    /// 加载精灵并创建SDL纹理
    /// 
    /// # 参数
    /// - `engine`: Engine引用，用于创建纹理
    /// - `sprite_path`: 精灵文件路径（CL2或CLX）
    /// - `palette_path`: 调色板文件路径
    /// - `sprite_name`: 精灵名称（用于纹理ID）
    /// - `state_name`: 动画状态名称（如 "idle", "walk"）
    /// - `frame_width`: CL2帧宽度（CLX可以忽略）
    /// 
    /// # 返回
    /// 纹理ID列表，格式：["{sprite_name}_{state_name}_0", "{sprite_name}_{state_name}_1", ...]
    /// 
    /// # 示例
    /// ```
    /// let texture_ids = rm.load_sprite_textures(
    ///     &mut engine,
    ///     "plrgfx/warrior/wmn/wmnas.cl2",
    ///     "levels/towndata/town.pal",
    ///     "warrior",
    ///     "idle",
    ///     Some(96),
    /// )?;
    /// // texture_ids = ["warrior_idle_0", "warrior_idle_1", ...]
    /// ```
    pub fn load_sprite_textures(
        &mut self,
        engine: &mut Engine,
        sprite_path: &str,
        palette_path: &str,
        sprite_name: &str,
        state_name: &str,
        frame_width: Option<u16>,
    ) -> Result<Vec<String>> {
        // 加载调色板
        let palette = self.load_palette(palette_path)?;
        
        // 判断文件类型并加载精灵
        let frames = if sprite_path.to_lowercase().ends_with(".cl2") {
            let width = frame_width.ok_or_else(|| anyhow::anyhow!("CL2 requires frame_width"))?;
            self.load_cl2(sprite_path, width)?
        } else if sprite_path.to_lowercase().ends_with(".clx") {
            self.load_clx(sprite_path)?
        } else {
            anyhow::bail!("Unsupported sprite format: {}", sprite_path);
        };
        
        // 创建纹理
        let mut texture_ids = Vec::new();
        
        for (i, frame) in frames.iter().enumerate() {
            let texture_id = format!("{}_{}_{}",  sprite_name, state_name, i);
            let rgba_data = frame.to_rgba(&palette);
            
            engine.load_texture_from_rgba(
                &texture_id,
                &rgba_data,
                frame.width as u32,
                frame.height as u32,
            ).with_context(|| format!("Failed to create texture: {}", texture_id))?;
            
            texture_ids.push(texture_id);
        }
        
        // 保存纹理ID映射
        let map_key = format!("{}_{}", sprite_name, state_name);
        self.texture_id_map.insert(map_key, texture_ids.clone());
        
        Ok(texture_ids)
    }
    
    /// 加载精灵并应用TRN颜色转换，然后创建纹理
    /// 
    /// 这个方法用于创建不同颜色的精灵变体（如不同颜色的怪物）
    /// 
    /// # 参数
    /// - `trn_path`: TRN文件路径
    /// - `variant_suffix`: 变体后缀（如 "_red", "_green"）
    /// 
    /// # 示例
    /// ```
    /// // 创建红色僵尸
    /// let red_zombie_textures = rm.load_sprite_with_trn(
    ///     &mut engine,
    ///     "monsters/zombie/zombie.cl2",
    ///     "levels/towndata/town.pal",
    ///     "monsters/zombie/red.trn",
    ///     "zombie",
    ///     "idle",
    ///     Some(96),
    ///     "_red",
    /// )?;
    /// // texture_ids = ["zombie_idle_red_0", "zombie_idle_red_1", ...]
    /// ```
    pub fn load_sprite_with_trn(
        &mut self,
        engine: &mut Engine,
        sprite_path: &str,
        palette_path: &str,
        trn_path: &str,
        sprite_name: &str,
        state_name: &str,
        frame_width: Option<u16>,
        variant_suffix: &str,
    ) -> Result<Vec<String>> {
        // 加载资源
        let palette = self.load_palette(palette_path)?;
        let trn = self.load_trn(trn_path)?;
        
        // 加载精灵
        let frames = if sprite_path.to_lowercase().ends_with(".cl2") {
            let width = frame_width.ok_or_else(|| anyhow::anyhow!("CL2 requires frame_width"))?;
            self.load_cl2(sprite_path, width)?
        } else if sprite_path.to_lowercase().ends_with(".clx") {
            self.load_clx(sprite_path)?
        } else {
            anyhow::bail!("Unsupported sprite format: {}", sprite_path);
        };
        
        // 创建纹理（应用TRN）
        let mut texture_ids = Vec::new();
        
        for (i, frame) in frames.iter().enumerate() {
            // 复制帧并应用TRN
            let mut modified_frame = frame.clone();
            trn.apply_to_pixels(&mut modified_frame.pixels);
            
            // 创建纹理
            let texture_id = format!("{}_{}{}_{}", sprite_name, state_name, variant_suffix, i);
            let rgba_data = modified_frame.to_rgba(&palette);
            
            engine.load_texture_from_rgba(
                &texture_id,
                &rgba_data,
                modified_frame.width as u32,
                modified_frame.height as u32,
            ).with_context(|| format!("Failed to create texture: {}", texture_id))?;
            
            texture_ids.push(texture_id);
        }
        
        // 保存纹理ID映射
        let map_key = format!("{}_{}{}", sprite_name, state_name, variant_suffix);
        self.texture_id_map.insert(map_key, texture_ids.clone());
        
        Ok(texture_ids)
    }
    
    /// 加载PCX图像并创建纹理
    /// 
    /// # 参数
    /// - `pcx_path`: PCX文件路径
    /// - `texture_id`: 纹理ID
    /// - `palette_path`: 调色板路径（可选，PCX可能自带调色板）
    /// - `transparent_index`: 透明色索引（可选）
    /// 
    /// # 返回
    /// 纹理ID
    pub fn load_pcx_texture(
        &mut self,
        engine: &mut Engine,
        pcx_path: &str,
        texture_id: &str,
        palette_path: Option<&str>,
        transparent_index: Option<u8>,
    ) -> Result<String> {
        let pcx = self.load_pcx(pcx_path)?;
        
        // 选择调色板
        let palette = if let Some(ref internal_pal) = pcx.palette {
            // 使用PCX内部调色板
            Arc::new(internal_pal.clone())
        } else if let Some(pal_path) = palette_path {
            // 使用外部调色板
            self.load_palette(pal_path)?
        } else {
            anyhow::bail!("PCX has no internal palette and no external palette provided");
        };
        
        // 转换为RGBA
        let rgba_data = pcx.to_rgba(&palette, transparent_index);
        
        // 创建纹理
        engine.load_texture_from_rgba(
            texture_id,
            &rgba_data,
            pcx.width as u32,
            pcx.height as u32,
        ).with_context(|| format!("Failed to create texture from PCX: {}", texture_id))?;
        
        Ok(texture_id.to_string())
    }
    
    // ========== 缓存管理 ==========
    
    /// 清空所有缓存
    pub fn clear_cache(&mut self) {
        self.palette_cache.clear();
        self.trn_cache.clear();
        self.clx_cache.clear();
        self.cl2_cache.clear();
        self.texture_id_map.clear();
    }
    
    /// 获取缓存统计信息
    pub fn cache_stats(&self) -> String {
        format!(
            "Cache Stats:\n  Palettes: {}\n  TRNs: {}\n  CLX: {}\n  CL2: {}\n  Texture Maps: {}",
            self.palette_cache.len(),
            self.trn_cache.len(),
            self.clx_cache.len(),
            self.cl2_cache.len(),
            self.texture_id_map.len(),
        )
    }
    
    /// 获取纹理ID列表
    pub fn get_texture_ids(&self, sprite_name: &str, state_name: &str) -> Option<&Vec<String>> {
        let key = format!("{}_{}", sprite_name, state_name);
        self.texture_id_map.get(&key)
    }
    
    /// 获取已加载的MPQ归档列表
    pub fn get_loaded_archives(&self) -> Vec<String> {
        self.mpq_manager.get_loaded_archives()
    }
    
    /// 获取MpqManager的可变引用（用于tiles加载）
    /// 
    /// # 返回
    /// MpqManager的可变引用
    /// 
    /// # 示例
    /// ```
    /// use rust_diablo::tiles::MinData;
    /// 
    /// let mut rm = ResourceManager::new_empty();
    /// let min_data = MinData::from_mpq(rm.mpq_manager_mut(), "levels/l1data/l1.min")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn mpq_manager_mut(&mut self) -> &mut MpqManager {
        &mut self.mpq_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_manager_creation() {
        let rm = ResourceManager::new_empty();
        assert_eq!(rm.palette_cache.len(), 0);
        assert_eq!(rm.trn_cache.len(), 0);
    }
    
    #[test]
    fn test_cache_stats() {
        let rm = ResourceManager::new_empty();
        let stats = rm.cache_stats();
        assert!(stats.contains("Palettes: 0"));
        assert!(stats.contains("TRNs: 0"));
    }
    
    #[test]
    fn test_get_texture_ids_empty() {
        let rm = ResourceManager::new_empty();
        assert!(rm.get_texture_ids("warrior", "idle").is_none());
    }
}

