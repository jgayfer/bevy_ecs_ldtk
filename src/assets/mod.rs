//! Assets and AssetLoaders for loading ldtk files.

use bevy::asset::AssetPath;
use std::path::Path;

mod ldtk_level;
pub use ldtk_level::{LdtkLevel, LdtkLevelLoader};

mod ldtk_project;
pub use ldtk_project::{LdtkProject, LdtkProjectLoader};

fn ldtk_path_to_asset_path<'b>(ldtk_path: &Path, rel_path: &str) -> AssetPath<'b> {
    ldtk_path.parent().unwrap().join(Path::new(rel_path)).into()
}
