mod support;

use std::error::Error;
use std::path::PathBuf;

use frames_translator::frame::Frame;
use frames_translator::frames_ocr::FramesOCR;
use frames_translator::frames_sourcer::FramesSourcer;
use frames_translator::implementations::{
    ArgosTextTranslator, BlockTextRenderer, FramesDetectorSourcer, MangaFramesOCR,
};
use frames_translator::text_region::Bounds;
use frames_translator::text_renderer::TextRenderer;
use frames_translator::text_translator::TextTranslator;

const IMAGE_PATH: &str = "test-case/Shadowrun_01.png";
const EXPECTED_FRAME_COUNT: usize = 1;
const RENDERED_BLOCK_COLOR: [u8; 4] = [255, 128, 0, 255];

#[test]
fn shadowrun_dialog_pipeline_recognizes_translates_and_renders() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let dialog = tests_config.shadowrun_dialog();
    let expected_frames = dialog.frames().frames();
    assert_eq!(expected_frames.len(), EXPECTED_FRAME_COUNT);
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(IMAGE_PATH);
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
            .get(frame_count)
            .expect("the detector must not produce more frames than expected");
        let expected_texts = dialog
            .expected_result_ocr()
            .get(expected_frame.name())
            .expect("every expected frame must define OCR lines");
        let expected_translations = dialog
            .expected_translation_result()
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
        let rendered_frame = text_renderer.render(&frame, &translated_texts)?;
        assert_eq!(rendered_frame.name(), frame.name());
        assert_eq!(rendered_frame.width(), frame.width());
        assert_eq!(rendered_frame.height(), frame.height());
        for translated in &translated_texts {
            assert_rendered_block(&rendered_frame, translated.source_region().bounds());
        }
        frame_count += 1;
    }
    assert_eq!(frame_count, EXPECTED_FRAME_COUNT);
    Ok(())
}

fn assert_rendered_block(frame: &Frame, bounds: Bounds) {
    let offset = (bounds.y() as usize * frame.width() as usize + bounds.x() as usize) * 4;
    assert_eq!(
        &frame.rgba_pixels()[offset..offset + 4],
        RENDERED_BLOCK_COLOR.as_slice()
    );
}
