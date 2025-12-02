/// Rust Diablo - A Rust rewrite of Diablo 1
/// 
/// This is a learning project to rewrite the Diablo 1 game engine in Rust.
/// We'll build it incrementally, starting with the most basic framework.

mod game;
mod engine;
mod math;
mod renderer;
mod entity;
mod world;
mod sprite;
mod assets;
mod resources;
mod tiles;  // Step 6.1: Tiles system
mod levels; // Step 6.3: Dungeon generation

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
    println!("===============================================\n");
    
    run_game()
}

