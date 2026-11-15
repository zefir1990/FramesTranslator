mod support;

use std::error::Error;

use frames_translator::factory::create_text_renderer;
use frames_translator::frame::Frame;
use frames_translator::text_region::{Bounds, TextRegion, TranslatedText};
use frames_translator::text_renderer::RenderingError;

#[test]
fn text_renderer_draws_translated_regions() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let frame_width = config.require_u32("TEXT_RENDERER_TEST_FRAME_WIDTH")?;
    let frame_height = config.require_u32("TEXT_RENDERER_TEST_FRAME_HEIGHT")?;
    let bounds = Bounds::new(
        config.require_u32("TEXT_RENDERER_TEST_REGION_X")?,
        config.require_u32("TEXT_RENDERER_TEST_REGION_Y")?,
        config.require_u32("TEXT_RENDERER_TEST_REGION_WIDTH")?,
        config.require_u32("TEXT_RENDERER_TEST_REGION_HEIGHT")?,
    );
    let expected_color = config.require_color("TEXT_RENDERER_COLOR")?;
    let frame = Frame::new(
        frame_width,
        frame_height,
        vec![0; frame_width as usize * frame_height as usize * 4],
    )?;
    let translated_text = TranslatedText::new(
        TextRegion::new(bounds, "Shadowrun"),
        "Shadowrun (translated)",
    );
    let text_renderer = create_text_renderer(&config)?;
    let rendered_frame = text_renderer.render(&frame, &[translated_text])?;
    assert_eq!(rendered_frame.width(), frame.width());
    assert_eq!(rendered_frame.height(), frame.height());
    assert_eq!(&rendered_frame.rgba_pixels()[..4], &[0; 4]);
    let region_offset = (bounds.y() as usize * frame_width as usize + bounds.x() as usize) * 4;
    assert_eq!(
        &rendered_frame.rgba_pixels()[region_offset..region_offset + 4],
        expected_color.as_slice()
    );
    assert_eq!(text_renderer.render(&frame, &[])?, frame);
    Ok(())
}

#[test]
fn text_renderer_rejects_regions_outside_the_frame() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let frame = Frame::new(4, 4, vec![0; 4 * 4 * 4])?;
    let translated_text =
        TranslatedText::new(TextRegion::new(Bounds::new(3, 3, 4, 4), "Text"), "Text");
    let text_renderer = create_text_renderer(&config)?;
    let result = text_renderer.render(&frame, &[translated_text]);
    assert!(matches!(
        result,
        Err(RenderingError::RegionOutOfBounds { .. })
    ));
    Ok(())
}
