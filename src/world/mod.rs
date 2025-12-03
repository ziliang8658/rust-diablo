/// World module - Game world and level system
/// 
/// This module manages the game world, including:
/// - Grid-based map
/// - Entity management
/// - World rendering

use crate::math::{Point, Rect};
use crate::renderer::{Color, Camera};
use crate::entity::Entity;
use crate::engine::Engine;
use crate::sprite::AnimationState;
use crate::tiles::{MinData, TilData, SolData, texture_manager::TileTextureManager};
use anyhow::Result;
use std::cell::RefCell;

pub mod collision;
pub mod town;
pub mod dungeon_map;

pub use collision::{TileType, CollisionMap};
pub use town::SimpleTown;
pub use dungeon_map::{DungeonMap, DMAXX, DMAXY, MAXDUNX, MAXDUNY};

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
    pub fn new(width: usize, height: usize, tile_size: u32) -> Self {
        // Initialize with a simple floor pattern
        let mut tiles = vec![vec![TileType::Empty; width]; height];
        // Ensure center is clear for player
        let center_x = width / 2;
        let center_y = height / 2;
        for y in center_y-2..=center_y+2 {
            for x in center_x-2..=center_x+2 {
                tiles[y][x] = TileType::Floor;
            }
        }

        // Create collision map
        let collision_map = CollisionMap::new(width, height, tiles);

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






    /// Update all entities
    pub fn update(&mut self, dt: f32) {
        // If we have tile data loaded, use tile-based collision
        // Otherwise use the legacy collision map
        if self.min_data.is_some() && self.sol_data.is_some() {
            // Tile-based collision (Step 6.1)
            // For now, disable collision to allow free movement
            // TODO: Implement proper tile-based collision detection
            for entity in &mut self.entities {
                entity.update(dt, None);
            }
        } else {
            // Legacy collision map
            let tile_size = self.tile_size;
            let map = &self.collision_map;

            for entity in &mut self.entities {
                entity.update(dt, Some((map, tile_size)));
            }
        }
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
                dungeon_map, mpq_manager, dun_path, &til_data, offset_x, offset_y, default_piece
            )?;
        } else {
            // Create new map and load first sector
            let dungeon_map = load_dun_to_dpiece(
                mpq_manager, dun_path, &til_data, offset_x, offset_y, default_piece
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
    pub fn is_tile_walkable(&self, world_x: i32, world_y: i32) -> bool {
        // If we have SOL data, use it for collision detection
        if let (Some(min_data), Some(til_data), Some(sol_data)) =
            (&self.min_data, &self.til_data, &self.sol_data) {

            // Convert world coordinates to tile coordinates
            // tile_size is typically 32 for the test world
            let tile_x = world_x / (self.tile_size as i32);
            let tile_y = world_y / (self.tile_size as i32);

            // Get MegaTile index (wrap around to create repeating pattern)
            let tile_index = ((tile_y.abs() * 10 + tile_x.abs()) as usize) % til_data.len();

            if let Some(mega_tile) = til_data.get(tile_index) {
                // Check the first MicroTile (micro1) for collision
                // micro1 is a levelPieceId, which points to a PieceMicros in MIN
                if let Some(piece) = min_data.get(mega_tile.micro1 as usize) {
                    // Check the first block in the piece
                    if let Some(block) = piece.mt.first() {
                        if block.has_value() {
                            // Get the tile's frame number to look up in SOL data
                            let frame = block.frame() as usize;

                            if let Some(properties) = sol_data.get(frame) {
                                // Check if tile is solid (not walkable)
                                return !properties.is_solid();
                            }
                        }
                    }
                }
            }

            // Default to walkable if we can't determine
            true
        } else {
            // Fallback to collision map if tile data not available
            let tile_x = (world_x / self.tile_size as i32).max(0).min(self.width as i32 - 1);
            let tile_y = (world_y / self.tile_size as i32).max(0).min(self.height as i32 - 1);
            self.collision_map.tiles[tile_y as usize][tile_x as usize].is_walkable()
        }
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
        for entity in &self.entities {
            // Skip entities not visible in camera
            if !camera.is_rect_visible(entity.bounds()) {
                continue;
            }

            let screen_pos = camera.world_to_screen(entity.position);
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

                        let texture_id = format!("{}_{}_{}",base_sprite_id, state_name, frame_index);

                        if engine.texture_manager().contains(&texture_id) {
                            if engine.draw_texture_by_id(&texture_id, None, dst_rect)? {
                                texture_found = true;
                            }
                        }
                    }

                    // Fallback: Try traditional sprite sheet with src_rect
                    if !texture_found {
                        let src_rect = entity.animation
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
    /// Original: `Source/engine/render/scrollrt.cpp::DrawGame()` Line 1132-1189
    /// Original: `Source/engine/render/scrollrt.cpp::DrawFloor()` Line 927-955
    /// Original: `Source/engine/render/scrollrt.cpp::DrawTileContent()` Line 966-1016
    fn render_with_texture_manager(&self, engine: &mut Engine, camera: &Camera) -> Result<()> {
        use crate::tiles::types::TileProperties;
        use std::fs::File;
        use std::io::Write;
        use std::sync::atomic::{AtomicBool, Ordering};

        // DEBUG: Output d_piece array to file for comparison
        static DPIECE_DUMPED: AtomicBool = AtomicBool::new(false);
        if !DPIECE_DUMPED.swap(true, Ordering::Relaxed) {
            if let Some(dm) = self.dungeon_map.as_ref() {
                if let Ok(mut f) = File::create("dpiece_rust.txt") {
                    writeln!(f, "=== Rust d_piece Array (MAXDUNX={}, MAXDUNY={}) ===",
                        crate::world::dungeon_map::MAXDUNX, crate::world::dungeon_map::MAXDUNY).ok();
                    for y in 0..crate::world::dungeon_map::MAXDUNY {
                        for x in 0..crate::world::dungeon_map::MAXDUNX {
                            write!(f, "{}", dm.d_piece[x][y]).ok();
                            if x < crate::world::dungeon_map::MAXDUNX - 1 {
                                write!(f, ",").ok();
                            }
                        }
                        writeln!(f).ok();
                    }
                    println!("✓ Rust d_piece array dumped to dpiece_rust.txt");
                }
            }
        }

        let texture_mgr_cell = self.texture_manager.as_ref().unwrap();
        let dungeon_map = match self.dungeon_map.as_ref() {
            Some(dm) => dm,
            None => return Ok(()), // No map data, skip rendering
        };
        let sol_data = self.sol_data.as_ref();

        // Use viewport dimensions (game area, excluding UI panels)
        // Reference: C++ uses GetViewportHeight() not GetScreenHeight()
        // Source/engine/render/scrollrt.cpp::CalcTileOffset() Line 1521
        // viewportHeight = screenHeight - MainPanel.height (if UI is visible)
        let screen_width = camera.viewport_width as i32;
        let viewport_height = camera.viewport_height as i32;

        // Tile dimensions
        const TILE_WIDTH: i32 = 64;
        const TILE_HEIGHT: i32 = 32;

        // Get player position in dPiece coordinates
        // Reference: dPiece has 16-tile border, so world(0,0) maps to dPiece(16,16)
        // From game.rs: world_x = ((x as i32 - 16) * 32)
        // So inverse: dPiece_x = (world_x / 32) + 16
        use crate::engine::isometric::BORDER_SIZE;
        let (view_x, view_y) = if let Some(player) = self.entities.first() {
            // Convert world position to dPiece coordinates
            // Add BORDER_SIZE (16) because dPiece has a 16-tile border
            let px = (player.position.x / 32) as i32 + BORDER_SIZE;
            let py = (player.position.y / 32) as i32 + BORDER_SIZE;


            (px.clamp(1, MAXDUNX as i32 - 10), py.clamp(1, MAXDUNY as i32 - 10))
        } else {
            // Default view position - center of loaded data
            // For Town sector1s loaded at (0,0), data spans 0..50 in dPiece
            (25, 25)
        };

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

        // Starting tile position (view center)
        let start_tile_x = view_x;
        let start_tile_y = view_y;

        // === Phase 1: Draw Floor (like C++ DrawFloor) ===
        // Render all floor tiles in the view

        let mut tile_x = start_tile_x;
        let mut tile_y = start_tile_y;
        let mut screen_x = offset_x;
        let mut screen_y = offset_y;
        let mut current_columns = columns;

        for row in 0..rows {
            let mut tx = tile_x;
            let mut ty = tile_y;
            let mut sx = screen_x;

            for _col in 0..current_columns {
                if dungeon_map.in_bounds(tx, ty) {
                    let level_piece_id = dungeon_map.get_piece(tx, ty) as usize;

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

                    // Render the tile
                    if is_floor {
                        let _ = self.draw_floor_at(engine, texture_mgr_cell, level_piece_id, sx, screen_y);
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

        // === Phase 2: Draw Walls/Cells (like C++ DrawTileContent/DrawCell) ===
        // Reference: Source/engine/render/scrollrt.cpp::DrawTileContent() Line 966-1016
        // After rendering all floors, render ALL tiles' walls and upper layers
        // NOTE: Original C++ DrawTileContent renders ALL tiles, not just walls!
        
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
                if dungeon_map.in_bounds(tx, ty) {
                    let level_piece_id = dungeon_map.get_piece(tx, ty) as usize;
                    
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
                    let _ = self.draw_cell_at(engine, texture_mgr_cell, level_piece_id, sx, wall_screen_y, is_floor);
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
    ) -> Result<()> {
        // Simple approach: decode first, then render
        // Reference: C++ DrawFloorTile() Line 662-677
        // Render block 0 (LeftTriangle) at screen_x
        // Render block 1 (RightTriangle) at screen_x + 32
        
        static mut RENDER_COUNT: usize = 0;
        unsafe {
            RENDER_COUNT += 1;
            if RENDER_COUNT <= 5 {
                println!("  [draw_floor_at] piece={}, screen=({}, {})", level_piece_id, screen_x, screen_y);
            }
        }
        
        // Render block 0 if it has value
        self.render_micro_tile(engine, texture_mgr, level_piece_id, 0, screen_x, screen_y)?;

        // Render block 1 if it has value
        self.render_micro_tile(engine, texture_mgr, level_piece_id, 1, screen_x + 32, screen_y)?;

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
        match texture_mgr.borrow_mut().get_decoded_foliage(level_piece_id, block_index) {
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
        is_floor: bool,  // From SOL data: !TileHasAny(Solid | BlockMissile)
    ) -> Result<()> {
        const TILE_HEIGHT: i32 = 32;

        let blocks_per_piece = texture_mgr.borrow().blocks_per_piece();

        // ✅ KEY FIX: Render blocks 0-1 based on original C++ logic
        // Reference: C++ DrawCell() Line 588-611
        // C++ logic: if (!isFloor || tileType == TileType::TransparentSquare)
        
        // Render blocks 0-1 based on C++ DrawCell() logic
        // Reference: C++ DrawCell() Line 588-611
        
        // Block 0 (left half)
        {
            let mgr = texture_mgr.borrow();
            if let Some(piece) = mgr.get_piece(level_piece_id) {
                if let Some(block) = piece.mt.get(0) {
                    if block.has_value() {
                        let tile_type = block.tile_type();
                        
                        // C++ condition: if (!isFloor || tileType == TransparentSquare)
                        if !is_floor || tile_type == crate::tiles::types::TileType::TransparentSquare {
                            drop(mgr);  // Release borrow
                            
                            // C++ nested condition: if (isFloor && tileType == TransparentSquare)
                            if is_floor && tile_type == crate::tiles::types::TileType::TransparentSquare {
                                // Render foliage (grass)
                                let _ = self.render_floor_foliage(engine, texture_mgr, level_piece_id, 0, screen_x, screen_y);
                            } else {
                                // Render normal tile
                                let _ = self.render_micro_tile(engine, texture_mgr, level_piece_id, 0, screen_x, screen_y);
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
                        
                        // C++ condition: if (!isFloor || tileType == TransparentSquare)
                        if !is_floor || tile_type == crate::tiles::types::TileType::TransparentSquare {
                            drop(mgr);  // Release borrow
                            
                            // C++ nested condition: if (isFloor && tileType == TransparentSquare)
                            if is_floor && tile_type == crate::tiles::types::TileType::TransparentSquare {
                                // Render foliage (grass)
                                let _ = self.render_floor_foliage(engine, texture_mgr, level_piece_id, 1, screen_x + 32, screen_y);
                            } else {
                                // Render normal tile
                                let _ = self.render_micro_tile(engine, texture_mgr, level_piece_id, 1, screen_x + 32, screen_y);
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
            self.render_micro_tile(engine, texture_mgr, level_piece_id, i, screen_x, y)?;
            if i + 1 < blocks_per_piece {
                self.render_micro_tile(engine, texture_mgr, level_piece_id, i + 1, screen_x + 32, y)?;
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
    ) -> Result<()> {
        // 🔍 DEBUG: Count render attempts and skips
        static mut RENDER_ATTEMPTS: usize = 0;
        static mut NO_VALUE_COUNT: usize = 0;
        static mut NO_BLOCK_COUNT: usize = 0;
        static mut NO_PIECE_COUNT: usize = 0;
        static mut ALL_TRANSPARENT_COUNT: usize = 0;
        static mut SIZE_MISMATCH_COUNT: usize = 0;
        static mut DECODE_ERROR_COUNT: usize = 0;
        static mut SUCCESS_COUNT: usize = 0;
        
        unsafe {
            RENDER_ATTEMPTS += 1;
            if RENDER_ATTEMPTS % 500 == 0 {
                let (att, suc, nov, tra, siz, dec) = (
                    RENDER_ATTEMPTS, SUCCESS_COUNT, NO_VALUE_COUNT,
                    ALL_TRANSPARENT_COUNT, SIZE_MISMATCH_COUNT, DECODE_ERROR_COUNT
                );
                println!("📊 att={} suc={} nov={} tra={} siz={} dec={}", 
                    att, suc, nov, tra, siz, dec);
            }
        }
        
        // First check if the block has a value (like C++ levelCelBlock.hasValue())
        {
            let mgr = texture_mgr.borrow();
            if let Some(piece) = mgr.get_piece(level_piece_id) {
                if let Some(block) = piece.mt.get(block_index) {
                    if !block.has_value() {
                        // Empty block - skip rendering (like C++)
                        unsafe {
                            NO_VALUE_COUNT += 1;
                            if NO_VALUE_COUNT <= 5 {
                                println!("⚠️ No value: piece={} block={}", level_piece_id, block_index);
                            }
                        }
                        return Ok(());
                    }
                } else {
                    unsafe {
                        NO_BLOCK_COUNT += 1;
                        if NO_BLOCK_COUNT <= 5 {
                            println!("⚠️ No block: piece={} block_idx={}", level_piece_id, block_index);
                        }
                    }
                    return Ok(());
                }
            } else {
                unsafe {
                    NO_PIECE_COUNT += 1;
                    if NO_PIECE_COUNT <= 5 {
                        println!("⚠️ No piece: piece_id={} (total pieces={})",
                            level_piece_id, mgr.len());
                    }
                }
                return Ok(());
            }
        }

        // First, get tile type and dimensions (using immutable borrow)
        let (width, height) = {
            let mgr = texture_mgr.borrow();
            if let Some(piece) = mgr.get_piece(level_piece_id) {
                if let Some(block) = piece.mt.get(block_index) {
                    use crate::tiles::types::TileType;
                    match block.tile_type() {
                        TileType::LeftTriangle | TileType::RightTriangle => {
                            (32u32, 31u32)  // Triangles are 32x31
                        }
                        _ => {
                            (32u32, 32u32)  // Other types are 32x32
                        }
                    }
                } else {
                    (32u32, 32u32)  // Default
                }
            } else {
                (32u32, 32u32)  // Default
            }
        };

        // Now decode (using mutable borrow - immutable borrow is released)
        match texture_mgr.borrow_mut().get_decoded_tile(level_piece_id, block_index) {
            Ok(rgba_pixels) => {
                let expected_size = (width * height * 4) as usize;

                if rgba_pixels.len() != expected_size {
                    unsafe {
                        SIZE_MISMATCH_COUNT += 1;
                        if SIZE_MISMATCH_COUNT <= 5 {
                            println!("⚠️ Size mismatch: piece={} block={} expected={} got={}", 
                                level_piece_id, block_index, expected_size, rgba_pixels.len());
                        }
                    }
                    return Ok(());
                }

                // Skip if all pixels are transparent
                let has_visible_pixels = rgba_pixels.chunks(4).any(|p| p[3] > 0);
                if !has_visible_pixels {
                    unsafe {
                        ALL_TRANSPARENT_COUNT += 1;
                        if ALL_TRANSPARENT_COUNT <= 5 {
                            println!("⚠️ All transparent: piece={} block={}", level_piece_id, block_index);
                        }
                    }
                    return Ok(());
                }

                // Use direct RGBA rendering instead of TextureCache to avoid unsafe pointer issues
                // TODO: Optimize with proper texture caching in the future
                // Reference: Source/engine/render/dun_render.cpp::RenderTile()
                
                static mut RENDER_COUNT: usize = 0;
                unsafe {
                    RENDER_COUNT += 1;
                    if RENDER_COUNT <= 10 {
                        // Count visible pixels
                        let visible = rgba_pixels.chunks(4).filter(|p| p[3] > 0).count();
                        println!("  [render_micro_tile] piece={} block={} at ({}, {}) size={}x{} visible={}", 
                            level_piece_id, block_index, screen_x, screen_y, width, height, visible);
                    }
                }

                let texture_id = format!("tile_{}_{}", level_piece_id, block_index);
                let rect = Rect::new(screen_x, screen_y, width, height);
                engine.draw_rgba_texture(&texture_id, rgba_pixels, width, height, rect)?;
                
                // 🔍 DEBUG: Count successful renders
                unsafe {
                    SUCCESS_COUNT += 1;
                }
            }
            Err(e) => {
                unsafe {
                    DECODE_ERROR_COUNT += 1;
                    if DECODE_ERROR_COUNT <= 10 {
                        eprintln!("⚠️ Decode error #{}: piece={} block={}", 
                            DECODE_ERROR_COUNT, level_piece_id, block_index);
                        eprintln!("   Error: {:?}", e);
                    }
                    if DECODE_ERROR_COUNT == 100 {
                        eprintln!("⚠️ ... and more decode errors (total so far: 100)");
                    }
                }
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
        engine.canvas_mut().set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        engine.canvas_mut().fill_rect(sdl_rect).ok();
        Ok(())
    }

    /// Render player sprite at screen center
    fn render_player_sprite(&self, engine: &mut Engine, screen_width: i32, screen_height: i32) -> Result<()> {
        if let Some(player) = self.entities.first() {
            if player.use_sprite {
                if let Some(ref base_sprite_id) = player.sprite_id {
                    let (anim_state, frame_index) = if let Some(ref anim) = player.animation {
                        (Some(anim.current_state()), anim.current_frame_index().unwrap_or(0))
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

                        let texture_id = format!("{}_{}_{}",base_sprite_id, state_name, frame_index);

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
                if a > 128 {  // Only count non-transparent pixels
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
