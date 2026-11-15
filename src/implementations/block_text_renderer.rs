use crate::frame::{CHANNELS_PER_PIXEL, Frame};
use crate::text_region::{Bounds, TranslatedText};
use crate::text_renderer::{RenderingError, TextRenderer};

const COLOR: [u8; 4] = [255, 128, 0, 255];

#[derive(Debug, Clone, Copy, Default)]
pub struct BlockTextRenderer;

impl BlockTextRenderer {
    pub const fn new() -> Self {
        Self
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
            fill_bounds(&mut rgba_pixels, frame.width(), bounds, COLOR);
        }
        Frame::new(frame.name(), frame.width(), frame.height(), rgba_pixels)
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
