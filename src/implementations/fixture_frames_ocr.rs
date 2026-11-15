use std::fs;
use std::path::Path;

use crate::config::{Config, ConfigError};
use crate::frame::Frame;
use crate::frames_ocr::{FramesOCR, RecognitionError};
use crate::text_region::{Bounds, TextRegion};

#[derive(Debug, Clone)]
pub struct FixtureFramesOCR {
    regions: Vec<TextRegion>,
}

impl FixtureFramesOCR {
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        let path = config.resolve_path("FRAMES_OCR_FIXTURE_PATH")?;
        let contents = fs::read_to_string(&path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        let regions = parse_regions(&path, &contents)?;
        Ok(Self { regions })
    }
}

impl FramesOCR for FixtureFramesOCR {
    fn recognize(&self, _frame: &Frame) -> Result<Vec<TextRegion>, RecognitionError> {
        Ok(self.regions.clone())
    }
}

fn parse_regions(path: &Path, contents: &str) -> Result<Vec<TextRegion>, ConfigError> {
    let mut regions = Vec::new();
    for (line_index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        regions.push(parse_region(path, line_index + 1, line)?);
    }
    Ok(regions)
}

fn parse_region(path: &Path, line: usize, line_text: &str) -> Result<TextRegion, ConfigError> {
    let (bounds_text, text) =
        line_text
            .split_once('|')
            .ok_or_else(|| ConfigError::MalformedFileEntry {
                path: path.to_path_buf(),
                line,
            })?;
    let bounds = parse_bounds(bounds_text).ok_or_else(|| ConfigError::MalformedFileEntry {
        path: path.to_path_buf(),
        line,
    })?;
    let text = text.trim();
    if text.is_empty() {
        return Err(ConfigError::MalformedFileEntry {
            path: path.to_path_buf(),
            line,
        });
    }
    Ok(TextRegion::new(bounds, text))
}

fn parse_bounds(bounds_text: &str) -> Option<Bounds> {
    let components: Vec<&str> = bounds_text.split(',').map(str::trim).collect();
    let [x, y, width, height] = components.as_slice() else {
        return None;
    };
    Some(Bounds::new(
        x.parse().ok()?,
        y.parse().ok()?,
        width.parse().ok()?,
        height.parse().ok()?,
    ))
}
