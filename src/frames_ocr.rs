use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::frame::Frame;
use crate::text_region::TextRegion;

pub trait FramesOCR {
    fn recognize(&self, frame: &Frame) -> Result<Vec<TextRegion>, RecognitionError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecognitionError {
    UnsupportedFrame,
    RecognitionFailed(String),
}

impl Display for RecognitionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFrame => write!(formatter, "frame format is not supported"),
            Self::RecognitionFailed(message) => {
                write!(formatter, "text recognition failed: {message}")
            }
        }
    }
}

impl Error for RecognitionError {}
