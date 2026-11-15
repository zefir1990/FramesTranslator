use std::error::Error;
use std::fs;
use std::path::PathBuf;

use frames_translator::application_config::ApplicationConfig;

const CONFIG_PATH: &str = "test-case/tests-application-config.jsonc";

#[test]
fn application_config_serializes_and_deserializes_jsonc() -> Result<(), Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CONFIG_PATH);
    let contents = fs::read_to_string(config_path)?;
    let application_config = ApplicationConfig::from_jsonc(&contents)?;
    assert_eq!(application_config.frames_sourcer(), "ImageFramesSourcer");
    assert_eq!(application_config.frames_ocr(), "FixtureFramesOCR");
    assert_eq!(
        application_config.text_translator(),
        "DictionaryTextTranslator"
    );
    assert_eq!(application_config.text_renderer(), "BlockTextRenderer");
    let serialized = application_config.to_jsonc()?;
    let serialized_value: serde_json::Value = serde_json::from_str(&serialized)?;
    for key in [
        "framesSourcer",
        "framesOcr",
        "textTranslator",
        "textRenderer",
    ] {
        assert!(serialized_value.get(key).is_some());
    }
    let restored = ApplicationConfig::from_jsonc(&serialized)?;
    assert_eq!(restored, application_config);
    Ok(())
}
