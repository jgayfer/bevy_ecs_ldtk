#[cfg(feature = "aseprite")]
use crate::assets::AsepriteImageLoader;
#[cfg(feature = "external_levels")]
use crate::assets::{ldtk_external_level::LdtkExternalLevelLoader, LdtkExternalLevel};
use crate::assets::{ldtk_project::LdtkProjectLoader, LdtkProject};
use bevy::prelude::*;

/// Plugin that registers LDtk-related assets.
///
/// With the `aseprite` feature enabled, this also registers an [`AssetLoader`] that loads
/// `.aseprite` and `.ase` files as [`Image`]s, so that Aseprite files can be used as tilesets.
///
/// [`AssetLoader`]: bevy::asset::AssetLoader
#[derive(Copy, Clone, Debug, Default)]
pub struct LdtkAssetPlugin;

impl Plugin for LdtkAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LdtkProject>()
            .init_asset_loader::<LdtkProjectLoader>();

        #[cfg(feature = "external_levels")]
        {
            app.init_asset::<LdtkExternalLevel>()
                .init_asset_loader::<LdtkExternalLevelLoader>()
                .register_asset_reflect::<LdtkExternalLevel>();
        }

        #[cfg(feature = "aseprite")]
        {
            app.register_asset_loader(AsepriteImageLoader);
        }
    }
}
