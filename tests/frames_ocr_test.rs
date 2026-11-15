mod support;

use std::error::Error;

use frames_translator::factory::create_frames_ocr;
use frames_translator::frame::Frame;

#[test]
fn frames_ocr_recognizes_regions_inside_the_frame() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let frame_width = config.require_u32("FRAMES_OCR_TEST_FRAME_WIDTH")?;
    let frame_height = config.require_u32("FRAMES_OCR_TEST_FRAME_HEIGHT")?;
    let rgba_pixels = vec![0; frame_width as usize * frame_height as usize * 4];
    let frame = Frame::new(frame_width, frame_height, rgba_pixels)?;
    let frames_ocr = create_frames_ocr(&config)?;
    let regions = frames_ocr.recognize(&frame)?;
    assert!(!regions.is_empty());
    for region in regions {
        assert!(!region.text().is_empty());
        assert!(region.bounds().right() <= frame.width());
        assert!(region.bounds().bottom() <= frame.height());
    }
    Ok(())
}
