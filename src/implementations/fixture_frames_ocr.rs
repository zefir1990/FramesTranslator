use crate::frame::Frame;
use crate::frames_ocr::{FramesOCR, RecognitionError};
use crate::text_region::{Bounds, TextRegion};

const REGIONS: [(&str, u32, u32, u32, u32); 2] = [
    ("Shadowrun", 10, 8, 40, 12),
    ("Second Edition", 10, 28, 40, 12),
];

#[derive(Debug, Clone)]
pub struct FixtureFramesOCR {
    regions: Vec<TextRegion>,
}

impl FixtureFramesOCR {
    pub fn new() -> Self {
        Self {
            regions: REGIONS
                .iter()
                .map(|(text, x, y, width, height)| {
                    TextRegion::new(Bounds::new(*x, *y, *width, *height), *text)
                })
                .collect(),
        }
    }
}

impl Default for FixtureFramesOCR {
    fn default() -> Self {
        Self::new()
    }
}

impl FramesOCR for FixtureFramesOCR {
    fn recognize(&self, _frame: &Frame) -> Result<Vec<TextRegion>, RecognitionError> {
        Ok(self.regions.clone())
    }
}
