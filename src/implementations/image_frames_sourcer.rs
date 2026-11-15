use std::collections::VecDeque;
use std::path::Path;

use image::RgbaImage;

use crate::frame::Frame;
use crate::frames_config::FramesConfig;
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};

#[derive(Debug)]
pub struct ImageFramesSourcer {
    frames: VecDeque<Frame>,
}

impl ImageFramesSourcer {
    pub fn open(
        path: impl AsRef<Path>,
        frames_config: &FramesConfig,
    ) -> Result<Self, FrameSourceError> {
        let path = path.as_ref();
        let image = image::open(path).map_err(|error| {
            FrameSourceError::Decoding(format!("failed to decode {}: {error}", path.display()))
        })?;
        let rgba_image = image.to_rgba8();
        Ok(Self {
            frames: extract_frames(&rgba_image, frames_config)?.into(),
        })
    }
}

impl FramesSourcer for ImageFramesSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError> {
        Ok(self.frames.pop_front())
    }
}

fn extract_frames(
    image: &RgbaImage,
    frames_config: &FramesConfig,
) -> Result<Vec<Frame>, FrameSourceError> {
    frames_config
        .frames()
        .iter()
        .map(|named_rect| {
            let bounds = named_rect.rect();
            if bounds.right() > image.width() || bounds.bottom() > image.height() {
                return Err(FrameSourceError::RegionOutOfBounds {
                    name: named_rect.name().to_owned(),
                    bounds,
                    image_width: image.width(),
                    image_height: image.height(),
                });
            }
            let cropped_image = image::imageops::crop_imm(
                image,
                bounds.x(),
                bounds.y(),
                bounds.width(),
                bounds.height(),
            )
            .to_image();
            Frame::new(
                named_rect.name(),
                cropped_image.width(),
                cropped_image.height(),
                cropped_image.into_raw(),
            )
            .map_err(|error| FrameSourceError::Decoding(error.to_string()))
        })
        .collect()
}
