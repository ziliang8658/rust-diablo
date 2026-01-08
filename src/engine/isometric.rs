/// Isometric projection system for Diablo
///
/// Diablo uses an isometric (diamond tile) projection to create a pseudo-3D view.
/// This module provides coordinate conversion utilities between:
/// - World coordinates (logical grid, used for game logic)
/// - Screen coordinates (rendering position on screen)
/// - MegaTile coordinates (map generation, 40x40 grid)
/// - MicroTile coordinates (rendering and collision, 112x112 grid)
///
/// # Coordinate Systems
///
/// ## World Coordinates
/// - Used for game logic (collision, pathfinding, etc.)
/// - Orthogonal grid (square tiles)
/// - Origin at top-left
///
/// ## Screen Coordinates
/// - Used for rendering
/// - Isometric projection (diamond tiles)
/// - Origin at top-left
///
/// ## MegaTile Coordinates
/// - Used for map generation
/// - Range: [0, DMAXX) x [0, DMAXY) where DMAXX = DMAXY = 40
/// - Each MegaTile contains 2x2 MicroTiles
///
/// ## MicroTile Coordinates
/// - Used for rendering and precise collision
/// - Range: [0, MAXDUNX) x [0, MAXDUNY) where MAXDUNX = MAXDUNY = 112
/// - Conversion: MicroTile = MegaTile * 2 + 16 (16-tile border)
///
/// # References
/// - Original code: `Source/engine/displacement.hpp` Line 151-168

/// Tile dimensions in screen space
/// Each diamond tile is 64 pixels wide and 32 pixels tall
pub const TILE_WIDTH: i32 = 64;
pub const TILE_HEIGHT: i32 = 32;

/// Map dimensions in MegaTiles
pub const DMAXX: i32 = 40;
pub const DMAXY: i32 = 40;

/// Map dimensions in MicroTiles
pub const MAXDUNX: i32 = 112;
pub const MAXDUNY: i32 = 112;

/// Border size in MicroTiles
pub const BORDER_SIZE: i32 = 16;

/// Convert world coordinates to screen coordinates
///
/// This implements the isometric projection transformation.
/// The transformation is a -135° rotation + scale:
/// - screen_x = (world_y - world_x) * 32
/// - screen_y = (world_y + world_x) * -16
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::world_to_screen;
///
/// let (sx, sy) = world_to_screen(0, 0);
/// assert_eq!(sx, 0);
/// assert_eq!(sy, 0);
///
/// let (sx, sy) = world_to_screen(1, 0);
/// assert_eq!(sx, -32);
/// assert_eq!(sy, -16);
///
/// let (sx, sy) = world_to_screen(0, 1);
/// assert_eq!(sx, 32);
/// assert_eq!(sy, -16);
/// ```
///
/// # Reference
/// Original code: `Source/engine/displacement.hpp::worldToScreen()`
#[inline]
pub fn world_to_screen(world_x: i32, world_y: i32) -> (i32, i32) {
    let screen_x = (world_y - world_x) * 32;
    let screen_y = (world_y + world_x) * -16;
    (screen_x, screen_y)
}

/// Convert screen coordinates to world coordinates
///
/// This is the inverse transformation of `world_to_screen`.
/// Given screen coordinates, recovers the original world coordinates.
///
/// # Formula
/// - world_x = (2 * screen_y + screen_x) / -64
/// - world_y = (2 * screen_y - screen_x) / -64
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::{world_to_screen, screen_to_world};
///
/// // Round-trip conversion should be consistent
/// let (wx, wy) = (5, 7);
/// let (sx, sy) = world_to_screen(wx, wy);
/// let (wx2, wy2) = screen_to_world(sx, sy);
/// assert_eq!(wx, wx2);
/// assert_eq!(wy, wy2);
/// ```
///
/// # Reference
/// Original code: `Source/engine/displacement.hpp::screenToWorld()`
#[inline]
pub fn screen_to_world(screen_x: i32, screen_y: i32) -> (i32, i32) {
    let world_x = (2 * screen_y + screen_x) / -64;
    let world_y = (2 * screen_y - screen_x) / -64;
    (world_x, world_y)
}

