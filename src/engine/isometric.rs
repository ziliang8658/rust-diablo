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
}
