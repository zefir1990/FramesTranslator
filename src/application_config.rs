use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationConfig {
    frames_sourcer: String,
    frames_ocr: String,
    text_translator: String,
    text_renderer: String,
    frames_detection_background_color: u32,
    frames_detection_text_color: u32,
}

impl ApplicationConfig {
    pub fn new(
        frames_sourcer: impl Into<String>,
        frames_ocr: impl Into<String>,
        text_translator: impl Into<String>,
        text_renderer: impl Into<String>,
        frames_detection_background_color: u32,
        frames_detection_text_color: u32,
    ) -> Self {
        Self {
            frames_sourcer: frames_sourcer.into(),
            frames_ocr: frames_ocr.into(),
            text_translator: text_translator.into(),
            text_renderer: text_renderer.into(),
            frames_detection_background_color,
            frames_detection_text_color,
        }
    }

    pub fn frames_sourcer(&self) -> &str {
        &self.frames_sourcer
    }

    pub fn frames_ocr(&self) -> &str {
        &self.frames_ocr
    }

    pub fn text_translator(&self) -> &str {
        &self.text_translator
    }

    pub fn text_renderer(&self) -> &str {
        &self.text_renderer
    }

    pub fn frames_detection_background_color(&self) -> u32 {
        self.frames_detection_background_color
    }

    pub fn frames_detection_text_color(&self) -> u32 {
        self.frames_detection_text_color
    }

    pub fn from_jsonc(source: &str) -> Result<Self, json5::Error> {
        json5::from_str(source)
    }

    pub fn to_jsonc(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_config_exposes_implementation_names() {
        let application_config = ApplicationConfig::new(
            "ImageFramesSourcer",
            "FixtureFramesOCR",
            "DictionaryTextTranslator",
            "BlockTextRenderer",
            0x000000,
            0xFCFCFC,
        );
        assert_eq!(application_config.frames_sourcer(), "ImageFramesSourcer");
        assert_eq!(application_config.frames_ocr(), "FixtureFramesOCR");
        assert_eq!(
            application_config.text_translator(),
            "DictionaryTextTranslator"
        );
        assert_eq!(application_config.text_renderer(), "BlockTextRenderer");
        assert_eq!(
            application_config.frames_detection_background_color(),
            0x000000
        );
        assert_eq!(application_config.frames_detection_text_color(), 0xFCFCFC);
    }
}
