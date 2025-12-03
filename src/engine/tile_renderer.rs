/// Tile Line-by-Line Renderer
/// 
/// Implements C++-style line-by-line rendering for tiles.
/// This matches the C++ RenderLeftTriangleFull implementation exactly.
/// 
/// # Reference
/// Original: `Source/engine/render/dun_render.cpp::RenderLeftTriangleFull()` Line 559-564

use anyhow::Result;
use crate::resources::Palette;
use crate::tiles::decoder::decode_tile;
use crate::tiles::types::TileType;

/// Constants matching C++ implementation
const WIDTH: usize = 32;  // DunFrameWidth
const LOWER_HEIGHT: usize = 16;  // DunFrameHeight / 2
const TRIANGLE_UPPER_HEIGHT: usize = 15;  // DunFrameHeight / 2 - 1
const TRIANGLE_HEIGHT: usize = 31;  // DunFrameTriangleHeight
const X_STEP: usize = 2;  // For triangles, 2 pixels per vertical step

#[allow(dead_code)]

/// Render a complete right triangle (32×31) line-by-line
/// 
/// This matches C++ RenderRightTriangleFull exactly:
/// 1. Render lower half (16 rows, from row 15 down to row 0) - no position adjustment
/// 2. Render upper half (15 rows, from row 16 up to row 30) - using dstPitch (not dstPitch - XStep)
/// 
/// # Arguments
/// * `dst_buffer` - RGBA output buffer (32×31×4 = 3968 bytes)
/// * `dst_pitch` - Bytes per row (32×4 = 128)
/// * `raw_data` - Raw encoded tile data
/// * `palette` - Color palette for index-to-RGBA conversion
/// * `transparent` - Whether to handle transparency (index 0 = transparent)
/// 
/// # Reference
/// Original: `Source/engine/render/dun_render.cpp::RenderRightTriangleFull()` Line 679-682
pub fn render_right_triangle_full(
    dst_buffer: &mut [u8],
    dst_pitch: usize,
    raw_data: &[u8],
    palette: &Palette,
    transparent: bool,
) -> Result<()> {
    // Ensure buffer is large enough
    let required_size = TRIANGLE_HEIGHT * dst_pitch;
    if dst_buffer.len() < required_size {
        return Err(anyhow::anyhow!(
            "Buffer too small: need {} bytes, got {}",
            required_size,
            dst_buffer.len()
        ));
    }
    
    // Decode the triangle to get indexed pixels
    let indexed_pixels = decode_tile(TileType::RightTriangle, raw_data)?;
    
    if indexed_pixels.len() != WIDTH * TRIANGLE_HEIGHT {
        return Err(anyhow::anyhow!(
            "Decoded triangle size mismatch: expected {} bytes, got {}",
            WIDTH * TRIANGLE_HEIGHT,
            indexed_pixels.len()
        ));
    }
    
    // Render lower half (rows 15 down to 0)
    // Reference: RenderRightTriangleLower -> RenderTriangleLower
    // C++: No position adjustment, directly render from row 15 down to row 0
    // Uses dstPitch (not dstPitch + XStep like LeftTriangle)
    let widths = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32];
    
    // Render lower half: from row 15 down to row 0
    // RightTriangle is right-aligned, so pixels start at offset (32 - width)
    for row_idx in (0..LOWER_HEIGHT).rev() {
        let row = row_idx; // 15, 14, 13, ..., 0
        let width = widths[row];
        
        // Calculate destination position
        // RightTriangle is right-aligned: pixels start at (32 - width) position
        let dst_row_offset = row * dst_pitch;
        let dst_x_offset = (WIDTH - width) * 4; // Right-align: start at (32 - width) pixels
        let dst_start = dst_row_offset + dst_x_offset;
        
        // Get source data for this row from decoded triangle
        // Decoded RightTriangle has row 0 at index 0, row 15 at index 15*32
        // The decoder already right-aligns the data, so pixels are at offset (32 - width) within each row
        let src_row_start = row * WIDTH;
        let src_start = src_row_start + (WIDTH - width); // Right-aligned in decoded array
        let src_end = src_start + width;
        
        if src_end > indexed_pixels.len() {
            return Err(anyhow::anyhow!(
                "Source data out of bounds: row {}, src_end {} > {}",
                row, src_end, indexed_pixels.len()
            ));
        }
        
        // Render this line: convert indexed pixels to RGBA
        let row_pixels = &indexed_pixels[src_start..src_end];
        render_line_to_rgba(
            &mut dst_buffer[dst_start..],
            row_pixels,
            width,
            palette,
            transparent,
        )?;
    }
    
    // Render upper half (rows 16 to 30)
    // C++: uses dstPitch (not dstPitch - XStep like LeftTriangle)
    let upper_widths = [30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2];
    
    // Render from row 16 up to row 30
    for row_idx in 0..TRIANGLE_UPPER_HEIGHT {
        let row = LOWER_HEIGHT + row_idx; // 16, 17, 18, ..., 30
        let width = upper_widths[row_idx];
        
        // Calculate destination position
        // RightTriangle is right-aligned: pixels start at (32 - width) position
        let dst_row_offset = row * dst_pitch;
        let dst_x_offset = (WIDTH - width) * 4; // Right-align
        let dst_start = dst_row_offset + dst_x_offset;
        
        // Get source data for this row from decoded triangle
        // The decoder already right-aligns the data, so pixels are at offset (32 - width) within each row
        let src_row_start = row * WIDTH;
        let src_start = src_row_start + (WIDTH - width); // Right-aligned in decoded array
        let src_end = src_start + width;
        
        if src_end > indexed_pixels.len() {
            return Err(anyhow::anyhow!(
                "Source data out of bounds: row {}, src_end {} > {}",
                row, src_end, indexed_pixels.len()
            ));
        }
        
        // Render this line
        let row_pixels = &indexed_pixels[src_start..src_end];
        render_line_to_rgba(
            &mut dst_buffer[dst_start..],
            row_pixels,
            width,
            palette,
            transparent,
        )?;
    }
    
    Ok(())
}

