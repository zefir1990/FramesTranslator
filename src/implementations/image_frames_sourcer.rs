use std::path::Path;

use crate::frame::Frame;
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};

#[derive(Debug)]
pub struct ImageFramesSourcer {
    frame: Option<Frame>,
}

impl ImageFramesSourcer {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, FrameSourceError> {
        let path = path.as_ref();
        let image = image::open(path).map_err(|error| {
            FrameSourceError::Decoding(format!("failed to decode {}: {error}", path.display()))
        })?;
        let rgba_image = image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let frame_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        let frame = Frame::new(frame_name, width, height, rgba_image.into_raw())
            .map_err(|error| FrameSourceError::Decoding(error.to_string()))?;
        Ok(Self { frame: Some(frame) })
    }
}

impl FramesSourcer for ImageFramesSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError> {
        Ok(self.frame.take())
    }
}