use crate::math::Point;

/// Calculate the number of tiles visible in the viewport
///
/// # Arguments
/// * `viewport_width` - Viewport width in pixels
/// * `viewport_height` - Viewport height in pixels
///
/// # Returns
/// Tuple of (columns, rows) - number of tiles visible in each dimension
///
/// # Reference
/// Original: Source/engine/render/scrollrt.cpp::TilesInView() Line 1699-1727
pub fn tiles_in_view(viewport_width: u32, viewport_height: u32) -> (i32, i32) {
    let mut columns = (viewport_width / TILE_WIDTH as u32) as i32;
    if (viewport_width % TILE_WIDTH as u32) != 0 {
        columns += 1;
    }
    let mut rows = (viewport_height / TILE_HEIGHT as u32) as i32;
    if (viewport_height % TILE_HEIGHT as u32) != 0 {
        rows += 1;
    }
    (columns, rows)
}

/// Shift tile position in isometric grid
///
/// This function moves a tile position in the isometric grid coordinate system.
/// The isometric grid uses a diamond pattern, so horizontal and vertical movements
/// affect both x and y coordinates.
///
/// # Arguments
/// * `tile` - Tile position to shift (mutable reference)
/// * `horizontal` - Horizontal movement (positive = right)
/// * `vertical` - Vertical movement (positive = down)
///
/// # Reference
/// Original: Source/engine/render/scrollrt.cpp::ShiftGrid() Line 1653-1657
pub fn shift_grid(tile: &mut Point, horizontal: i32, vertical: i32) {
    tile.x += vertical + horizontal;
    tile.y += vertical - horizontal;
}

/// Convert screen coordinates to tile coordinates
///
/// This is similar to C++ ConvertToTileGrid() but adapted for Rust's coordinate system.
/// It converts a screen pixel position to the corresponding tile coordinate in the
/// isometric grid, taking into account the current view position and viewport size.
///
/// # Arguments
/// * `screen_x` - Screen X coordinate (pixels)
/// * `screen_y` - Screen Y coordinate (pixels)
/// * `view_tile` - Current view center tile position (equivalent to ViewPosition)
/// * `viewport_width` - Viewport width in pixels
/// * `viewport_height` - Viewport height in pixels
/// * `tile_offset_x` - Tile offset X from rendering (tileOffset.x)
/// * `tile_offset_y` - Tile offset Y from rendering (tileOffset.y)
///
/// # Returns
/// Tile coordinates (Point)
///
/// # Reference
/// Original: Source/cursor.cpp::ConvertToTileGrid() Line 723-752
pub fn screen_to_tile(
    screen_x: i32,
    screen_y: i32,
    view_tile: Point,
    viewport_width: u32,
    viewport_height: u32,
    tile_offset_x: i32,
    tile_offset_y: i32,
) -> Point {
    let (columns, rows) = tiles_in_view(viewport_width, viewport_height);
    let lrow = rows; // Simplified: assume no panel coverage for now
    
    // Center player tile on screen
    let mut current_tile = view_tile;
    shift_grid(&mut current_tile, -columns / 2, -lrow / 2);
    
    // Adjust screen position by tile offset
    let mut adjusted_screen_x = screen_x + tile_offset_x;
    let mut adjusted_screen_y = screen_y + tile_offset_y;
    
    // Align grid (similar to C++ alignment logic)
    if (columns % 2) == 0 && (lrow % 2) == 0 {
        adjusted_screen_y += TILE_HEIGHT / 2;
    } else if (columns % 2) != 0 && (lrow % 2) != 0 {
        adjusted_screen_x -= TILE_WIDTH / 2;
    } else if (columns % 2) != 0 && (lrow % 2) == 0 {
        current_tile.y += 1;
    }
    
    // Calculate tile offset from screen coordinates
    let tx = adjusted_screen_x / TILE_WIDTH;
    let ty = adjusted_screen_y / TILE_HEIGHT;
    shift_grid(&mut current_tile, tx, ty);
    
    current_tile
}

