/// Game module - Core game loop and state management
/// 
/// This module contains the main game loop and game state management.
/// It's the central coordinator for all game systems.

use anyhow::Result;
use crate::engine::Engine;
use crate::world::{World, SimpleTown};
use crate::entity::Entity;
use crate::math::{Point, Rect};
use crate::assets::AssetPaths;
use crate::engine::Direction;
use crate::sprite::{AnimationState, Animation};
use std::time::Instant;
use crate::renderer::Camera;
use crate::resources::{MpqManager, Palette, PcxImage, Cl2Sprite, ResourceManager};
use crate::tiles::{MinData, TilData, SolData, MegaTile, LevelCelBlock, TileType, TileProperties};

/// Create simple test dungeon data with controlled frame indices
/// 
/// This creates a simple dungeon with:
/// - Frame 1-6: Floor tiles (map to tile_0-5, 64x32)
/// 
/// Note: Wall tiles temporarily removed. Only floor rendering is active.
fn create_test_dungeon_data() -> (MinData, TilData, SolData) {
    // Create MicroTiles (MIN data)
    // Frame indices: 1-6 = floors only
    let mut micro_tiles = Vec::new();
    
    // 0: Empty tile
    micro_tiles.push(LevelCelBlock::new(0));
    
    // 1-6: Floor tiles (various types)
    for i in 1..=6 {
        micro_tiles.push(LevelCelBlock::from_parts(TileType::Square, i));
    }
    
    // Note: Wall tiles (7-10) removed for simplification
    // Will be re-implemented with proper architecture later
    
    // Create PieceMicros - each piece contains 10 blocks (for Cathedral)
    let pieces: Vec<crate::tiles::PieceMicros> = micro_tiles.iter().map(|block| {
        crate::tiles::PieceMicros {
            mt: vec![*block; 10], // 10 blocks per piece for Cathedral
        }
    }).collect();
    
    let min_data = MinData { pieces, blocks_per_piece: 10 };
    
    // Create MegaTiles (TIL data) - combinations of MicroTiles
    let mut mega_tiles = Vec::new();
    
    // MegaTile 0: Empty
    mega_tiles.push(MegaTile::default());
    
    // MegaTile 1-6: Different floor patterns
    for i in 1..=6 {
        mega_tiles.push(MegaTile {
            micro1: i as u16,
            micro2: i as u16,
            micro3: i as u16,
            micro4: i as u16,
        });
    }
    
    // Note: Wall MegaTiles (7-8) removed for simplification
    // Currently only using floor tiles for rendering
    
    let til_data = TilData { mega_tiles };
    
    // Create SOL data (tile properties)
    let mut sol_properties = vec![TileProperties::empty(); 7];
    
    // All floor tiles (1-6) are walkable
    for i in 1..=6 {
        sol_properties[i] = TileProperties::empty();
    }
    
    let sol_data = SolData { properties: sol_properties };
    
    (min_data, til_data, sol_data)
}

/// Scene type enum for different game scenes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneType {
    /// Test world scene (with Step 6.1 tiles system)
    TestWorld,
    /// Town preview scene (Step 5.3 integration)
    TownPreview,
}

/// Main game structure that holds all game state
pub struct Game {
    engine: Engine,
    event_pump: sdl2::EventPump,
    running: bool,
    world: World,
    player_index: usize,
    last_frame_time: Instant,
    
    // Step 4.2: Camera system
    camera: Camera,
    
    // Step 4.1: 键盘状态追踪（用于持续移动）
    key_up: bool,
    key_down: bool,
    key_left: bool,
    key_right: bool,
    
