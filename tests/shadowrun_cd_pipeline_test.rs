mod support;

use std::error::Error;

use frames_translator::factory::{
    create_frames_ocr, create_frames_sourcer, create_text_translator,
};

const EXPECTED_FRAME_COUNT: usize = 1;
const EXPECTED_FRAME_NAME: &str = "Actions Menu";
const EXPECTED_LINE_COUNT: usize = 4;

#[test]
fn shadowrun_cd_pipeline_recognizes_and_translates_four_lines() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let shadowrun_cd = tests_config.shadowrun_cd();
    let expected_texts = shadowrun_cd.expected_result_ocr();
    let expected_translations = shadowrun_cd.expected_translation_result();
    assert_eq!(expected_texts.len(), EXPECTED_LINE_COUNT);
    assert_eq!(expected_translations.len(), EXPECTED_LINE_COUNT);
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let frames_ocr = create_frames_ocr(&application_config)?;
    let mut text_translator = create_text_translator(&application_config)?;
    let mut frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        assert_eq!(frame.name(), EXPECTED_FRAME_NAME);
        let regions = frames_ocr.recognize(&frame)?;
        assert_eq!(regions.len(), expected_texts.len());
        for (region, expected_text) in regions.iter().zip(expected_texts) {
            assert_eq!(region.text(), expected_text.as_str());
        }
        let translated_texts = text_translator.translate_all(&regions)?;
        assert_eq!(translated_texts.len(), expected_translations.len());
        for ((region, translated), expected_translation) in regions
            .iter()
            .zip(&translated_texts)
            .zip(expected_translations)
        {
            assert_eq!(translated.source_region(), region);
            assert_eq!(translated.text(), expected_translation.as_str());
        }
        frame_count += 1;
    }
    assert_eq!(frame_count, EXPECTED_FRAME_COUNT);
    Ok(())
}
