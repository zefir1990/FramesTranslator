use std::collections::BTreeMap;

use crate::frames_config::FramesConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestsConfig {
    shadowrun_cd: ShadowrunCdTestConfig,
    shadowrun_dialog: ShadowrunDialogTestConfig,
    shadowrun_street: ShadowrunStreetTestConfig,
    shadowrun_hospital: ShadowrunHospitalTestConfig,
}

impl TestsConfig {
    pub fn new(
        shadowrun_cd: ShadowrunCdTestConfig,
        shadowrun_dialog: ShadowrunDialogTestConfig,
        shadowrun_street: ShadowrunStreetTestConfig,
        shadowrun_hospital: ShadowrunHospitalTestConfig,
    ) -> Self {
        Self {
            shadowrun_cd,
            shadowrun_dialog,
            shadowrun_street,
            shadowrun_hospital,
        }
    }

    pub fn shadowrun_cd(&self) -> &ShadowrunCdTestConfig {
        &self.shadowrun_cd
    }

    pub fn shadowrun_dialog(&self) -> &ShadowrunDialogTestConfig {
        &self.shadowrun_dialog
    }

    pub fn shadowrun_street(&self) -> &ShadowrunStreetTestConfig {
        &self.shadowrun_street
    }

    pub fn shadowrun_hospital(&self) -> &ShadowrunHospitalTestConfig {
        &self.shadowrun_hospital
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
    expected_result_ocr: BTreeMap<String, Vec<String>>,
    #[serde(rename = "expectedTranslationResult")]
    expected_translation_result: BTreeMap<String, Vec<String>>,
    #[serde(rename = "timeoutSeconds")]
    timeout_seconds: u64,
}

impl ShadowrunCdTestConfig {
    pub fn new(
        frames: FramesConfig,
        expected_result_ocr: BTreeMap<String, Vec<String>>,
        expected_translation_result: BTreeMap<String, Vec<String>>,
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

    pub fn expected_result_ocr(&self) -> &BTreeMap<String, Vec<String>> {
        &self.expected_result_ocr
    }

    pub fn expected_translation_result(&self) -> &BTreeMap<String, Vec<String>> {
        &self.expected_translation_result
    }

    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowrunDialogTestConfig {
    #[serde(flatten)]
    frames: FramesConfig,
    #[serde(rename = "expectedResultOCR")]
    expected_result_ocr: BTreeMap<String, Vec<String>>,
    #[serde(rename = "expectedTranslationResult")]
    expected_translation_result: BTreeMap<String, Vec<String>>,
    #[serde(rename = "timeoutSeconds")]
    timeout_seconds: u64,
}

impl ShadowrunDialogTestConfig {
    pub fn new(
        frames: FramesConfig,
        expected_result_ocr: BTreeMap<String, Vec<String>>,
        expected_translation_result: BTreeMap<String, Vec<String>>,
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

    pub fn expected_result_ocr(&self) -> &BTreeMap<String, Vec<String>> {
        &self.expected_result_ocr
    }

    pub fn expected_translation_result(&self) -> &BTreeMap<String, Vec<String>> {
        &self.expected_translation_result
    }

    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowrunStreetTestConfig {
    #[serde(flatten)]
    frames: FramesConfig,
    #[serde(rename = "expectedResultOCR")]
    expected_result_ocr: BTreeMap<String, Vec<String>>,
    #[serde(rename = "expectedTranslationResult")]
    expected_translation_result: BTreeMap<String, Vec<String>>,
    #[serde(rename = "timeoutSeconds")]
    timeout_seconds: u64,
}

impl ShadowrunStreetTestConfig {
    pub fn new(
        frames: FramesConfig,
        expected_result_ocr: BTreeMap<String, Vec<String>>,
        expected_translation_result: BTreeMap<String, Vec<String>>,
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

    pub fn expected_result_ocr(&self) -> &BTreeMap<String, Vec<String>> {
        &self.expected_result_ocr
    }

    pub fn expected_translation_result(&self) -> &BTreeMap<String, Vec<String>> {
        &self.expected_translation_result
    }

    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowrunHospitalTestConfig {
    #[serde(flatten)]
    frames: FramesConfig,
    #[serde(rename = "expectedResultOCR")]
    expected_result_ocr: BTreeMap<String, Vec<String>>,
    #[serde(rename = "expectedTranslationResult")]
    expected_translation_result: BTreeMap<String, Vec<String>>,
    #[serde(rename = "timeoutSeconds")]
    timeout_seconds: u64,
}

impl ShadowrunHospitalTestConfig {
    pub fn new(
        frames: FramesConfig,
        expected_result_ocr: BTreeMap<String, Vec<String>>,
        expected_translation_result: BTreeMap<String, Vec<String>>,
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

    pub fn expected_result_ocr(&self) -> &BTreeMap<String, Vec<String>> {
        &self.expected_result_ocr
    }

    pub fn expected_translation_result(&self) -> &BTreeMap<String, Vec<String>> {
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
        let frames_config = FramesConfig::new(vec![
            NamedRect::new("Actions Menu", Bounds::new(18, 86, 91, 68)),
            NamedRect::new("Contacts", Bounds::new(146, 86, 75, 68)),
        ]);
        let expected_result_ocr = BTreeMap::from([
            (
                "Actions Menu".to_owned(),
                ["見る", "話す", "移動", "アイテム"]
                    .map(str::to_owned)
                    .to_vec(),
            ),
            (
                "Contacts".to_owned(),
                ["仲間", "通行人１", "通行人２", "通行人３"]
                    .map(str::to_owned)
                    .to_vec(),
            ),
        ]);
        let expected_translation_result = BTreeMap::from([
            (
                "Actions Menu".to_owned(),
                ["See", "Talk", "Contact Us", "Item."]
                    .map(str::to_owned)
                    .to_vec(),
            ),
            (
                "Contacts".to_owned(),
                ["Contact Us", "Traffic1.", "person 2.", "Traffic 3."]
                    .map(str::to_owned)
                    .to_vec(),
            ),
        ]);
        let tests_config = TestsConfig::new(
            ShadowrunCdTestConfig::new(
                frames_config,
                expected_result_ocr.clone(),
                expected_translation_result.clone(),
                5,
            ),
            empty_dialog_config(),
            empty_street_config(),
            empty_hospital_config(),
        );
        let shadowrun_cd = tests_config.shadowrun_cd();
        assert_eq!(shadowrun_cd.frames().frames().len(), 2);
        let actions_menu = &shadowrun_cd.frames().frames()[0];
        assert_eq!(actions_menu.name(), "Actions Menu");
        assert_eq!(actions_menu.rect(), Bounds::new(18, 86, 91, 68));
        let contacts = &shadowrun_cd.frames().frames()[1];
        assert_eq!(contacts.name(), "Contacts");
        assert_eq!(contacts.rect(), Bounds::new(146, 86, 75, 68));
        assert_eq!(shadowrun_cd.expected_result_ocr(), &expected_result_ocr);
        assert_eq!(
            shadowrun_cd.expected_translation_result(),
            &expected_translation_result
        );
        assert_eq!(shadowrun_cd.timeout_seconds(), 5);
    }

    #[test]
    fn tests_config_exposes_shadowrun_dialog_test_config() {
        let frames_config = FramesConfig::new(vec![NamedRect::new(
            "Dialog",
            Bounds::new(26, 158, 267, 52),
        )]);
        let expected_result_ocr = BTreeMap::from([(
            "Dialog".to_owned(),
            ["意外に手ごたえのない", "連中だったわね"]
                .map(str::to_owned)
                .to_vec(),
        )]);
        let expected_translation_result = BTreeMap::from([(
            "Dialog".to_owned(),
            ["Unexpected", "It was a long time."]
                .map(str::to_owned)
                .to_vec(),
        )]);
        let tests_config = TestsConfig::new(
            empty_cd_config(),
            ShadowrunDialogTestConfig::new(
                frames_config,
                expected_result_ocr.clone(),
                expected_translation_result.clone(),
                5,
            ),
            empty_street_config(),
            empty_hospital_config(),
        );
        let shadowrun_dialog = tests_config.shadowrun_dialog();
        assert_eq!(shadowrun_dialog.frames().frames().len(), 1);
        let dialog = &shadowrun_dialog.frames().frames()[0];
        assert_eq!(dialog.name(), "Dialog");
        assert_eq!(dialog.rect(), Bounds::new(26, 158, 267, 52));
        assert_eq!(shadowrun_dialog.expected_result_ocr(), &expected_result_ocr);
        assert_eq!(
            shadowrun_dialog.expected_translation_result(),
            &expected_translation_result
        );
        assert_eq!(shadowrun_dialog.timeout_seconds(), 5);
    }

    #[test]
    fn tests_config_exposes_shadowrun_street_test_config() {
        let frames_config = FramesConfig::new(vec![NamedRect::new(
            "Street Dialog",
            Bounds::new(26, 158, 267, 52),
        )]);
        let expected_result_ocr = BTreeMap::from([(
            "Street Dialog".to_owned(),
            ["渋谷は", "とてもきらびやかな街だ"]
                .map(str::to_owned)
                .to_vec(),
        )]);
        let expected_translation_result = BTreeMap::from([(
            "Street Dialog".to_owned(),
            ["Shibuya", "A very nice city"].map(str::to_owned).to_vec(),
        )]);
        let tests_config = TestsConfig::new(
            empty_cd_config(),
            empty_dialog_config(),
            ShadowrunStreetTestConfig::new(
                frames_config,
                expected_result_ocr.clone(),
                expected_translation_result.clone(),
                5,
            ),
            empty_hospital_config(),
        );
        let shadowrun_street = tests_config.shadowrun_street();
        assert_eq!(shadowrun_street.frames().frames().len(), 1);
        let street_dialog = &shadowrun_street.frames().frames()[0];
        assert_eq!(street_dialog.name(), "Street Dialog");
        assert_eq!(street_dialog.rect(), Bounds::new(26, 158, 267, 52));
        assert_eq!(shadowrun_street.expected_result_ocr(), &expected_result_ocr);
        assert_eq!(
            shadowrun_street.expected_translation_result(),
            &expected_translation_result
        );
        assert_eq!(shadowrun_street.timeout_seconds(), 5);
    }

    #[test]
    fn tests_config_exposes_shadowrun_hospital_test_config() {
        let frames_config = FramesConfig::new(vec![NamedRect::new(
            "Hospital Dialog",
            Bounds::new(39, 261, 411, 84),
        )]);
        let expected_result_ocr = BTreeMap::from([(
            "Hospital Dialog".to_owned(),
            ["おまえさんに", "尻をたたかれて", "やる気になったはいいが"]
                .map(str::to_owned)
                .to_vec(),
        )]);
        let expected_translation_result = BTreeMap::from([(
            "Hospital Dialog".to_owned(),
            ["I'd like to", "be motivated by", "my wife."]
                .map(str::to_owned)
                .to_vec(),
        )]);
        let tests_config = TestsConfig::new(
            empty_cd_config(),
            empty_dialog_config(),
            empty_street_config(),
            ShadowrunHospitalTestConfig::new(
                frames_config,
                expected_result_ocr.clone(),
                expected_translation_result.clone(),
                5,
            ),
        );
        let shadowrun_hospital = tests_config.shadowrun_hospital();
        assert_eq!(shadowrun_hospital.frames().frames().len(), 1);
        let hospital_dialog = &shadowrun_hospital.frames().frames()[0];
        assert_eq!(hospital_dialog.name(), "Hospital Dialog");
        assert_eq!(hospital_dialog.rect(), Bounds::new(39, 261, 411, 84));
        assert_eq!(
            shadowrun_hospital.expected_result_ocr(),
            &expected_result_ocr
        );
        assert_eq!(
            shadowrun_hospital.expected_translation_result(),
            &expected_translation_result
        );
        assert_eq!(shadowrun_hospital.timeout_seconds(), 5);
    }

    fn empty_cd_config() -> ShadowrunCdTestConfig {
        ShadowrunCdTestConfig::new(
            FramesConfig::new(Vec::new()),
            BTreeMap::new(),
            BTreeMap::new(),
            5,
        )
    }

    fn empty_dialog_config() -> ShadowrunDialogTestConfig {
        ShadowrunDialogTestConfig::new(
            FramesConfig::new(Vec::new()),
            BTreeMap::new(),
            BTreeMap::new(),
            5,
        )
    }

    fn empty_street_config() -> ShadowrunStreetTestConfig {
        ShadowrunStreetTestConfig::new(
            FramesConfig::new(Vec::new()),
            BTreeMap::new(),
            BTreeMap::new(),
            5,
        )
    }

    fn empty_hospital_config() -> ShadowrunHospitalTestConfig {
        ShadowrunHospitalTestConfig::new(
            FramesConfig::new(Vec::new()),
            BTreeMap::new(),
            BTreeMap::new(),
            5,
        )
    }
}
