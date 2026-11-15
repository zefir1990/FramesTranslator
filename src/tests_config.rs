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
}

impl ShadowrunCdTestConfig {
    pub fn new(frames: FramesConfig) -> Self {
        Self { frames }
    }

    pub fn frames(&self) -> &FramesConfig {
        &self.frames
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
        let tests_config = TestsConfig::new(ShadowrunCdTestConfig::new(frames_config));
        let actions_menu = &tests_config.shadowrun_cd().frames().frames()[0];
        assert_eq!(actions_menu.name(), "Actions Menu");
        assert_eq!(actions_menu.rect(), Bounds::new(18, 86, 91, 68));
    }
}
