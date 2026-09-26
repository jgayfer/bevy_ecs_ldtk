use crate::ldtk::TilesetDefinition;
use bevy::{asset::AssetPath, reflect::Reflect};

/// A tileset in an LDtk project, resolved against the project's location.
///
/// Holds no asset handle, so the tileset file can be loaded with any loader, not just Bevy's
/// image loader.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct LdtkTilesetRef {
    /// The tileset's definition from the LDtk project.
    pub definition: TilesetDefinition,
    /// Asset path of the tileset's source file.
    pub path: AssetPath<'static>,
}
