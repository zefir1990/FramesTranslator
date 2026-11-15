use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

use crate::frame::Frame;

pub trait FramesSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError>;
}

#[derive(Debug)]
pub enum FrameSourceError {
    Io(io::Error),
    Decoding(String),
}

impl Display for FrameSourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "frame source I/O error: {error}"),
            Self::Decoding(message) => write!(formatter, "frame decoding failed: {message}"),
        }
    }
}

impl Error for FrameSourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Decoding(_) => None,
        }
    }
}

impl From<io::Error> for FrameSourceError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
