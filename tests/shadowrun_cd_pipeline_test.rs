mod support;

use std::error::Error;
use std::time::{Duration, Instant};

use frames_translator::factory::{
    create_frames_ocr, create_frames_sourcer, create_text_renderer, create_text_translator,
};

const EXPECTED_FRAME_COUNT: usize = 2;

#[test]
fn shadowrun_cd_pipeline_recognizes_and_translates_each_detected_frame()
-> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let shadowrun_cd = tests_config.shadowrun_cd();
    let expected_frames = shadowrun_cd.frames().frames();
    assert_eq!(expected_frames.len(), EXPECTED_FRAME_COUNT);
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let frames_ocr = create_frames_ocr(&application_config)?;
    let mut text_translator = create_text_translator(&application_config)?;
    let text_renderer = create_text_renderer(&application_config)?;
    let mut frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        let expected_frame = expected_frames
            .get(frame_count)
            .expect("the detector must not produce more frames than expected");
        let expected_texts = shadowrun_cd
            .expected_result_ocr()
            .get(expected_frame.name())
            .expect("every expected frame must define OCR lines");
        let expected_translations = shadowrun_cd
            .expected_translation_result()
            .get(expected_frame.name())
            .expect("every expected frame must define translations");
        assert_eq!(frame.width(), expected_frame.rect().width());
        assert_eq!(frame.height(), expected_frame.rect().height());
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
        let timeout = Duration::from_secs(shadowrun_cd.timeout_seconds());
        let render_started_at = Instant::now();
        let rendered_frame = text_renderer.render(&frame, &translated_texts)?;
        let render_elapsed = render_started_at.elapsed();
        assert!(render_elapsed >= timeout);
        assert!(render_elapsed < timeout + Duration::from_secs(5));
        assert_eq!(rendered_frame.name(), frame.name());
        assert_eq!(rendered_frame.width(), frame.width());
        assert_eq!(rendered_frame.height(), frame.height());
        frame_count += 1;
    }
    assert_eq!(frame_count, EXPECTED_FRAME_COUNT);
    Ok(())
}