/// Render a complete left triangle (32×31) line-by-line
/// 
/// This matches C++ RenderLeftTriangleFull exactly:
/// 1. Render lower half (16 rows, from row 15 down to row 0)
/// 2. Adjust position: dst += 2 * XStep
/// 3. Render upper half (15 rows, from row 16 up to row 30)
/// 
/// # Arguments
/// * `dst_buffer` - RGBA output buffer (32×31×4 = 3968 bytes)
/// * `dst_pitch` - Bytes per row (32×4 = 128)
/// * `raw_data` - Raw encoded tile data
/// * `palette` - Color palette for index-to-RGBA conversion
/// * `transparent` - Whether to handle transparency (index 0 = transparent)
/// 
/// # Reference
/// Original: `Source/engine/render/dun_render.cpp::RenderLeftTriangleFull()` Line 559-564
pub fn render_left_triangle_full(
    dst_buffer: &mut [u8],
    dst_pitch: usize,
    raw_data: &[u8],
    palette: &Palette,
    transparent: bool,
) -> Result<()> {
    // Ensure buffer is large enough
    let required_size = TRIANGLE_HEIGHT * dst_pitch;
    if dst_buffer.len() < required_size {
        return Err(anyhow::anyhow!(
            "Buffer too small: need {} bytes, got {}",
            required_size,
            dst_buffer.len()
        ));
    }
    
    // Decode the triangle to get indexed pixels
    // We decode the whole triangle first, then render line-by-line
    let indexed_pixels = decode_tile(TileType::LeftTriangle, raw_data)?;
    
    if indexed_pixels.len() != WIDTH * TRIANGLE_HEIGHT {
        return Err(anyhow::anyhow!(
            "Decoded triangle size mismatch: expected {} bytes, got {}",
            WIDTH * TRIANGLE_HEIGHT,
            indexed_pixels.len()
        ));
    }
    
    // Render lower half (rows 15 down to 0)
    // Reference: RenderLeftTriangleLower -> RenderTriangleLower
    // C++: dst += XStep * (LowerHeight - 1) = 2 * 15 = 30 pixels
    // Then uses dstPitch + XStep for line offset
    // We render from row 15 (widest) down to row 0 (narrowest)
    
    // The decoded triangle has rows in order: row 0 at index 0, row 15 at index 15*32
    // But C++ renders from row 15 down to row 0
    let widths = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32];
    
    // Render lower half: from row 15 down to row 0
    // C++: dst starts at row 15, then goes up (dst -= dstLineOffset)
    for row_idx in (0..LOWER_HEIGHT).rev() {
        let row = row_idx; // 15, 14, 13, ..., 0
        let width = widths[row];
        
        // Calculate destination position
        // C++: starts at row 15, then moves up
        // In our buffer: row 15 is at offset (LOWER_HEIGHT - 1) * dst_pitch
        // But C++ also adds XStep offset: dst += XStep * (LowerHeight - 1)
        let dst_row_offset = row * dst_pitch;
        let dst_x_offset = (LOWER_HEIGHT - 1 - row) * X_STEP * 4; // XStep offset decreases as we go up
        let dst_start = dst_row_offset + dst_x_offset;
        
        // Get source data for this row from decoded triangle
        // Decoded triangle has row 0 at index 0, row 15 at index 15*32
        let src_start = row * WIDTH;
        let src_end = src_start + width;
        
        if src_end > indexed_pixels.len() {
            return Err(anyhow::anyhow!(
                "Source data out of bounds: row {}, src_end {} > {}",
                row, src_end, indexed_pixels.len()
            ));
        }
        
        // Render this line: convert indexed pixels to RGBA
        let row_pixels = &indexed_pixels[src_start..src_end];
        render_line_to_rgba(
            &mut dst_buffer[dst_start..],
            row_pixels,
            width,
            palette,
            transparent,
        )?;
    }
    
    // Adjust position: dst += 2 * XStep = 4 pixels = 16 bytes
    // C++: dst += 2 * XStep (after RenderLeftTriangleLower)
    let upper_x_offset = 2 * X_STEP * 4; // 16 bytes
    
    // Render upper half (rows 16 to 30)
    // C++: uses dstPitch - XStep for line offset
    let upper_widths = [30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2];
    
    // Render from row 16 up to row 30
    for row_idx in 0..TRIANGLE_UPPER_HEIGHT {
        let row = LOWER_HEIGHT + row_idx; // 16, 17, 18, ..., 30
        let width = upper_widths[row_idx];
        
        // Calculate destination position
        // C++: dst starts at adjusted position, then goes up
        // dstPitch is reduced by XStep: dstPitch - XStep
        // X offset increases as we go up: row_idx * XStep
        let dst_row_offset = row * dst_pitch;
        let dst_x_offset = upper_x_offset + row_idx * X_STEP * 4;
        let dst_start = dst_row_offset + dst_x_offset;
        
        // Get source data for this row from decoded triangle
        let src_start = row * WIDTH;
        let src_end = src_start + width;
        
        if src_end > indexed_pixels.len() {
            return Err(anyhow::anyhow!(
                "Source data out of bounds: row {}, src_end {} > {}",
                row, src_end, indexed_pixels.len()
            ));
        }
        
        // Render this line
        let row_pixels = &indexed_pixels[src_start..src_end];
        render_line_to_rgba(
            &mut dst_buffer[dst_start..],
            row_pixels,
            width,
            palette,
            transparent,
        )?;
    }
    
    Ok(())
}

