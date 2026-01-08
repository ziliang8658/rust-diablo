# Tile Indexed Pixel Reordering Requirements (Temporary Render Fix)

Rust currently renders dungeon micro-tiles by decoding each `TileType` to a padded row-major indexed buffer (`width=32`, `height=31/32`) and then uploading it as a 2D texture (`render_micro_tile()`).

DevilutionX C++ does **not** treat triangle/trapezoid sources as 32-byte-padded rows. Instead, it consumes *packed* variable-width rows and applies per-row horizontal positioning in the renderer (e.g. `RenderLeftTriangleLower`, `RenderRightTriangleLower`, `RenderTrapezoidUpperHalf`).

Because of this, some `TileType` decoders in Rust produce correct pixels but with a different per-row horizontal alignment than what the C++ renderer expects. The current workaround is to do a small per-row horizontal reordering in the render path.

This document records what needs to be reordered so it can later be moved into the decode stage.

## Where The Fix Lives Today

- `rust-diablo/src/world/mod.rs:892` (`World::render_micro_tile`)

## Row Order Convention

- Row 0 is the **bottom** row.
- Triangle lower half is rows `0..=15`.
- Triangle upper half is rows `16..=30`.
- Trapezoid lower (triangle) part is rows `0..=15`; trapezoid upper (rectangle) part is rows `16..=31`.

## Required Reorderings

### `LeftTriangle` (32×31)

Goal: match the C++ triangle rendering layout.

- Rows `0..=15`: right-align `width = 2*(row+1)` pixels within the 32-wide row.
- Rows `16..=30`: shift pixels right by `offset = 2*(row-15)` (i.e. keep the same pixel count but move it right).

### `RightTriangle` (32×31)

Goal: match the C++ triangle rendering layout.

- Rows `0..=30`: left-align `width` pixels within the 32-wide row:
  - `width = 2*(row+1)` for rows `0..=15`
  - `width = 32 - 2*(row-15)` for rows `16..=30`

### `LeftTrapezoid` (32×32)

C++ renders the **lower half** using `RenderLeftTriangleLower(...)` and the **upper half** using `RenderTrapezoidUpperHalf(...)`:

- Rows `0..=15` (triangle part): right-align `width = 2*(row+1)` pixels within the 32-wide row.
- Rows `16..=31` (rectangle part): no reordering.

### `RightTrapezoid` (32×32)

C++ renders the **lower half** using `RenderRightTriangleLower(...)` and the **upper half** using `RenderTrapezoidUpperHalf(...)`:

- Rows `0..=15` (triangle part): left-align `width = 2*(row+1)` pixels within the 32-wide row.
- Rows `16..=31` (rectangle part): no reordering.

## Future Work (Preferred)

Move all of the above per-row horizontal alignment to the decode phase so that:

- `decode_tile()` produces buffers already matching the C++ renderer’s expected layout.
- `render_micro_tile()` becomes a simple “decode → palette → upload” step with no tile-type-specific pixel shuffling.