    // Step 5.3: Scene system and resource manager
    resource_manager: Option<ResourceManager>,
    town_scene: Option<SimpleTown>,
    current_scene: SceneType,
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Result<Self> {
        let mut engine = Engine::new()?;
        let event_pump = engine.sdl_context()
            .event_pump()
            .map_err(|e| anyhow::anyhow!("Failed to create event pump: {}", e))?;
        
        // 禁用文本输入模式，避免输入法拦截按键
        // Disable text input to prevent IME from intercepting key events
        engine.sdl_context().video()
            .map_err(|e| anyhow::anyhow!("Failed to get video subsystem: {}", e))?
            .text_input()
            .stop();
        
        // Load player sprite
        let player_sprite_path = AssetPaths::sprite("player.png");
        if let Err(e) = engine.load_texture("player", &player_sprite_path) {
            eprintln!("Warning: Failed to load player sprite: {}", e);
            eprintln!("Player will be rendered as a colored rectangle.");
        }
        
        // Load tile textures
        // Try loading each texture; if it fails, we'll just see a colored square fallback
        let tile_textures = vec![
            ("tile_floor", "tile_floor.png"),
            ("tile_wall", "tile_wall.png"),
            ("tile_grass", "tile_grass.png"),
            ("tile_water", "tile_water.png"),
        ];
        
        for (id, filename) in tile_textures {
            let path = AssetPaths::sprite(filename);
            // We don't panic on failure, just log warning
            if let Err(e) = engine.load_texture(id, &path) {
                eprintln!("Warning: Failed to load tile texture '{}' ({}): {}", id, filename, e);
            } else {
                println!("Loaded texture: {}", id);
            }
        }
        
        // Create world using Diablo 1 dimensions
        // Reference: Source/levels/gendung.cpp - DMAXX=40, DMAXY=40 (MegaTile scale)
        // For test world, we use orthogonal tiles (32px) matching the original test setup
        // Note: Actual dungeon uses isometric tiles (64x32px) but test world uses orthogonal
        let tile_size = 32;
        let map_width = 112;  // Match DMAXX from original code
        let map_height = 112; // Match DMAXY from original code
        let mut world = World::new(map_width, map_height, tile_size);
        
        // Create player at center
        let center_x = (map_width as u32 * tile_size) as i32 / 2;
        let center_y = (map_height as u32 * tile_size) as i32 / 2;
        let player = Entity::create_player(Point::new(center_x, center_y));
        world.add_entity(player);
        let player_index = 0;
        
        // Create camera
        let mut camera = Camera::new(640, 480);
        // Set camera bounds to map size
        camera.set_bounds(Rect::new(0, 0, map_width as u32 * tile_size, map_height as u32 * tile_size));
        
        // Step 5.1: MPQ and Palette System Integration Test
        println!("\n=== Step 5.1: MPQ and Palette System Test ===");
        
        // Initialize MPQ manager
        let mut mpq_manager = MpqManager::new();
        
        // Try to load DIABDAT.MPQ using stormlib-rs
        // Search in multiple locations (priority order)
        let mpq_paths = vec![
            "assets/Diabdat.mpq",  // First try assets directory (where user placed it)
            "assets/DIABDAT.MPQ",  // Uppercase version
            "Diabdat.mpq",         // Current directory
            "DIABDAT.MPQ",         // Uppercase in current directory
        ];
        
        let mut mpq_loaded = false;
        for path in &mpq_paths {
            match mpq_manager.load_mpq(path, 1000) {
                Ok(_) => {
                    println!("✓ Loaded MPQ archive: {} (priority: 1000)", path);
                    mpq_loaded = true;
                    break;
                }
                Err(e) => {
                    // Continue trying other paths
                    if path == &mpq_paths[mpq_paths.len() - 1] {
                        // Last path, print warning
                        eprintln!("⚠ Warning: Could not load MPQ from any path.");
                        eprintln!("  Last error: {}", e);
                        eprintln!("  Searched paths: {:?}", mpq_paths);
                        eprintln!("  Note: Please ensure Diabdat.mpq is in the assets/ directory.");
                    }
                }
            }
        }
        
        // Try to load a palette file
        let palette_paths = vec![
            "levels/towndata/town.pal",
            "levels\\towndata\\town.pal",  // Windows path separator
        ];
        
        let mut palette_loaded = false;
        let mut loaded_palette: Option<Palette> = None;
        for path in &palette_paths {
            if let Some(palette_data) = mpq_manager.find_file(path) {
                match Palette::from_bytes(&palette_data) {
                    Ok(palette) => {
                        println!("✓ Loaded palette: {}", path);
                        println!("  Palette size: {} colors", palette.len());
                        
                        // Print first 10 colors
                        println!("  First 10 colors:");
                        for i in 0..10.min(256) {
                            let color = palette.colors[i];
                            println!("    Color {}: RGB({}, {}, {})", i, color.r, color.g, color.b);
                        }
                        
                        // Test RGBA conversion
                        let test_rgba = palette.to_rgba(0, true);
                        println!("  Test: Index 0 with transparency -> RGBA({}, {}, {}, {})", 
                                 test_rgba[0], test_rgba[1], test_rgba[2], test_rgba[3]);
                        
                        palette_loaded = true;
                        loaded_palette = Some(palette);
                        break;
                    }
                    Err(e) => {
                        eprintln!("⚠ Failed to parse palette from {}: {}", path, e);
                    }
                }
            }
        }
        
        if !palette_loaded {
            if mpq_loaded {
                eprintln!("⚠ Warning: MPQ loaded but could not find palette file.");
                eprintln!("  Searched paths: {:?}", palette_paths);
            } else {
                eprintln!("⚠ Warning: Could not load palette file from MPQ.");
                eprintln!("  MPQ file was not loaded, so palette cannot be read.");
            }
        }
        
        // Print loaded archives
        let loaded_archives = mpq_manager.get_loaded_archives();
        if !loaded_archives.is_empty() {
            println!("  Loaded MPQ archives:");
            for archive in &loaded_archives {
                println!("    - {}", archive);
            }
        }
        
        println!("=== Step 5.1 Test Complete ===\n");
        
        // Step 5.2: PCX and CLX Integration
        println!("=== Step 5.2: PCX and CLX Integration ===");
        
        // Phase 4: Load PCX background (logo)
        if mpq_loaded {
            let pcx_paths = vec![
                "ui_art/logo.pcx",
                "ui_art\\logo.pcx",
            ];
            
            let mut pcx_loaded = false;
            for path in &pcx_paths {
                if let Some(pcx_data) = mpq_manager.find_file(path) {
                    match PcxImage::from_bytes(&pcx_data) {
                        Ok(pcx) => {
                            println!("✓ Loaded PCX: {}", path);
                            println!("  Size: {}x{}", pcx.width, pcx.height);
                            println!("  Has internal palette: {}", pcx.palette.is_some());
                            
                            // Use internal palette if available, otherwise use loaded palette
                            let palette_to_use = if let Some(ref internal_pal) = pcx.palette {
                                internal_pal
                            } else if let Some(ref external_pal) = loaded_palette {
                                external_pal
                            } else {
                                eprintln!("⚠ No palette available for PCX conversion");
                                continue;
                            };
                            
                            // Convert to RGBA (transparent index 0)
                            let rgba_data = pcx.to_rgba(palette_to_use, Some(0));
                            
                            // Load texture
                            match engine.load_texture_from_rgba("logo", &rgba_data, pcx.width as u32, pcx.height as u32) {
                                Ok(_) => {
                                    println!("✓ Created texture 'logo' from PCX");
                                    pcx_loaded = true;
                                }
                                Err(e) => {
                                    eprintln!("⚠ Failed to create texture from PCX: {}", e);
                                }
                            }
                            break;
                        }
                        Err(e) => {
                            eprintln!("⚠ Failed to parse PCX from {}: {}", path, e);
                        }
                    }
                }
            }
            
            if !pcx_loaded {
                eprintln!("⚠ Could not load PCX logo from any path");
                eprintln!("  Searched paths: {:?}", pcx_paths);
            }
        }
        
        // Phase 5: Load CLX/CL2 sprites (idle and walk animations)
        let mut idle_frame_count = 0;
        let mut walk_frame_count = 0;
        let mut clx_sprite_name = String::new();
        
        if mpq_loaded && loaded_palette.is_some() {
            // Load idle and walk animations
            let animation_sets = vec![
                // (idle_path, walk_path, sprite_name)
                ("plrgfx/warrior/wmn/wmnas.cl2", "plrgfx/warrior/wmn/wmnaw.cl2", "warrior"),
                ("plrgfx\\warrior\\wmn\\wmnas.cl2", "plrgfx\\warrior\\wmn\\wmnaw.cl2", "warrior"),
                ("plrgfx/warrior/wmd/wmdas.cl2", "plrgfx/warrior/wmd/wmdaw.cl2", "warrior"),
                ("plrgfx\\warrior\\wmd\\wmdas.cl2", "plrgfx\\warrior\\wmd\\wmdaw.cl2", "warrior"),
            ];
            
            let mut animations_loaded = false;
            let palette = loaded_palette.as_ref().unwrap();
            
            for (idle_path, walk_path, sprite_name) in &animation_sets {
                // Try to load idle animation
                let idle_data = mpq_manager.find_file(idle_path);
                let walk_data = mpq_manager.find_file(walk_path);
                
                if idle_data.is_none() {
                    continue;
                }
                
                // Load idle animation
                let frame_width = 96u16;
                match Cl2Sprite::from_bytes(&idle_data.unwrap(), frame_width) {
                    Ok(idle_sprite) => {
                        println!("✓ Loaded idle CL2: {}", idle_path);
                        println!("  Idle frames: {}", idle_sprite.frames.len());
                        
                        // Load idle frames as textures
                        for (i, frame) in idle_sprite.frames.iter().enumerate() {
                            let rgba_data = frame.to_rgba(palette);
                            let texture_id = format!("{}_idle_{}", sprite_name, i);
                            
                            if let Err(e) = engine.load_texture_from_rgba(&texture_id, &rgba_data, frame.width as u32, frame.height as u32) {
                                eprintln!("⚠ Failed to create idle texture {}: {}", i, e);
                            } else {
                                idle_frame_count += 1;
                            }
                        }
                        
                        // Try to load walk animation
                        if let Some(walk_data_bytes) = walk_data {
                            match Cl2Sprite::from_bytes(&walk_data_bytes, frame_width) {
                                Ok(walk_sprite) => {
                                    println!("✓ Loaded walk CL2: {}", walk_path);
                                    println!("  Walk frames: {}", walk_sprite.frames.len());
                                    
                                    // Load walk frames as textures
                                    for (i, frame) in walk_sprite.frames.iter().enumerate() {
                                        let rgba_data = frame.to_rgba(palette);
                                        let texture_id = format!("{}_walk_{}", sprite_name, i);
                                        
                                        if let Err(e) = engine.load_texture_from_rgba(&texture_id, &rgba_data, frame.width as u32, frame.height as u32) {
                                            eprintln!("⚠ Failed to create walk texture {}: {}", i, e);
                                        } else {
                                            walk_frame_count += 1;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("⚠ Failed to parse walk CL2: {}", e);
                                }
                            }
                        } else {
                            eprintln!("⚠ Walk animation not found: {}", walk_path);
                        }
                        
                        clx_sprite_name = sprite_name.to_string();
                        animations_loaded = true;
                        break;
                    }
                    Err(e) => {
                        eprintln!("⚠ Failed to parse idle CL2 from {}: {}", idle_path, e);
                    }
                }
            }
            
            if !animations_loaded {
                eprintln!("⚠ Could not load warrior animations");
            }
        }
        
        println!("=== Step 5.2 Integration Complete ===\n");
        
        // Phase 6: Update player sprite if animations were loaded
        let mut sprite_width = 96u32;
        let mut sprite_height = 96u32;
        
        if idle_frame_count > 0 && !clx_sprite_name.is_empty() {
            // Get the first frame dimensions
            if let Some(first_frame_texture) = engine.texture_manager().get(&format!("{}_idle_0", clx_sprite_name)) {
                sprite_width = first_frame_texture.width();
                sprite_height = first_frame_texture.height();
                println!("✓ Sprite size: {}x{}", sprite_width, sprite_height);
            }
            
            if let Some(player) = world.get_entity_mut(player_index) {
                println!("✓ Updating player to use warrior sprite: {}", clx_sprite_name);
                player.sprite_id = Some(clx_sprite_name.clone());
                player.size = (sprite_width, sprite_height);
                println!("  Player size set to: {}x{}", player.size.0, player.size.1);
                
                // Reconfigure animations
                if let Some(ref mut anim_controller) = player.animation {
                    // Idle animation
                    let idle_frames: Vec<Rect> = (0..idle_frame_count)
                        .map(|_| Rect::new(0, 0, 64, 64))
                        .collect();
                    let idle_anim = Animation::new(idle_frames, 0.15, true);
                    anim_controller.add_animation(AnimationState::Idle, idle_anim);
                    
                    // Walk animation
                    if walk_frame_count > 0 {
                        let walk_frames: Vec<Rect> = (0..walk_frame_count)
                            .map(|_| Rect::new(0, 0, 64, 64))
                            .collect();
                        let walk_anim = Animation::new(walk_frames, 0.1, true);
                        anim_controller.add_animation(AnimationState::Walk, walk_anim);
                        println!("  Configured walk animation with {} frames", walk_frame_count);
                    }
                    
                    anim_controller.set_state(AnimationState::Idle);
                    println!("  Configured idle animation with {} frames", idle_frame_count);
                }
            }
        }
        
        // Step 5.3: Initialize ResourceManager and load town scene
        println!("\n=== Step 5.3: ResourceManager and Town Scene ===");
        let mut resource_manager_opt = None;
        let mut town_scene_opt = None;
        
        // Try to create ResourceManager
        let mpq_paths = vec![
            ("assets/Diabdat.mpq", 1000),
            ("assets/DIABDAT.MPQ", 1000),
            ("Diabdat.mpq", 1000),
            ("DIABDAT.MPQ", 1000),
        ];
        
        let mut res_mgr = ResourceManager::new_empty();
        let loaded_mpqs = res_mgr.try_load_mpqs(mpq_paths);
        
        if !loaded_mpqs.is_empty() {
            println!("✓ ResourceManager initialized with MPQ: {:?}", loaded_mpqs[0]);
            
            // Try to load town resources
            let mut town_loaded = false;
            
            // Load town palette
            match res_mgr.load_palette("levels/towndata/town.pal") {
                Ok(palette) => {
                    println!("✓ Loaded town palette ({} colors)", palette.len());
                    
                    // Try to load background PCX
                    let bg_paths = vec![
                        "ui_art/logo.pcx",
                        "ui_art/smlogo.pcx",
                    ];
                    
                    for bg_path in bg_paths {
                        match res_mgr.load_pcx_texture(
                            &mut engine,
                            bg_path,
                            "town_bg",
                            None,
                            Some(0),
                        ) {
                            Ok(_) => {
                                println!("✓ Loaded town background: {}", bg_path);
                                
                                // Get background size (from logo.pcx: 550x3240)
                                let bg_width = 550;
                                let bg_height = 480; // Use screen height for now
                                
                                // Create SimpleTown with background
                                let mut town = SimpleTown::with_background(
                                    "town_bg".to_string(),
                                    bg_width,
                                    bg_height,
                                );
                                
                                // Set walkable area (slightly inset from edges)
                                town.set_walkable_area(Rect::new(32, 32, bg_width - 64, bg_height - 64));
                                
                                println!("✓ Created SimpleTown scene");
                                println!("  Size: {}x{}", bg_width, bg_height);
                                
                                town_scene_opt = Some(town);
                                town_loaded = true;
                                break;
                            }
                            Err(e) => {
                                // Try next path
                                println!("⚠ Could not load {}: {}", bg_path, e);
                            }
                        }
                    }
                    
                    // Load warrior sprites for town scene
                    let warrior_idle = res_mgr.load_sprite_textures(
                        &mut engine,
                        "plrgfx/warrior/wmn/wmnas.cl2",
                        "levels/towndata/town.pal",
                        "warrior_town",
                        "idle",
                        Some(96),
                    );
                    
                    let warrior_walk = res_mgr.load_sprite_textures(
                        &mut engine,
                        "plrgfx/warrior/wmn/wmnaw.cl2",
                        "levels/towndata/town.pal",
                        "warrior_town",
                        "walk",
                        Some(96),
                    );
                    
                    if warrior_idle.is_ok() && warrior_walk.is_ok() {
                        println!("✓ Loaded warrior sprites for town scene");
                        println!("  Idle frames: {}", warrior_idle.as_ref().unwrap().len());
                        println!("  Walk frames: {}", warrior_walk.as_ref().unwrap().len());
                    }
                    
                    if !town_loaded {
                        println!("⚠ Could not create town scene (no background)");
                    }
                }
                Err(e) => {
                    println!("⚠ Could not load town palette: {}", e);
                }
            }
            
            resource_manager_opt = Some(res_mgr);
        } else {
            println!("⚠ ResourceManager not initialized (no MPQ files found)");
        }
        
        println!("=== Step 5.3 Initialization Complete ===\n");
        
        // Step 6.1 & 6.2: Load tiles system with TileTextureManager
        println!("\n=== Step 6.1/6.2: Tiles System with TextureManager ===");
        
        let mut texture_manager_opt = None;
        
        if mpq_loaded {
            // Step 6.2: Load Town tileset
            println!("Loading Town tileset from MPQ...");
            
            use crate::tiles::texture_manager::TileTextureManager;
            use crate::tiles::DungeonType;
            
            match TileTextureManager::load_for_dungeon(DungeonType::Town, &mut mpq_manager) {
                Ok(tex_mgr) => {
                    println!("✓ TileTextureManager loaded successfully");
                    println!("  Total tiles: {}", tex_mgr.len());
                    println!("  Total frames: {}", tex_mgr.total_frames());
                    
                    // Load MIN/TIL/SOL data for World
                    use crate::tiles::{MinData, TilData, SolData};
                    
                    let min_result = MinData::load_for_dungeon(&mut mpq_manager, DungeonType::Town);
                    let til_result = TilData::load_for_dungeon(&mut mpq_manager, DungeonType::Town);
                    let sol_result = SolData::load_for_dungeon(&mut mpq_manager, DungeonType::Town);
                    
                    if let (Ok(min_data), Ok(til_data), Ok(sol_data)) = (min_result, til_result, sol_result) {
                        println!("✓ Loaded MIN: {} pieces", min_data.len());
                        println!("✓ Loaded TIL: {} mega tiles", til_data.len());
                        println!("✓ Loaded SOL: {} properties", sol_data.len());
                        
                        // Debug: Check piece 16 and 856 blocks
                        if let Some(piece) = min_data.pieces.get(16) {
                            println!("\n=== Piece 16 Blocks ===");
                            for (i, block) in piece.mt.iter().enumerate() {
                                println!("  block[{}]: data={:#06x} has_value={} frame={} type={:?}", 
                                    i, block.data, block.has_value(), block.frame(), block.tile_type());
                            }
                        }
                        
                        // Debug: Check piece 856 (the problematic one)
                        if let Some(piece) = min_data.pieces.get(856) {
                            println!("\n=== Piece 856 Blocks (PROBLEMATIC) ===");
                            for (i, block) in piece.mt.iter().enumerate() {
                                println!("  block[{}]: data={:#06x} has_value={} frame={} type={:?}", 
                                    i, block.data, block.has_value(), block.frame(), block.tile_type());
                            }
                        }
                        // Load all Town sectors directly to dPiece (like C++ FillSector)
                        // Town default piece is 218 (used for empty areas)
                        let default_piece = 218u16;
                        
                        // Load all 4 sectors to match C++ behavior
                        // Reference: Source/levels/town.cpp:202-205
                        // FillSector("levels\\towndata\\sector1s.dun", 46, 46);
                        // FillSector("levels\\towndata\\sector2s.dun", 46, 0);
                        // FillSector("levels\\towndata\\sector3s.dun", 0, 46);
                        // FillSector("levels\\towndata\\sector4s.dun", 0, 0);
                        
                        // Load first sector (creates dungeon_map and stores data)
                        if let Err(e) = world.load_town_sector(
                            &mut mpq_manager,
                            "levels\\towndata\\sector1s.dun",
                            min_data,
                            til_data,
                            sol_data,
                            Some(tex_mgr),
                            46,  // offset_x
                            46,  // offset_y
                            default_piece,
                        ) {
                            println!("⚠ Failed to load town sector1s: {}", e);
                        }
                        
                        // Load remaining sectors (append to existing dungeon_map)
                        let sectors = [
                            ("levels\\towndata\\sector2s.dun", 46, 0),   // Top-right
                            ("levels\\towndata\\sector3s.dun", 0, 46),   // Bottom-left
                            ("levels\\towndata\\sector4s.dun", 0, 0),    // Top-left
                        ];
                        
                        for (dun_path, offset_x, offset_y) in sectors.iter() {
                            if let Some(ref mut dungeon_map) = world.dungeon_map {
                                use crate::world::dungeon_map::load_sector_to_dpiece;
                                if let Some(ref til_data) = world.til_data {
                                    if let Err(e) = load_sector_to_dpiece(
                                        dungeon_map,
                                        &mut mpq_manager,
                                        dun_path,
                                        til_data,
                                        *offset_x,
                                        *offset_y,
                                        default_piece,
                                    ) {
                                        println!("⚠ Failed to load town sector {}: {}", dun_path, e);
                                    } else {
                                        println!("✓ Loaded Town sector: {}", dun_path);
                                    }
                                }
                            }
                        }
                        println!("✓ Town tiles integrated into World");
                        // Move player to a floor tile
                        if let Some(ref dm) = world.dungeon_map {
                            // Find a floor tile for player spawn
                            // Floor tile 13 -> TIL[12] -> pieces 22, 1, 6, 3
                            // Search from center of map area
                            'outer: for y in 30..80 {
                                for x in 30..80 {
                                    let piece = dm.d_piece[x][y];
                                    // Floor pieces from TIL[12]: 22, 1, 6, 3
                                    // Also check for common floor pieces
                                    if piece == 22 || piece == 1 || piece == 6 || piece == 3 {
                                        // Convert dPiece coords to world coords
                                        let world_x = ((x as i32 - 16) * 32) as i32;
                                        let world_y = ((y as i32 - 16) * 32) as i32;
                                        if let Some(player) = world.get_entity_mut(0) {
                                            player.position = crate::math::Point::new(world_x, world_y);
                                            println!("✓ Player moved to floor at d_piece[{},{}], world ({},{})", x, y, world_x, world_y);
                                        }
                                        break 'outer;
                                    }
                                }
                            }
                        }
                        
                        // Mark that we're using texture manager mode
                        texture_manager_opt = Some(true);
                    } else {
                        println!("⚠ Failed to load some tile data files");
                    }
                }
                Err(e) => {
                    println!("⚠ Could not load TileTextureManager: {}", e);
                    println!("  Falling back to PNG tiles...");
                }
            }
        }
        

        
        println!("=== Step 6.1/6.2 Initialization Complete ===\n");
        
        Ok(Self {
            engine,
            event_pump,
            running: true,
            world,
            player_index,
            last_frame_time: Instant::now(),
            camera,
            key_up: false,
            key_down: false,
            key_left: false,
            key_right: false,
            resource_manager: resource_manager_opt,
            town_scene: town_scene_opt,
            current_scene: SceneType::TestWorld, // Start with test world
        })
    }

    /// Run the main game loop
    /// 
    /// This is the core game loop that runs until the game is closed.
    /// It handles:
    /// - Event processing (input, window events)
    /// - Game logic updates
    /// - Rendering
    pub fn run(&mut self) -> Result<()> {
        while self.running {
            // Process events (input, window close, etc.)
            self.process_events()?;
            
            // Update game logic
            self.update()?;
            
            // Render the current frame
            self.render()?;
        }
        
        Ok(())
    }

    /// Process all pending events
    fn process_events(&mut self) -> Result<()> {
        // Collect events first to avoid borrow conflict
        let events: Vec<_> = self.event_pump.poll_iter().collect();
        
        for event in events {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    self.running = false;
                }
                sdl2::event::Event::KeyDown { keycode: Some(keycode), .. } => {
                    self.handle_keydown(keycode);
                }
                sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } => {
                    self.handle_keyup(keycode);
                }
                sdl2::event::Event::MouseMotion { x, y, .. } => {
                    self.handle_mouse_motion(x, y);
                }
                sdl2::event::Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    self.handle_mouse_down(mouse_btn, x, y);
                }
                sdl2::event::Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                    self.handle_mouse_up(mouse_btn, x, y);
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Handle keyboard key press
    fn handle_keydown(&mut self, keycode: sdl2::keyboard::Keycode) {
        match keycode {
            sdl2::keyboard::Keycode::Escape => {
                self.running = false;
            }
            sdl2::keyboard::Keycode::W | sdl2::keyboard::Keycode::Up => {
                self.key_up = true;
            }
            sdl2::keyboard::Keycode::S | sdl2::keyboard::Keycode::Down => {
                self.key_down = true;
            }
            sdl2::keyboard::Keycode::A | sdl2::keyboard::Keycode::Left => {
                self.key_left = true;
            }
            sdl2::keyboard::Keycode::D | sdl2::keyboard::Keycode::Right => {
                self.key_right = true;
            }
            // Step 5.3 & 6.1: Scene switching
            sdl2::keyboard::Keycode::F1 => {
                self.switch_scene(SceneType::TestWorld);
            }
            sdl2::keyboard::Keycode::F2 => {
                self.switch_scene(SceneType::TownPreview);
            }
            _ => {}
        }
    }

