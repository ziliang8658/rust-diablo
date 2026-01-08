/// Light ray casting module
///
/// Reference: Source/lighting.cpp::DoCrawl()
use super::light_source::LightSource;
use super::{MAXDUNX, MAXDUNY};

/// Ray cast lighting - calculate illuminated area from light source
///
/// Reference: Source/lighting.cpp::DoCrawl() Line 124-156
///
/// # Arguments
/// * `grid` - Light intensity grid to update
/// * `source` - Light source
/// * `block_map` - Wall blocking map
pub fn crawl_light(
    grid: &mut [[u8; MAXDUNY]; MAXDUNX],
    source: &LightSource,
    block_map: &[[bool; MAXDUNY]; MAXDUNX],
) {
    let (cx, cy) = source.position;
    let radius = source.radius as i32;

    // Clear previous lighting in radius
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = cx as i32 + dx;
            let y = cy as i32 + dy;

            if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
                grid[x as usize][y as usize] = 0;
            }
        }
    }

    // Cast light rays from center outward
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();

            if dist > radius as f32 {
                continue;
            }

            let x = cx as i32 + dx;
            let y = cy as i32 + dy;

            if x < 0 || x >= MAXDUNX as i32 || y < 0 || y >= MAXDUNY as i32 {
                continue;
            }

            // Check if light ray is clear
            if is_line_clear(cx, cy, x as usize, y as usize, block_map) {
                // Calculate light intensity (farther = darker)
                let intensity = 15.0 * (1.0 - dist / radius as f32);
                grid[x as usize][y as usize] = intensity.max(0.0).min(15.0) as u8;
            }
        }
    }
}

/// Bresenham line algorithm - check if light ray is blocked
///
/// Reference: Bresenham's line algorithm
///
/// # Returns
/// true if line is clear, false if blocked by wall
fn is_line_clear(
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    block_map: &[[bool; MAXDUNY]; MAXDUNX],
) -> bool {
    // Clamp coordinates to valid range to prevent overflow
    let x0 = x0.min(MAXDUNX - 1);
    let y0 = y0.min(MAXDUNY - 1);
    let x1 = x1.min(MAXDUNX - 1);
    let y1 = y1.min(MAXDUNY - 1);
    
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        // Check current point (bounds already checked by clamping)
        if x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32 {
            if block_map[x as usize][y as usize] {
                return false; // Blocked by wall
            }
        }

        if x == x1 as i32 && y == y1 as i32 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            // Check for overflow before adding
            let new_x = x.checked_add(sx);
            if let Some(nx) = new_x {
                if nx >= 0 && nx < MAXDUNX as i32 {
                    x = nx;
                } else {
                    break; // Out of bounds, stop
                }
            } else {
                break; // Overflow, stop
            }
        }
        if e2 < dx {
            err += dx;
            // Check for overflow before adding
            let new_y = y.checked_add(sy);
            if let Some(ny) = new_y {
                if ny >= 0 && ny < MAXDUNY as i32 {
                    y = ny;
                } else {
                    break; // Out of bounds, stop
                }
            } else {
                break; // Overflow, stop
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lighting::LightSource;

    #[test]
    fn test_is_line_clear_straight() {
        let block_map = [[false; MAXDUNY]; MAXDUNX];

        // Unblocked straight lines should be clear
        assert!(is_line_clear(50, 50, 60, 50, &block_map));
        assert!(is_line_clear(50, 50, 50, 60, &block_map));
    }

    #[test]
    fn test_is_line_clear_blocked() {
        let mut block_map = [[false; MAXDUNY]; MAXDUNX];

        // Place wall
        block_map[55][50] = true;

        // Line through wall should be blocked
        assert!(!is_line_clear(50, 50, 60, 50, &block_map));
    }

    #[test]
    fn test_crawl_light_attenuation() {
        let mut grid = [[0u8; MAXDUNY]; MAXDUNX];
        let source = LightSource::player((50, 50));
        let block_map = [[false; MAXDUNY]; MAXDUNX];

        crawl_light(&mut grid, &source, &block_map);

        // Center should be brightest
        let center_light = grid[50][50];
        assert!(
            center_light >= 13,
            "Center should be bright, got {}",
            center_light
        );

        // Distance increases, light decreases
        let dist_1 = grid[51][50];
        let dist_2 = grid[52][50];
        let dist_3 = grid[53][50];

        assert!(center_light > dist_1);
        assert!(dist_1 >= dist_2);
        assert!(dist_2 >= dist_3);
    }

    #[test]
    fn test_crawl_light_radius() {
        let mut grid = [[0u8; MAXDUNY]; MAXDUNX];
        let source = LightSource::player((50, 50));
        let block_map = [[false; MAXDUNY]; MAXDUNX];

        crawl_light(&mut grid, &source, &block_map);

        // Within radius should be lit
        assert!(grid[55][50] > 0, "Within radius should be lit");
        assert!(grid[50][55] > 0, "Within radius should be lit");

        // Beyond radius should be dark
        let far_x = 50 + source.radius as usize + 2;
        let far_y = 50 + source.radius as usize + 2;
        if far_x < MAXDUNX {
            assert_eq!(grid[far_x][50], 0, "Beyond radius should be dark");
        }
        if far_y < MAXDUNY {
            assert_eq!(grid[50][far_y], 0, "Beyond radius should be dark");
        }
    }

    #[test]
    fn test_light_blocking() {
        let mut grid = [[0u8; MAXDUNY]; MAXDUNX];
        let source = LightSource::player((50, 50));
        let mut block_map = [[false; MAXDUNY]; MAXDUNX];

        // Place wall
        block_map[55][50] = true;

        crawl_light(&mut grid, &source, &block_map);

        // Before wall should be lit
        assert!(grid[54][50] > 0, "Before wall should be lit");

        // Behind wall should be dark
        assert!(
            grid[56][50] < 5,
            "Behind wall should be dark, got {}",
            grid[56][50]
        );
        assert!(grid[57][50] < 5, "Far behind wall should be dark");
    }
}