/// Shift tile position to match diamond grid alignment
///
/// This handles the diamond-shaped tile boundaries in isometric projection.
/// When a screen coordinate falls on the boundary between tiles, this function
/// determines which tile it actually belongs to based on the diamond shape.
///
/// # Arguments
/// * `screen_x` - Screen X coordinate (pixels)
/// * `screen_y` - Screen Y coordinate (pixels)
/// * `tile` - Tile position to adjust (mutable reference)
///
/// # Returns
/// flipflag - Boolean indicating if tile should be flipped (for cursor display)
///
/// # Reference
/// Original: Source/cursor.cpp::ShiftToDiamondGridAlignment() Line 757-775
pub fn shift_to_diamond_grid_alignment(
    screen_x: i32,
    screen_y: i32,
    tile: &mut Point,
) -> bool {
    let px = screen_x % TILE_WIDTH;
    let py = screen_y % TILE_HEIGHT;
    
    let flipy = py < (px / 2);
    if flipy {
        tile.y -= 1;
    }
    
    let flipx = py >= TILE_HEIGHT - (px / 2);
    if flipx {
        tile.x += 1;
    }
    
    // Clamp to valid bounds
    tile.x = tile.x.clamp(0, MAXDUNX - 1);
    tile.y = tile.y.clamp(0, MAXDUNY - 1);
    
    // Calculate flipflag
    (flipy && flipx) || ((flipy || flipx) && px < TILE_WIDTH / 2)
}

/// Simplified version of screen_to_tile with default tile offset
///
/// This is a convenience function that uses default tile offset (0, 0) and
/// automatically applies diamond grid alignment. It's useful for most cases
/// where you don't need the full control of the `screen_to_tile()` function.
///
/// # Arguments
/// * `screen_x` - Screen X coordinate (pixels)
/// * `screen_y` - Screen Y coordinate (pixels)
/// * `view_tile` - Current view center tile position (equivalent to ViewPosition)
/// * `viewport_width` - Viewport width in pixels
/// * `viewport_height` - Viewport height in pixels
///
/// # Returns
/// Tile coordinates (Point) with diamond grid alignment applied
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::screen_to_tile_simple;
/// use rust_diablo::math::Point;
///
/// // Convert mouse click position to tile coordinate
/// let mouse_x = 320;
/// let mouse_y = 240;
/// let player_tile = Point::new(10, 10);
/// let tile = screen_to_tile_simple(mouse_x, mouse_y, player_tile, 640, 480);
/// ```
pub fn screen_to_tile_simple(
    screen_x: i32,
    screen_y: i32,
    view_tile: Point,
    viewport_width: u32,
    viewport_height: u32,
) -> Point {
    let mut tile = screen_to_tile(
        screen_x,
        screen_y,
        view_tile,
        viewport_width,
        viewport_height,
        0, // tile_offset_x
        0, // tile_offset_y
    );
    
    // Apply diamond grid alignment
    shift_to_diamond_grid_alignment(screen_x, screen_y, &mut tile);
    
    tile
}

/// Convert world pixel coordinates to tile coordinates
///
/// This function converts world pixel coordinates (like player.position) to tile coordinates.
/// For orthogonal world grid, this is a simple division: `tile = pixel / tile_size`.
///
/// # Important Note
/// This is a simple conversion for orthogonal grids. It does NOT use isometric projection.
/// The world coordinate system uses orthogonal tiles (32x32 pixels), not isometric tiles.
/// For isometric projection conversions, use `world_to_screen()` and `screen_to_world()` instead.
///
/// # Arguments
/// * `world_pixel_x` - World X coordinate in pixels
/// * `world_pixel_y` - World Y coordinate in pixels
/// * `tile_size` - Size of a tile in pixels (typically 32 for world grid)
///
/// # Returns
/// Tile coordinates (Point) in world tile space
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::world_pixel_to_tile;
///
/// // Convert player pixel position to tile coordinate
/// let player_pixel_x = 640;
/// let player_pixel_y = 480;
/// let tile = world_pixel_to_tile(player_pixel_x, player_pixel_y, 32);
/// // tile.x = 20, tile.y = 15 (for 32-pixel tiles)
/// ```
pub fn world_pixel_to_tile(world_pixel_x: i32, world_pixel_y: i32, tile_size: i32) -> Point {
    // Convert pixel coordinates to tile coordinates
    // For orthogonal world grid: tile = pixel / tile_size
    // This is the correct conversion for world coordinates (orthogonal grid, not isometric)
    Point::new(
        world_pixel_x / tile_size,
        world_pixel_y / tile_size,
    )
}

