use std::error::Error;
use std::fs;
use std::path::PathBuf;

use frames_translator::tests_config::TestsConfig;
use frames_translator::text_region::Bounds;

const CONFIG_PATH: &str = "test-case/tests-config.jsonc";

#[test]
fn tests_config_serializes_and_deserializes_jsonc() -> Result<(), Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CONFIG_PATH);
    let contents = fs::read_to_string(config_path)?;
    let tests_config = TestsConfig::from_jsonc(&contents)?;
    let frames = tests_config.shadowrun_cd().frames().frames();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].name(), "Actions Menu");
    assert_eq!(frames[0].rect(), Bounds::new(18, 86, 91, 68));
    let expected_result_ocr = tests_config.shadowrun_cd().expected_result_ocr();
    assert_eq!(expected_result_ocr.len(), 4);
    for (actual_text, expected_text) in
        expected_result_ocr
            .iter()
            .zip(["見る", "話す", "移動", "アイテム"])
    {
        assert_eq!(actual_text.as_str(), expected_text);
    }
    let expected_translation_result = tests_config.shadowrun_cd().expected_translation_result();
    assert_eq!(expected_translation_result.len(), 4);
    for (actual_text, expected_text) in
        expected_translation_result
            .iter()
            .zip(["See", "Talk", "Contact Us", "Item."])
    {
        assert_eq!(actual_text.as_str(), expected_text);
    }
    let serialized = tests_config.to_jsonc()?;
    let restored = TestsConfig::from_jsonc(&serialized)?;
    assert_eq!(restored, tests_config);
    Ok(())
}
