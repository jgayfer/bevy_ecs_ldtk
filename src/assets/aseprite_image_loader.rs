//! Loader that flattens the first frame of an Aseprite file into an [`Image`].
//!
//! Requires the `aseprite` feature to be enabled.
use std::io;

use aseprite_loader::loader::{AsepriteFile, LayerSelection, LoadImageError, LoadSpriteError};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, RenderAssetUsages},
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur when loading an Aseprite file as an [`Image`].
#[derive(Debug, Error)]
pub enum AsepriteImageLoaderError {
    /// Encountered IO error reading Aseprite file
    #[error("encountered IO error reading Aseprite file: {0}")]
    Io(#[from] io::Error),
    /// Unable to parse Aseprite file
    #[error("unable to parse Aseprite file: {0}")]
    Parse(#[from] LoadSpriteError),
    /// Unable to render Aseprite frame
    #[error("unable to render Aseprite frame: {0}")]
    Render(#[from] LoadImageError),
    /// Aseprite file has no frames
    #[error("Aseprite file has no frames")]
    NoFrames,
}

/// Settings for loading an [`Image`] using an [`AsepriteImageLoader`].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AsepriteImageLoaderSettings {
    /// Sampler to use for the loaded image.
    ///
    /// Defaults to [`ImageSampler::Default`], which defers to the app's default image sampler.
    /// This matches the behavior of tilesets loaded from `.png` files.
    pub sampler: ImageSampler,
}

/// [`AssetLoader`] that loads `.aseprite` and `.ase` files as [`Image`]s.
///
/// Only the first frame is rendered, and only layers marked visible in the file are included.
/// This matches how LDtk itself displays Aseprite tilesets.
///
/// This loader produces a plain [`Image`], so it does not conflict with loaders from other
/// crates that produce their own asset type from the same file extension.
/// Bevy selects a loader by both the requested asset type and the file extension.
///
/// Requires the `aseprite` feature to be enabled.
#[derive(Copy, Clone, Debug, Default, TypePath)]
pub struct AsepriteImageLoader;

impl AssetLoader for AsepriteImageLoader {
    type Asset = Image;
    type Settings = AsepriteImageLoaderSettings;
    type Error = AsepriteImageLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut image = flatten_first_frame(&bytes)?;
        image.sampler = settings.sampler.clone();

        Ok(image)
    }

    fn extensions(&self) -> &[&str] {
        &["aseprite", "ase"]
    }
}

/// Render the first frame of an Aseprite file, using its visible layers, as an RGBA8 [`Image`].
fn flatten_first_frame(bytes: &[u8]) -> Result<Image, AsepriteImageLoaderError> {
    let file = AsepriteFile::load(bytes)?;

    if file.frames().is_empty() {
        return Err(AsepriteImageLoaderError::NoFrames);
    }

    let (width, height) = file.size();
    let mut buffer = vec![0; usize::from(width) * usize::from(height) * 4];

    file.render_frame(0, &mut buffer, &LayerSelection::Visible)?;

    Ok(Image::new(
        Extent3d {
            width: u32::from(width),
            height: u32::from(height),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        buffer,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}
