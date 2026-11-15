mod support;

use std::error::Error;

use frames_translator::factory::{create_frames_ocr, create_frames_sourcer};

const EXPECTED_FRAME_COUNT: usize = 2;

#[test]
fn frames_ocr_recognizes_lines_for_each_detected_frame() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let shadowrun_cd = tests_config.shadowrun_cd();
    let expected_frames = shadowrun_cd.frames().frames();
    assert_eq!(expected_frames.len(), EXPECTED_FRAME_COUNT);
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let frames_ocr = create_frames_ocr(&application_config)?;
    let mut frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        let expected_frame = expected_frames
            .get(frame_count)
            .expect("the detector must not produce more frames than expected");
        let expected_texts = shadowrun_cd
            .expected_result_ocr()
            .get(expected_frame.name())
            .expect("every expected frame must define OCR lines");
        assert_eq!(frame.width(), expected_frame.rect().width());
        assert_eq!(frame.height(), expected_frame.rect().height());
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
