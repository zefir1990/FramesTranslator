use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use crate::application_config::ApplicationConfig;
use crate::frames_ocr::FramesOCR;
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};
use crate::implementations::{
    BlockTextRenderer, DictionaryTextTranslator, FixtureFramesOCR, ImageFramesSourcer,
    SyntheticFramesSourcer,
};
use crate::text_renderer::TextRenderer;
use crate::text_translator::TextTranslator;

const DEFAULT_IMAGE_PATH: &str = "test-case/Shadowrun_02.png";

pub fn create_frames_sourcer(
    config: &ApplicationConfig,
) -> Result<Box<dyn FramesSourcer>, FactoryError> {
    match config.frames_sourcer() {
        "SyntheticFramesSourcer" => Ok(Box::new(SyntheticFramesSourcer::new())),
        "ImageFramesSourcer" => {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_IMAGE_PATH);
            Ok(Box::new(ImageFramesSourcer::open(path)?))
        }
        name => Err(FactoryError::UnknownImplementation {
            variable: "FRAMES_SOURCER",
            name: name.to_owned(),
        }),
    }
}

pub fn create_frames_ocr(config: &ApplicationConfig) -> Result<Box<dyn FramesOCR>, FactoryError> {
    match config.frames_ocr() {
        "FixtureFramesOCR" => Ok(Box::new(FixtureFramesOCR::new())),
        name => Err(FactoryError::UnknownImplementation {
            variable: "FRAMES_OCR",
            name: name.to_owned(),
        }),
    }
}

pub fn create_text_translator(
    config: &ApplicationConfig,
) -> Result<Box<dyn TextTranslator>, FactoryError> {
    match config.text_translator() {
        "DictionaryTextTranslator" => Ok(Box::new(DictionaryTextTranslator::new())),
        name => Err(FactoryError::UnknownImplementation {
            variable: "TEXT_TRANSLATOR",
            name: name.to_owned(),
        }),
    }
}

pub fn create_text_renderer(
    config: &ApplicationConfig,
) -> Result<Box<dyn TextRenderer>, FactoryError> {
    match config.text_renderer() {
        "BlockTextRenderer" => Ok(Box::new(BlockTextRenderer::new())),
        name => Err(FactoryError::UnknownImplementation {
            variable: "TEXT_RENDERER",
            name: name.to_owned(),
        }),
    }
}

#[derive(Debug)]
pub enum FactoryError {
    FrameSource(FrameSourceError),
    UnknownImplementation {
        variable: &'static str,
        name: String,
    },
}

impl Display for FactoryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameSource(error) => write!(formatter, "{error}"),
            Self::UnknownImplementation { variable, name } => {
                write!(formatter, "unknown implementation {name:?} for {variable}")
            }
        }
    }
}

impl Error for FactoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FrameSource(error) => Some(error),
            Self::UnknownImplementation { .. } => None,
        }
    }
}

impl From<FrameSourceError> for FactoryError {
    fn from(error: FrameSourceError) -> Self {
        Self::FrameSource(error)
    }
}
