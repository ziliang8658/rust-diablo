# Rust Diablo

Rust Diablo is a learning-oriented Rust rewrite of selected Diablo 1 engine systems. The project focuses on resource loading, binary format parsing, isometric rendering, animation, collision, and long-term engine architecture.

This repository contains code, tests, tools, and development notes. It does not include or redistribute Blizzard Entertainment game assets.

## Project Goals

- Rebuild Diablo 1 engine concepts in idiomatic Rust.
- Document each major step so the project can be used as a learning resource.
- Keep the implementation testable, readable, and easy to compare with known engine behavior.
- Explore safe Rust boundaries around legacy file formats, decompression, FFI, and SDL2 rendering.

## Current Status

The project has implemented the foundation needed to load and render original-style assets:

| Area | Status | Notes |
| --- | --- | --- |
| Core framework | Done | Game loop, SDL2 windowing, renderer structure, entity basics |
| Animation and collision | Done | Frame animation, movement, collision checks |
| Resource formats | Done | MPQ, palette, PCX, CLX, CL2, CEL, TRN |
| Compression | Done | PKWare, Zlib, Huffman-related support |
| Tile system | Done | MIN, TIL, SOL, tile types, texture management |
| Town rendering | Done | Two-pass floor/wall rendering with CEL/CLX support |
| Lighting | In progress | Step 6.4 planning and partial implementation |
| Dungeon generation | Planned | Room generation and gameplay integration |

The current milestone is Step 6.4: lighting and player animation improvements. See [docs/ROADMAP.md](docs/ROADMAP.md) for the longer roadmap.

## Technical Highlights

- Native MPQ archive reader with hash/block table handling.
- Palette and TRN color transformation support.
- PCX, CLX, CL2, and CEL sprite/image format parsers.
- MIN/TIL/SOL tile data loading and tile type decoding.
- SDL2 rendering pipeline for floor, wall, transparency, and town-scene rendering.
- Test utilities and visual validation samples for decoded tiles and rendering behavior.
- Optional libmpq FFI path for comparison and validation work.

## Repository Layout

```text
rust-diablo/
├── src/
│   ├── game.rs              # Game loop and high-level orchestration
│   ├── resources/           # MPQ, palette, PCX, CLX, CL2, CEL, TRN loaders
│   ├── tiles/               # MIN/TIL/SOL loading and tile decoders
│   ├── world/               # Town and dungeon rendering logic
│   ├── renderer/            # Direct framebuffer and camera helpers
│   ├── sprite/              # Animation and sprite abstractions
│   └── lighting/            # Lighting system work in progress
├── tests/                   # Unit and integration tests
├── examples/                # Debugging and extraction utilities
├── tools/                   # Development comparison helpers
├── docs/                    # Design notes, implementation summaries, debugging guides
└── assets/                  # Local-only asset location; commercial assets are not tracked
```

## Legal Asset Boundary

You need a legally owned copy of Diablo 1 to run the parts of the project that load original assets. Place `Diabdat.mpq` in `assets/` locally:

```text
rust-diablo/
└── assets/
    └── Diabdat.mpq
```

Do not commit `Diabdat.mpq`, extracted Blizzard assets, screenshots containing copyrighted asset dumps, or other proprietary game data. This repository is for engine research, compatibility study, and Rust learning.

## Requirements

- Rust 1.70 or newer
- SDL2 development libraries
- SDL2_image development libraries
- A local `Diabdat.mpq` file for asset-backed examples and integration tests

On Windows, install SDL2/SDL2_image through your preferred toolchain setup. On Linux, packages are typically named `libsdl2-dev` and `libsdl2-image-dev`.

## Build And Test

```powershell
cargo build
cargo check
```

Useful targeted tests that do not require proprietary assets:

```powershell
cargo test --lib engine::direction
cargo test --lib engine::isometric
cargo test --lib tiles::decoder
cargo test --lib tiles::types
```

Run the main demo:

```powershell
cargo run
```

Some examples and tests expect local game assets and will not work without `assets/Diabdat.mpq`. The full test suite is still being split into pure unit tests, asset-backed integration tests, and work-in-progress regression tests.

## Documentation

- [Roadmap](docs/ROADMAP.md)
- [Step 5 resource formats](docs/step-5/step-5-resource-formats.md)
- [Step 6 implementation plan](docs/step-6/step-6-implementation-plan.md)
- [Step 6.3 completion summary](docs/step-6.3/step-6.3-completion-summary.md)
- [Step 6.4 lighting and player animation](docs/step-6.4/step-6.4-lighting-and-player-animation.md)
- [Step 7 player animation design](docs/step-7/step-7-player-movement-8direction-animation-design.md)

The `docs/` directory intentionally contains detailed debugging notes and implementation logs. They are part of the learning record, not polished end-user documentation.

## Roadmap

Near-term work:

- Finish Step 6.4 lighting integration.
- Improve player animation direction handling.
- Separate asset-dependent tests from pure parser/unit tests.
- Add more deterministic rendering validation.
- Clean up documentation indexes and contributor onboarding.

Longer-term work:

- Dungeon room generation and level transitions.
- Monster, combat, item, spell, and UI systems.
- Save/load support.
- Optional networking and scripting experiments after the single-player prototype is stable.

## Contributing

Contributions are welcome, especially in parser tests, documentation cleanup, rendering validation, and small Rust refactors. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

Important rule: do not include proprietary Diablo assets in issues, pull requests, tests, fixtures, or screenshots unless you have verified they are safe to publish.

## License

The project code is licensed under the MIT License. See [LICENSE](LICENSE).

Diablo and related game assets are owned by Blizzard Entertainment. This repository does not grant any rights to those assets.
