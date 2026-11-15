use crate::frames_config::FramesConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestsConfig {
    shadowrun_cd: ShadowrunCdTestConfig,
}

impl TestsConfig {
    pub fn new(shadowrun_cd: ShadowrunCdTestConfig) -> Self {
        Self { shadowrun_cd }
    }

    pub fn shadowrun_cd(&self) -> &ShadowrunCdTestConfig {
        &self.shadowrun_cd
    }

    pub fn from_jsonc(source: &str) -> Result<Self, json5::Error> {
        json5::from_str(source)
    }

    pub fn to_jsonc(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowrunCdTestConfig {
    #[serde(flatten)]
    frames: FramesConfig,
    #[serde(rename = "expectedResultOCR")]
    expected_result_ocr: Vec<String>,
    #[serde(rename = "expectedTranslationResult")]
    expected_translation_result: Vec<String>,
    #[serde(rename = "timeoutSeconds")]
    timeout_seconds: u64,
}

impl ShadowrunCdTestConfig {
    pub fn new(
        frames: FramesConfig,
        expected_result_ocr: Vec<String>,
        expected_translation_result: Vec<String>,
        timeout_seconds: u64,
    ) -> Self {
        Self {
            frames,
            expected_result_ocr,
            expected_translation_result,
            timeout_seconds,
        }
    }

    pub fn frames(&self) -> &FramesConfig {
        &self.frames
    }

    pub fn expected_result_ocr(&self) -> &[String] {
        &self.expected_result_ocr
    }

    pub fn expected_translation_result(&self) -> &[String] {
        &self.expected_translation_result
    }

    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frames_config::NamedRect;
    use crate::text_region::Bounds;

    #[test]
    fn tests_config_exposes_shadowrun_cd_test_config() {
        let frames_config = FramesConfig::new(vec![NamedRect::new(
            "Actions Menu",
            Bounds::new(18, 86, 91, 68),
        )]);
        let expected_result_ocr = vec![
            "見る".to_owned(),
            "話す".to_owned(),
            "移動".to_owned(),
            "アイテム".to_owned(),
        ];
        let expected_translation_result = vec![
            "See".to_owned(),
            "Talk".to_owned(),
            "Contact Us".to_owned(),
            "Item.".to_owned(),
        ];
        let tests_config = TestsConfig::new(ShadowrunCdTestConfig::new(
            frames_config,
            expected_result_ocr.clone(),
            expected_translation_result.clone(),
            5,
        ));
        let shadowrun_cd = tests_config.shadowrun_cd();
        let actions_menu = &shadowrun_cd.frames().frames()[0];
        assert_eq!(actions_menu.name(), "Actions Menu");
        assert_eq!(actions_menu.rect(), Bounds::new(18, 86, 91, 68));
        assert_eq!(shadowrun_cd.expected_result_ocr(), expected_result_ocr);
        assert_eq!(
            shadowrun_cd.expected_translation_result(),
            expected_translation_result
        );
        assert_eq!(shadowrun_cd.timeout_seconds(), 5);
    }
}