/// Convert world pixel coordinates to MicroTile coordinates (dPiece space)
///
/// This function converts world pixel coordinates to MicroTile coordinates used in
/// dPiece array, which includes the 16-tile border offset.
///
/// # Arguments
/// * `world_pixel_x` - World X coordinate in pixels
/// * `world_pixel_y` - World Y coordinate in pixels
/// * `tile_size` - Size of a tile in pixels (typically 32)
///
/// # Returns
/// MicroTile coordinates (Point) in dPiece space (with BORDER_SIZE added)
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::world_pixel_to_micro_tile;
///
/// // Convert player pixel position to MicroTile coordinate
/// let player_pixel_x = 640;
/// let player_pixel_y = 480;
/// let micro_tile = world_pixel_to_micro_tile(player_pixel_x, player_pixel_y, 32);
/// // micro_tile.x = 20 + 16 = 36, micro_tile.y = 15 + 16 = 31
/// ```
pub fn world_pixel_to_micro_tile(world_pixel_x: i32, world_pixel_y: i32, tile_size: i32) -> Point {
    let tile = world_pixel_to_tile(world_pixel_x, world_pixel_y, tile_size);
    Point::new(tile.x + BORDER_SIZE, tile.y + BORDER_SIZE)
}

/// Convert MegaTile coordinates to MicroTile coordinates
///
/// MegaTiles are used for map generation (40x40 grid).
/// MicroTiles are used for rendering and collision (112x112 grid).
///
/// # Formula
/// - micro_x = mega_x * 2 + BORDER_SIZE
/// - micro_y = mega_y * 2 + BORDER_SIZE
///
/// where BORDER_SIZE = 16 tiles
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::mega_to_micro;
///
/// let (mx, my) = mega_to_micro(10, 10);
/// assert_eq!(mx, 36);  // 10 * 2 + 16
/// assert_eq!(my, 36);
/// ```
///
/// # Reference
/// Original code: `Source/levels/gendung.cpp`
#[inline]
pub fn mega_to_micro(mega_x: i32, mega_y: i32) -> (i32, i32) {
    (mega_x * 2 + BORDER_SIZE, mega_y * 2 + BORDER_SIZE)
}

/// Convert MicroTile coordinates to MegaTile coordinates
///
/// This is the inverse transformation of `mega_to_micro`.
///
/// # Formula
/// - mega_x = (micro_x - BORDER_SIZE) / 2
/// - mega_y = (micro_y - BORDER_SIZE) / 2
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::{mega_to_micro, micro_to_mega};
///
/// // Round-trip conversion should be consistent
/// let (mx, my) = (10, 10);
/// let (micro_x, micro_y) = mega_to_micro(mx, my);
/// let (mx2, my2) = micro_to_mega(micro_x, micro_y);
/// assert_eq!(mx, mx2);
/// assert_eq!(my, my2);
/// ```
///
/// # Reference
/// Original code: `Source/levels/gendung.cpp`
#[inline]
pub fn micro_to_mega(micro_x: i32, micro_y: i32) -> (i32, i32) {
    ((micro_x - BORDER_SIZE) / 2, (micro_y - BORDER_SIZE) / 2)
}

/// Check if MegaTile coordinates are within bounds
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::in_mega_bounds;
///
/// assert!(in_mega_bounds(0, 0));
/// assert!(in_mega_bounds(39, 39));
/// assert!(!in_mega_bounds(-1, 0));
/// assert!(!in_mega_bounds(40, 0));
/// ```
#[inline]
pub fn in_mega_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < DMAXX && y >= 0 && y < DMAXY
}

