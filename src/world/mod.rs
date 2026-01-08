use crate::debug::RenderDebugFlags;
use crate::engine::Engine;
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
use sdl2::log;

pub mod collision;
pub mod dungeon_map;
pub mod town;

pub use collision::{CollisionMap, TileType};
pub use dungeon_map::{DungeonMap, DMAXX, DMAXY, MAXDUNX, MAXDUNY};
pub use town::SimpleTown;

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
        }
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
                entity.update(dt, Some(move |x, y| {
                    let dpiece_x = x + BORDER_SIZE;
                    let dpiece_y = y + BORDER_SIZE;
                    map.in_bounds(dpiece_x, dpiece_y)
                }));
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
            let has_player_light = self.lighting.light_sources.iter().any(|s| s.id == player_light_id);
            
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
                self.lighting.update_light_position(player_light_id, (micro_x, micro_y));
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
        use crate::engine::isometric::BORDER_SIZE;
        
        if world_x <0 || world_y <0 {
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
        // Draw entities (common for both rendering modes)
        self.render_entities(engine, camera)?;

        Ok(())
    }

    /// Render entities (common for both rendering modes)
    fn render_entities(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
        // Check if we're in isometric rendering mode (dungeon_map exists)
        let is_isometric = self.dungeon_map.is_some();
        
        // Get player position for isometric rendering (if available)
        let player_tile_pos = if is_isometric {
            self.entities.first().map(|p| (p.tile_position.x, p.tile_position.y))
        } else {
            None
        };
        
        // Calculate screen center for isometric rendering
        let screen_center_x = camera.viewport_width as i32 / 2;
        let screen_center_y = camera.viewport_height as i32 / 2;
        
        for entity in &self.entities {
            let screen_pos = if is_isometric {
                // Isometric rendering mode: use isometric projection
                if let Some((player_x, player_y)) = player_tile_pos {
                    // Calculate relative position from player
                    let delta_x = entity.tile_position.x - player_x;
                    let delta_y = entity.tile_position.y - player_y;
                    
                    // Convert to screen coordinates using isometric projection
                    // Reference: Source/engine/displacement.hpp::worldToScreen()
                    use crate::engine::isometric::world_to_screen;
                    let (screen_delta_x, screen_delta_y) = world_to_screen(delta_x, delta_y);
                    
                    // Player is at screen center, other entities are offset from center
                    Point::new(
                        screen_center_x + screen_delta_x,
                        screen_center_y + screen_delta_y,
                    )
                } else {
                    // No player, use entity position directly
                    let (screen_x, screen_y) = crate::engine::isometric::world_to_screen(
                        entity.tile_position.x,
                        entity.tile_position.y,
                    );
                    Point::new(screen_x, screen_y)
                }
            } else {
                // 2D rendering mode: use camera world_to_screen
                let entity_world_pixel = Point::new(
                    entity.tile_position.x * self.tile_size as i32,
                    entity.tile_position.y * self.tile_size as i32,
                );
                camera.world_to_screen(entity_world_pixel)
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

                    // For CL2/CLX sprites, construct texture ID with animation state
                    // Format: "{base}_{state}_{frame}" e.g. "warrior_idle_0", "warrior_walk_3"
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

                        let texture_id =
                            format!("{}_{}_{}", base_sprite_id, state_name, frame_index);

                        if engine.texture_manager().contains(&texture_id) {
                            if engine.draw_texture_by_id(&texture_id, None, dst_rect)? {
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
                            // Final fallback: colored rectangle
                            engine.draw_rect(dst_rect, entity.color)?;
                        }
                    }
                } else {
                    engine.draw_rect(dst_rect, entity.color)?;
                }
            } else {
                // Render as colored rectangle
                engine.draw_rect(dst_rect, entity.color)?;
            }
        }

        Ok(())
    }

    /// Render using TileTextureManager with proper tile decoding (Step 6.2)
    ///
    /// # Reference
    fn render_with_texture_manager(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
        use crate::tiles::types::TileProperties;
        use std::io::Write;



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
            (25 - BORDER_SIZE, 25 - BORDER_SIZE)  // Convert from dPiece to world if needed
        };

        // Debug: focus render to the 2x2 region around the view center (in dPiece coordinates).
        // This is intentionally simple (skip outside tiles) to help isolate rendering issues.
        // Toggle via F7 (RenderDebugFlags.show_debug_info).
        let focus_only_2x2 = self.render_debug.show_debug_info;
        let (focus_min_x, focus_max_x, focus_min_y, focus_max_y) = if focus_only_2x2 {
            let cx = view_x + BORDER_SIZE;
            let cy = view_y + BORDER_SIZE;
            (cx, cx + 1, cy, cy + 1)
        } else {
            (0, 0, 0, 0)
        };
        if focus_only_2x2 {
            println!(
                "[RENDER_FOCUS_2X2] view_world=({}, {}), focus_dpiece_x=[{}..{}], focus_dpiece_y=[{}..{}]",
                view_x, view_y, focus_min_x, focus_max_x, focus_min_y, focus_max_y
            );
        }

        // Calculate how many tiles to render
        // C++ uses columns and rows based on screen size
        let columns = (screen_width / TILE_WIDTH) + 2;
        let rows = (viewport_height / (TILE_HEIGHT / 2)) + 4;

        // Calculate starting screen offset (center the view)
        // C++ CalcTileOffset: offset for centering partial tiles
        // Reference: Source/engine/render/scrollrt.cpp::CalcTileOffset() Line 1518-1541
        // If remainder != 0, use (TILE_WIDTH - remainder) / 2, not remainder / 2
        let remainder_x = screen_width % TILE_WIDTH;
        let remainder_y = viewport_height % TILE_HEIGHT;
        let offset_x = if remainder_x != 0 {
            (TILE_WIDTH - remainder_x) / 2
        } else {
            0
        };
        let offset_y = if remainder_y != 0 {
            (TILE_HEIGHT - remainder_y) / 2
        } else {
            0
        };

        // Calculate starting tile position using isometric coordinate conversion
        // Reference: Source/engine/render/scrollrt.cpp::CalcViewportGeometry() Line 1722-1767
        // Player screen position (center of viewport)
        let player_screen_x = screen_width / 2;
        let player_screen_y = viewport_height / 2;
        
        // Calculate how many tiles from screen center to top/left
        // Reference: Source/engine/render/scrollrt.cpp::CalcViewportGeometry() Line 1734-1735
        let tiles_to_top = (player_screen_y + TILE_HEIGHT - 1) / TILE_HEIGHT;
        let tiles_to_left = (player_screen_x + TILE_WIDTH - 1) / TILE_WIDTH;
        
        // Calculate tileShift using isometric direction vectors
        // Reference: Source/engine/render/scrollrt.cpp::CalcViewportGeometry() Line 1742-1744
        // tileShift += Displacement(Direction::North) * tilesToTop;
        // tileShift += Displacement(Direction::West) * tilesToLeft;
        // Direction::North = (-1, -1), Direction::West = (-1, 1)
        // tileShift = (-tilesToTop - tilesToLeft, -tilesToTop + tilesToLeft)
        let tile_shift_x = -tiles_to_top - tiles_to_left;
        let tile_shift_y = -tiles_to_top + tiles_to_left;
        
        // Starting tile position in world coordinates
        // Reference: Source/engine/render/scrollrt.cpp::CalcFirstTilePosition() Line 1197
        // position += tileShift;
        let start_tile_x = view_x + tile_shift_x;
        let start_tile_y = view_y + tile_shift_y;

        // === Phase 1: Draw Floor (like C++ DrawFloor) ===
        // Reference: Source/engine/render/scrollrt.cpp::DrawGame() Line 1306-1308
        // Render all floor tiles in the view (only if render_floor is enabled)
        if self.render_debug.render_floor {
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

                    if focus_only_2x2
                        && !(dpiece_x >= focus_min_x
                            && dpiece_x <= focus_max_x
                            && dpiece_y >= focus_min_y
                            && dpiece_y <= focus_max_y)
                    {
                        tx += 1;
                        ty -= 1;
                        sx += 64;
                        continue;
                    }
                    if dungeon_map.in_bounds(dpiece_x, dpiece_y) {
                        let level_piece_id = dungeon_map.get_piece(dpiece_x, dpiece_y) as usize;
                        // Check IsFloor
                        let is_floor = if let Some(sol) = sol_data {
                            if let Some(props) = sol.get(level_piece_id) {
                                !props.contains(TileProperties::SOLID)
                                    && !props.contains(TileProperties::BLOCK_MISSILE)
                            } else {
                                true
                            }
                        } else {
                            true
                        };

                        // Render the tile (frame 311 filtering is done in render_micro_tile)
                        if is_floor {
                            if focus_only_2x2 {
                                println!(
                                    "  - phase=floor dpiece=({}, {}) piece_id={}",
                                    dpiece_x, dpiece_y, level_piece_id
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
                            );
                        }
                    } else {
                        // Out of bounds: render black invisible tile
                        // This ensures that areas outside the map are explicitly black
                        use crate::math::Rect;
                        
                        // Create a black color manually since we can't easily import Color constructors here
                        // assuming Color is available from imports
                        let black = crate::renderer::Color { r: 0, g: 0, b: 0};
                        
                        let rect = Rect::new(sx, screen_y, 64, 32);
                        engine.draw_rect(rect, black)?;
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
        }

        // === Phase 2: Draw Walls/Cells (like C++ DrawTileContent/DrawCell) ===
        // Reference: Source/engine/render/scrollrt.cpp::DrawGame() Line 1311-1313
        // After rendering all floors, render ALL tiles' walls and upper layers
        // NOTE: Original C++ DrawTileContent renders ALL tiles, not just walls!
        // Only render if render_walls is enabled
        if self.render_debug.render_walls {
            // ✅ KEY FIX: Extend rows for wall rendering
            // Reference: C++ DrawTileContent() Line 969: rows += MicroTileLen
            // MicroTileLen is typically 8 (for 16 blocks / 2)
            // This ensures tall walls are fully rendered
            const MICRO_TILE_LEN: i32 = 8;
            let wall_rows = rows + MICRO_TILE_LEN;

            // Reset to starting position
            let mut wall_tile_x = start_tile_x;
            let mut wall_tile_y = start_tile_y;
            let mut wall_screen_x = offset_x;
            let mut wall_screen_y = offset_y;
            let mut wall_current_columns = columns;

            for row in 0..wall_rows {
                let mut tx = wall_tile_x;
                let mut ty = wall_tile_y;
                let mut sx = wall_screen_x;

                for _col in 0..wall_current_columns {
                    // Convert world coordinates to dPiece coordinates for array access
                    let dpiece_x = tx + BORDER_SIZE;
                    let dpiece_y = ty + BORDER_SIZE;

                    if focus_only_2x2
                        && !(dpiece_x >= focus_min_x
                            && dpiece_x <= focus_max_x
                            && dpiece_y >= focus_min_y
                            && dpiece_y <= focus_max_y)
                    {
                        tx += 1;
                        ty -= 1;
                        sx += 64;
                        continue;
                    }
                    
                    // Get piece_id (returns 0 if out of bounds, matching C++ behavior)
                    // Reference: C++ DrawTileContent() Line 1031: if (InDungeonBounds(tilePosition))
                    // C++ returns 0 for out-of-bounds: Line 1192: piece_id = InDungeonBounds(...) ? dPiece[...] : 0
                    let level_piece_id = dungeon_map.get_piece(dpiece_x, dpiece_y) as usize;

                    // Check IsFloor (same logic as Phase 1)
                    let is_floor = if let Some(sol) = sol_data {
                        if let Some(props) = sol.get(level_piece_id) {
                            !props.contains(TileProperties::SOLID)
                                && !props.contains(TileProperties::BLOCK_MISSILE)
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    // ✅ KEY FIX: Render ALL tiles in Phase 2, not just non-floor tiles
                    // Reference: C++ DrawTileContent() Line 996 calls DrawDungeon() for ALL tiles
                    // Even if out of bounds (piece_id = 0), we still call draw_cell_at
                    // draw_cell_at will handle piece_id = 0 gracefully (no rendering)
                    if focus_only_2x2 {
                        println!(
                            "  - phase=cell dpiece=({}, {}) piece_id={}",
                            dpiece_x, dpiece_y, level_piece_id
                        );
                    }
                    let _ = self.draw_cell_at(
                        engine,
                        texture_mgr_cell,
                        level_piece_id,
                        sx,
                        wall_screen_y,
                        is_floor,
                        dpiece_x,
                        dpiece_y,
                    );

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
        }

        Ok(())
    }

    /// Draw floor tile at screen position
    ///
    /// # Reference
    /// Draw floor tile at screen position
    ///
    /// # Reference
    /// Original: `Source/engine/render/scrollrt.cpp::DrawFloorTile()` Line 652-677
    /// C++ DrawFloorTile:
    /// - Only renders block 0 and block 1
    /// - Forces TileType::LeftTriangle for block 0, TileType::RightTriangle for block 1
    /// - Only checks hasValue(), ignores actual TileType stored in block
    /// - Uses fixed height: DunFrameTriangleHeight = 31
    ///
    /// Rust implementation: Use simple decode-then-render approach
    fn draw_floor_at(
        &self,
        engine: &mut Engine,
        texture_mgr: &RefCell<TileTextureManager>,
        level_piece_id: usize,
        screen_x: i32,
        screen_y: i32,
        micro_x: i32,
        micro_y: i32,
    ) -> Result<()> {
        // Render block 0 (LeftTriangle) at screen_x
        self.render_micro_tile(engine, texture_mgr, level_piece_id, 0, screen_x, screen_y, micro_x, micro_y)?;

        // Render block 1 (RightTriangle) at screen_x + 32
        self.render_micro_tile(
            engine,
            texture_mgr,
            level_piece_id,
            1,
            screen_x + 32,
            screen_y,
            micro_x + 1,
            micro_y,
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
    ) -> Result<()> {
        // C++: RenderTileFoliage renders at position.y - 16
        let foliage_y = screen_y - 16;

        // Get foliage data (offset 512 bytes from main tile data)
        match texture_mgr
            .borrow_mut()
            .get_decoded_foliage(level_piece_id, block_index)
        {
            Ok(rgba_pixels) => {
                // Foliage is 16 pixels high, 32 pixels wide
                let width = 32u32;
                let height = 16u32;
                let expected_size = (width * height * 4) as usize;

                if rgba_pixels.len() != expected_size {
                    return Ok(());
                }

                // Skip if all pixels are transparent
                let has_visible_pixels = rgba_pixels.chunks(4).any(|p| p[3] > 0);
                if !has_visible_pixels {
                    return Ok(());
                }

                let texture_id = format!("foliage_{}_{}", level_piece_id, block_index);
                let rect = Rect::new(screen_x, foliage_y, width, height);
                engine.draw_rgba_texture(&texture_id, rgba_pixels, width, height, rect)?;
            }
            Err(_e) => {
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
        micro_x: i32,
        micro_y: i32,
    ) -> Result<()> {
        const TILE_HEIGHT: i32 = 32;

        let blocks_per_piece = texture_mgr.borrow().blocks_per_piece();
        

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
                                );
                            } else {
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
                                );
                            } else {
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
            self.render_micro_tile(engine, texture_mgr, level_piece_id, i, screen_x, y, micro_x, micro_y)?;
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
    ) -> Result<()> {
        // Get tile type and dimensions (using immutable borrow)
        let (width, height, tile_type_num) = {
            let mgr = texture_mgr.borrow();
            if let Some(piece) = mgr.get_piece(level_piece_id) {
                if let Some(block) = piece.mt.get(block_index) {
                    if !block.has_value() {
                        return Ok(()); // Empty block
                    }
                    use crate::tiles::types::TileType;
                    let tt = block.tile_type();
                    let tt_num = tt as u8;
                    match tt {
                        TileType::LeftTriangle | TileType::RightTriangle => {
                            (32u32, 31u32, tt_num) // Triangles are 32x31
                        }
                        _ => {
                            (32u32, 32u32, tt_num) // Other types are 32x32
                        }
                    }
                } else {
                    return Ok(()); // No block
                }
            } else {
                return Ok(()); // No piece
            }
        };

        // Get indexed pixels (using mutable borrow)
        let indexed_pixels_result = texture_mgr
            .borrow_mut()
            .get_indexed_tile_pixels(level_piece_id, block_index);

        match indexed_pixels_result {
            Ok(mut indexed_pixels) => {
                let expected_indexed_size = (width * height) as usize;
                if indexed_pixels.len() != expected_indexed_size {
                    return Ok(());
                }

                // Rearrange triangle pixels to match C++ rendering layout
                use crate::tiles::types::TileType;
                
                 if tile_type_num == TileType::LeftTriangle as u8 {
                     // LeftTriangle: Rust decoder outputs left-aligned, C++ renders right-aligned
                     // - Lower half (row 0-15): need to right-align (pixels at end of row)
                     // - Upper half (row 16-30): need left-padding (offset from start)
                     let mut rearranged = vec![0u8; indexed_pixels.len()];
                    for row in 0..31usize {
                        let row_start = row * 32;
                        if row <= 15 {
                            // Lower half: width = 2*(row+1), right-align
                            let pixel_width = 2 * (row + 1);
                            let offset = 32 - pixel_width;
                            for i in 0..pixel_width {
                                rearranged[row_start + offset + i] = indexed_pixels[row_start + i];
                            }
                        } else {
                            // Upper half: left-pad with offset = 2*(row-15)
                            let offset = 2 * (row - 15);
                            let pixel_width = 32 - offset;
                            for i in 0..pixel_width {
                                rearranged[row_start + offset + i] = indexed_pixels[row_start + i];
                            }
                        }
                    }
                     indexed_pixels = rearranged;
                 } else if tile_type_num == TileType::RightTriangle as u8 {
                     // RightTriangle: Rust decoder outputs RIGHT-aligned, C++ renders LEFT-aligned
                     // Need to move pixels from right side to left side of each row
                     let mut rearranged = vec![0u8; indexed_pixels.len()];
                    for row in 0..31usize {
                        let row_start = row * 32;
                        let pixel_width = if row <= 15 {
                            2 * (row + 1)  // 2, 4, 6, ..., 32
                        } else {
                            32 - 2 * (row - 15)  // 30, 28, 26, ..., 2
                        };
                        // Rust decoder has pixels at the END of row (right-aligned)
                        // C++ expects pixels at the START of row (left-aligned)
                        let src_offset = 32 - pixel_width;
                        for i in 0..pixel_width {
                            rearranged[row_start + i] = indexed_pixels[row_start + src_offset + i];
                        }
                    }
                     indexed_pixels = rearranged;
                 } else if tile_type_num == TileType::LeftTrapezoid as u8 {
                     // LeftTrapezoid: lower half is rendered using LeftTriangleLower in C++,
                     // which expects the triangle part to be right-aligned. Upper half is a
                     // full-width rectangle and does not need reordering.
                     let mut rearranged = vec![0u8; indexed_pixels.len()];
                     for row in 0..32usize {
                         let row_start = row * 32;
                         if row <= 15 {
                             let pixel_width = 2 * (row + 1); // 2, 4, 6, ..., 32
                             let offset = 32 - pixel_width;
                             rearranged[row_start + offset..row_start + offset + pixel_width]
                                 .copy_from_slice(
                                     &indexed_pixels[row_start..row_start + pixel_width],
                                 );
                         } else {
                             rearranged[row_start..row_start + 32]
                                 .copy_from_slice(&indexed_pixels[row_start..row_start + 32]);
                         }
                     }
                     indexed_pixels = rearranged;
                 } else if tile_type_num == TileType::RightTrapezoid as u8 {
                     // RightTrapezoid: lower half is rendered using RightTriangleLower in C++,
                     // which expects the triangle part to be left-aligned. Upper half is a
                     // full-width rectangle and does not need reordering.
                     let mut rearranged = vec![0u8; indexed_pixels.len()];
                     for row in 0..32usize {
                         let row_start = row * 32;
                         if row <= 15 {
                             let pixel_width = 2 * (row + 1); // 2, 4, 6, ..., 32
                             let src_offset = 32 - pixel_width;
                             rearranged[row_start..row_start + pixel_width].copy_from_slice(
                                 &indexed_pixels[row_start + src_offset
                                     ..row_start + src_offset + pixel_width],
                             );
                         } else {
                             rearranged[row_start..row_start + 32]
                                 .copy_from_slice(&indexed_pixels[row_start..row_start + 32]);
                         }
                     }
                     indexed_pixels = rearranged;
                 }

                 // Convert indexed pixels to RGBA using palette
                 let mgr = texture_mgr.borrow();
                 let rgba_pixels = mgr.palette().indices_to_rgba(&indexed_pixels, true);
                drop(mgr);

                let expected_size = (width * height * 4) as usize;
                if rgba_pixels.len() != expected_size {
                    return Ok(());
                }

                // Skip if all pixels are transparent
                let has_visible_pixels = rgba_pixels.chunks(4).any(|p| p[3] > 0);
                if !has_visible_pixels {
                    return Ok(());
                }

                let texture_id = format!("tile_{}_{}", level_piece_id, block_index);
                // screen_y is the BOTTOM of the tile (like C++ position.y)
                // But Rect::new expects TOP-LEFT corner, so convert
                let rect_y = screen_y - height as i32 + 1;
                let rect = Rect::new(screen_x, rect_y, width, height);
                
                engine.draw_rgba_texture(&texture_id, &rgba_pixels, width, height, rect)?;
            }
            Err(_) => {
                // Silently ignore decode errors
            }
        }
        Ok(())
    }
}

impl World {
    /// Draw a black tile (for out-of-bounds areas)
    /// Reference: Source/engine/render/scrollrt.cpp::world_draw_black_tile()
    fn draw_black_tile(&self, engine: &mut Engine, screen_x: i32, screen_y: i32) -> Result<()> {
        // Draw a 64x32 black rectangle (isometric tile area)
        let sdl_rect = sdl2::rect::Rect::new(screen_x, screen_y, 64, 32);
        engine
            .canvas_mut()
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        engine.canvas_mut().fill_rect(sdl_rect).ok();
        Ok(())
    }

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
