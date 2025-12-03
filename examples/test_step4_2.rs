use anyhow::Result;
use rust_diablo::engine::{Direction, Engine};
use rust_diablo::entity::{Entity, EntityType};
use rust_diablo::math::{Point, Rect};
use rust_diablo::renderer::{Camera, Color};
use rust_diablo::world::{TileType, World};
use std::time::Instant;

fn main() -> Result<()> {
    let mut engine = Engine::new()?;

    // Create a larger world (40x30)
    let tile_size = 32;
    let mut world = World::new(40, 30, tile_size);

    // Add some obstacles (walls) in the middle
    // Accessing collision map directly might be hard if fields are private?
    // World fields are public. CollisionMap fields are public.
    // But CollisionMap::update_tiles takes Vec<Vec<TileType>>.
    // We can modify world generation or just test with default walls (borders).

    // Let's try to modify tiles if possible, or just use the default border walls.
    // World::new creates border walls.

    // Add a player
    let mut player = Entity::create_player(Point::new(100, 100));
    player.speed = 300.0; // Fast player
    world.add_entity(player);

    // Setup camera
    let mut camera = Camera::new(640, 480);
    camera.set_bounds(Rect::new(0, 0, 40 * tile_size, 30 * tile_size));

    let mut event_pump = engine
        .sdl_context()
        .event_pump()
        .map_err(|e| anyhow::anyhow!("Failed to create event pump: {}", e))?;
    let mut running = true;
    let mut last_time = Instant::now();

    println!("Use Arrow Keys to move. Escape to quit.");
    println!("Test: 1. Try to move outside screen (Camera should follow)");
    println!("Test: 2. Try to hit walls (Collision should stop you)");

    while running {
        let dt = last_time.elapsed().as_secs_f32();
        last_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                _ => {}
            }
        }

        // Simple input handling
        let keys: Vec<sdl2::keyboard::Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(sdl2::keyboard::Keycode::from_scancode)
            .collect();

        let mut move_x = 0.0;
        let mut move_y = 0.0;

        if keys.contains(&sdl2::keyboard::Keycode::Up) {
            move_y -= 1.0;
        }
        if keys.contains(&sdl2::keyboard::Keycode::Down) {
            move_y += 1.0;
        }
        if keys.contains(&sdl2::keyboard::Keycode::Left) {
            move_x -= 1.0;
        }
        if keys.contains(&sdl2::keyboard::Keycode::Right) {
            move_x += 1.0;
        }

        // Update player direction manually since we don't use Game struct
        if let Some(player) = world.get_entity_mut(0) {
            player.set_direction(Direction::from_velocity(move_x, move_y));
        }

        // Update world (handles movement and collision)
        world.update(dt);

        // Update camera
        if let Some(player) = world.get_entity_mut(0) {
            camera.follow(player.position);
        }

        // Render
        engine.clear()?;
        world.render(&mut engine, &camera)?;
        engine.present();
    }

    Ok(())
}
