use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::frame::Frame;
use crate::text_region::{Bounds, TranslatedText};

pub trait TextRenderer {
    fn render(
        &self,
        frame: &Frame,
        translated_texts: &[TranslatedText],
    ) -> Result<Frame, RenderingError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderingError {
    RegionOutOfBounds {
        bounds: Bounds,
        frame_width: u32,
        frame_height: u32,
    },
    UnsupportedFrame,
    InvalidFrame(String),
}

impl Display for RenderingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegionOutOfBounds {
                bounds,
                frame_width,
                frame_height,
            } => write!(
                formatter,
                "text region {}x{} at ({}, {}) exceeds the {frame_width}x{frame_height} frame",
                bounds.width(),
                bounds.height(),
                bounds.x(),
                bounds.y()
            ),
            Self::UnsupportedFrame => write!(formatter, "frame format is not supported"),
            Self::InvalidFrame(message) => write!(formatter, "frame is invalid: {message}"),
        }
    }
}

impl Error for RenderingError {}
