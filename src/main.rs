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

use anyhow::Result;
use game::Game;

/// Run the game (exported for examples)
pub fn run_game() -> Result<()> {
    let mut game = Game::new()?;
    game.run()?;
    Ok(())
}

fn main() -> Result<()> {
    run_game()
}

