mod assets;
mod debug;
#[cfg(any(debug_assertions, feature = "devtools"))]
mod devtools;
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
use tracing_subscriber::EnvFilter;

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,rust_diablo=info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

/// Run the game (exported for examples)
pub fn run_game() -> Result<()> {
    let mut game = Game::new()?;
    game.run()?;
    Ok(())
}

fn main() -> Result<()> {
    init_logging();

    #[cfg(any(debug_assertions, feature = "devtools"))]
    devtools::print_startup_help();

    run_game()
}