    /// Handle keyboard key release
    fn handle_keyup(&mut self, keycode: sdl2::keyboard::Keycode) {
        match keycode {
            sdl2::keyboard::Keycode::W | sdl2::keyboard::Keycode::Up => {
                self.key_up = false;
            }
            sdl2::keyboard::Keycode::S | sdl2::keyboard::Keycode::Down => {
                self.key_down = false;
            }
            sdl2::keyboard::Keycode::A | sdl2::keyboard::Keycode::Left => {
                self.key_left = false;
            }
            sdl2::keyboard::Keycode::D | sdl2::keyboard::Keycode::Right => {
                self.key_right = false;
            }
            _ => {}
        }
    }

    /// Handle mouse movement
    fn handle_mouse_motion(&mut self, _x: i32, _y: i32) {
        // Placeholder for future mouse handling
    }

    /// Handle mouse button press
    fn handle_mouse_down(&mut self, _button: sdl2::mouse::MouseButton, _x: i32, _y: i32) {
        // Placeholder for future mouse handling
    }

    /// Handle mouse button release
    fn handle_mouse_up(&mut self, _button: sdl2::mouse::MouseButton, _x: i32, _y: i32) {
        // Placeholder for future mouse handling
    }

    /// Update game logic
    /// 
    /// This is where all game state updates happen:
    /// - Player movement
    /// - Monster AI
    /// - Item interactions
    /// - etc.
    fn update(&mut self) -> Result<()> {
        // Calculate delta time
        let current_time = Instant::now();
        let dt = current_time.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;
        
        // Step 4.1: 计算玩家移动方向
        let mut move_x = 0.0;
        let mut move_y = 0.0;
        
        if self.key_up {
            move_y -= 1.0;
        }
        if self.key_down {
            move_y += 1.0;
        }
        if self.key_left {
            move_x -= 1.0;
        }
        if self.key_right {
            move_x += 1.0;
        }
        
        // 更新玩家方向和动画
        if let Some(player) = self.world.get_entity_mut(self.player_index) {
            let direction = Direction::from_velocity(move_x, move_y);
            
            player.set_direction(direction);
            
            // 根据方向切换动画状态
            let new_state = if direction.is_moving() {
                AnimationState::Walk
            } else {
                AnimationState::Idle
            };
            
            if let Some(ref mut anim) = player.animation {
                let old_state = anim.current_state();
                anim.set_state(new_state);
                
                // DEBUG: 输出状态切换
                if old_state != new_state {
                    println!("\x1b[33m>>> Animation State Changed: {:?} -> {:?}\x1b[0m", old_state, new_state);
                }
            }
            
            // Note: Animation frames are now handled by the sprite sheet
            // No need to change color/size based on animation state
        }
        
        // Update world (includes physics and collision)
        match self.current_scene {
            SceneType::TestWorld => {
                // Use normal world update with tile collision
                self.world.update(dt);
            }
            SceneType::TownPreview => {
                // For town scene, we handle collision manually
                // First, save player position
                let player_pos_before = if let Some(player) = self.world.get_entity_mut(self.player_index) {
                    player.position
                } else {
                    Point::new(0, 0)
                };
                
                // Update world physics
                self.world.update(dt);
                
                // Apply town-specific collision
                if let Some(ref town) = self.town_scene {
                    if let Some(player) = self.world.get_entity_mut(self.player_index) {
                        let new_x = player.position.x as f32;
                        let new_y = player.position.y as f32;
                        
                        // Check if new position is walkable
                        if !town.is_walkable(new_x, new_y) {
                            // Revert to previous position
                            player.position = player_pos_before;
                            player.velocity = Point::new(0, 0);
                        }
                    }
                }
            }
        }
        
        // Update camera to follow player
        let player_pos = self.world.get_entity_mut(self.player_index)
            .map(|player| player.position);
        
        if let Some(pos) = player_pos {
            match self.current_scene {
                SceneType::TestWorld => {
                    // Camera follows player in test world
                    self.camera.follow(pos);
                }
                SceneType::TownPreview => {
                    // In town, player moves freely without camera following
                    // (background is fixed for now)
                }
            }
            
            // DEBUG: Camera info (only when moving)
            if move_x != 0.0 || move_y != 0.0 {
                // println!("\n=== Camera Update ===");
                // println!("Player Pos: {:?}", pos);
                // println!("Camera Pos: {:?}", self.camera.position);
                // println!("Visible: {}", self.camera.is_visible(pos));
            }
        }
        
        Ok(())
    }

