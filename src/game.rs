/// Game module - Core game loop and state management
/// 
/// This module contains the main game loop and game state management.
/// It's the central coordinator for all game systems.

use anyhow::Result;
use crate::engine::Engine;
use crate::world::World;
use crate::entity::Entity;
use crate::math::{Point, Rect};
use crate::assets::AssetPaths;
use crate::engine::Direction;
use crate::sprite::AnimationState;
use std::time::Instant;
use crate::renderer::Camera;

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
        
        // Create world (20x15 tiles, 32 pixels per tile = 640x480)
        let tile_size = 32;
        let map_width = 60;  // Increased map size for exploration
        let map_height = 40;
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
                    // println!("\x1b[33m>>> Animation State Changed: {:?} -> {:?}\x1b[0m", old_state, new_state);
                }
            }
            
            // Note: Animation frames are now handled by the sprite sheet
            // No need to change color/size based on animation state
        }
        
        // Update world (includes physics and collision)
        self.world.update(dt);
        
        // Update camera to follow player
        if let Some(player) = self.world.get_entity_mut(self.player_index) {
            self.camera.follow(player.position);
            
            // DEBUG: Camera info (only when moving)
            if move_x != 0.0 || move_y != 0.0 {
                // println!("\n=== Camera Update ===");
                // println!("Player Pos: {:?}", player.position);
                // println!("Camera Pos: {:?}", self.camera.position);
                // println!("Visible: {}", self.camera.is_visible(player.position));
            }
        }
        
        Ok(())
    }

    /// Render the current frame
    /// 
    /// This is where all rendering happens:
    /// - Clear the screen
    /// - Draw game world
    /// - Draw UI
    /// - Present to screen
    fn render(&mut self) -> Result<()> {
        self.engine.clear()?;
        
        // Render the world (tiles and entities) using camera
        self.world.render(&mut self.engine, &self.camera)?;
        
        self.engine.present();
        Ok(())
    }
}
