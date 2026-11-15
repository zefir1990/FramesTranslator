use crate::config::{Config, ConfigError};
use crate::frame::{CHANNELS_PER_PIXEL, Frame};
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};

#[derive(Debug, Clone)]
pub struct SyntheticFramesSourcer {
    frame_template: Frame,
    remaining_frames: usize,
}

impl SyntheticFramesSourcer {
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        let width = config.require_u32("FRAMES_SOURCER_FRAME_WIDTH")?;
        let height = config.require_u32("FRAMES_SOURCER_FRAME_HEIGHT")?;
        let remaining_frames = config.require_usize("FRAMES_SOURCER_FRAME_COUNT")?;
        let fill_color = config.require_color("FRAMES_SOURCER_FILL_COLOR")?;
        let frame_template = build_filled_frame(width, height, fill_color)?;
        Ok(Self {
            frame_template,
            remaining_frames,
        })
    }
}

impl FramesSourcer for SyntheticFramesSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError> {
        if self.remaining_frames == 0 {
            return Ok(None);
        }
        self.remaining_frames -= 1;
        Ok(Some(self.frame_template.clone()))
    }
}

fn build_filled_frame(width: u32, height: u32, fill_color: [u8; 4]) -> Result<Frame, ConfigError> {
    let buffer_length = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixel_count| pixel_count.checked_mul(CHANNELS_PER_PIXEL))
        .ok_or_else(|| frame_dimensions_error(width, height))?;
    let mut rgba_pixels = vec![0; buffer_length];
    for pixel in rgba_pixels.as_chunks_mut::<CHANNELS_PER_PIXEL>().0 {
        pixel.copy_from_slice(&fill_color);
    }
    Frame::new(width, height, rgba_pixels).map_err(|_| frame_dimensions_error(width, height))
}

fn frame_dimensions_error(width: u32, height: u32) -> ConfigError {
    ConfigError::InvalidValue {
        key: "FRAMES_SOURCER_FRAME_WIDTH".to_owned(),
        value: format!("{width}x{height}"),
        expected: "frame dimensions that fit in memory",
    }
}
