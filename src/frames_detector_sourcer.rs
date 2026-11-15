use std::collections::VecDeque;
use std::path::Path;

use image::RgbaImage;

use crate::frame::Frame;
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};
use crate::text_region::Bounds;

pub trait FramesDetectorSourcer: FramesSourcer {
    fn detect(&self, image: &RgbaImage) -> Result<Vec<DetectedFrame>, FrameSourceError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedFrame {
    bounds: Bounds,
    frame: Frame,
}

impl DetectedFrame {
    pub(crate) fn new(bounds: Bounds, frame: Frame) -> Self {
        Self { bounds, frame }
    }

    pub const fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    pub(crate) fn into_frame(self) -> Frame {
        self.frame
    }
}

#[derive(Debug, Default)]
pub(crate) struct DetectedFrameQueue {
    frames: VecDeque<Frame>,
}

impl DetectedFrameQueue {
    pub(crate) fn from_detected_frames(detected_frames: Vec<DetectedFrame>) -> Self {
        Self {
            frames: detected_frames
                .into_iter()
                .map(DetectedFrame::into_frame)
                .collect(),
        }
    }
}

impl FramesSourcer for DetectedFrameQueue {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError> {
        Ok(self.frames.pop_front())
    }
}

pub(crate) fn decode_image(path: &Path) -> Result<RgbaImage, FrameSourceError> {
    image::open(path)
        .map(|image| image.to_rgba8())
        .map_err(|error| {
            FrameSourceError::Decoding(format!("failed to decode {}: {error}", path.display()))
        })
}

pub(crate) fn detected_frames_from_bounds(
    image: &RgbaImage,
    detected_bounds: Vec<Bounds>,
) -> Result<Vec<DetectedFrame>, FrameSourceError> {
    detected_bounds
        .into_iter()
        .enumerate()
        .map(|(index, bounds)| {
            let cropped_image = image::imageops::crop_imm(
                image,
                bounds.x(),
                bounds.y(),
                bounds.width(),
                bounds.height(),
            )
            .to_image();
            let frame = Frame::new(
                format!("Detected Frame {}", index + 1),
                cropped_image.width(),
                cropped_image.height(),
                cropped_image.into_raw(),
            )
            .map_err(|error| FrameSourceError::Decoding(error.to_string()))?;
            Ok(DetectedFrame::new(bounds, frame))
        })
        .collect()
}

pub(crate) fn validate_colors(name: &'static str, values: &[u32]) -> Result<(), FrameSourceError> {
    for &value in values {
        if value > 0xFF_FFFF {
            return Err(FrameSourceError::InvalidColor { name, value });
        }
    }
    Ok(())
}
