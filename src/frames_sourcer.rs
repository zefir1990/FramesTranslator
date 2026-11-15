use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

use crate::frame::Frame;
use crate::text_region::Bounds;

pub trait FramesSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError>;
}

#[derive(Debug)]
pub enum FrameSourceError {
    Io(io::Error),
    Decoding(String),
    RegionOutOfBounds {
        name: String,
        bounds: Bounds,
        image_width: u32,
        image_height: u32,
    },
}

impl Display for FrameSourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "frame source I/O error: {error}"),
            Self::Decoding(message) => write!(formatter, "frame decoding failed: {message}"),
            Self::RegionOutOfBounds {
                name,
                bounds,
                image_width,
                image_height,
            } => write!(
                formatter,
                "frame {name:?} with bounds {}x{} at ({}, {}) exceeds the {image_width}x{image_height} image",
                bounds.width(),
                bounds.height(),
                bounds.x(),
                bounds.y()
            ),
        }
    }
}

impl Error for FrameSourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Decoding(_) | Self::RegionOutOfBounds { .. } => None,
        }
    }
}

impl From<io::Error> for FrameSourceError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
