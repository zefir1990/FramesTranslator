mod support;

use std::error::Error;

const IMAGE_PATH: &str = "test-case/Shadowrun_03.png";

#[test]
fn shadowrun_street_pipeline_recognizes_translates_and_renders() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let street = tests_config.shadowrun_street();
    support::assert_detected_frame_pipeline(
        &application_config,
        IMAGE_PATH,
        street.frames(),
        street.expected_result_ocr(),
        street.expected_translation_result(),
    )
}
