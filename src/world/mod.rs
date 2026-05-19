use crate::debug::RenderDebugFlags;
use crate::engine::{Direction, Engine};
use crate::entity::Entity;
/// World module - Game world and level system
///
/// This module manages the game world, including:
/// - Grid-based map
/// - Entity management
/// - World rendering
use crate::lighting::LightingSystem;
use crate::math::{Point, Rect};
use crate::renderer::{Camera, Color};
use crate::resources::Palette;
use crate::sprite::AnimationState;
use crate::tiles::{texture_manager::TileTextureManager, MinData, SolData, TilData};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use std::time::{Duration, Instant};

pub mod collision;
pub mod dungeon_map;
pub mod town;

pub use collision::{CollisionMap, TileType};
pub use dungeon_map::{DungeonMap, DMAXX, DMAXY, MAXDUNX, MAXDUNY};
pub use town::SimpleTown;

const RENDER_FOCUS_TRACE_SAMPLE_INTERVAL: u64 = 8;
const RENDER_PERF_REPORT_INTERVAL: Duration = Duration::from_secs(1);
static RENDER_FOCUS_TRACE_FRAME: AtomicU64 = AtomicU64::new(0);
static RENDER_PERF_ACCUMULATOR: Mutex<RenderPerfAccumulator> =
    Mutex::new(RenderPerfAccumulator::new());
static MICRO_TEXTURE_CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static MICRO_TEXTURE_CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static FOLIAGE_TEXTURE_CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static FOLIAGE_TEXTURE_CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Default)]
struct RenderPerfFrame {
    total_time: Duration,
    floor_time: Duration,
    wall_time: Duration,
    floor_tiles: u64,
    wall_scan_tiles: u64,
    wall_cells: u64,
    predraw_cells: u64,
    micro_cache_hits: u64,
    micro_cache_misses: u64,
    foliage_cache_hits: u64,
    foliage_cache_misses: u64,
    texture_cache_size: u64,
}

struct RenderPerfAccumulator {
    frames: u64,
    total_time: Duration,
    floor_time: Duration,
    wall_time: Duration,
    floor_tiles: u64,
    wall_scan_tiles: u64,
    wall_cells: u64,
    predraw_cells: u64,
    micro_cache_hits: u64,
    micro_cache_misses: u64,
    foliage_cache_hits: u64,
    foliage_cache_misses: u64,
    texture_cache_size: u64,
    last_report: Option<Instant>,
}

impl RenderPerfAccumulator {
    const fn new() -> Self {
        Self {
            frames: 0,
            total_time: Duration::ZERO,
            floor_time: Duration::ZERO,
            wall_time: Duration::ZERO,
            floor_tiles: 0,
            wall_scan_tiles: 0,
            wall_cells: 0,
            predraw_cells: 0,
            micro_cache_hits: 0,
            micro_cache_misses: 0,
            foliage_cache_hits: 0,
            foliage_cache_misses: 0,
            texture_cache_size: 0,
            last_report: None,
        }
    }

    fn add_frame(&mut self, frame: RenderPerfFrame) {
        self.frames += 1;
        self.total_time += frame.total_time;
        self.floor_time += frame.floor_time;
        self.wall_time += frame.wall_time;
        self.floor_tiles += frame.floor_tiles;
        self.wall_scan_tiles += frame.wall_scan_tiles;
        self.wall_cells += frame.wall_cells;
        self.predraw_cells += frame.predraw_cells;
        self.micro_cache_hits += frame.micro_cache_hits;
        self.micro_cache_misses += frame.micro_cache_misses;
        self.foliage_cache_hits += frame.foliage_cache_hits;
        self.foliage_cache_misses += frame.foliage_cache_misses;
        self.texture_cache_size = frame.texture_cache_size;
    }

    fn reset(&mut self, now: Instant) {
        *self = Self::new();
        self.last_report = Some(now);
    }
}

#[derive(Clone, Copy, Debug)]
struct RenderFocus {
    enabled: bool,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

impl RenderFocus {
    fn disabled() -> Self {
        Self {
            enabled: false,
            min_x: 0,
            max_x: 0,
            min_y: 0,
            max_y: 0,
        }
    }

    fn centered_2x2(view_world_x: i32, view_world_y: i32, border_size: i32) -> Self {
        let cx = view_world_x + border_size;
        let cy = view_world_y + border_size;
        Self {
            enabled: true,
            min_x: cx,
            max_x: cx + 1,
            min_y: cy,
            max_y: cy + 1,
        }
    }

