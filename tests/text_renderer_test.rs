mod support;

use std::error::Error;

use frames_translator::factory::create_text_renderer;
use frames_translator::frame::Frame;
use frames_translator::text_region::{Bounds, TextRegion, TranslatedText};
use frames_translator::text_renderer::RenderingError;

const FRAME_WIDTH: u32 = 16;
const FRAME_HEIGHT: u32 = 16;
const FRAME_NAME: &str = "Renderer test frame";
const REGION_X: u32 = 4;
const REGION_Y: u32 = 4;
const REGION_WIDTH: u32 = 8;
const REGION_HEIGHT: u32 = 8;
const EXPECTED_COLOR: [u8; 4] = [255, 128, 0, 255];

#[test]
fn text_renderer_draws_translated_regions() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let bounds = Bounds::new(REGION_X, REGION_Y, REGION_WIDTH, REGION_HEIGHT);
    let frame = Frame::new(
        FRAME_NAME,
        FRAME_WIDTH,
        FRAME_HEIGHT,
        vec![0; FRAME_WIDTH as usize * FRAME_HEIGHT as usize * 4],
    )?;
    let translated_text = TranslatedText::new(
        TextRegion::new(bounds, "Shadowrun"),
        "Shadowrun (translated)",
    );
    let text_renderer = create_text_renderer(&application_config)?;
    let rendered_frame = text_renderer.render(&frame, &[translated_text])?;
    assert_eq!(rendered_frame.name(), frame.name());
    assert_eq!(rendered_frame.width(), frame.width());
    assert_eq!(rendered_frame.height(), frame.height());
    assert_eq!(&rendered_frame.rgba_pixels()[..4], &[0; 4]);
    let region_offset = (REGION_Y as usize * FRAME_WIDTH as usize + REGION_X as usize) * 4;
    assert_eq!(
        &rendered_frame.rgba_pixels()[region_offset..region_offset + 4],
        EXPECTED_COLOR.as_slice()
    );
    assert_eq!(text_renderer.render(&frame, &[])?, frame);
    Ok(())
}

#[test]
fn text_renderer_rejects_regions_outside_the_frame() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let frame = Frame::new("Small frame", 4, 4, vec![0; 4 * 4 * 4])?;
    let translated_text =
        TranslatedText::new(TextRegion::new(Bounds::new(3, 3, 4, 4), "Text"), "Text");
    let text_renderer = create_text_renderer(&application_config)?;
    let result = text_renderer.render(&frame, &[translated_text]);
    assert!(matches!(
        result,
        Err(RenderingError::RegionOutOfBounds { .. })
    ));
    Ok(())
}
