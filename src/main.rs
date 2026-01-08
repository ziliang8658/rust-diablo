mod assets;
mod debug;
mod engine;
mod entity;
/// Rust Diablo - A Rust rewrite of Diablo 1
///
/// This is a learning project to rewrite the Diablo 1 game engine in Rust.
/// We'll build it incrementally, starting with the most basic framework.
mod game;
mod levels;
mod lighting; // Step 6.4.1: Lighting system
mod math;
mod renderer;
mod resources;
mod sprite;
mod tiles; // Step 6.1: Tiles system
mod world; // Step 6.3: Dungeon generation

use anyhow::Result;
use game::Game;

/// Run the game (exported for examples)
pub fn run_game() -> Result<()> {
    let mut game = Game::new()?;
    game.run()?;
    Ok(())
}

fn main() -> Result<()> {
    println!("=== Rust Diablo - Step 6.1: Tiles System Demo ===");
    println!("Controls:");
    println!("  WASD / Arrow Keys - Move");
    println!("  ESC - Quit");
    println!("  F1 - Cathedral Dungeon (Step 6.1 Tiles System)");
    println!("  F2 - Town Preview (Step 5.3)");
    println!("\nRendering Debug (for troubleshooting):");
    println!("  F4 - Toggle Floor Layer");
    println!("  F5 - Toggle Wall Layer");
    println!("  F6 - Toggle Entity Layer");
    println!("  F7 - Toggle Debug Info");
    println!("  F8 - Reset All Layers (All Enabled)");
    println!("  F9 - Floor Only Mode (for debugging)");
    println!("\nNote: Default mode is FLOOR ONLY for easier debugging.");
    println!("      Press F8 to enable all layers for normal gameplay.");
    println!("===============================================\n");

    run_game()
}
