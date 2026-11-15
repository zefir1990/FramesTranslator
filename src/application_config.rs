use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationConfig {
    frames_sourcer: String,
    frames_ocr: String,
    text_translator: String,
    text_renderer: String,
    frames_detection_background_colors: Vec<u32>,
    frames_detection_text_colors: Vec<u32>,
    #[serde(default = "default_verbose_mode")]
    verbose_mode: bool,
}

impl ApplicationConfig {
    pub fn new(
        frames_sourcer: impl Into<String>,
        frames_ocr: impl Into<String>,
        text_translator: impl Into<String>,
        text_renderer: impl Into<String>,
        frames_detection_background_colors: Vec<u32>,
        frames_detection_text_colors: Vec<u32>,
        verbose_mode: bool,
    ) -> Self {
        Self {
            frames_sourcer: frames_sourcer.into(),
            frames_ocr: frames_ocr.into(),
            text_translator: text_translator.into(),
            text_renderer: text_renderer.into(),
            frames_detection_background_colors,
            frames_detection_text_colors,
            verbose_mode,
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

    pub fn frames_detection_background_colors(&self) -> &[u32] {
        &self.frames_detection_background_colors
    }

    pub fn frames_detection_text_colors(&self) -> &[u32] {
        &self.frames_detection_text_colors
    }

    pub fn verbose_mode(&self) -> bool {
        self.verbose_mode
    }

    pub fn from_jsonc(source: &str) -> Result<Self, json5::Error> {
        json5::from_str(source)
    }

    pub fn to_jsonc(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn default_verbose_mode() -> bool {
    true
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
            vec![0x000000],
            vec![0xFCFCFC, 0xFFFFFF, 0xFCFCB4],
            true,
        );
        assert_eq!(application_config.frames_sourcer(), "ImageFramesSourcer");
        assert_eq!(application_config.frames_ocr(), "FixtureFramesOCR");
        assert_eq!(
            application_config.text_translator(),
            "DictionaryTextTranslator"
        );
        assert_eq!(application_config.text_renderer(), "BlockTextRenderer");
        assert_eq!(
            application_config.frames_detection_background_colors(),
            [0x000000]
        );
        assert_eq!(
            application_config.frames_detection_text_colors(),
            [0xFCFCFC, 0xFFFFFF, 0xFCFCB4]
        );
        assert!(application_config.verbose_mode());
    }

    #[test]
    fn verbose_mode_defaults_to_true_when_missing() {
        let application_config = ApplicationConfig::from_jsonc(
            r#"{
                "framesSourcer": "FramesDetectorSourcer",
                "framesOcr": "MangaFramesOCR",
                "textTranslator": "ArgosTextTranslator",
                "textRenderer": "SdlThreeTextRenderer",
                "framesDetectionBackgroundColors": [0x000000],
                "framesDetectionTextColors": [0xFFFFFF]
            }"#,
        )
        .unwrap();
        assert!(application_config.verbose_mode());
    }
}
