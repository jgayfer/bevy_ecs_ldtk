use crate::{
    assets::LdtkLevel,
    ldtk::{LdtkJson, Level},
    resources::LevelSelection,
};
use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use derive_getters::Getters;
use std::collections::HashMap;
use std::path::Path;

fn ldtk_path_to_asset_path<'b>(ldtk_path: &Path, rel_path: &str) -> AssetPath<'b> {
    ldtk_path.parent().unwrap().join(Path::new(rel_path)).into()
}

/// Main asset for loading ldtk files.
///
/// Load your ldtk project with the asset server, then insert the handle into the
/// [`LdtkWorldBundle`].
///
/// [`LdtkWorldBundle`]: crate::components::LdtkWorldBundle
#[derive(Clone, TypeUuid, Getters)]
#[uuid = "ecfb87b7-9cd9-4970-8482-f2f68b770d31"]
pub struct LdtkProject {
    /// Raw ldtk project data.
    data: LdtkJson,
    /// Map from tileset uids to image handles for the loaded tileset.
    tileset_map: HashMap<i32, Handle<Image>>,
    /// Map from level iids to level handles.
    level_map: HashMap<String, Handle<LdtkLevel>>,
    /// Image used for rendering int grid colors.
    int_grid_image_handle: Option<Handle<Image>>,
}

impl LdtkProject {
    /// Get an iterator of all the levels in the LDtk file.
    ///
    /// This abstraction avoids compatibility issues between pre-multi-world and post-multi-world
    /// LDtk projects.
    ///
    /// Note: the returned levels are the ones existent in the [`LdtkProject`].
    /// These levels will have "incomplete" data if you use LDtk's external levels feature.
    /// To always get full level data, you'll need to access `Assets<LdtkLevel>`.
    pub fn iter_levels(&self) -> impl Iterator<Item = &Level> {
        self.data.iter_levels()
    }

    /// Find a particular level using a [`LevelSelection`].
    ///
    /// Note: the returned level is the one existent in the [`LdtkProject`].
    /// This level will have "incomplete" data if you use LDtk's external levels feature.
    /// To always get full level data, you'll need to access `Assets<LdtkLevel>`.
    pub fn get_level(&self, level_selection: &LevelSelection) -> Option<&Level> {
        self.iter_levels()
            .enumerate()
            .find(|(i, l)| level_selection.is_match(i, l))
            .map(|(_, l)| l)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct LdtkProjectLoader;

impl AssetLoader for LdtkProjectLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let data: LdtkJson = serde_json::from_slice(bytes)?;

            let mut external_level_paths = Vec::new();
            let mut level_map = HashMap::new();
            let mut background_images = Vec::new();
            if data.external_levels {
                for level in data.iter_levels() {
                    if let Some(external_rel_path) = &level.external_rel_path {
                        let asset_path =
                            ldtk_path_to_asset_path(load_context.path(), external_rel_path);

                        external_level_paths.push(asset_path.clone());
                        level_map.insert(level.iid.clone(), load_context.get_handle(asset_path));
                    }
                }
            } else {
                for level in data.iter_levels() {
                    let label = level.identifier.as_ref();

                    let mut background_image: Option<Handle<Image>> = None;
                    if let Some(rel_path) = &level.bg_rel_path {
                        let asset_path = ldtk_path_to_asset_path(load_context.path(), rel_path);
                        background_images.push(asset_path.clone());
                        background_image = Some(load_context.get_handle(asset_path));
                    }

                    let ldtk_level = LdtkLevel::new(level.clone());
                    let level_handle =
                        load_context.set_labeled_asset(label, LoadedAsset::new(ldtk_level));

                    level_map.insert(level.iid.clone(), level_handle);
                }
            }

            let mut tileset_rel_paths = Vec::new();
            let mut tileset_map = HashMap::new();
            for tileset in &data.defs.tilesets {
                if let Some(tileset_path) = &tileset.rel_path {
                    let asset_path = ldtk_path_to_asset_path(load_context.path(), tileset_path);

                    tileset_rel_paths.push(asset_path.clone());
                    tileset_map.insert(tileset.uid, load_context.get_handle(asset_path));
                } else if tileset.embed_atlas.is_some() {
                    warn!("Ignoring LDtk's Internal_Icons. They cannot be displayed due to their license.");
                } else {
                    let identifier = &tileset.identifier;
                    warn!("{identifier} tileset cannot be loaded, it has a null relative path.");
                }
            }

            let int_grid_image_handle = data.defs.create_int_grid_image().map(|image| {
                load_context.set_labeled_asset("int_grid_image", LoadedAsset::new(image))
            });

            let ldtk_asset = LdtkProject {
                data,
                tileset_map,
                level_map,
                int_grid_image_handle,
            };
            load_context.set_default_asset(
                LoadedAsset::new(ldtk_asset)
                    .with_dependencies(tileset_rel_paths)
                    .with_dependencies(external_level_paths)
                    .with_dependencies(background_images),
            );
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtk"]
    }
}