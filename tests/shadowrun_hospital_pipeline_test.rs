mod support;

use std::error::Error;

const IMAGE_PATH: &str = "test-case/Shadowrun_04.png";

#[test]
fn shadowrun_hospital_pipeline_recognizes_translates_and_renders() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let hospital = tests_config.shadowrun_hospital();
    support::assert_detected_frame_pipeline(
        &application_config,
        IMAGE_PATH,
        hospital.frames(),
        hospital.expected_result_ocr(),
        hospital.expected_translation_result(),
    )
}
