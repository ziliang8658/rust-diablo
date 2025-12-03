/// Assets module - Resource management
///
/// Manages game assets and resources
use std::path::Path;

/// Asset paths
pub struct AssetPaths;

impl AssetPaths {
    /// Get path to assets directory
    pub fn assets_dir() -> &'static str {
        "assets"
    }

    /// Get path to sprites directory
    pub fn sprites_dir() -> String {
        format!("{}/sprites", Self::assets_dir())
    }

    /// Get path to a sprite file
    pub fn sprite(name: &str) -> String {
        format!("{}/{}", Self::sprites_dir(), name)
    }

    /// Check if assets directory exists
    pub fn check_assets_exist() -> bool {
        Path::new(Self::assets_dir()).exists()
    }
}
