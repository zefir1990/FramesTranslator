mod support;

use std::error::Error;

const IMAGE_PATH: &str = "test-case/Shadowrun_01.png";

#[test]
fn shadowrun_dialog_pipeline_recognizes_translates_and_renders() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let dialog = tests_config.shadowrun_dialog();
    support::assert_detected_frame_pipeline(
        &application_config,
        IMAGE_PATH,
        dialog.frames(),
        dialog.expected_result_ocr(),
        dialog.expected_translation_result(),
    )
}
