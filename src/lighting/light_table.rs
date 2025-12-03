/// Light color transformation tables
///
/// Reference: Source/lighting.cpp::MakeLightTable()
use crate::resources::Palette;

/// Light color transformation tables
///
/// LightTables[level][color_index] = darkened_color_index
///
/// - level: 0 (complete darkness) - 15 (full brightness)
/// - color_index: Original palette index (0-255)
/// - darkened_color_index: Darkened palette index
pub struct LightTables {
    tables: [[u8; 256]; 16],
}

impl LightTables {
    /// Generate light tables from palette
    ///
    /// Reference: Source/lighting.cpp::MakeLightTable() Line 201-234
    pub fn from_palette(palette: &Palette) -> Self {
        let mut tables = [[0u8; 256]; 16];

        for level in 0..16 {
            // level 0: complete darkness (all map to black)
            // level 15: full brightness (keep original color)
            let brightness = level as f32 / 15.0;

            for color_idx in 0..256 {
                let color = palette.colors[color_idx];
                let darkened = Self::darken_color((color.r, color.g, color.b), brightness);
                let nearest_idx = palette.find_nearest_color(darkened);
                tables[level][color_idx] = nearest_idx;
            }
        }

        Self { tables }
    }

    /// Get darkened color index
    ///
    /// # Arguments
    /// * `light_level` - Light level (0-15)
    /// * `color_index` - Original palette index (0-255)
    ///
    /// # Returns
    /// Darkened palette index
    pub fn get(&self, light_level: u8, color_index: u8) -> u8 {
        let level = light_level.min(15) as usize;
        self.tables[level][color_index as usize]
    }

    /// Darken color by brightness factor
    ///
    /// # Arguments
    /// * `color` - Original RGB color
    /// * `brightness` - Brightness factor (0.0-1.0)
    ///
    /// # Returns
    /// Darkened RGB color
    fn darken_color(color: (u8, u8, u8), brightness: f32) -> (u8, u8, u8) {
        (
            (color.0 as f32 * brightness) as u8,
            (color.1 as f32 * brightness) as u8,
            (color.2 as f32 * brightness) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_light_tables_generation() {
        let palette = create_test_palette();
        let tables = LightTables::from_palette(&palette);

        // Level 0: complete darkness → should map to black
        assert_eq!(tables.get(0, 128), 0);
        assert_eq!(tables.get(0, 255), 0);

        // Level 15: full brightness → keep original color
        assert_eq!(tables.get(15, 128), 128);
        assert_eq!(tables.get(15, 255), 255);
    }

    #[test]
    fn test_light_tables_gradual_darkening() {
        let palette = create_test_palette();
        let tables = LightTables::from_palette(&palette);

        let test_color = 200;

        let mut prev_value = 255;
        for level in (0..=15).rev() {
            let value = tables.get(level, test_color);

            // Darker light should produce darker colors
            assert!(
                value <= prev_value,
                "Level {} value {} should be <= prev {}",
                level,
                value,
                prev_value
            );

            prev_value = value;
        }
    }

    #[test]
    fn test_darken_color() {
        let color = (200, 150, 100);

        // 50% brightness
        let darkened = LightTables::darken_color(color, 0.5);
        assert_eq!(darkened, (100, 75, 50));

        // 0% brightness (complete darkness)
        let darkened = LightTables::darken_color(color, 0.0);
        assert_eq!(darkened, (0, 0, 0));

        // 100% brightness (keep original)
        let darkened = LightTables::darken_color(color, 1.0);
        assert_eq!(darkened, color);
    }
}