/// Render a single line of indexed pixels to RGBA
fn render_line_to_rgba(
    dst: &mut [u8],
    src: &[u8],
    width: usize,
    palette: &Palette,
    transparent: bool,
) -> Result<()> {
    if dst.len() < width * 4 {
        return Err(anyhow::anyhow!(
            "Destination buffer too small: need {} bytes, got {}",
            width * 4,
            dst.len()
        ));
    }
    
    for i in 0..width {
        let color_idx = src[i];
        let rgba = if transparent && color_idx == 0 {
            [0, 0, 0, 0] // Transparent
        } else {
            let color = palette.to_rgb(color_idx);
            [color.r, color.g, color.b, 255]
        };
        
        let dst_offset = i * 4;
        dst[dst_offset..dst_offset + 4].copy_from_slice(&rgba);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::Palette;
    
    #[test]
    fn test_render_left_triangle_full_buffer_size() {
        let palette = Palette::from_bytes(&[0u8; 768]).unwrap();
        let raw_data = vec![0u8; 544]; // Minimum size for triangle
        let mut buffer = vec![0u8; 32 * 31 * 4];
        
        // Should not panic
        let result = render_left_triangle_full(&mut buffer, 32 * 4, &raw_data, &palette, true);
        // May fail due to invalid data, but shouldn't panic
        let _ = result;
    }
}