    fn contains(&self, dpiece_x: i32, dpiece_y: i32) -> bool {
        !self.enabled
            || (dpiece_x >= self.min_x
                && dpiece_x <= self.max_x
                && dpiece_y >= self.min_y
                && dpiece_y <= self.max_y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ViewportGeometry {
    tile_shift: Point,
    tile_offset: Point,
    columns: i32,
    rows: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WalkingViewportExpansion {
    tile_shift: Point,
    screen_offset: Point,
    extra_columns: i32,
    extra_rows: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RenderScanGeometry {
    start_tile: Point,
    tile_offset: Point,
    columns: i32,
    rows: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WallContentScanPadding {
    tile_shift: Point,
    screen_x_shift: i32,
    extra_columns: i32,
}

impl Default for WalkingViewportExpansion {
    fn default() -> Self {
        Self {
            tile_shift: Point::zero(),
            screen_offset: Point::zero(),
            extra_columns: 0,
            extra_rows: 0,
        }
    }
}

fn scaled_direction(direction: Direction, scale: i32) -> Point {
    let (x, y) = direction.to_tile_offset();
    Point::new(x * scale, y * scale)
}

fn calc_viewport_geometry(screen_width: i32, viewport_height: i32) -> ViewportGeometry {
    // Port of DevilutionX CalcViewportGeometry() for the current Rust renderer:
    // no zoom and no control-panel split yet, so the player sits at viewport center.
    let player_position = Point::new(screen_width / 2, viewport_height / 2);
    let tiles_to_top = (player_position.y + TILE_HEIGHT - 1) / TILE_HEIGHT;
    let tiles_to_left = (player_position.x + TILE_WIDTH - 1) / TILE_WIDTH;

    let mut start_position = Point::new(
        player_position.x - tiles_to_left * TILE_WIDTH,
        player_position.y - tiles_to_top * TILE_HEIGHT,
    );
    let mut tile_shift = scaled_direction(Direction::North, tiles_to_top)
        + scaled_direction(Direction::West, tiles_to_left);

    // The original render loop expects to start on the short row of the diamond.
    // Missing this half-row alignment is enough to drop tall wall base tiles at edges.
    if tiles_to_left * TILE_WIDTH >= player_position.x {
        start_position.x += TILE_WIDTH / 2;
        start_position.y -= TILE_HEIGHT / 2;
        tile_shift = tile_shift + Point::from(Direction::NorthEast.to_tile_offset());
    } else if tiles_to_top * TILE_HEIGHT < player_position.y {
        start_position.y -= TILE_HEIGHT;
        tile_shift = tile_shift + Point::from(Direction::North.to_tile_offset());
    }

    let tile_offset = Point::new(
        start_position.x - TILE_WIDTH / 2,
        start_position.y + TILE_HEIGHT / 2 - 1,
    );
    let render_start = Point::new(
        start_position.x - TILE_WIDTH / 2,
        start_position.y - TILE_HEIGHT / 2,
    );
    let rows = (viewport_height - render_start.y + TILE_HEIGHT / 2 - 1) / (TILE_HEIGHT / 2);
    let columns = (screen_width - render_start.x + TILE_WIDTH - 1) / TILE_WIDTH;

    ViewportGeometry {
        tile_shift,
        tile_offset,
        columns,
        rows,
    }
}

fn walking_viewport_expansion(direction: Direction) -> WalkingViewportExpansion {
    let mut expansion = WalkingViewportExpansion::default();

    match direction {
        Direction::North | Direction::NorthEast => {
            expansion.tile_shift = Point::from(Direction::North.to_tile_offset());
            expansion.screen_offset.y -= TILE_HEIGHT;
            expansion.extra_rows = 2;
            if direction == Direction::NorthEast {
                expansion.extra_columns = 1;
            }
        }
        Direction::South => {
            expansion.extra_rows = 2;
        }
        Direction::East | Direction::West => {
            expansion.extra_columns = 1;
            if direction == Direction::West {
                expansion.tile_shift = Point::from(Direction::West.to_tile_offset());
                expansion.screen_offset.x -= TILE_WIDTH;
            }
        }
        Direction::SouthEast | Direction::SouthWest => {
            expansion.extra_columns = 1;
            expansion.extra_rows = 1;
            if direction == Direction::SouthWest {
                expansion.tile_shift = Point::from(Direction::West.to_tile_offset());
                expansion.screen_offset.x -= TILE_WIDTH;
            }
        }
        Direction::NorthWest => {
            expansion.tile_shift = Point::from(Direction::NorthWest.to_tile_offset());
            expansion.screen_offset.x -= TILE_WIDTH / 2;
            expansion.screen_offset.y -= TILE_HEIGHT / 2;
            expansion.extra_columns = 1;
            expansion.extra_rows = 1;
        }
        Direction::None => {}
    }

    expansion
}

fn wall_content_scan_padding() -> WallContentScanPadding {
    // Keep a narrow content-only overscan while the Rust renderer still lacks
    // the original object/entity grids and panel/zoom variants. Floors stay on
    // the exact viewport geometry; tall wall content gets one safety column.
    const WALL_EDGE_PADDING_COLUMNS: i32 = 1;

    let west = Point::from(Direction::West.to_tile_offset());
    WallContentScanPadding {
        tile_shift: Point::new(
            west.x * WALL_EDGE_PADDING_COLUMNS,
            west.y * WALL_EDGE_PADDING_COLUMNS,
        ),
        screen_x_shift: -TILE_WIDTH * WALL_EDGE_PADDING_COLUMNS,
        extra_columns: WALL_EDGE_PADDING_COLUMNS * 2,
    }
}

fn wall_content_rows(rows: i32, micro_tile_len: usize) -> i32 {
    // DevilutionX DrawTileContent() does `rows += MicroTileLen`. MicroTileLen is
    // the number of micro blocks per piece, not the number of vertical pairs.
    rows + micro_tile_len as i32
}

fn calc_render_scan_geometry(
    view_position: Point,
    screen_width: i32,
    viewport_height: i32,
    walking_direction: Option<Direction>,
    walking_camera_offset: Point,
) -> RenderScanGeometry {
    let viewport_geometry = calc_viewport_geometry(screen_width, viewport_height);
    let mut scan = RenderScanGeometry {
        start_tile: view_position + viewport_geometry.tile_shift,
        tile_offset: Point::new(
            viewport_geometry.tile_offset.x - walking_camera_offset.x,
            viewport_geometry.tile_offset.y - walking_camera_offset.y,
        ),
        columns: viewport_geometry.columns,
        rows: viewport_geometry.rows,
    };

    if let Some(direction) = walking_direction {
        let expansion = walking_viewport_expansion(direction);
        scan.start_tile = scan.start_tile + expansion.tile_shift;
        scan.tile_offset = scan.tile_offset + expansion.screen_offset;
        scan.columns += expansion.extra_columns;
        scan.rows += expansion.extra_rows;
    }

    scan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_geometry_matches_cpp_calc_viewport_geometry_640x480() {
        assert_eq!(
            calc_viewport_geometry(640, 480),
            ViewportGeometry {
                tile_shift: Point::new(-13, -4),
                tile_offset: Point::new(0, -17),
                columns: 10,
                rows: 33,
            }
        );
    }

    #[test]
    fn test_viewport_geometry_matches_cpp_calc_viewport_geometry_640x352() {
        assert_eq!(
            calc_viewport_geometry(640, 352),
            ViewportGeometry {
                tile_shift: Point::new(-11, -2),
                tile_offset: Point::new(0, -17),
                columns: 10,
                rows: 25,
            }
        );
    }

    #[test]
    fn test_walking_offset_matches_world_projection() {
        for direction in Direction::walk_animation_order() {
            let (dx, dy) = direction.to_tile_offset();
            let (screen_x, screen_y) = world_to_screen(dx, dy);

            assert_eq!(
                direction.walking_render_offset(1.0),
                Point::new(screen_x, screen_y),
                "walking offset should match the rendered target tile for {:?}",
                direction
            );
        }
    }

    #[test]
    fn test_walking_viewport_expansion_matches_cpp_draw_areas() {
        let cases = [
            (Direction::None, Point::zero(), Point::zero(), 0, 0),
            (
                Direction::North,
                Point::new(-1, -1),
                Point::new(0, -TILE_HEIGHT),
                0,
                2,
            ),
            (
                Direction::NorthEast,
                Point::new(-1, -1),
                Point::new(0, -TILE_HEIGHT),
                1,
                2,
            ),
            (Direction::East, Point::zero(), Point::zero(), 1, 0),
            (Direction::SouthEast, Point::zero(), Point::zero(), 1, 1),
            (Direction::South, Point::zero(), Point::zero(), 0, 2),
            (
                Direction::SouthWest,
                Point::new(-1, 1),
                Point::new(-TILE_WIDTH, 0),
                1,
                1,
            ),
            (
                Direction::West,
                Point::new(-1, 1),
                Point::new(-TILE_WIDTH, 0),
                1,
                0,
            ),
            (
                Direction::NorthWest,
                Point::new(-1, 0),
                Point::new(-TILE_WIDTH / 2, -TILE_HEIGHT / 2),
                1,
                1,
            ),
        ];

        for (direction, tile_shift, screen_offset, extra_columns, extra_rows) in cases {
            assert_eq!(
                walking_viewport_expansion(direction),
                WalkingViewportExpansion {
                    tile_shift,
                    screen_offset,
                    extra_columns,
                    extra_rows,
                },
                "walking expansion should match CalcFirstTilePosition/DrawGame for {direction:?}"
            );
        }
    }

    #[test]
    fn test_render_scan_geometry_applies_camera_mode_offset() {
        let view_position = Point::new(56, 56);
        let base = calc_viewport_geometry(640, 480);
        let scan = calc_render_scan_geometry(
            view_position,
            640,
            480,
            Some(Direction::East),
            Direction::East.walking_render_offset(0.5),
        );

        assert_eq!(base.tile_offset, Point::new(0, -17));
        assert_eq!(
            scan.tile_offset,
            Point::new(base.tile_offset.x - 32, base.tile_offset.y)
        );
        assert_eq!(scan.start_tile, view_position + base.tile_shift);
        assert_eq!(scan.columns, base.columns + 1);
        assert_eq!(scan.rows, base.rows);
    }

    #[test]
    fn test_render_scan_geometry_stays_base_when_camera_offset_disabled() {
        let view_position = Point::new(56, 56);
        let base = calc_viewport_geometry(640, 480);
        let scan = calc_render_scan_geometry(view_position, 640, 480, None, Point::zero());

        assert_eq!(
            scan,
            RenderScanGeometry {
                start_tile: view_position + base.tile_shift,
                tile_offset: base.tile_offset,
                columns: base.columns,
                rows: base.rows,
            }
        );
    }

    #[test]
    fn test_wall_content_scan_pads_one_column_each_side() {
        assert_eq!(
            wall_content_scan_padding(),
            WallContentScanPadding {
                tile_shift: Point::new(-1, 1),
                screen_x_shift: -TILE_WIDTH,
                extra_columns: 2,
            }
        );
    }

    #[test]
    fn test_wall_content_rows_use_original_micro_tile_len() {
        let base_rows = calc_viewport_geometry(640, 480).rows;

        assert_eq!(wall_content_rows(base_rows, 16), base_rows + 16);
        assert_eq!(wall_content_rows(base_rows, 10), base_rows + 10);
    }

    #[test]
    fn test_entity_render_order_tile_uses_target_for_south_and_east_priority() {
        let mut entity = Entity::new(
            crate::entity::EntityType::Player,
            Point::new(10, 10),
            (32, 32),
            Color::CYAN,
        );

        assert!(entity.start_walk::<fn(i32, i32) -> bool>(Direction::South, None));
        assert_eq!(World::entity_render_order_tile(&entity), Point::new(11, 11));

        entity.cancel_walk();
        assert!(entity.start_walk::<fn(i32, i32) -> bool>(Direction::East, None));
        assert_eq!(World::entity_render_order_tile(&entity), Point::new(11, 9));
    }

    #[test]
    fn test_entity_scan_anchor_uses_source_tile_when_east_walk_sorts_on_target() {
        let mut entity = Entity::new(
            crate::entity::EntityType::Player,
            Point::new(10, 10),
            (32, 32),
            Color::CYAN,
        );

        assert!(entity.start_walk::<fn(i32, i32) -> bool>(Direction::East, None));
        entity.walk_progress = 0.5;

        let scan_tile = World::entity_render_order_tile(&entity);
        let scan_screen = Point::new(100, 200);
        let anchor = World::entity_scan_anchor(&entity, scan_tile, scan_screen);
        let walk_offset = entity.walking_render_offset();
        let final_screen = Point::new(anchor.x + walk_offset.x, anchor.y + walk_offset.y);

        assert_eq!(scan_tile, Point::new(11, 9));
        assert_eq!(anchor, Point::new(36, 200));
        assert_eq!(walk_offset, Point::new(32, 0));
        assert_eq!(final_screen, Point::new(68, 200));
    }

    #[test]
    fn test_entity_render_order_tile_keeps_source_for_north_and_west() {
        let mut entity = Entity::new(
            crate::entity::EntityType::Player,
            Point::new(10, 10),
            (32, 32),
            Color::CYAN,
        );

        assert!(entity.start_walk::<fn(i32, i32) -> bool>(Direction::North, None));
        assert_eq!(World::entity_render_order_tile(&entity), Point::new(10, 10));

        entity.cancel_walk();
        assert!(entity.start_walk::<fn(i32, i32) -> bool>(Direction::West, None));
        assert_eq!(World::entity_render_order_tile(&entity), Point::new(10, 10));
    }

    #[test]
    fn test_wall_content_predraw_matches_cpp_right_edge_gate() {
        use crate::tiles::types::TileProperties;

        let mut dungeon_map = DungeonMap::new();
        let mut sol_data = SolData {
            properties: vec![TileProperties::NONE; 3],
        };
        sol_data.properties[1] = TileProperties::SOLID;

        let wall_piece_id = 1;
        let floor_piece_id = 2;
        let dpiece_x = 20;
        let dpiece_y = 20;

        dungeon_map.d_piece[dpiece_x][dpiece_y] = wall_piece_id;
        dungeon_map.d_piece[dpiece_x + 1][dpiece_y] = wall_piece_id;
        dungeon_map.d_piece[dpiece_x][dpiece_y - 1] = floor_piece_id;
        dungeon_map.d_piece[dpiece_x + 1][dpiece_y - 1] = floor_piece_id;

        assert!(
            World::should_predraw_east_wall_content(
                Some(&sol_data),
                &dungeon_map,
                dpiece_x as i32,
                dpiece_y as i32,
                576,
                640
            ),
            "x-axis walls with walkable space behind them should pre-draw the east tile"
        );
        assert!(
            !World::should_predraw_east_wall_content(
                Some(&sol_data),
                &dungeon_map,
                dpiece_x as i32,
                dpiece_y as i32,
                577,
                640
            ),
            "the original gate stops once the tile would extend past the right edge"
        );
        assert!(
            !World::should_predraw_east_wall_content(
                Some(&sol_data),
                &dungeon_map,
                dpiece_x as i32,
                dpiece_y as i32,
                641,
                640
            ),
            "the pre-draw should stop once the source tile starts beyond the viewport"
        );

        sol_data.properties[floor_piece_id as usize] = TileProperties::SOLID;
        assert!(
            !World::should_predraw_east_wall_content(
                Some(&sol_data),
                &dungeon_map,
                dpiece_x as i32,
                dpiece_y as i32,
                576,
                640
            ),
            "the pre-draw is only for walls with walkable space behind them"
        );
    }

    #[test]
    fn test_tile_texture_cache_key_separates_floor_from_draw_cell() {
        let normal = World::tile_texture_cache_key(
            7,
            1,
            TileTextureKind::Normal,
            DrawCellMask::Solid,
            false,
        );
        let floor =
            World::tile_texture_cache_key(7, 1, TileTextureKind::Floor, DrawCellMask::Solid, false);
        let foliage = World::tile_texture_cache_key(
            7,
            1,
            TileTextureKind::Foliage,
            DrawCellMask::Solid,
            false,
        );

        assert_ne!(normal, floor);
        assert_ne!(normal, foliage);
        assert_ne!(floor, foliage);
    }

    #[test]
    fn test_draw_cell_mask_first_tiles_match_cpp_rules() {
        use crate::tiles::types::{TileProperties, TileType};

        assert_eq!(
            DrawCellMask::first_left(TileType::Square, false, TileProperties::NONE),
            DrawCellMask::Solid
        );
        assert_eq!(
            DrawCellMask::first_left(
                TileType::TransparentSquare,
                true,
                TileProperties::TRANSPARENT | TileProperties::TRANSPARENT_LEFT,
            ),
            DrawCellMask::Left
        );
        assert_eq!(
            DrawCellMask::first_right(
                TileType::TransparentSquare,
                true,
                TileProperties::TRANSPARENT | TileProperties::TRANSPARENT_RIGHT,
            ),
            DrawCellMask::Right
        );
        assert_eq!(
            DrawCellMask::first_left(TileType::RightTrapezoid, true, TileProperties::TRANSPARENT),
            DrawCellMask::Transparent
        );
        assert_eq!(
            DrawCellMask::first_right(TileType::LeftTriangle, true, TileProperties::TRANSPARENT),
            DrawCellMask::Transparent
        );
    }

    #[test]
    fn test_draw_cell_mask_upper_blocks_match_cpp_rules() {
        assert_eq!(DrawCellMask::upper_block(false), DrawCellMask::Solid);
        assert_eq!(DrawCellMask::upper_block(true), DrawCellMask::Transparent);
        assert_eq!(
            DrawCellMask::Transparent.enabled(false),
            DrawCellMask::Solid
        );
    }

    #[test]
    fn test_draw_cell_mask_transparent_alpha_affects_visible_pixels_only() {
        let mut pixels = vec![10u8, 20, 30, 255, 40, 50, 60, 0];

        World::apply_draw_cell_mask_to_rgba(&mut pixels, 2, 1, DrawCellMask::Transparent);

        assert_eq!(pixels[3], DrawCellMask::TRANSPARENT_ALPHA);
        assert_eq!(pixels[7], 0);
    }

    #[test]
    fn test_draw_cell_mask_left_and_right_are_upper_half_diagonals() {
        let mut left_pixels = [1u8, 1, 1, 255].repeat(32 * 32);
        let mut right_pixels = left_pixels.clone();

        World::apply_draw_cell_mask_to_rgba(&mut left_pixels, 32, 32, DrawCellMask::Left);
        World::apply_draw_cell_mask_to_rgba(&mut right_pixels, 32, 32, DrawCellMask::Right);

        let alpha = |pixels: &[u8], x: usize, y: usize| pixels[(y * 32 + x) * 4 + 3];

        assert_eq!(alpha(&left_pixels, 0, 15), 255);
        assert_eq!(alpha(&right_pixels, 31, 15), 255);

        assert_eq!(alpha(&left_pixels, 0, 16), DrawCellMask::TRANSPARENT_ALPHA);
        assert_eq!(alpha(&left_pixels, 31, 16), 255);
        assert_eq!(
            alpha(&right_pixels, 31, 16),
            DrawCellMask::TRANSPARENT_ALPHA
        );
        assert_eq!(alpha(&right_pixels, 0, 16), 255);

        assert_eq!(alpha(&left_pixels, 0, 31), 255);
        assert_eq!(alpha(&right_pixels, 31, 31), 255);
    }

    #[test]
    fn test_floor_triangle_extraction_uses_original_halves() {
        use crate::tiles::types::TileType;

        let mut source = vec![0u8; 32 * 32];
        for row in 0..32usize {
            for col in 0..32usize {
                source[row * 32 + col] = col as u8;
            }
        }

        let left = World::extract_floor_triangle_pixels(&source, 32, 32, TileType::LeftTriangle);
        let right = World::extract_floor_triangle_pixels(&source, 32, 32, TileType::RightTriangle);

        assert_eq!(left[0], 30);
        assert_eq!(left[1], 31);
        assert_eq!(right[30], 0);
        assert_eq!(right[31], 1);

        let row15 = 15 * 32;
        assert_eq!(left[row15], 0);
        assert_eq!(left[row15 + 31], 31);
        assert_eq!(right[row15], 0);
        assert_eq!(right[row15 + 31], 31);
    }
}

#[derive(Default)]
struct RenderScratch {
    luminance: Vec<u8>,
    edges: Vec<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawCellMask {
    Solid,
    Transparent,
    Left,
    Right,
}

impl DrawCellMask {
    const TRANSPARENT_ALPHA: u8 = 128;

    fn first_left(
        tile: crate::tiles::types::TileType,
        transparency: bool,
        props: crate::tiles::types::TileProperties,
    ) -> Self {
        use crate::tiles::types::{TileProperties, TileType};

        if !transparency {
            return Self::Solid;
        }

        match tile {
            TileType::LeftTrapezoid | TileType::TransparentSquare => {
                if props.contains(TileProperties::TRANSPARENT_LEFT) {
                    Self::Left
                } else {
                    Self::Solid
                }
            }
            TileType::LeftTriangle => Self::Solid,
            _ => Self::Transparent,
        }
    }

    fn first_right(
        tile: crate::tiles::types::TileType,
        transparency: bool,
        props: crate::tiles::types::TileProperties,
    ) -> Self {
        use crate::tiles::types::{TileProperties, TileType};

        if !transparency {
            return Self::Solid;
        }

        match tile {
            TileType::RightTrapezoid | TileType::TransparentSquare => {
                if props.contains(TileProperties::TRANSPARENT_RIGHT) {
                    Self::Right
                } else {
                    Self::Solid
                }
            }
            TileType::RightTriangle => Self::Solid,
            _ => Self::Transparent,
        }
    }

    fn upper_block(transparency: bool) -> Self {
        if transparency {
            Self::Transparent
        } else {
            Self::Solid
        }
    }

    fn enabled(self, enabled: bool) -> Self {
        if enabled {
            self
        } else {
            Self::Solid
        }
    }

    fn cache_index(self) -> usize {
        match self {
            Self::Solid => 0,
            Self::Transparent => 1,
            Self::Left => 2,
            Self::Right => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TileTextureKind {
    Normal,
    Floor,
    Foliage,
}

impl TileTextureKind {
    fn cache_index(self) -> usize {
        match self {
            Self::Normal => 0,
            Self::Floor => 1,
            Self::Foliage => 2,
        }
    }

    fn texture_id_fragment(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Floor => "floor",
            Self::Foliage => "foliage",
        }
    }
}

/// World - Game world representation
pub struct World {
    /// Grid dimensions (in tiles)
    pub width: usize,
    pub height: usize,
    /// Tile size in pixels
    pub tile_size: u32,
    /// Collision map (holds tile data)
    pub collision_map: CollisionMap,
    /// Entities in the world
    entities: Vec<Entity>,

    // Step 6.1: Tiles system (optional, for dungeon rendering)
    pub min_data: Option<MinData>,
    pub til_data: Option<TilData>,
    pub sol_data: Option<SolData>,
    pub dungeon_tileset: Option<crate::tiles::DungeonTileset>,

    // Step 6.2: Texture manager for tile rendering (wrapped in RefCell for interior mutability)
    pub texture_manager: Option<RefCell<TileTextureManager>>,

    // Dungeon map data (dPiece equivalent)
    pub dungeon_map: Option<DungeonMap>,

    // Step 6.4.1: Lighting system
    pub lighting: LightingSystem,

    // Render debug flags (for controlling rendering phases)
    pub render_debug: RenderDebugFlags,
    require_entity_textures: bool,
    render_scratch: RefCell<RenderScratch>,
    skipped_foliage_cache: RefCell<HashSet<usize>>,
}

// Isometric projection constants
pub const TILE_WIDTH: i32 = 64;
pub const TILE_HEIGHT: i32 = 32;

/// Convert world coordinates to screen coordinates (isometric projection)
///
/// # Reference
/// Original: Source/engine/displacement.hpp
pub fn world_to_screen(world_x: i32, world_y: i32) -> (i32, i32) {
    let screen_x = (world_x - world_y) * (TILE_WIDTH / 2);
    let screen_y = (world_x + world_y) * (TILE_HEIGHT / 2);
    (screen_x, screen_y)
}

/// Convert screen coordinates to world coordinates
///
/// # Reference
/// Original: Source/engine/displacement.hpp
pub fn screen_to_world(screen_x: i32, screen_y: i32) -> (i32, i32) {
    let world_x = (screen_x / (TILE_WIDTH / 2) + screen_y / (TILE_HEIGHT / 2)) / 2;
    let world_y = (screen_y / (TILE_HEIGHT / 2) - screen_x / (TILE_WIDTH / 2)) / 2;
    (world_x, world_y)
}

impl World {
    fn next_render_focus_trace_frame(enabled: bool) -> Option<u64> {
        if !enabled {
            return None;
        }

        let frame = RENDER_FOCUS_TRACE_FRAME.fetch_add(1, Ordering::Relaxed) + 1;
        if frame % RENDER_FOCUS_TRACE_SAMPLE_INTERVAL == 0 {
            Some(frame)
        } else {
            None
        }
    }

    fn record_render_perf(frame_perf: RenderPerfFrame, debug: &RenderDebugFlags) {
        let now = Instant::now();
        let Ok(mut accumulator) = RENDER_PERF_ACCUMULATOR.lock() else {
            return;
        };

        let last_report = *accumulator.last_report.get_or_insert(now);
        let mut frame_perf = frame_perf;
        frame_perf.micro_cache_hits = MICRO_TEXTURE_CACHE_HITS.swap(0, Ordering::Relaxed);
        frame_perf.micro_cache_misses = MICRO_TEXTURE_CACHE_MISSES.swap(0, Ordering::Relaxed);
        frame_perf.foliage_cache_hits = FOLIAGE_TEXTURE_CACHE_HITS.swap(0, Ordering::Relaxed);
        frame_perf.foliage_cache_misses = FOLIAGE_TEXTURE_CACHE_MISSES.swap(0, Ordering::Relaxed);
        accumulator.add_frame(frame_perf);

        let elapsed = now.duration_since(last_report);
        if elapsed < RENDER_PERF_REPORT_INTERVAL {
            return;
        }

        let frames = accumulator.frames.max(1);
        let frames_f64 = frames as f64;
        println!(
            "[render-perf] frames={} seconds={:.2} avg_world_ms={:.2} avg_floor_ms={:.2} avg_wall_ms={:.2} floor_tiles/frame={:.1} wall_scan/frame={:.1} wall_cells/frame={:.1} predraw/frame={:.1} micro_cache_hit/frame={:.1} micro_cache_miss/frame={:.1} foliage_cache_hit/frame={:.1} foliage_cache_miss/frame={:.1} texture_cache_size={} floor={} walls={} cam={} mask={}",
            frames,
            elapsed.as_secs_f64(),
            accumulator.total_time.as_secs_f64() * 1000.0 / frames_f64,
            accumulator.floor_time.as_secs_f64() * 1000.0 / frames_f64,
            accumulator.wall_time.as_secs_f64() * 1000.0 / frames_f64,
            accumulator.floor_tiles as f64 / frames_f64,
            accumulator.wall_scan_tiles as f64 / frames_f64,
            accumulator.wall_cells as f64 / frames_f64,
            accumulator.predraw_cells as f64 / frames_f64,
            accumulator.micro_cache_hits as f64 / frames_f64,
            accumulator.micro_cache_misses as f64 / frames_f64,
            accumulator.foliage_cache_hits as f64 / frames_f64,
            accumulator.foliage_cache_misses as f64 / frames_f64,
            accumulator.texture_cache_size,
            debug.render_floor,
            debug.render_walls,
            debug.walking_camera_offset,
            debug.mask_aware_draw_cell
        );

        accumulator.reset(now);
    }

    /// Create a new world
    ///
    /// # Arguments
    /// * `width` - World width in tiles
    /// * `height` - World height in tiles
    /// * `tile_size` - Tile size in pixels
    /// * `palette` - Game palette for lighting system
    pub fn new(width: usize, height: usize, tile_size: u32, palette: &Palette) -> Self {
        // Initialize with a simple floor pattern
        let mut tiles = vec![vec![TileType::Empty; width]; height];
        // Ensure center is clear for player
        let center_x = width / 2;
        let center_y = height / 2;
        for y in center_y - 2..=center_y + 2 {
            for x in center_x - 2..=center_x + 2 {
                tiles[y][x] = TileType::Floor;
            }
        }

        // Create collision map
        let collision_map = CollisionMap::new(width, height, tiles);

        // Create lighting system
        let lighting = LightingSystem::new(palette);

        Self {
            width,
            height,
            tile_size,
            collision_map,
            entities: Vec::new(),
            min_data: None,
            til_data: None,
            sol_data: None,
            dungeon_tileset: None,
            texture_manager: None,
            dungeon_map: None,
            lighting,
            render_debug: RenderDebugFlags::default(), // Default: floor-only mode (like C++)
            require_entity_textures: false,
            render_scratch: RefCell::new(RenderScratch::default()),
            skipped_foliage_cache: RefCell::new(HashSet::new()),
        }
    }

    /// Require sprite entities to draw real textures instead of debug rectangles.
    pub fn require_entity_textures(&mut self) {
        self.require_entity_textures = true;
    }

    /// Add an entity to the world
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Get a mutable reference to an entity by index
    pub fn get_entity_mut(&mut self, index: usize) -> Option<&mut Entity> {
        self.entities.get_mut(index)
    }

    /// Get all entities
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Update all entities (tile-based movement system)
    pub fn update(&mut self, dt: f32) {
        // Update entities with tile-based collision
        // Extract dungeon_map reference before the mutable borrow to avoid borrow checker issues
        // Simplified: only check if position is within bounds
        use crate::engine::isometric::BORDER_SIZE;

        // Get dungeon_map reference before borrowing entities mutably
        let dungeon_map_opt = self.dungeon_map.as_ref();

        for entity in &mut self.entities {
            if let Some(dungeon_map) = dungeon_map_opt {
                // Create a closure that checks bounds using dungeon_map
                // Capture dungeon_map by reference (it's already a reference, so this is fine)
                let map = dungeon_map;
                entity.update(
                    dt,
                    Some(move |x, y| {
                        let dpiece_x = x + BORDER_SIZE;
                        let dpiece_y = y + BORDER_SIZE;
                        map.in_bounds(dpiece_x, dpiece_y)
                    }),
                );
            } else {
                // Type annotation needed for None
                entity.update::<fn(i32, i32) -> bool>(dt, None);
            }
        }

        // Step 6.4.1: Update lighting system
        // Convert collision map to block map for lighting
        let block_map = self.collision_map.to_block_map();

        // Update player light source position
        // Find player entity (assuming first entity is player)
        if let Some(player) = self.entities.get(0) {
            // Convert world coordinates to dPiece coordinates for lighting system
            // Lighting system uses dPiece coordinates (0-111), not world coordinates
            use crate::engine::isometric::BORDER_SIZE;
            let micro_x = (player.tile_position.x + BORDER_SIZE) as usize;
            let micro_y = (player.tile_position.y + BORDER_SIZE) as usize;

            // Check if player light source exists, if not create it
            let player_light_id = 1; // Use ID 1 for player light
            let has_player_light = self
                .lighting
                .light_sources
                .iter()
                .any(|s| s.id == player_light_id);

            if !has_player_light {
                // Create player light source
                use crate::lighting::{LightSource, LightType};
                let player_light = LightSource {
                    id: player_light_id,
                    position: (micro_x, micro_y),
                    radius: 8, // Player light radius
                    light_type: LightType::Player,
                    active: true,
                };
                self.lighting.add_light(player_light);
            } else {
                // Update player light position
                self.lighting
                    .update_light_position(player_light_id, (micro_x, micro_y));
            }
        }

        // Update lighting system (calculate light propagation)
        self.lighting.update(&block_map);
    }

    /// Load Town sector directly to dPiece (like C++ FillSector)
    ///
    /// Town uses a different loading method - it writes directly to dPiece
    /// instead of going through dungeon array.
    pub fn load_town_sector(
        &mut self,
        mpq_manager: &mut crate::resources::MpqManager,
        dun_path: &str,
        min_data: MinData,
        til_data: TilData,
        sol_data: SolData,
        texture_manager: Option<TileTextureManager>,
        offset_x: usize,
        offset_y: usize,
        default_piece: u16,
    ) -> Result<(), String> {
        use crate::world::dungeon_map::{load_dun_to_dpiece, load_sector_to_dpiece};

        // If dungeon_map already exists, append to it; otherwise create new
        if let Some(ref mut dungeon_map) = self.dungeon_map {
            // Append sector to existing map
            load_sector_to_dpiece(
                dungeon_map,
                mpq_manager,
                dun_path,
                &til_data,
                offset_x,
                offset_y,
                default_piece,
            )?;
        } else {
            // Create new map and load first sector
            let dungeon_map = load_dun_to_dpiece(
                mpq_manager,
                dun_path,
                &til_data,
                offset_x,
                offset_y,
                default_piece,
            )?;
            self.dungeon_map = Some(dungeon_map);
        }

        println!("✓ Loaded Town sector: {}", dun_path);

        self.min_data = Some(min_data);
        self.til_data = Some(til_data);
        self.sol_data = Some(sol_data);
        self.dungeon_tileset = None;
        self.texture_manager = texture_manager.map(RefCell::new);

        Ok(())
    }

    /// Check if a world position is walkable based on tile data
    ///
    /// Returns true if the position is walkable (not solid), false otherwise.
    /// If tile data is not loaded, defaults to using the collision map.
    ///
    /// # Arguments
    /// * `world_x` - World tile X coordinate (world coordinates, not dPiece)
    /// * `world_y` - World tile Y coordinate (world coordinates, not dPiece)
    ///
    /// # Reference
    /// C++ TileHasAny/IsFloor: Source/engine/render/scrollrt.cpp Line 114-117
    /// C++ uses dPiece coordinates for TileHasAny, but player position is in world coordinates
    pub fn is_tile_walkable(&self, world_x: i32, world_y: i32) -> bool {
        if world_x < 0 || world_y < 0 {
            return false;
        }

        if world_x > (DMAXX * 2) as i32 || world_y > (DMAXY * 2) as i32 {
            return false;
        }

        return true;
    }

    /// Render the world
    pub fn render(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
        self.render_with_texture_manager(engine, camera)?;
        // In isometric dungeon mode with the wall layer enabled, entities are drawn
        // inside the DrawTileContent-style tile scan so foreground walls can cover them.
        if self.dungeon_map.is_none() || !self.render_debug.render_walls {
            self.render_entities(engine, camera)?;
        }

        Ok(())
    }

    /// Render entities (common for both rendering modes)
    fn render_entities(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
        // Check if we're in isometric rendering mode (dungeon_map exists)
        let is_isometric = self.dungeon_map.is_some();

        // Get player position for isometric rendering (if available)
        let player_tile_pos = if is_isometric {
            self.entities
                .first()
                .map(|p| (p.tile_position.x, p.tile_position.y))
        } else {
            None
        };

        for (entity_index, entity) in self.entities.iter().enumerate() {
            self.render_entity(
                engine,
                camera,
                entity_index,
                entity,
                is_isometric,
                player_tile_pos,
                None,
            )?;
        }

        Ok(())
    }

    fn render_entity(
        &self,
        engine: &mut Engine,
        camera: &Camera,
        entity_index: usize,
        entity: &Entity,
        is_isometric: bool,
        player_tile_pos: Option<(i32, i32)>,
        isometric_scan_anchor: Option<Point>,
    ) -> Result<()> {
        let render_offset = if is_isometric {
            if isometric_scan_anchor.is_some() {
                entity.walking_render_offset()
            } else if entity_index == 0 {
                Point::zero()
            } else {
                entity.walking_render_offset()
            }
        } else {
            entity.walking_pixel_offset(self.tile_size as i32)
        };

        let screen_pos = if let Some(anchor) = isometric_scan_anchor {
            Point::new(anchor.x + render_offset.x, anchor.y + render_offset.y)
        } else if is_isometric {
            // Isometric rendering mode: use isometric projection
            if let Some((player_x, player_y)) = player_tile_pos {
                // Calculate relative position from player
                let delta_x = entity.tile_position.x - player_x;
                let delta_y = entity.tile_position.y - player_y;

                // Convert to screen coordinates using isometric projection
                // Reference: Source/engine/displacement.hpp::worldToScreen()
                let (screen_delta_x, screen_delta_y) = world_to_screen(delta_x, delta_y);

                // Player is at screen center, other entities are offset from center
                Point::new(
                    camera.viewport_width as i32 / 2 + screen_delta_x + render_offset.x,
                    camera.viewport_height as i32 / 2 + screen_delta_y + render_offset.y,
                )
            } else {
                // No player, use entity position directly
                let (screen_x, screen_y) =
                    world_to_screen(entity.tile_position.x, entity.tile_position.y);
                Point::new(screen_x + render_offset.x, screen_y + render_offset.y)
            }
        } else {
            // 2D rendering mode: use camera world_to_screen
            let entity_world_pixel = Point::new(
                entity.tile_position.x * self.tile_size as i32,
                entity.tile_position.y * self.tile_size as i32,
            );
            let mut screen_pos = camera.world_to_screen(entity_world_pixel);
            screen_pos.x += render_offset.x;
            screen_pos.y += render_offset.y;
            screen_pos
        };

        let dst_rect = Rect::from_center(screen_pos, entity.size.0, entity.size.1);

        if entity.use_sprite {
            // Render sprite if available
            if let Some(ref base_sprite_id) = entity.sprite_id {
                // Get animation state and frame index
                let (anim_state, frame_index) = if let Some(ref anim) = entity.animation {
                    let state = anim.current_state();
                    let frame = anim.current_frame_index().unwrap_or(0);
                    (Some(state), frame)
                } else {
                    (None, 0)
                };
                let direction = entity.render_direction();

                // For CL2/CLX sprites, construct texture ID with animation state
                // Format: "{base}_{state}_{direction}_{frame}"
                // e.g. "warrior_idle_south_0", "warrior_walk_north_east_3"
                let mut texture_found = false;

                if let Some(state) = anim_state {
                    let state_name = match state {
                        AnimationState::Idle => "idle",
                        AnimationState::Walk => "walk",
                        AnimationState::Attack => "attack",
                        AnimationState::Hit => "hit",
                        AnimationState::Death => "death",
                        AnimationState::Cast => "cast",
                    };

                    let directional_texture_id = format!(
                        "{}_{}_{}_{}",
                        base_sprite_id,
                        state_name,
                        direction.animation_suffix(),
                        frame_index
                    );

                    if engine.draw_texture_by_id(&directional_texture_id, None, dst_rect)? {
                        texture_found = true;
                    } else {
                        let legacy_texture_id =
                            format!("{}_{}_{}", base_sprite_id, state_name, frame_index);
                        if engine.draw_texture_by_id(&legacy_texture_id, None, dst_rect)? {
                            texture_found = true;
                        }
                    }
                }

                // Fallback: Try traditional sprite sheet with src_rect
                if !texture_found {
                    let src_rect = entity
                        .animation
                        .as_ref()
                        .and_then(|anim| anim.current_frame_rect());

                    if !engine.draw_texture_by_id(base_sprite_id, src_rect, dst_rect)? {
                        if self.require_entity_textures {
                            anyhow::bail!(
                                "Missing entity texture '{}' for {:?} at tile ({}, {})",
                                base_sprite_id,
                                entity.entity_type,
                                entity.tile_position.x,
                                entity.tile_position.y
                            );
                        } else {
                            // Final fallback for debug-only entities without loaded art.
                            engine.draw_rect(dst_rect, entity.color)?;
                        }
                    }
                }
            } else {
                if self.require_entity_textures {
                    anyhow::bail!(
                        "Sprite entity {:?} has no sprite_id at tile ({}, {})",
                        entity.entity_type,
                        entity.tile_position.x,
                        entity.tile_position.y
                    );
                } else {
                    engine.draw_rect(dst_rect, entity.color)?;
                }
            }
        } else {
            if self.require_entity_textures {
                anyhow::bail!(
                    "Sprite rendering disabled for {:?} at tile ({}, {})",
                    entity.entity_type,
                    entity.tile_position.x,
                    entity.tile_position.y
                );
            } else {
                // Render as colored rectangle
                engine.draw_rect(dst_rect, entity.color)?;
            }
        }

        Ok(())
    }

    fn entity_render_order_tile(entity: &Entity) -> Point {
        if entity.walking {
            let direction = entity.render_direction();
            if matches!(
                direction,
                Direction::South | Direction::SouthWest | Direction::SouthEast | Direction::East
            ) {
                return entity.walk_target_tile.unwrap_or(entity.tile_position);
            }
        }

        entity.tile_position
    }

    fn entity_scan_anchor(entity: &Entity, scan_tile: Point, scan_screen: Point) -> Point {
        let delta_x = entity.tile_position.x - scan_tile.x;
        let delta_y = entity.tile_position.y - scan_tile.y;
        let (screen_delta_x, screen_delta_y) = world_to_screen(delta_x, delta_y);

        Point::new(
            scan_screen.x + screen_delta_x,
            scan_screen.y + screen_delta_y,
        )
    }

    fn render_entities_at_dpiece(
        &self,
        engine: &mut Engine,
        camera: &Camera,
        dpiece_x: i32,
        dpiece_y: i32,
        screen_x: i32,
        screen_y: i32,
        border_size: i32,
    ) -> Result<()> {
        if !self.render_debug.render_entities {
            return Ok(());
        }

        let tile_position = Point::new(dpiece_x - border_size, dpiece_y - border_size);
        let player_tile_pos = self
            .entities
            .first()
            .map(|player| (player.tile_position.x, player.tile_position.y));

        for (entity_index, entity) in self.entities.iter().enumerate() {
            if Self::entity_render_order_tile(entity) == tile_position {
                let scan_anchor =
                    Self::entity_scan_anchor(entity, tile_position, Point::new(screen_x, screen_y));
                self.render_entity(
                    engine,
                    camera,
                    entity_index,
                    entity,
                    true,
                    player_tile_pos,
                    Some(scan_anchor),
                )?;
            }
        }

        Ok(())
    }

    fn tile_texture_cache_key(
        level_piece_id: usize,
        block_index: usize,
        kind: TileTextureKind,
        mask: DrawCellMask,
        stylized: bool,
    ) -> usize {
        ((((level_piece_id * 32) + block_index) * 3 + kind.cache_index()) * 4 + mask.cache_index())
            * 2
            + usize::from(stylized)
    }

    fn is_floor(sol: Option<&SolData>, piece_id: usize) -> bool {
        sol.and_then(|sol| sol.get(piece_id))
            .map(|props| {
                !props.contains(crate::tiles::types::TileProperties::SOLID)
                    && !props.contains(crate::tiles::types::TileProperties::BLOCK_MISSILE)
            })
            .unwrap_or(true)
    }

    fn is_wall(
        sol: Option<&SolData>,
        dungeon_map: &DungeonMap,
        dpiece_x: i32,
        dpiece_y: i32,
    ) -> bool {
        if !dungeon_map.in_bounds(dpiece_x, dpiece_y) {
            return false;
        }

        let level_piece_id = dungeon_map.get_piece(dpiece_x, dpiece_y) as usize;
        !Self::is_floor(sol, level_piece_id)
    }

    fn is_tile_not_solid(
        sol: Option<&SolData>,
        dungeon_map: &DungeonMap,
        dpiece_x: i32,
        dpiece_y: i32,
    ) -> bool {
        if !dungeon_map.in_bounds(dpiece_x, dpiece_y) {
            return false;
        }

        let level_piece_id = dungeon_map.get_piece(dpiece_x, dpiece_y) as usize;
        sol.and_then(|sol| sol.get(level_piece_id))
            .map(|props| !props.contains(crate::tiles::types::TileProperties::SOLID))
            .unwrap_or(true)
    }

    fn should_predraw_east_wall_content(
        sol: Option<&SolData>,
        dungeon_map: &DungeonMap,
        dpiece_x: i32,
        dpiece_y: i32,
        screen_x: i32,
        screen_width: i32,
    ) -> bool {
        if dpiece_x + 1 >= MAXDUNX as i32 || dpiece_y <= 0 || screen_x + TILE_WIDTH > screen_width {
            return false;
        }

        let wall_axis_aligned = Self::is_wall(sol, dungeon_map, dpiece_x, dpiece_y)
            && (Self::is_wall(sol, dungeon_map, dpiece_x + 1, dpiece_y)
                || (dpiece_x > 0 && Self::is_wall(sol, dungeon_map, dpiece_x - 1, dpiece_y)));
        let has_walkable_area_behind =
            Self::is_tile_not_solid(sol, dungeon_map, dpiece_x + 1, dpiece_y - 1)
                && Self::is_tile_not_solid(sol, dungeon_map, dpiece_x, dpiece_y - 1);

        wall_axis_aligned && has_walkable_area_behind
    }

    fn draw_tile_content_at(
        &self,
        engine: &mut Engine,
        camera: &Camera,
        texture_mgr: &RefCell<TileTextureManager>,
        dungeon_map: &DungeonMap,
        sol_data: Option<&SolData>,
        dpiece_x: i32,
        dpiece_y: i32,
        screen_x: i32,
        screen_y: i32,
        border_size: i32,
        trace_frame: Option<u64>,
    ) -> Result<()> {
        let level_piece_id = dungeon_map.get_piece(dpiece_x, dpiece_y) as usize;
        let is_floor = Self::is_floor(sol_data, level_piece_id);
        let tile_props = sol_data
            .and_then(|sol| sol.get(level_piece_id))
            .unwrap_or(crate::tiles::types::TileProperties::NONE);

        if let Some(frame) = trace_frame {
            println!(
                "  [render-trace #{:06}] pass=cell-scan dpiece=({}, {}) piece={} is_floor={} props={:?} dst=({}, {})",
                frame, dpiece_x, dpiece_y, level_piece_id, is_floor, tile_props, screen_x, screen_y
            );
        }

        self.draw_cell_at(
            engine,
            texture_mgr,
            level_piece_id,
            screen_x,
            screen_y,
            is_floor,
            tile_props,
            dpiece_x,
            dpiece_y,
            trace_frame,
        )?;
        self.render_entities_at_dpiece(
            engine,
            camera,
            dpiece_x,
            dpiece_y,
            screen_x,
            screen_y,
            border_size,
        )
    }

    /// Render using TileTextureManager with proper tile decoding (Step 6.2)
    ///
    /// # Reference
    fn render_with_texture_manager(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
        let render_start = Instant::now();
        let mut perf = RenderPerfFrame::default();
        let texture_mgr_cell = self.texture_manager.as_ref().unwrap();
        let dungeon_map = match self.dungeon_map.as_ref() {
            Some(dm) => dm,
            None => return Ok(()), // No map data, skip rendering
        };
        let sol_data = self.sol_data.as_ref();

        let screen_width = camera.viewport_width as i32;
        let viewport_height = camera.viewport_height as i32;

        // Tile dimensions
        const TILE_WIDTH: i32 = 64;
        const TILE_HEIGHT: i32 = 32;

        use crate::engine::isometric::BORDER_SIZE;
        use crate::math::Point;

        // ViewPosition is in world coordinates (like C++ ViewPosition)
        // Reference: Source/engine/render/scrollrt.cpp::DrawView() Line 1364
        // DrawView(out, ViewPosition) where ViewPosition is world tile coordinates
        let (view_x, view_y) = if let Some(player) = self.entities.first() {
            // player.tile_position is in world coordinates (like MyPlayer->position.tile)
            // ViewPosition should be world coordinates, NOT dPiece coordinates
            (player.tile_position.x, player.tile_position.y)
        } else {
            // Default view position - center of loaded data (in world coordinates)
            (25 - BORDER_SIZE, 25 - BORDER_SIZE) // Convert from dPiece to world if needed
        };

        // Debug: focus render to the 2x2 region around the view center (in dPiece coordinates).
        // This is intentionally simple (skip outside tiles) to help isolate rendering issues.
        // Toggle focus via Shift+F7 (RenderDebugFlags.focus_render_2x2).
        let focus_render_2x2 = self.render_debug.focus_render_2x2;
        let log_render_focus = self.render_debug.log_render_focus && focus_render_2x2;
        let render_focus = if focus_render_2x2 {
            RenderFocus::centered_2x2(view_x, view_y, BORDER_SIZE)
        } else {
            RenderFocus::disabled()
        };
        let trace_frame = Self::next_render_focus_trace_frame(log_render_focus);
        if let Some(frame) = trace_frame {
            println!(
                "[render-trace #{:06}] view_world=({}, {}) focus_dpiece_x=[{}..{}] focus_dpiece_y=[{}..{}]",
                frame,
                view_x,
                view_y,
                render_focus.min_x,
                render_focus.max_x,
                render_focus.min_y,
                render_focus.max_y
            );
        }

        let walking_player = self
            .entities
            .first()
            .filter(|player| self.render_debug.walking_camera_offset && player.walking);
        let walking_direction = walking_player.map(|player| player.render_direction());
        let walking_camera_offset = walking_player
            .map(|player| player.walking_render_offset())
            .unwrap_or_else(Point::zero);
        let scan = calc_render_scan_geometry(
            Point::new(view_x, view_y),
            screen_width,
            viewport_height,
            walking_direction,
            walking_camera_offset,
        );
        if let Some(frame) = trace_frame {
            println!(
                "[render-trace #{:06}] scan start_tile=({}, {}) tile_offset=({}, {}) columns={} rows={} walking_dir={:?} walking_offset=({}, {}) cam={}",
                frame,
                scan.start_tile.x,
                scan.start_tile.y,
                scan.tile_offset.x,
                scan.tile_offset.y,
                scan.columns,
                scan.rows,
                walking_direction,
                walking_camera_offset.x,
                walking_camera_offset.y,
                if self.render_debug.walking_camera_offset { "ON" } else { "OFF" }
            );
        }

        let columns = scan.columns;
        let rows = scan.rows;
        let offset_x = scan.tile_offset.x;
        let offset_y = scan.tile_offset.y;
        let start_tile_x = scan.start_tile.x;
        let start_tile_y = scan.start_tile.y;

        // === Phase 1: Draw Floor (like C++ DrawFloor) ===
        // Reference: Source/engine/render/scrollrt.cpp::DrawGame() Line 1306-1308
        // Render all floor tiles in the view (only if render_floor is enabled)
        if self.render_debug.render_floor {
            let floor_start = Instant::now();
            let mut tile_x = start_tile_x;
            let mut tile_y = start_tile_y;
            let mut screen_x = offset_x;
            let mut screen_y = offset_y;
            let mut current_columns = columns;

            for row in 0..rows {
                let mut tx = tile_x;
                let mut ty = tile_y;
                let mut sx = screen_x;
                let col_count = current_columns;

                for _col in 0..col_count {
                    // Convert world coordinates to dPiece coordinates for array access
                    let dpiece_x = tx + BORDER_SIZE;
                    let dpiece_y = ty + BORDER_SIZE;

                    if !render_focus.contains(dpiece_x, dpiece_y) {
                        tx += 1;
                        ty -= 1;
                        sx += 64;
                        continue;
                    }
                    if dungeon_map.in_bounds(dpiece_x, dpiece_y) {
                        let level_piece_id = dungeon_map.get_piece(dpiece_x, dpiece_y) as usize;
                        // Check IsFloor
                        let is_floor = Self::is_floor(sol_data, level_piece_id);

                        // Render the tile (frame 311 filtering is done in render_micro_tile)
                        if is_floor {
                            if let Some(frame) = trace_frame {
                                println!(
                                    "  [render-trace #{:06}] pass=floor-scan dpiece=({}, {}) piece={} dst=({}, {})",
                                    frame, dpiece_x, dpiece_y, level_piece_id, sx, screen_y
                                );
                            }
                            let _ = self.draw_floor_at(
                                engine,
                                texture_mgr_cell,
                                level_piece_id,
                                sx,
                                screen_y,
                                dpiece_x,
                                dpiece_y,
                                trace_frame,
                            );
                            perf.floor_tiles += 1;
                        }
                    }

                    tx += 1;
                    ty -= 1;
                    sx += 64;
                }

                screen_y += TILE_HEIGHT / 2;

                if (row & 1) != 0 {
                    tile_x += 1;
                    current_columns -= 1;
                    screen_x += TILE_WIDTH / 2;
                } else {
                    tile_y += 1;
                    current_columns += 1;
                    screen_x -= TILE_WIDTH / 2;
                }
            }
            perf.floor_time = floor_start.elapsed();
        }

        // === Phase 2: Draw Walls/Cells (like C++ DrawTileContent/DrawCell) ===
        // Reference: Source/engine/render/scrollrt.cpp::DrawGame() Line 1311-1313
        // After rendering all floors, render ALL tiles' walls and upper layers
        // NOTE: Original C++ DrawTileContent renders ALL tiles, not just walls!
        // Only render if render_walls is enabled
        if self.render_debug.render_walls {
            let wall_start = Instant::now();
            // Match C++ DrawTileContent(): rows += MicroTileLen.
            // Town uses 16 micro blocks per piece, so 8 rows loses tall house roots at viewport edges.
            let blocks_per_piece = texture_mgr_cell.borrow().blocks_per_piece();
            let wall_rows = wall_content_rows(rows, blocks_per_piece);
            let wall_padding = wall_content_scan_padding();

            // Reset to starting position
            let mut wall_tile_x = start_tile_x + wall_padding.tile_shift.x;
            let mut wall_tile_y = start_tile_y + wall_padding.tile_shift.y;
            let mut wall_screen_x = offset_x + wall_padding.screen_x_shift;
            let mut wall_screen_y = offset_y;
            let mut wall_current_columns = columns + wall_padding.extra_columns;

            for row in 0..wall_rows {
                let mut tx = wall_tile_x;
                let mut ty = wall_tile_y;
                let mut sx = wall_screen_x;
                let mut skip = false;

                for _col in 0..wall_current_columns {
                    // Convert world coordinates to dPiece coordinates for array access
                    let dpiece_x = tx + BORDER_SIZE;
                    let dpiece_y = ty + BORDER_SIZE;

                    if !render_focus.contains(dpiece_x, dpiece_y) {
                        skip = false;
                        tx += 1;
                        ty -= 1;
                        sx += 64;
                        continue;
                    }

                    if dungeon_map.in_bounds(dpiece_x, dpiece_y) {
                        perf.wall_scan_tiles += 1;
                        let mut skip_next = false;

                        if self.render_debug.wall_predraw
                            && Self::should_predraw_east_wall_content(
                                sol_data,
                                dungeon_map,
                                dpiece_x,
                                dpiece_y,
                                sx,
                                screen_width,
                            )
                        {
                            let behind_x = dpiece_x + 1;
                            let behind_y = dpiece_y - 1;

                            if render_focus.contains(behind_x, behind_y) {
                                // C++ DrawTileContent renders the tile behind this wall first,
                                // then skips it when the row scan reaches it normally.
                                self.draw_tile_content_at(
                                    engine,
                                    camera,
                                    texture_mgr_cell,
                                    dungeon_map,
                                    sol_data,
                                    behind_x,
                                    behind_y,
                                    sx + TILE_WIDTH,
                                    wall_screen_y,
                                    BORDER_SIZE,
                                    trace_frame,
                                )?;
                                perf.predraw_cells += 1;
                                skip_next = true;
                                if let Some(frame) = trace_frame {
                                    println!(
                                        "  [render-trace #{:06}] pass=wall-predraw source_dpiece=({}, {}) predraw_dpiece=({}, {}) dst=({}, {}) skip_next=true",
                                        frame,
                                        dpiece_x,
                                        dpiece_y,
                                        behind_x,
                                        behind_y,
                                        sx + TILE_WIDTH,
                                        wall_screen_y
                                    );
                                }
                            }
                        }

                        if !skip {
                            self.draw_tile_content_at(
                                engine,
                                camera,
                                texture_mgr_cell,
                                dungeon_map,
                                sol_data,
                                dpiece_x,
                                dpiece_y,
                                sx,
                                wall_screen_y,
                                BORDER_SIZE,
                                trace_frame,
                            )?;
                            perf.wall_cells += 1;
                        }

                        skip = skip_next;
                    }

                    tx += 1;
                    ty -= 1;
                    sx += 64;
                }

                wall_screen_y += TILE_HEIGHT / 2;

                if (row & 1) != 0 {
                    wall_tile_x += 1;
                    wall_current_columns -= 1;
                    wall_screen_x += TILE_WIDTH / 2;
                } else {
                    wall_tile_y += 1;
                    wall_current_columns += 1;
                    wall_screen_x -= TILE_WIDTH / 2;
                }
            }
            perf.wall_time = wall_start.elapsed();
        }

        perf.total_time = render_start.elapsed();
        perf.texture_cache_size = engine.tile_texture_cache_stats().0 as u64;
        Self::record_render_perf(perf, &self.render_debug);
        Ok(())
    }

    /// # Reference
    /// Original: `Source/engine/render/scrollrt.cpp::DrawFloorTile()` Line 652-677
    /// C++ DrawFloorTile:
    /// - Only renders block 0 and block 1
    /// - Forces TileType::LeftTriangle for block 0, TileType::RightTriangle for block 1
    /// - Only checks hasValue(), ignores actual TileType stored in block
    /// - Uses fixed height: DunFrameTriangleHeight = 31
    ///
    /// Rust implementation decodes the source frame by its real MIN type, then
    /// clips it to the floor triangle footprint used by the original renderer.
    fn draw_floor_at(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        level_piece_id: usize,
        screen_x: i32,
        screen_y: i32,
        micro_x: i32,
        micro_y: i32,
        trace_frame: Option<u64>,
    ) -> Result<()> {
        use crate::tiles::types::TileType;

        // Render block 0 (LeftTriangle) at screen_x
        self.render_floor_micro_tile(
            engine,
            texture_mgr,
            level_piece_id,
            0,
            screen_x,
            screen_y,
            micro_x,
            micro_y,
            TileType::LeftTriangle,
            trace_frame,
        )?;

        // Render block 1 (RightTriangle) at screen_x + 32
        self.render_floor_micro_tile(
            engine,
            texture_mgr,
            level_piece_id,
            1,
            screen_x + 32,
            screen_y,
            micro_x + 1,
            micro_y,
            TileType::RightTriangle,
            trace_frame,
        )?;

        Ok(())
    }

    /// Render floor foliage (grass) for TransparentSquare blocks
    ///
    /// # Reference
    /// Original: `Source/engine/render/dun_render.hpp::RenderTileFoliage()` Line 162-167
    /// - Foliage data is at: GetDunFrame(frame) + ReencodedTriangleFrameSize
    /// - ReencodedTriangleFrameSize = 544 - 32 = 512 bytes
    /// - Foliage is 16 pixels high, rendered at position.y - 16
    /// - TileType: TransparentSquare, height: 16
    fn render_floor_foliage(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        level_piece_id: usize,
        block_index: usize,
        screen_x: i32,
        screen_y: i32,
        trace_frame: Option<u64>,
    ) -> Result<()> {
        // C++: RenderTileFoliage renders at position.y - 16
        let foliage_y = screen_y - 16;
        let width = 32u32;
        let height = 16u32;
        let cache_key = Self::tile_texture_cache_key(
            level_piece_id,
            block_index,
            TileTextureKind::Foliage,
            DrawCellMask::Solid,
            self.render_debug.toon_filter,
        );
        let rect = Rect::new(screen_x, foliage_y, width, height);

        if self.skipped_foliage_cache.borrow().contains(&cache_key) {
            return Ok(());
        }

        if engine.draw_cached_texture(cache_key, rect)? {
            FOLIAGE_TEXTURE_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            if let Some(frame) = trace_frame {
                println!(
                    "    [render-trace #{:06}] pass=foliage-cache-hit kind={:?} piece={} block={} dst=({}, {}) rect=({}, {}) size={}x{}",
                    frame,
                    TileTextureKind::Foliage,
                    level_piece_id,
                    block_index,
                    screen_x,
                    foliage_y,
                    rect.x,
                    rect.y,
                    width,
                    height
                );
            }
            return Ok(());
        }
        FOLIAGE_TEXTURE_CACHE_MISSES.fetch_add(1, Ordering::Relaxed);

        // Get foliage data (offset 512 bytes from main tile data)
        match texture_mgr
            .borrow_mut()
            .get_decoded_foliage(level_piece_id, block_index)
        {
            Ok(rgba_pixels) => {
                // Foliage is 16 pixels high, 32 pixels wide
                let expected_size = (width * height * 4) as usize;

                if rgba_pixels.len() != expected_size {
                    self.skipped_foliage_cache.borrow_mut().insert(cache_key);
                    return Ok(());
                }

                // Skip if all pixels are transparent
                let has_visible_pixels = rgba_pixels.chunks(4).any(|p| p[3] > 0);
                if !has_visible_pixels {
                    self.skipped_foliage_cache.borrow_mut().insert(cache_key);
                    return Ok(());
                }

                let texture_id = format!("foliage_{}_{}", level_piece_id, block_index);
                if let Some(frame) = trace_frame {
                    println!(
                        "    [render-trace #{:06}] pass=foliage kind={:?} piece={} block={} dst=({}, {}) rect=({}, {}) size={}x{}",
                        frame,
                        TileTextureKind::Foliage,
                        level_piece_id,
                        block_index,
                        screen_x,
                        foliage_y,
                        rect.x,
                        rect.y,
                        width,
                        height
                    );
                }
                engine
                    .draw_cached_rgba_texture(cache_key, rgba_pixels, width, height, rect)
                    .or_else(|_| {
                        engine.draw_rgba_texture(&texture_id, rgba_pixels, width, height, rect)
                    })?;
            }
            Err(_e) => {
                self.skipped_foliage_cache.borrow_mut().insert(cache_key);
                // Foliage decode failed, skip
            }
        }
        Ok(())
    }

    /// Draw cell (walls) at screen position
    ///
    /// # Reference
    /// Original: `Source/engine/render/scrollrt.cpp::DrawCell()` Line 521-643
    fn draw_cell_at(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        level_piece_id: usize,
        screen_x: i32,
        screen_y: i32,
        is_floor: bool, // From SOL data: !TileHasAny(Solid | BlockMissile)
        tile_props: crate::tiles::types::TileProperties,
        micro_x: i32,
        micro_y: i32,
        trace_frame: Option<u64>,
    ) -> Result<()> {
        const TILE_HEIGHT: i32 = 32;

        let blocks_per_piece = texture_mgr.borrow().blocks_per_piece();
        let transparency = tile_props.contains(crate::tiles::types::TileProperties::TRANSPARENT);
        let mask_aware = self.render_debug.mask_aware_draw_cell;

        // Block 0 (left half)
        {
            let mgr = texture_mgr.borrow();
            if let Some(piece) = mgr.get_piece(level_piece_id) {
                if let Some(block) = piece.mt.get(0) {
                    if block.has_value() {
                        let tile_type = block.tile_type();

                        // C++ condition: if (!isFloor || tileType == TransparentSquare)
                        if !is_floor
                            || tile_type == crate::tiles::types::TileType::TransparentSquare
                        {
                            drop(mgr); // Release borrow

                            // C++ nested condition: if (isFloor && tileType == TransparentSquare)
                            if is_floor
                                && tile_type == crate::tiles::types::TileType::TransparentSquare
                            {
                                // Render foliage (grass)
                                let _ = self.render_floor_foliage(
                                    engine,
                                    texture_mgr,
                                    level_piece_id,
                                    0,
                                    screen_x,
                                    screen_y,
                                    trace_frame,
                                );
                            } else {
                                let mask =
                                    DrawCellMask::first_left(tile_type, transparency, tile_props)
                                        .enabled(mask_aware);
                                // Render normal tile
                                let _ = self.render_micro_tile(
                                    engine,
                                    texture_mgr,
                                    level_piece_id,
                                    0,
                                    screen_x,
                                    screen_y,
                                    micro_x,
                                    micro_y,
                                    mask,
                                    trace_frame,
                                );
                            }
                        }
                    }
                }
            }
        }

        // Block 1 (right half)
        {
            let mgr = texture_mgr.borrow();
            if let Some(piece) = mgr.get_piece(level_piece_id) {
                if let Some(block) = piece.mt.get(1) {
                    if block.has_value() {
                        let tile_type = block.tile_type();

                        if !is_floor
                            || tile_type == crate::tiles::types::TileType::TransparentSquare
                        {
                            drop(mgr); // Release borrow

                            // C++ nested condition: if (isFloor && tileType == TransparentSquare)
                            if is_floor
                                && tile_type == crate::tiles::types::TileType::TransparentSquare
                            {
                                // Render foliage (grass)
                                let _ = self.render_floor_foliage(
                                    engine,
                                    texture_mgr,
                                    level_piece_id,
                                    1,
                                    screen_x + 32,
                                    screen_y,
                                    trace_frame,
                                );
                            } else {
                                let mask =
                                    DrawCellMask::first_right(tile_type, transparency, tile_props)
                                        .enabled(mask_aware);
                                // Render normal tile
                                let _ = self.render_micro_tile(
                                    engine,
                                    texture_mgr,
                                    level_piece_id,
                                    1,
                                    screen_x + 32,
                                    screen_y,
                                    micro_x + 1,
                                    micro_y,
                                    mask,
                                    trace_frame,
                                );
                            }
                        }
                    }
                }
            }
        }

        // ✅ KEY FIX: ALWAYS render blocks 2+ (wall layers), regardless of blocks 0-1
        // Reference: C++ DrawCell() Line 614-632
        // The loop for blocks 2+ is NOT inside the blocks 0-1 checks
        //
        // Draw blocks 2 and above (wall layers)
        // Each pair of blocks goes up one TILE_HEIGHT
        let mut y = screen_y - TILE_HEIGHT;
        for i in (2..blocks_per_piece).step_by(2) {
            self.render_micro_tile(
                engine,
                texture_mgr,
                level_piece_id,
                i,
                screen_x,
                y,
                micro_x,
                micro_y,
                DrawCellMask::upper_block(transparency).enabled(mask_aware),
                trace_frame,
            )?;
            if i + 1 < blocks_per_piece {
                self.render_micro_tile(
                    engine,
                    texture_mgr,
                    level_piece_id,
                    i + 1,
                    screen_x + 32,
                    y,
                    micro_x + 1,
                    micro_y,
                    DrawCellMask::upper_block(transparency).enabled(mask_aware),
                    trace_frame,
                )?;
            }
            y -= TILE_HEIGHT;
        }

        Ok(())
    }

    /// Render a single micro tile
    ///
    /// # Reference
    /// Original: `Source/engine/render/dun_render.cpp::RenderTile()`
    fn render_micro_tile(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        level_piece_id: usize,
        block_index: usize,
        screen_x: i32,
        screen_y: i32,
        micro_x: i32,
        micro_y: i32,
        mask: DrawCellMask,
        trace_frame: Option<u64>,
    ) -> Result<()> {
        self.render_micro_tile_with_kind(
            engine,
            texture_mgr,
            level_piece_id,
            block_index,
            screen_x,
            screen_y,
            TileTextureKind::Normal,
            None,
            mask,
            micro_x,
            micro_y,
            trace_frame,
        )
    }

    fn render_floor_micro_tile(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        level_piece_id: usize,
        block_index: usize,
        screen_x: i32,
        screen_y: i32,
        micro_x: i32,
        micro_y: i32,
        forced_tile_type: crate::tiles::types::TileType,
        trace_frame: Option<u64>,
    ) -> Result<()> {
        self.render_micro_tile_with_kind(
            engine,
            texture_mgr,
            level_piece_id,
            block_index,
            screen_x,
            screen_y,
            TileTextureKind::Floor,
            Some(forced_tile_type),
            DrawCellMask::Solid,
            micro_x,
            micro_y,
            trace_frame,
        )
    }

    fn render_micro_tile_with_kind(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        level_piece_id: usize,
        block_index: usize,
        screen_x: i32,
        screen_y: i32,
        texture_kind: TileTextureKind,
        forced_tile_type: Option<crate::tiles::types::TileType>,
        mask: DrawCellMask,
        micro_x: i32,
        micro_y: i32,
        trace_frame: Option<u64>,
    ) -> Result<()> {
        use crate::tiles::types::TileType;

        // Get tile type and dimensions (using immutable borrow)
        let actual_tile_type = {
            let mgr = texture_mgr.borrow();
            if let Some(piece) = mgr.get_piece(level_piece_id) {
                if let Some(block) = piece.mt.get(block_index) {
                    if !block.has_value() {
                        return Ok(()); // Empty block
                    }
                    block.tile_type()
                } else {
                    return Ok(()); // No block
                }
            } else {
                return Ok(()); // No piece
            }
        };
        let render_tile_type = forced_tile_type.unwrap_or(actual_tile_type);
        let (actual_width, actual_height) = Self::tile_dimensions(actual_tile_type);
        let (width, height) = Self::tile_dimensions(render_tile_type);
        let cache_key = Self::tile_texture_cache_key(
            level_piece_id,
            block_index,
            texture_kind,
            mask,
            self.render_debug.toon_filter,
        );
        let rect_y = screen_y - height as i32 + 1;
        let rect = Rect::new(screen_x, rect_y, width, height);

        if engine.draw_cached_texture(cache_key, rect)? {
            MICRO_TEXTURE_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            if let Some(frame) = trace_frame {
                println!(
                    "    [render-trace #{:06}] pass=micro-cache-hit kind={:?} dpiece=({}, {}) piece={} block={} actual={:?} forced={:?} render={:?} mask={:?} dst=({}, {}) rect=({}, {}) size={}x{}",
                    frame,
                    texture_kind,
                    micro_x,
                    micro_y,
                    level_piece_id,
                    block_index,
                    actual_tile_type,
                    forced_tile_type,
                    render_tile_type,
                    mask,
                    screen_x,
                    screen_y,
                    rect.x,
                    rect.y,
                    width,
                    height
                );
            }
            return Ok(());
        }
        MICRO_TEXTURE_CACHE_MISSES.fetch_add(1, Ordering::Relaxed);

        // Get indexed pixels (using mutable borrow)
        let indexed_pixels_result = texture_mgr
            .borrow_mut()
            .get_indexed_tile_pixels(level_piece_id, block_index);

        match indexed_pixels_result {
            Ok(mut indexed_pixels) => {
                let expected_actual_size = (actual_width * actual_height) as usize;
                if indexed_pixels.len() != expected_actual_size {
                    return Ok(());
                }

                indexed_pixels =
                    Self::rearrange_indexed_pixels_for_render(indexed_pixels, actual_tile_type);

                if let Some(forced_tile_type) = forced_tile_type {
                    if !matches!(
                        forced_tile_type,
                        TileType::LeftTriangle | TileType::RightTriangle
                    ) {
                        return Ok(());
                    }
                    indexed_pixels = Self::extract_floor_triangle_pixels(
                        &indexed_pixels,
                        actual_width as usize,
                        actual_height as usize,
                        forced_tile_type,
                    );
                    indexed_pixels =
                        Self::rearrange_indexed_pixels_for_render(indexed_pixels, forced_tile_type);
                }

                let expected_indexed_size = (width * height) as usize;
                if indexed_pixels.len() != expected_indexed_size {
                    return Ok(());
                }

                // Convert indexed pixels to RGBA using palette
                let mgr = texture_mgr.borrow();
                let rgba_pixels = mgr.palette().indices_to_rgba(&indexed_pixels, true);
                drop(mgr);

                let expected_size = (width * height * 4) as usize;
                if rgba_pixels.len() != expected_size {
                    return Ok(());
                }

                let mut rgba_pixels = rgba_pixels;
                Self::apply_draw_cell_mask_to_rgba(
                    &mut rgba_pixels,
                    width as usize,
                    height as usize,
                    mask,
                );

                // Skip if all pixels are transparent.
                // This prevents doing toon edge detection on fully transparent tiles.
                let has_visible_pixels = rgba_pixels.chunks(4).any(|p| p[3] > 0);
                if !has_visible_pixels {
                    return Ok(());
                }

                if self.render_debug.toon_filter {
                    let mut scratch = self.render_scratch.borrow_mut();
                    Self::apply_toon_filter(
                        &mut rgba_pixels,
                        width as usize,
                        height as usize,
                        &mut scratch,
                    );
                }

                let texture_id = format!(
                    "tile_{}_{}_{}_{}",
                    texture_kind.texture_id_fragment(),
                    level_piece_id,
                    block_index,
                    mask.cache_index()
                );
                if let Some(frame) = trace_frame {
                    println!(
                        "    [render-trace #{:06}] pass=micro kind={:?} dpiece=({}, {}) piece={} block={} actual={:?} forced={:?} render={:?} mask={:?} dst=({}, {}) rect=({}, {}) size={}x{}",
                        frame,
                        texture_kind,
                        micro_x,
                        micro_y,
                        level_piece_id,
                        block_index,
                        actual_tile_type,
                        forced_tile_type,
                        render_tile_type,
                        mask,
                        screen_x,
                        screen_y,
                        rect.x,
                        rect.y,
                        width,
                        height
                    );
                }

                engine
                    .draw_cached_rgba_texture(cache_key, &rgba_pixels, width, height, rect)
                    .or_else(|_| {
                        engine.draw_rgba_texture(&texture_id, &rgba_pixels, width, height, rect)
                    })?;
            }
            Err(_) => {
                // Silently ignore decode errors
            }
        }
        Ok(())
    }

    fn tile_dimensions(tile_type: crate::tiles::types::TileType) -> (u32, u32) {
        use crate::tiles::types::TileType;

        match tile_type {
            TileType::LeftTriangle | TileType::RightTriangle => (32, 31),
            _ => (32, 32),
        }
    }

    fn rearrange_indexed_pixels_for_render(
        indexed_pixels: Vec<u8>,
        tile_type: crate::tiles::types::TileType,
    ) -> Vec<u8> {
        use crate::tiles::types::TileType;

        match tile_type {
            TileType::LeftTriangle => {
                if indexed_pixels.len() != 32 * 31 {
                    return indexed_pixels;
                }
                let mut rearranged = vec![0u8; indexed_pixels.len()];
                for row in 0..31usize {
                    let row_start = row * 32;
                    let pixel_width = Self::triangle_row_width(row);
                    let offset = 32 - pixel_width;
                    rearranged[row_start + offset..row_start + offset + pixel_width]
                        .copy_from_slice(&indexed_pixels[row_start..row_start + pixel_width]);
                }
                rearranged
            }
            TileType::RightTriangle => {
                if indexed_pixels.len() != 32 * 31 {
                    return indexed_pixels;
                }
                let mut rearranged = vec![0u8; indexed_pixels.len()];
                for row in 0..31usize {
                    let row_start = row * 32;
                    let pixel_width = Self::triangle_row_width(row);
                    let src_offset = 32 - pixel_width;
                    rearranged[row_start..row_start + pixel_width].copy_from_slice(
                        &indexed_pixels
                            [row_start + src_offset..row_start + src_offset + pixel_width],
                    );
                }
                rearranged
            }
            TileType::LeftTrapezoid => {
                if indexed_pixels.len() != 32 * 32 {
                    return indexed_pixels;
                }
                let mut rearranged = vec![0u8; indexed_pixels.len()];
                for row in 0..32usize {
                    let row_start = row * 32;
                    if row <= 15 {
                        let pixel_width = 2 * (row + 1);
                        let offset = 32 - pixel_width;
                        rearranged[row_start + offset..row_start + offset + pixel_width]
                            .copy_from_slice(&indexed_pixels[row_start..row_start + pixel_width]);
                    } else {
                        rearranged[row_start..row_start + 32]
                            .copy_from_slice(&indexed_pixels[row_start..row_start + 32]);
                    }
                }
                rearranged
            }
            TileType::RightTrapezoid => {
                if indexed_pixels.len() != 32 * 32 {
                    return indexed_pixels;
                }
                let mut rearranged = vec![0u8; indexed_pixels.len()];
                for row in 0..32usize {
                    let row_start = row * 32;
                    if row <= 15 {
                        let pixel_width = 2 * (row + 1);
                        let src_offset = 32 - pixel_width;
                        rearranged[row_start..row_start + pixel_width].copy_from_slice(
                            &indexed_pixels
                                [row_start + src_offset..row_start + src_offset + pixel_width],
                        );
                    } else {
                        rearranged[row_start..row_start + 32]
                            .copy_from_slice(&indexed_pixels[row_start..row_start + 32]);
                    }
                }
                rearranged
            }
            TileType::Square | TileType::TransparentSquare => indexed_pixels,
        }
    }

    fn triangle_row_width(row: usize) -> usize {
        if row <= 15 {
            2 * (row + 1)
        } else {
            32 - 2 * (row - 15)
        }
    }

    fn extract_floor_triangle_pixels(
        source: &[u8],
        source_width: usize,
        source_height: usize,
        forced_tile_type: crate::tiles::types::TileType,
    ) -> Vec<u8> {
        use crate::tiles::types::TileType;

        let mut output = vec![0u8; 32 * 31];
        if source_width != 32 || source_height < 31 || source.len() < source_width * source_height {
            return output;
        }

        for row in 0..31usize {
            let pixel_width = Self::triangle_row_width(row);
            let source_row = row * source_width;
            let dest_row = row * 32;
            match forced_tile_type {
                TileType::LeftTriangle => {
                    let src_start = source_row + 32 - pixel_width;
                    output[dest_row..dest_row + pixel_width]
                        .copy_from_slice(&source[src_start..src_start + pixel_width]);
                }
                TileType::RightTriangle => {
                    let src_start = source_row;
                    let dest_start = dest_row + 32 - pixel_width;
                    output[dest_start..dest_start + pixel_width]
                        .copy_from_slice(&source[src_start..src_start + pixel_width]);
                }
                _ => {}
            }
        }

        output
    }

    fn apply_draw_cell_mask_to_rgba(
        pixels: &mut [u8],
        width: usize,
        height: usize,
        mask: DrawCellMask,
    ) {
        if mask == DrawCellMask::Solid {
            return;
        }

        let expected = width
            .checked_mul(height)
            .and_then(|v| v.checked_mul(4))
            .unwrap_or(0);
        if expected == 0 || pixels.len() != expected {
            return;
        }

        for y in 0..height {
            for x in 0..width {
                let alpha_idx = (y * width + x) * 4 + 3;
                if pixels[alpha_idx] == 0 {
                    continue;
                }

                let masked = match mask {
                    DrawCellMask::Solid => false,
                    DrawCellMask::Transparent => true,
                    DrawCellMask::Left => Self::is_left_mask_pixel(x, y, width, height),
                    DrawCellMask::Right => Self::is_right_mask_pixel(x, y, width, height),
                };

                if masked {
                    pixels[alpha_idx] = DrawCellMask::TRANSPARENT_ALPHA;
                }
            }
        }
    }

    fn is_left_mask_pixel(x: usize, y: usize, width: usize, height: usize) -> bool {
        if width == 0 || height == 0 {
            return false;
        }

        let upper_start = height.saturating_sub(16);
        if y < upper_start {
            return false;
        }

        let upper_row = y - upper_start;
        let opaque_width = ((upper_row + 1) * 2).min(width);
        x < width.saturating_sub(opaque_width)
    }

    fn is_right_mask_pixel(x: usize, y: usize, width: usize, height: usize) -> bool {
        if width == 0 || height == 0 {
            return false;
        }

        let upper_start = height.saturating_sub(16);
        if y < upper_start {
            return false;
        }

        let upper_row = y - upper_start;
        let opaque_width = ((upper_row + 1) * 2).min(width);
        x >= opaque_width
    }

    /// Apply a simple toon/comic post-process to a tile's RGBA buffer.
    /// - Posterize colors to fewer bands.
    /// - Add black outlines based on Sobel edge detection on luminance.
    fn apply_toon_filter(
        pixels: &mut [u8],
        width: usize,
        height: usize,
        scratch: &mut RenderScratch,
    ) {
        let expected = width
            .checked_mul(height)
            .and_then(|v| v.checked_mul(4))
            .unwrap_or(0);
        if expected == 0 || pixels.len() != expected {
            return;
        }

        // Posterize step: reduce color bands for a flat look.
        // Use u16 math to avoid `256` overflowing `u8` literals.
        const POSTERIZE_LEVELS: u16 = 6;
        const EDGE_THRESHOLD: i32 = 600;
        const SOBEL_KX: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
        const SOBEL_KY: [[i32; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

        let step: u16 = (256u16 / POSTERIZE_LEVELS).max(1);

        scratch.luminance.resize(width * height, 0);
        scratch.luminance.fill(0);
        let luminance = &mut scratch.luminance;

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let a = pixels[idx + 3];
                if a == 0 {
                    continue;
                }
                let r = pixels[idx];
                let g = pixels[idx + 1];
                let b = pixels[idx + 2];

                // Compute luminance for edge detection (simple luma approx).
                let l = (r as u16 * 30 + g as u16 * 59 + b as u16 * 11) / 100;
                luminance[y * width + x] = l as u8;

                // Posterize color channels.
                pixels[idx] = ((r as u16 / step) * step).min(255) as u8;
                pixels[idx + 1] = ((g as u16 / step) * step).min(255) as u8;
                pixels[idx + 2] = ((b as u16 / step) * step).min(255) as u8;
            }
        }

        // Skip edge detection if the tile is too small for a 3x3 kernel.
        if width < 3 || height < 3 {
            return;
        }

        scratch.edges.resize(width * height, false);
        scratch.edges.fill(false);
        let edges = &mut scratch.edges;

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut gx: i32 = 0;
                let mut gy: i32 = 0;
                for j in 0..3 {
                    for i in 0..3 {
                        let v = luminance[(y + j - 1) * width + (x + i - 1)] as i32;
                        gx += v * SOBEL_KX[j][i];
                        gy += v * SOBEL_KY[j][i];
                    }
                }
                // Use L1 magnitude to avoid sqrt.
                let mag = gx.abs() + gy.abs();
                if mag > EDGE_THRESHOLD {
                    edges[y * width + x] = true;
                }
            }
        }

        // Apply outlines: turn edge pixels black (keep alpha).
        for y in 0..height {
            for x in 0..width {
                if edges[y * width + x] {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0;
                    pixels[idx + 1] = 0;
                    pixels[idx + 2] = 0;
                }
            }
        }
    }
}

impl World {
    /// Render player sprite at screen center
    fn render_player_sprite(
        &self,
        engine: &mut Engine,
        screen_width: i32,
        screen_height: i32,
    ) -> Result<()> {
        if let Some(player) = self.entities.first() {
            if player.use_sprite {
                if let Some(ref base_sprite_id) = player.sprite_id {
                    let (anim_state, frame_index) = if let Some(ref anim) = player.animation {
                        (
                            Some(anim.current_state()),
                            anim.current_frame_index().unwrap_or(0),
                        )
                    } else {
                        (None, 0)
                    };

                    if let Some(state) = anim_state {
                        let state_name = match state {
                            AnimationState::Idle => "idle",
                            AnimationState::Walk => "walk",
                            AnimationState::Attack => "attack",
                            AnimationState::Hit => "hit",
                            AnimationState::Death => "death",
                            AnimationState::Cast => "cast",
                        };

                        let texture_id =
                            format!("{}_{}_{}", base_sprite_id, state_name, frame_index);

                        if engine.texture_manager().contains(&texture_id) {
                            let player_rect = Rect::from_center(
                                Point::new(screen_width / 2, screen_height / 2 + 120),
                                player.size.0,
                                player.size.1,
                            );

                            let _ = engine.draw_texture_by_id(&texture_id, None, player_rect);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Sample average color from RGBA pixel data
    ///
    /// Takes a sample of pixels and computes the average non-transparent color.
    fn sample_tile_color(rgba_pixels: &[u8]) -> Color {
        let mut r_sum: u32 = 0;
        let mut g_sum: u32 = 0;
        let mut b_sum: u32 = 0;
        let mut count: u32 = 0;

        // Sample every 8th pixel for performance
        for i in (0..rgba_pixels.len()).step_by(32) {
            if i + 3 < rgba_pixels.len() {
                let a = rgba_pixels[i + 3];
                if a > 128 {
                    // Only count non-transparent pixels
                    r_sum += rgba_pixels[i] as u32;
                    g_sum += rgba_pixels[i + 1] as u32;
                    b_sum += rgba_pixels[i + 2] as u32;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Color::new(
                (r_sum / count) as u8,
                (g_sum / count) as u8,
                (b_sum / count) as u8,
            )
        } else {
            // Default gray for fully transparent tiles
            Color::new(60, 60, 70)
        }
    }
}
