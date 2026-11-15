mod support;

use std::error::Error;

use frames_translator::factory::{create_frames_ocr, create_frames_sourcer};
use frames_translator::text_region::Bounds;

const EXPECTED_FRAME_COUNT: usize = 1;
const EXPECTED_TEXT_FRAGMENT: &str = "話す";

#[test]
fn frames_ocr_recognizes_shadowrun_frame_text() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let frames_ocr = create_frames_ocr(&application_config)?;
    let mut frame_count = 0;
    let mut recognized_text = String::new();
    while let Some(frame) = frames_sourcer.next_frame()? {
        let regions = frames_ocr.recognize(&frame)?;
        assert!(!regions.is_empty());
        for region in regions {
            assert!(!region.text().is_empty());
            assert_eq!(
                region.bounds(),
                Bounds::new(0, 0, frame.width(), frame.height())
            );
            recognized_text.push_str(region.text());
        }
        frame_count += 1;
    }
    assert_eq!(frame_count, EXPECTED_FRAME_COUNT);
    assert!(
        recognized_text.contains(EXPECTED_TEXT_FRAGMENT),
        "recognized text: {recognized_text:?}"
    );
    Ok(())
}
