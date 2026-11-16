use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{Canvas, FRect};
use sdl3::surface::Surface;

use crate::debug_text::DebugTextLayout;
use crate::frame::{CHANNELS_PER_PIXEL, Frame};
use crate::text_region::{Bounds, TranslatedText};
use crate::text_renderer::{RenderingError, TextRenderer};

const FRAME_BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);
const TRANSLATED_TEXT_COLOR: Color = Color::RGB(255, 255, 255);

#[derive(Debug, Clone, Copy, Default)]
pub struct SoftwareTextRenderer;

impl SoftwareTextRenderer {
    pub const fn new() -> Self {
        Self
    }
}

impl TextRenderer for SoftwareTextRenderer {
    fn render(
        &self,
        frame: &Frame,
        translated_texts: &[TranslatedText],
    ) -> Result<Frame, RenderingError> {
        let width = frame.width();
        let height = frame.height();
        let pitch = width
            .checked_mul(CHANNELS_PER_PIXEL as u32)
            .ok_or_else(|| RenderingError::InvalidFrame("frame pitch overflows".to_owned()))?;
        let mut rgba_pixels = frame.rgba_pixels().to_vec();
        let surface =
            Surface::from_data(&mut rgba_pixels, width, height, pitch, PixelFormat::RGBA32)
                .map_err(|error| rendering_failed("failed to create frame surface", error))?;
        let mut canvas = Canvas::from_surface(surface)
            .map_err(|error| rendering_failed("failed to create software renderer", error))?;
        canvas.set_draw_color(FRAME_BACKGROUND_COLOR);
        canvas
            .fill_rect(FRect::new(0.0, 0.0, width as f32, height as f32))
            .map_err(|error| rendering_failed("failed to fill frame background", error))?;
        let bounds = Bounds::new(0, 0, width, height);
        let layout = DebugTextLayout::for_translated_texts(translated_texts, bounds);
        layout
            .draw(&mut canvas, (0.0, 0.0), 1.0, TRANSLATED_TEXT_COLOR)
            .map_err(|error| rendering_failed("failed to draw translated text", error))?;
        drop(canvas);
        Frame::new(frame.name(), width, height, rgba_pixels)
            .map_err(|error| RenderingError::InvalidFrame(error.to_string()))
    }
}

fn rendering_failed(operation: &str, error: impl std::fmt::Display) -> RenderingError {
    RenderingError::RenderingFailed(format!("{operation}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_region::{Bounds, TextRegion};

    #[test]
    fn renders_translated_text_into_a_new_frame() {
        let frame = Frame::new("frame", 64, 32, vec![10; 64 * 32 * CHANNELS_PER_PIXEL]).unwrap();
        let translated_texts = [TranslatedText::new(
            TextRegion::new(Bounds::new(0, 0, 64, 32), "source"),
            "Hello",
        )];
        let rendered = SoftwareTextRenderer::new()
            .render(&frame, &translated_texts)
            .unwrap();
        assert_eq!(rendered.width(), frame.width());
        assert_eq!(rendered.height(), frame.height());
        assert_eq!(rendered.name(), frame.name());
        assert!(
            rendered
                .rgba_pixels()
                .as_chunks::<CHANNELS_PER_PIXEL>()
                .0
                .iter()
                .any(|pixel| pixel[0] > 200 && pixel[1] > 200 && pixel[2] > 200)
        );
        assert!(
            frame
                .rgba_pixels()
                .as_chunks::<CHANNELS_PER_PIXEL>()
                .0
                .iter()
                .all(|pixel| pixel[0] == 10)
        );
    }
}
