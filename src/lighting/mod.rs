pub mod crawl;
/// Lighting system module
///
/// Implements Diablo 1's lighting and vision system.
/// Reference: Source/lighting.cpp
pub mod light_source;
pub mod light_table;

pub use crawl::crawl_light;
pub use light_source::{LightSource, LightType};
pub use light_table::LightTables;

use crate::resources::Palette;

/// Maximum dungeon size in MicroTiles
pub const MAXDUNX: usize = 112;
pub const MAXDUNY: usize = 112;

/// Lighting system - manages scene lighting
///
/// Reference: Source/lighting.cpp::DoLighting()
pub struct LightingSystem {
    /// Light intensity grid (MicroTile level)
    /// Values: 0 (complete darkness) - 15 (full brightness)
    pub light_grid: [[u8; MAXDUNY]; MAXDUNX],

    /// List of light sources
    pub light_sources: Vec<LightSource>,

    /// Color transformation tables (16 brightness levels × 256 colors)
    pub light_tables: LightTables,

    /// Ambient light level (0-15)
    pub ambient_light: u8,

    /// Next light ID counter
    next_light_id: usize,
}

impl LightingSystem {
    /// Create new lighting system
    ///
    /// # Arguments
    /// * `palette` - Game palette for generating light tables
    pub fn new(palette: &Palette) -> Self {
        Self {
            light_grid: [[0u8; MAXDUNY]; MAXDUNX],
            light_sources: Vec::new(),
            light_tables: LightTables::from_palette(palette),
            ambient_light: 3, // Default ambient light
            next_light_id: 1,
        }
    }

    /// Add a light source
    ///
    /// # Returns
    /// Light source ID for future reference
    pub fn add_light(&mut self, mut source: LightSource) -> usize {
        source.id = self.next_light_id;
        self.next_light_id += 1;
        let id = source.id;
        self.light_sources.push(source);
        id
    }

    /// Remove a light source by ID
    pub fn remove_light(&mut self, id: usize) {
        self.light_sources.retain(|s| s.id != id);
    }

    /// Update light source position
    pub fn update_light_position(&mut self, id: usize, new_pos: (usize, usize)) {
        if let Some(source) = self.light_sources.iter_mut().find(|s| s.id == id) {
            source.position = new_pos;
        }
    }

    /// Get light level at position
    ///
    /// # Returns
    /// Light level (0-15), at least ambient_light
    pub fn get_light_level(&self, x: usize, y: usize) -> u8 {
        if x < MAXDUNX && y < MAXDUNY {
            self.light_grid[x][y].max(self.ambient_light)
        } else {
            self.ambient_light
        }
    }

    /// Apply lighting to color index
    ///
    /// # Arguments
    /// * `color_index` - Original palette index
    /// * `light_level` - Light level (0-15)
    ///
    /// # Returns
    /// Darkened palette index
    pub fn apply_lighting(&self, color_index: u8, light_level: u8) -> u8 {
        self.light_tables.get(light_level, color_index)
    }

    /// Update lighting system
    ///
    /// Calculates light propagation for all active light sources.
    ///
    /// # Arguments
    /// * `block_map` - Wall blocking map
    pub fn update(&mut self, block_map: &[[bool; MAXDUNY]; MAXDUNX]) {
        // Clear light grid
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                self.light_grid[x][y] = 0;
            }
        }

        // Calculate lighting for all active sources
        for source in &self.light_sources {
            if source.active {
                crawl_light(&mut self.light_grid, source, block_map);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighting_system_creation() {
        // Create test palette
        let palette = create_test_palette();
        let lighting = LightingSystem::new(&palette);

        assert_eq!(lighting.light_sources.len(), 0);
        assert_eq!(lighting.ambient_light, 3);
        assert_eq!(lighting.next_light_id, 1);

        // Light grid should be all zeros
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                assert_eq!(lighting.light_grid[x][y], 0);
            }
        }
    }

    #[test]
    fn test_add_remove_light() {
        let palette = create_test_palette();
        let mut lighting = LightingSystem::new(&palette);

        // Add lights
        let id1 = lighting.add_light(LightSource::player((50, 50)));
        assert_eq!(lighting.light_sources.len(), 1);
        assert_eq!(id1, 1);

        let id2 = lighting.add_light(LightSource::torch((60, 60)));
        assert_eq!(lighting.light_sources.len(), 2);
        assert_eq!(id2, 2);

        // Remove lights
        lighting.remove_light(id1);
        assert_eq!(lighting.light_sources.len(), 1);

        lighting.remove_light(id2);
        assert_eq!(lighting.light_sources.len(), 0);
    }

    #[test]
    fn test_update_light_position() {
        let palette = create_test_palette();
        let mut lighting = LightingSystem::new(&palette);

        let id = lighting.add_light(LightSource::player((50, 50)));

        // Update position
        lighting.update_light_position(id, (60, 60));

        let source = lighting.light_sources.iter().find(|s| s.id == id).unwrap();

        assert_eq!(source.position, (60, 60));
    }

    #[test]
    fn test_get_light_level() {
        let palette = create_test_palette();
        let mut lighting = LightingSystem::new(&palette);

        lighting.ambient_light = 5;

        // Unlit area should return ambient light
        assert_eq!(lighting.get_light_level(0, 0), 5);

        // Manually set lighting
        lighting.light_grid[10][10] = 12;
        assert_eq!(lighting.get_light_level(10, 10), 12);

        // Out of bounds returns ambient
        assert_eq!(lighting.get_light_level(MAXDUNX, MAXDUNY), 5);
    }

    // Helper function to create test palette
    fn create_test_palette() -> Palette {
        use crate::resources::palette::Color;

        let mut palette = Palette::new();
        // Initialize with gradient colors
        for i in 0..256 {
            let val = i as u8;
            palette.colors[i] = Color::new(val, val, val);
        }
        palette
    }
}
