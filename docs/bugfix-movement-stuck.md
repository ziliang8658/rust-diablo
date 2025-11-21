# Bug Fix: Player Stuck on Walkable Tiles

## Issue Description
**Date:** 2025-11-20
**Symptom:** The player character would not move despite input being registered and the target tile being walkable (Floor).
**Context:** This issue appeared after implementing the collision detection system in Step 4.2.

## Root Cause Analysis
The issue was caused by **sub-pixel movement loss** due to integer truncation in the game loop.

1.  **Sub-pixel Movement:** At high frame rates (e.g., 60+ FPS) or low movement speeds, the calculated movement per frame (`speed * dt`) is often less than 1.0 pixel (e.g., `200 px/s * 0.001s = 0.2 px`).
2.  **Integer Truncation:** The game's position logic calculates the new *integer* position (`new_pos`) based on the float velocity.
    ```rust
    // Before fix
    let new_pos = Point::new(
        new_pos_f.0.round() as i32,
        new_pos_f.1.round() as i32
    );
    ```
3.  **Conditional Update Logic:** The `Entity::update` method only updated the accumulated float position (`position_f`) *if* the integer position (`new_pos`) changed.
    ```rust
    // Buggy logic
    if new_pos != self.position {
        // ... validate and update ...
    }
    // MISSING: else branch to update float position when integer position doesn't change
    ```
4.  **Result:** If the movement was small (< 0.5 px), `new_pos` would equal `self.position`. The float accumulator `position_f` was never updated, effectively discarding the fractional movement every frame. The player remained stuck indefinitely.

## The Fix
We modified `src/entity/mod.rs` to ensure `position_f` is updated every frame, regardless of whether the integer position changes.

```rust
// Fixed logic in src/entity/mod.rs
if let Some((map, tile_size)) = collision_info {
    if new_pos != self.position {
        // ... collision validation ...
    } else {
        // CRITICAL FIX: Even if integer position didn't change,
        // we MUST update the float position to accumulate sub-pixel movement.
        self.position_f = new_pos_f;
    }
}
```

## Verification
*   **Logic Check:** sub-pixel movements now correctly accumulate in `position_f`. When they eventually exceed the rounding threshold, `new_pos` changes, triggering the actual movement.
*   **Testing:** Verified that player moves smoothly even at high frame rates.

## Compilation Fix
During debugging, a compilation error (`cannot find value 'direction'`) was introduced in `src/game.rs` due to a debug print being placed outside the variable's scope. This was identified and removed/fixed.

