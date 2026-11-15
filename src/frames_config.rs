use crate::text_region::Bounds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedRect {
    name: String,
    rect: Bounds,
}

impl NamedRect {
    pub fn new(name: impl Into<String>, rect: Bounds) -> Self {
        Self {
            name: name.into(),
            rect,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn rect(&self) -> Bounds {
        self.rect
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FramesConfig {
    frames: Vec<NamedRect>,
}

impl FramesConfig {
    pub fn new(frames: Vec<NamedRect>) -> Self {
        Self { frames }
    }

    pub fn frames(&self) -> &[NamedRect] {
        &self.frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_config_exposes_named_frames() {
        let frames_config = FramesConfig::new(vec![
            NamedRect::new("left", Bounds::new(0, 0, 10, 20)),
            NamedRect::new("right", Bounds::new(10, 0, 10, 20)),
        ]);
        assert_eq!(frames_config.frames().len(), 2);
        assert_eq!(frames_config.frames()[0].name(), "left");
        assert_eq!(frames_config.frames()[0].rect(), Bounds::new(0, 0, 10, 20));
        assert_eq!(frames_config.frames()[1].name(), "right");
        assert_eq!(frames_config.frames()[1].rect(), Bounds::new(10, 0, 10, 20));
    }
}
