use crate::frame::{CHANNELS_PER_PIXEL, Frame};
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};

const FRAME_WIDTH: u32 = 32;
const FRAME_HEIGHT: u32 = 24;
const FRAME_COUNT: usize = 2;
const FRAME_NAME: &str = "Synthetic";
const FILL_COLOR: [u8; 4] = [12, 34, 56, 255];

#[derive(Debug, Clone)]
pub struct SyntheticFramesSourcer {
    remaining_frames: usize,
}

impl SyntheticFramesSourcer {
    pub const fn new() -> Self {
        Self {
            remaining_frames: FRAME_COUNT,
        }
    }
}

impl Default for SyntheticFramesSourcer {
    fn default() -> Self {
        Self::new()
    }
}

impl FramesSourcer for SyntheticFramesSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError> {
        if self.remaining_frames == 0 {
            return Ok(None);
        }
        self.remaining_frames -= 1;
        let buffer_length = FRAME_WIDTH as usize * FRAME_HEIGHT as usize * CHANNELS_PER_PIXEL;
        let mut rgba_pixels = vec![0; buffer_length];
        for pixel in rgba_pixels.as_chunks_mut::<CHANNELS_PER_PIXEL>().0 {
            pixel.copy_from_slice(&FILL_COLOR);
        }
        Frame::new(FRAME_NAME, FRAME_WIDTH, FRAME_HEIGHT, rgba_pixels)
            .map(Some)
            .map_err(|error| FrameSourceError::Decoding(error.to_string()))
    }
}
