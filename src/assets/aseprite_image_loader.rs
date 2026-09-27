use std::io;

use aseprite_loader::loader::{AsepriteFile, LayerSelection, LoadImageError, LoadSpriteError};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, RenderAssetUsages},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AsepriteImageLoaderError {
    #[error("encountered IO error reading Aseprite file: {0}")]
    Io(#[from] io::Error),
    #[error("unable to parse Aseprite file: {0}")]
    Parse(#[from] LoadSpriteError),
    #[error("unable to render Aseprite frame: {0}")]
    Render(#[from] LoadImageError),
    #[error("Aseprite file has no frames")]
    NoFrames,
}

#[derive(Copy, Clone, Debug, Default, TypePath)]
pub struct AsepriteImageLoader;

impl AssetLoader for AsepriteImageLoader {
    type Asset = Image;
    type Settings = ();
    type Error = AsepriteImageLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        render_frame_grid(&bytes)
    }

    fn extensions(&self) -> &[&str] {
        &["aseprite", "ase"]
    }
}

/// Side length, in frames, of the smallest square grid that fits `frame_count` frames.
/// This is the layout LDtk uses for the frames of an Aseprite file.
fn grid_side(frame_count: usize) -> usize {
    (1..).find(|side| side * side >= frame_count).unwrap_or(1)
}

fn render_frame_grid(bytes: &[u8]) -> Result<Image, AsepriteImageLoaderError> {
    let file = AsepriteFile::load(bytes)?;

    let frame_count = file.frames().len();
    if frame_count == 0 {
        return Err(AsepriteImageLoaderError::NoFrames);
    }

    let (frame_width, frame_height) = file.size();
    let frame_width = usize::from(frame_width);
    let frame_height = usize::from(frame_height);
    let frame_row_bytes = frame_width * 4;

    let side = grid_side(frame_count);
    let image_width = frame_width * side;
    let image_height = frame_height * side;
    let image_row_bytes = image_width * 4;

    let mut image_buffer = vec![0; image_width * image_height * 4];
    let mut frame_buffer = vec![0; frame_width * frame_height * 4];

    for frame_index in 0..frame_count {
        frame_buffer.fill(0);
        file.render_frame(frame_index, &mut frame_buffer, &LayerSelection::Visible)?;

        let cell_x = frame_index % side;
        let cell_y = frame_index / side;

        for row in 0..frame_height {
            let src_start = row * frame_row_bytes;
            let dst_start =
                (cell_y * frame_height + row) * image_row_bytes + cell_x * frame_row_bytes;

            image_buffer[dst_start..dst_start + frame_row_bytes]
                .copy_from_slice(&frame_buffer[src_start..src_start + frame_row_bytes]);
        }
    }

    Ok(Image::new(
        Extent3d {
            width: image_width as u32,
            height: image_height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image_buffer,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_side_is_smallest_square_that_fits() {
        assert_eq!(grid_side(1), 1);
        assert_eq!(grid_side(2), 2);
        assert_eq!(grid_side(4), 2);
        assert_eq!(grid_side(5), 3);
        assert_eq!(grid_side(9), 3);
        assert_eq!(grid_side(10), 4);
    }

    #[test]
    fn invalid_bytes_are_an_error() {
        assert!(render_frame_grid(b"not an aseprite file").is_err());
    }
}