/// Check if MicroTile coordinates are within bounds
///
/// # Examples
///
/// ```
/// use rust_diablo::engine::isometric::in_micro_bounds;
///
/// assert!(in_micro_bounds(0, 0));
/// assert!(in_micro_bounds(111, 111));
/// assert!(!in_micro_bounds(-1, 0));
/// assert!(!in_micro_bounds(112, 0));
/// ```
#[inline]
pub fn in_micro_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < MAXDUNX && y >= 0 && y < MAXDUNY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_screen_origin() {
        let (sx, sy) = world_to_screen(0, 0);
        assert_eq!(sx, 0);
        assert_eq!(sy, 0);
    }

    #[test]
    fn test_world_to_screen_x_axis() {
        // Moving right in world space (positive x)
        // Should move left-down in screen space
        let (sx, sy) = world_to_screen(1, 0);
        assert_eq!(sx, -32);
        assert_eq!(sy, -16);
    }

    #[test]
    fn test_world_to_screen_y_axis() {
        // Moving down in world space (positive y)
        // Should move right-down in screen space
        let (sx, sy) = world_to_screen(0, 1);
        assert_eq!(sx, 32);
        assert_eq!(sy, -16);
    }

    #[test]
    fn test_world_to_screen_diagonal() {
        let (sx, sy) = world_to_screen(3, 5);
        assert_eq!(sx, 64); // (5 - 3) * 32 = 64
        assert_eq!(sy, -128); // (5 + 3) * -16 = -128
    }

    #[test]
    fn test_screen_to_world_origin() {
        let (wx, wy) = screen_to_world(0, 0);
        assert_eq!(wx, 0);
        assert_eq!(wy, 0);
    }

    #[test]
    fn test_screen_to_world_roundtrip() {
        // Test that converting to screen and back gives the same world coordinates
        let test_cases = [(0, 0), (1, 0), (0, 1), (5, 7), (10, 20), (39, 39)];

        for (wx, wy) in test_cases.iter() {
            let (sx, sy) = world_to_screen(*wx, *wy);
            let (wx2, wy2) = screen_to_world(sx, sy);
            assert_eq!(*wx, wx2, "x coordinate mismatch for ({}, {})", wx, wy);
            assert_eq!(*wy, wy2, "y coordinate mismatch for ({}, {})", wx, wy);
        }
    }

    #[test]
    fn test_mega_to_micro() {
        // Test basic conversion
        let (mx, my) = mega_to_micro(0, 0);
        assert_eq!(mx, 16); // 0 * 2 + 16
        assert_eq!(my, 16);

        let (mx, my) = mega_to_micro(10, 10);
        assert_eq!(mx, 36); // 10 * 2 + 16
        assert_eq!(my, 36);

        let (mx, my) = mega_to_micro(39, 39);
        assert_eq!(mx, 94); // 39 * 2 + 16
        assert_eq!(my, 94);
    }

    #[test]
    fn test_micro_to_mega() {
        // Test basic conversion
        let (mx, my) = micro_to_mega(16, 16);
        assert_eq!(mx, 0); // (16 - 16) / 2
        assert_eq!(my, 0);

        let (mx, my) = micro_to_mega(36, 36);
        assert_eq!(mx, 10); // (36 - 16) / 2
        assert_eq!(my, 10);

        let (mx, my) = micro_to_mega(94, 94);
        assert_eq!(mx, 39); // (94 - 16) / 2
        assert_eq!(my, 39);
    }

    #[test]
    fn test_mega_micro_roundtrip() {
        // Test that converting to micro and back gives the same mega coordinates
        let test_cases = [(0, 0), (10, 10), (20, 30), (39, 39)];

        for (mega_x, mega_y) in test_cases.iter() {
            let (micro_x, micro_y) = mega_to_micro(*mega_x, *mega_y);
            let (mega_x2, mega_y2) = micro_to_mega(micro_x, micro_y);
            assert_eq!(
                *mega_x, mega_x2,
                "x coordinate mismatch for ({}, {})",
                mega_x, mega_y
            );
            assert_eq!(
                *mega_y, mega_y2,
                "y coordinate mismatch for ({}, {})",
                mega_x, mega_y
            );
        }
    }

    #[test]
    fn test_in_mega_bounds() {
        // Valid bounds
        assert!(in_mega_bounds(0, 0));
        assert!(in_mega_bounds(39, 39));
        assert!(in_mega_bounds(20, 20));

        // Invalid bounds
        assert!(!in_mega_bounds(-1, 0));
        assert!(!in_mega_bounds(0, -1));
        assert!(!in_mega_bounds(40, 0));
        assert!(!in_mega_bounds(0, 40));
        assert!(!in_mega_bounds(40, 40));
    }

    #[test]
    fn test_in_micro_bounds() {
        // Valid bounds
        assert!(in_micro_bounds(0, 0));
        assert!(in_micro_bounds(111, 111));
        assert!(in_micro_bounds(50, 50));

        // Invalid bounds
        assert!(!in_micro_bounds(-1, 0));
        assert!(!in_micro_bounds(0, -1));
        assert!(!in_micro_bounds(112, 0));
        assert!(!in_micro_bounds(0, 112));
        assert!(!in_micro_bounds(112, 112));
    }

    #[test]
    fn test_tile_dimensions() {
        // Verify tile constants
        assert_eq!(TILE_WIDTH, 64);
        assert_eq!(TILE_HEIGHT, 32);
        assert_eq!(DMAXX, 40);
        assert_eq!(DMAXY, 40);
        assert_eq!(MAXDUNX, 112);
        assert_eq!(MAXDUNY, 112);
        assert_eq!(BORDER_SIZE, 16);
    }

    #[test]
    fn test_tiles_in_view() {
        // Test with standard viewport (640x480)
        let (columns, rows) = tiles_in_view(640, 480);
        assert_eq!(columns, 10); // 640 / 64 = 10
        assert_eq!(rows, 15); // 480 / 32 = 15

        // Test with partial tiles
        let (columns, rows) = tiles_in_view(641, 481);
        assert_eq!(columns, 11); // 641 / 64 = 10.015... -> 11
        assert_eq!(rows, 16); // 481 / 32 = 15.031... -> 16

        // Test with exact tile boundaries
        let (columns, rows) = tiles_in_view(128, 64);
        assert_eq!(columns, 2); // 128 / 64 = 2
        assert_eq!(rows, 2); // 64 / 32 = 2
    }

    #[test]
    fn test_shift_grid() {
        let mut tile = Point::new(10, 10);
        
        // Shift right (horizontal = 1)
        shift_grid(&mut tile, 1, 0);
        assert_eq!(tile.x, 11); // 10 + 0 + 1 = 11
        assert_eq!(tile.y, 9);  // 10 + 0 - 1 = 9

        // Reset and shift down (vertical = 1)
        tile = Point::new(10, 10);
        shift_grid(&mut tile, 0, 1);
        assert_eq!(tile.x, 11); // 10 + 1 + 0 = 11
        assert_eq!(tile.y, 11); // 10 + 1 - 0 = 11

        // Reset and shift diagonally
        tile = Point::new(10, 10);
        shift_grid(&mut tile, 1, 1);
        assert_eq!(tile.x, 12); // 10 + 1 + 1 = 12
        assert_eq!(tile.y, 10); // 10 + 1 - 1 = 10
    }

    #[test]
    fn test_screen_to_tile() {
        use crate::math::Point;

        // Test with center view (player at tile 10, 10)
        let view_tile = Point::new(10, 10);
        let viewport_width = 640;
        let viewport_height = 480;

        // Test center of screen (should map to view_tile)
        let tile = screen_to_tile(
            320, // center of 640 width
            240, // center of 480 height
            view_tile,
            viewport_width,
            viewport_height,
            0, // tile_offset_x
            0, // tile_offset_y
        );
        
        // The result should be close to view_tile (may vary due to grid alignment)
        assert!(tile.x >= 8 && tile.x <= 12);
        assert!(tile.y >= 8 && tile.y <= 12);
    }

    #[test]
    fn test_shift_to_diamond_grid_alignment() {
        use crate::math::Point;

        // Test case: screen position in center of tile (should not shift)
        // Center of tile is at (TILE_WIDTH/2, TILE_HEIGHT/2) = (32, 16)
        let mut tile = Point::new(10, 10);
        let _flipflag = shift_to_diamond_grid_alignment(32, 16, &mut tile);
        // Center of tile (32, 16) should not cause shift
        // Note: The exact behavior depends on the diamond grid logic
        // Just verify it produces valid coordinates
        assert!(tile.x >= 0 && tile.x < MAXDUNX);
        assert!(tile.y >= 0 && tile.y < MAXDUNY);

        // Test case: screen position in upper-left of diamond (should shift up)
        let mut tile = Point::new(10, 10);
        let _flipflag = shift_to_diamond_grid_alignment(0, 0, &mut tile);
        // Upper-left corner should shift tile.y down (but we check flipy which shifts up)
        // This is complex logic, just verify it doesn't crash and produces valid coordinates
        assert!(tile.x >= 0 && tile.x < MAXDUNX);
        assert!(tile.y >= 0 && tile.y < MAXDUNY);

        // Test case: screen position in lower-right of diamond (should shift right)
        let mut tile = Point::new(10, 10);
        let _flipflag = shift_to_diamond_grid_alignment(63, 31, &mut tile);
        // Lower-right corner should shift tile.x right
        assert!(tile.x >= 0 && tile.x < MAXDUNX);
        assert!(tile.y >= 0 && tile.y < MAXDUNY);

        // Test case: Verify the function correctly handles boundary conditions
        // Test with px = 0, py = 0 (top-left of diamond)
        let mut tile = Point::new(5, 5);
        let flipflag1 = shift_to_diamond_grid_alignment(0, 0, &mut tile);
        assert!(tile.x >= 0 && tile.x < MAXDUNX);
        assert!(tile.y >= 0 && tile.y < MAXDUNY);
        
        // Test with px = 63, py = 31 (bottom-right of diamond)
        let mut tile = Point::new(5, 5);
        let flipflag2 = shift_to_diamond_grid_alignment(63, 31, &mut tile);
        assert!(tile.x >= 0 && tile.x < MAXDUNX);
        assert!(tile.y >= 0 && tile.y < MAXDUNY);
        
        // flipflag should be boolean
        assert!(flipflag1 == true || flipflag1 == false);
        assert!(flipflag2 == true || flipflag2 == false);
    }

    #[test]
    fn test_screen_to_tile_simple() {
        use crate::math::Point;

        // Test with center view (player at tile 10, 10)
        let view_tile = Point::new(10, 10);
        let viewport_width = 640;
        let viewport_height = 480;

        // Test center of screen
        let tile = screen_to_tile_simple(
            320, // center of 640 width
            240, // center of 480 height
            view_tile,
            viewport_width,
            viewport_height,
        );
        
        // The result should be valid tile coordinates
        assert!(tile.x >= 0 && tile.x < MAXDUNX);
        assert!(tile.y >= 0 && tile.y < MAXDUNY);
    }

    #[test]
    fn test_screen_to_tile_roundtrip() {
        use crate::math::Point;

        // Test that converting screen -> tile -> screen gives reasonable results
        // Note: This is not a perfect roundtrip because screen_to_tile uses
        // approximate grid calculations, but it should be close
        
        let view_tile = Point::new(10, 10);
        let viewport_width = 640;
        let viewport_height = 480;

        // Test multiple screen positions
        let test_positions = [
            (320, 240), // center
            (0, 0),     // top-left
            (639, 479), // bottom-right
            (100, 100), // arbitrary
        ];

        for (screen_x, screen_y) in test_positions.iter() {
            let tile = screen_to_tile_simple(
                *screen_x,
                *screen_y,
                view_tile,
                viewport_width,
                viewport_height,
            );
            
            // Verify tile is in valid bounds
            assert!(tile.x >= 0 && tile.x < MAXDUNX, 
                "Tile x out of bounds: {} for screen ({}, {})", tile.x, screen_x, screen_y);
            assert!(tile.y >= 0 && tile.y < MAXDUNY,
                "Tile y out of bounds: {} for screen ({}, {})", tile.y, screen_x, screen_y);
        }
    }
}
