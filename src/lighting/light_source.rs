/// Light source module
///
/// Reference: Source/lighting.h

/// Light source type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Player,  // Player light aura
    Torch,   // Torch light
    Spell,   // Spell light effect
    Monster, // Glowing monster
}

/// Light source
///
/// Reference: Source/lighting.h::Light
#[derive(Debug, Clone)]
pub struct LightSource {
    pub id: usize,
    pub light_type: LightType,
    pub position: (usize, usize), // MicroTile coordinates
    pub radius: u8,               // Light radius (1-15)
    pub active: bool,             // Is active
}

impl LightSource {
    /// Create player light source
    ///
    /// Reference: Source/lighting.cpp - Player light radius
    pub fn player(position: (usize, usize)) -> Self {
        Self {
            id: 0, // Will be set by LightingSystem
            light_type: LightType::Player,
            position,
            radius: 10, // Original player light radius
            active: true,
        }
    }

    /// Create torch light source
    pub fn torch(position: (usize, usize)) -> Self {
        Self {
            id: 0, // Will be set by LightingSystem
            light_type: LightType::Torch,
            position,
            radius: 8, // Torch radius
            active: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_light_creation() {
        let light = LightSource::player((50, 50));
        assert_eq!(light.radius, 10);
        assert_eq!(light.position, (50, 50));
        assert!(light.active);
        assert_eq!(light.light_type, LightType::Player);
    }

    #[test]
    fn test_torch_light_creation() {
        let light = LightSource::torch((30, 40));
        assert_eq!(light.radius, 8);
        assert_eq!(light.position, (30, 40));
        assert!(light.active);
        assert_eq!(light.light_type, LightType::Torch);
    }
}