    /// Render the current frame
    /// 
    /// This is where all rendering happens:
    /// - Clear the screen
    /// - Draw game world or scene
    /// - Draw UI
    /// - Present to screen
    fn render(&mut self) -> Result<()> {
        self.engine.clear()?;
        
        // Render based on current scene
        match self.current_scene {
            SceneType::TestWorld => {
                // Render the world with Cathedral tiles system (Step 6.1)
                self.world.render(&mut self.engine, &self.camera)?;
            }
            SceneType::TownPreview => {
                // Render the town scene
                self.render_town()?;
            }
        }
        
        self.engine.present();
        Ok(())
    }
    
    /// Switch to a different scene
    /// 
    /// This method switches between different game scenes (TestWorld, TownPreview)
    fn switch_scene(&mut self, scene: SceneType) {
        if self.current_scene != scene {
            println!("Switching scene: {:?} -> {:?}", self.current_scene, scene);
            self.current_scene = scene;
            
            // Reset player velocity when switching scenes
            if let Some(player) = self.world.get_entity_mut(self.player_index) {
                player.velocity = Point::new(0, 0);
            }
        }
    }
    
    /// Render the town scene
    /// 
    /// This renders the town preview scene (Step 5.3)
    fn render_town(&mut self) -> Result<()> {
        if let Some(ref town) = self.town_scene {
            // Render town background if available
            if let Some(bg_id) = town.get_background_texture_id() {
                let bg_rect = Rect::new(
                    (640 - town.background_width as i32) / 2,
                    (480 - town.background_height as i32) / 2,
                    town.background_width,
                    town.background_height,
                );
                
                let _ = self.engine.draw_texture_by_id(bg_id, None, bg_rect);
            }
            
            // Render player in town
            if let Some(player) = self.world.get_entity_mut(self.player_index) {
                // Determine animation state
                let is_moving = player.velocity.x != 0 || player.velocity.y != 0;
                let sprite_base = "warrior_town";
                let state = if is_moving { "walk" } else { "idle" };
                
                // Get texture IDs for current animation
                if let Some(ref res_mgr) = self.resource_manager {
                    if let Some(texture_ids) = res_mgr.get_texture_ids(sprite_base, state) {
                        // Get current frame index from animation
                        let frame_index = if let Some(ref anim) = player.animation {
                            if let Some(idx) = anim.current_frame_index() {
                                idx % texture_ids.len()
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        
                        let texture_id = &texture_ids[frame_index];
                        
                        // Calculate player position on screen (centered)
                        let player_screen_x = player.position.x - (player.size.0 as i32 / 2);
                        let player_screen_y = player.position.y - (player.size.1 as i32 / 2);
                        
                        let player_rect = Rect::new(
                            player_screen_x,
                            player_screen_y,
                            player.size.0,
                            player.size.1,
                        );
                        
                        let _ = self.engine.draw_texture_by_id(texture_id, None, player_rect);
                    }
                }
            }
        } else {
            // No town scene available, show message
            // TODO: Render text message
        }
        
        Ok(())
    }
}
