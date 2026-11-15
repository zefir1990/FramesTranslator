use std::error::Error;
use std::fs;
use std::path::PathBuf;

use frames_translator::tests_config::TestsConfig;
use frames_translator::text_region::Bounds;

const CONFIG_PATH: &str = "test-case/tests-config.jsonc";
const EXPECTED_FRAME_COUNT: usize = 2;

#[test]
fn tests_config_serializes_and_deserializes_jsonc() -> Result<(), Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CONFIG_PATH);
    let contents = fs::read_to_string(config_path)?;
    let tests_config = TestsConfig::from_jsonc(&contents)?;
    let frames = tests_config.shadowrun_cd().frames().frames();
    assert_eq!(frames.len(), EXPECTED_FRAME_COUNT);
    assert_eq!(frames[0].name(), "Actions Menu");
    assert_eq!(frames[0].rect(), Bounds::new(18, 86, 91, 68));
    assert_eq!(frames[1].name(), "Contacts");
    assert_eq!(frames[1].rect(), Bounds::new(146, 86, 75, 68));
    assert_expected_texts(
        tests_config.shadowrun_cd().expected_result_ocr(),
        "Actions Menu",
        ["見る", "話す", "移動", "アイテム"],
    );
    assert_expected_texts(
        tests_config.shadowrun_cd().expected_result_ocr(),
        "Contacts",
        ["仲間", "通行人１", "通行人２", "通行人９"],
    );
    assert_expected_texts(
        tests_config.shadowrun_cd().expected_translation_result(),
        "Actions Menu",
        ["See", "Talk", "Contact Us", "Item."],
    );
    assert_expected_texts(
        tests_config.shadowrun_cd().expected_translation_result(),
        "Contacts",
        ["Contact Us", "Traffic1.", "person 2.", "Traffic 9."],
    );
    assert_eq!(tests_config.shadowrun_cd().timeout_seconds(), 5);
    let serialized = tests_config.to_jsonc()?;
    let restored = TestsConfig::from_jsonc(&serialized)?;
    assert_eq!(restored, tests_config);
    Ok(())
}

fn assert_expected_texts<const TEXT_COUNT: usize>(
    expected_texts: &std::collections::BTreeMap<String, Vec<String>>,
    frame_name: &str,
    expected_lines: [&str; TEXT_COUNT],
) {
    let actual_lines = expected_texts
        .get(frame_name)
        .expect("every frame must have expected texts");
    assert_eq!(actual_lines.len(), TEXT_COUNT);
    for (actual_line, expected_line) in actual_lines.iter().zip(expected_lines) {
        assert_eq!(actual_line.as_str(), expected_line);
    }
}
