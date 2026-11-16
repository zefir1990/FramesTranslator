use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::frame::{Frame, FrameError};

pub trait Screenshoter {
    fn sources(&self) -> Result<Vec<String>, ScreenshotError>;
    fn screenshot(&self, source: &str) -> Result<Frame, ScreenshotError>;
}

#[derive(Debug)]
pub enum ScreenshotError {
    Enumeration {
        reason: String,
    },
    UnknownSource {
        source: String,
    },
    Capture {
        source: String,
        reason: String,
    },
    InvalidDimensions {
        source: String,
        width: u32,
        height: u32,
    },
    BufferSizeOverflow {
        source: String,
        width: u32,
        height: u32,
    },
    Frame(FrameError),
}

impl Display for ScreenshotError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enumeration { reason } => {
                write!(
                    formatter,
                    "failed to enumerate screenshot sources: {reason}"
                )
            }
            Self::UnknownSource { source } => {
                write!(formatter, "screenshot source {source:?} is not available")
            }
            Self::Capture { source, reason } => {
                write!(
                    formatter,
                    "failed to capture screenshot source {source:?}: {reason}"
                )
            }
            Self::InvalidDimensions {
                source,
                width,
                height,
            } => write!(
                formatter,
                "screenshot source {source:?} has invalid dimensions {width}x{height}"
            ),
            Self::BufferSizeOverflow {
                source,
                width,
                height,
            } => write!(
                formatter,
                "screenshot source {source:?} pixel buffer for {width}x{height} overflows"
            ),
            Self::Frame(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ScreenshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frame(error) => Some(error),
            Self::Enumeration { .. }
            | Self::UnknownSource { .. }
            | Self::Capture { .. }
            | Self::InvalidDimensions { .. }
            | Self::BufferSizeOverflow { .. } => None,
        }
    }
}

impl From<FrameError> for ScreenshotError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}
