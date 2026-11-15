mod support;

use std::error::Error;

use frames_translator::factory::{create_frames_ocr, create_frames_sourcer};

const EXPECTED_FRAME_COUNT: usize = 1;
const EXPECTED_LINE_COUNT: usize = 4;

#[test]
fn frames_ocr_recognizes_four_shadowrun_frame_lines() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let expected_texts = tests_config.shadowrun_cd().expected_result_ocr();
    assert_eq!(expected_texts.len(), EXPECTED_LINE_COUNT);
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let frames_ocr = create_frames_ocr(&application_config)?;
    let mut frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        let regions = frames_ocr.recognize(&frame)?;
        assert_eq!(regions.len(), expected_texts.len());
        for (region, expected_text) in regions.iter().zip(expected_texts) {
            assert_eq!(region.text(), expected_text.as_str());
            assert!(region.bounds().right() <= frame.width());
            assert!(region.bounds().bottom() <= frame.height());
        }
        frame_count += 1;
    }
    assert_eq!(frame_count, EXPECTED_FRAME_COUNT);
    Ok(())
}
