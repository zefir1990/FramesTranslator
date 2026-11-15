#![allow(dead_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use frames_translator::application_config::ApplicationConfig;
use frames_translator::frame::Frame;
use frames_translator::frame_translation::translate_frame_regions;
use frames_translator::frames_config::FramesConfig;
use frames_translator::frames_ocr::FramesOCR;
use frames_translator::frames_sourcer::FramesSourcer;
use frames_translator::implementations::{
    ArgosTextTranslator, BlockTextRenderer, FramesDetectorSourcer, MangaFramesOCR,
};
use frames_translator::tests_config::TestsConfig;
use frames_translator::text_region::Bounds;
use frames_translator::text_renderer::TextRenderer;

const APPLICATION_CONFIG_PATH: &str = "test-case/tests-application-config.jsonc";
const TESTS_CONFIG_PATH: &str = "test-case/tests-config.jsonc";
const RENDERED_BLOCK_COLOR: [u8; 4] = [255, 128, 0, 255];

pub fn load_application_config() -> Result<ApplicationConfig, Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(APPLICATION_CONFIG_PATH);
    let contents = fs::read_to_string(config_path)?;
    Ok(ApplicationConfig::from_jsonc(&contents)?)
}

pub fn load_tests_config() -> Result<TestsConfig, Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TESTS_CONFIG_PATH);
    let contents = fs::read_to_string(config_path)?;
    Ok(TestsConfig::from_jsonc(&contents)?)
}

pub fn assert_detected_frame_pipeline(
    application_config: &ApplicationConfig,
    image_relative_path: &str,
    expected_frames: &FramesConfig,
    expected_result_ocr: &BTreeMap<String, Vec<String>>,
    expected_translation_result: &BTreeMap<String, Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(image_relative_path);
    let mut frames_sourcer = FramesDetectorSourcer::open(
        &image_path,
        application_config.frames_detection_background_colors(),
        application_config.frames_detection_text_colors(),
    )?;
    let frames_ocr = MangaFramesOCR::new()?;
    let mut text_translator = ArgosTextTranslator::new()?;
    let text_renderer = BlockTextRenderer::new();
    let mut frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        let expected_frame = expected_frames
            .frames()
            .get(frame_count)
            .expect("the detector must not produce more frames than expected");
        let expected_texts = expected_result_ocr
            .get(expected_frame.name())
            .expect("every expected frame must define OCR lines");
        let expected_translations = expected_translation_result
            .get(expected_frame.name())
            .expect("every expected frame must define translations");
        assert_eq!(frame.width(), expected_frame.rect().width());
        assert_eq!(frame.height(), expected_frame.rect().height());
        let regions = frames_ocr.recognize(&frame)?;
        assert_eq!(regions.len(), expected_texts.len());
        for (region, expected_text) in regions.iter().zip(expected_texts) {
            assert_eq!(region.text(), expected_text.as_str());
            assert!(region.bounds().right() <= frame.width());
            assert!(region.bounds().bottom() <= frame.height());
        }
        let translated_texts = translate_frame_regions(&frame, &regions, &mut text_translator)?;
        assert_eq!(translated_texts.len(), expected_translations.len());
        for ((region, translated), expected_translation) in regions
            .iter()
            .zip(&translated_texts)
            .zip(expected_translations)
        {
            assert_eq!(translated.source_region(), region);
            assert_eq!(translated.text(), expected_translation.as_str());
        }
        let rendered_frame = text_renderer.render(&frame, &translated_texts)?;
        assert_eq!(rendered_frame.name(), frame.name());
        assert_eq!(rendered_frame.width(), frame.width());
        assert_eq!(rendered_frame.height(), frame.height());
        for translated in &translated_texts {
            assert_rendered_block(&rendered_frame, translated.source_region().bounds());
        }
        frame_count += 1;
    }
    assert_eq!(frame_count, expected_frames.frames().len());
    Ok(())
}

fn assert_rendered_block(frame: &Frame, bounds: Bounds) {
    let offset = (bounds.y() as usize * frame.width() as usize + bounds.x() as usize) * 4;
    assert_eq!(
        &frame.rgba_pixels()[offset..offset + 4],
        RENDERED_BLOCK_COLOR.as_slice()
    );
}
