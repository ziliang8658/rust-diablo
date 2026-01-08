/// Debug utility to export tile textures to PNG files for inspection
use anyhow::Result;
use image::{ImageBuffer, Rgba};

/// Export a single block's RGBA data to PNG
///
/// # Arguments
/// * `rgba_data` - RGBA pixel data (4 bytes per pixel)
/// * `width` - Image width
/// * `height` - Image height
/// * `output_path` - Path to save PNG file
pub fn export_rgba_to_png(
    rgba_data: &[u8],
    width: u32,
    height: u32,
    output_path: &str,
) -> Result<()> {
    // Create image buffer from RGBA data
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba_data.to_vec())
            .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

    // Save as PNG
    img.save(output_path)?;

    println!("✓ Exported PNG: {}", output_path);
    Ok(())
}

/// Export all blocks of a specific piece to PNG files
///
/// This will create PNG files named: piece_{piece_id}_block_{block_idx}.png
///
/// # Arguments
/// * `texture_mgr` - Tile texture manager  
/// * `piece_id` - The piece ID to export
/// * `output_dir` - Directory to save PNG files (will be created if not exists)
pub fn export_piece_blocks_to_png(
    texture_mgr: &mut crate::tiles::texture_manager::TileTextureManager,
    piece_id: usize,
    output_dir: &str,
) -> Result<()> {
    use std::fs;

    // Create output directory
    fs::create_dir_all(output_dir)?;

    println!("\n=== Exporting Piece {} to PNG ===", piece_id);

    // First, collect block info without borrowing texture_mgr
    let block_info: Vec<(usize, crate::tiles::types::TileType, bool)> = {
        let piece = texture_mgr
            .get_piece(piece_id)
            .ok_or_else(|| anyhow::anyhow!("Piece {} not found", piece_id))?;
        
        piece.mt.iter().enumerate()
            .map(|(idx, block)| (idx, block.tile_type(), block.has_value()))
            .collect()
    };

    let mut exported_count = 0;

    // Export each block
    for (block_idx, tile_type, has_value) in block_info {
        if !has_value {
            println!("  Block {}: (empty)", block_idx);
            continue;
        }

        println!("  Block {}: type={:?}", block_idx, tile_type);

        // Get decoded indexed data
        match texture_mgr.get_decoded_tile_with_type(piece_id, block_idx, tile_type) {
            Ok(indexed_data) => {
                // Clone indexed data to avoid borrow issues
                let indexed_data_cloned = indexed_data.to_vec();
                
                // Apply palette to get RGBA
                let rgba_data = texture_mgr.palette().indices_to_rgba(&indexed_data_cloned, false);
                
                // Determine dimensions based on tile type
                let (width, height) = match tile_type {
                    crate::tiles::types::TileType::Square |
                    crate::tiles::types::TileType::TransparentSquare => (32, 32),
                    crate::tiles::types::TileType::LeftTriangle |
                    crate::tiles::types::TileType::RightTriangle => (32, 31),
                    crate::tiles::types::TileType::LeftTrapezoid |
                    crate::tiles::types::TileType::RightTrapezoid => (32, 32),
                };
                
                let filename = format!(
                    "{}/piece_{:04}_block_{}.png",
                    output_dir, piece_id, block_idx
                );
                export_rgba_to_png(&rgba_data, width, height, &filename)?;
                exported_count += 1;
            }
            Err(e) => {
                eprintln!("  ⚠️  Block {}: Failed to decode: {}", block_idx, e);
            }
        }
    }

    println!(
        "=== Exported {} blocks from piece {} ===\n",
        exported_count, piece_id
    );
    Ok(())
}

/// Export multiple pieces to PNG files
///
/// # Arguments
/// * `texture_mgr` - Tile texture manager
/// * `piece_ids` - List of piece IDs to export
/// * `output_dir` - Directory to save PNG files
pub fn export_multiple_pieces_to_png(
    texture_mgr: &mut crate::tiles::texture_manager::TileTextureManager,
    piece_ids: &[usize],
    output_dir: &str,
) -> Result<()> {
    for &piece_id in piece_ids {
        export_piece_blocks_to_png(texture_mgr, piece_id, output_dir)?;
    }
    Ok(())
}

