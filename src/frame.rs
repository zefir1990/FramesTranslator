use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub const CHANNELS_PER_PIXEL: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    name: String,
    width: u32,
    height: u32,
    rgba_pixels: Vec<u8>,
}

impl Frame {
    pub fn new(
        name: impl Into<String>,
        width: u32,
        height: u32,
        rgba_pixels: Vec<u8>,
    ) -> Result<Self, FrameError> {
        let expected_length = expected_rgba_buffer_length(width, height)?;
        let actual_length = rgba_pixels.len();
        if actual_length != expected_length {
            return Err(FrameError::PixelBufferSizeMismatch {
                expected: expected_length,
                actual: actual_length,
            });
        }
        Ok(Self {
            name: name.into(),
            width,
            height,
            rgba_pixels,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba_pixels(&self) -> &[u8] {
        &self.rgba_pixels
    }
}

fn expected_rgba_buffer_length(width: u32, height: u32) -> Result<usize, FrameError> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixel_count| pixel_count.checked_mul(CHANNELS_PER_PIXEL))
        .ok_or(FrameError::DimensionsOverflow { width, height })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    DimensionsOverflow { width: u32, height: u32 },
    PixelBufferSizeMismatch { expected: usize, actual: usize },
}

impl Display for FrameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionsOverflow { width, height } => write!(
                formatter,
                "frame dimensions {width}x{height} overflow the pixel buffer size"
            ),
            Self::PixelBufferSizeMismatch { expected, actual } => {
                write!(
                    formatter,
                    "expected {expected} RGBA bytes, received {actual}"
                )
            }
        }
    }
}

impl Error for FrameError {}
