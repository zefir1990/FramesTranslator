use crate::config::{Config, ConfigError};
use crate::frame::{CHANNELS_PER_PIXEL, Frame};
use crate::text_region::{Bounds, TranslatedText};
use crate::text_renderer::{RenderingError, TextRenderer};

#[derive(Debug, Clone, Copy)]
pub struct BlockTextRenderer {
    color: [u8; 4],
}

impl BlockTextRenderer {
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        Ok(Self {
            color: config.require_color("TEXT_RENDERER_COLOR")?,
        })
    }
}

impl TextRenderer for BlockTextRenderer {
    fn render(
        &self,
        frame: &Frame,
        translated_texts: &[TranslatedText],
    ) -> Result<Frame, RenderingError> {
        let mut rgba_pixels = frame.rgba_pixels().to_vec();
        for translated_text in translated_texts {
            let bounds = translated_text.source_region().bounds();
            if bounds.right() > frame.width() || bounds.bottom() > frame.height() {
                return Err(RenderingError::RegionOutOfBounds {
                    bounds,
                    frame_width: frame.width(),
                    frame_height: frame.height(),
                });
            }
            fill_bounds(&mut rgba_pixels, frame.width(), bounds, self.color);
        }
        Frame::new(frame.width(), frame.height(), rgba_pixels)
            .map_err(|error| RenderingError::InvalidFrame(error.to_string()))
    }
}

fn fill_bounds(rgba_pixels: &mut [u8], frame_width: u32, bounds: Bounds, color: [u8; 4]) {
    let frame_width = frame_width as usize;
    for y in bounds.y()..bounds.bottom() {
        for x in bounds.x()..bounds.right() {
            let offset = (y as usize * frame_width + x as usize) * CHANNELS_PER_PIXEL;
            rgba_pixels[offset..offset + CHANNELS_PER_PIXEL].copy_from_slice(&color);
        }
    }
}
