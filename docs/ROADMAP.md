# Roadmap

Rust Diablo is developed as a sequence of small engine milestones. Each milestone should leave the project buildable, documented, and easier to test.

## Completed

| Step | Area | Result |
| --- | --- | --- |
| 1-3 | Foundation | Game loop, rendering basics, entities, sprites |
| 4 | Animation and collision | Frame animation, movement, collision checks |
| 5.1 | MPQ and palette | MPQ access, palette loading, decompression support |
| 5.2 | PCX, CLX, CL2 | Image and sprite format parsing |
| 5.3 | TRN and town scene | Color transforms, resource manager, town preview |
| 6.1 | Tile foundations | MIN, TIL, SOL loading and tile data structures |
| 6.2 | Tile decoding | Tile type decoders and texture management |
| 6.3 | Wall and town rendering | CEL support, two-pass floor/wall rendering |

## In Progress

### Step 6.4: Lighting And Rendering Refinement

- Lighting tables and light sources.
- Mask-aware rendering improvements.
- Wall-behind and transparency behavior.
- Better validation around town and dungeon rendering.

Primary docs:

- [Step 6.4 phase split](step-6.4/step-6.4-phase-split.md)
- [Step 6.4 implementation plan](step-6.4/step-6.4-implementation-plan.md)
- [Step 6.4 lighting and player animation](step-6.4/step-6.4-lighting-and-player-animation.md)

## Next

### Step 7: Player Movement And 8-Direction Animation

- Direction-aware movement.
- Directional animation selection.
- Better animation-state integration.
- Tests for movement, direction mapping, and asset-backed animation behavior.

Primary docs:

- [Step 7 requirements](step-7/step-7-player-movement-8direction-animation-requirements.md)
- [Step 7 design](step-7/step-7-player-movement-8direction-animation-design.md)
- [Step 7 test plan](step-7/step-7-player-movement-8direction-animation-test-plan.md)

## Later Milestones

- Dungeon room generation and level transitions.
- Monster spawning and basic AI.
- Combat and damage systems.
- Item loading, drops, and inventory.
- Spell and missile systems.
- Save/load support.
- UI and automap.
- Optional networking and scripting work after the single-player prototype is stable.

## Maintenance Priorities

- Keep proprietary assets out of the repository.
- Separate pure parser tests from asset-backed integration tests.
- Make malformed binary inputs fail with clear errors instead of panics.
- Expand rendering validation with deterministic fixtures.
- Improve documentation indexes so new contributors can find the current step quickly.

