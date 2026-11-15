use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::config::{Config, ConfigError};
use crate::frames_ocr::FramesOCR;
use crate::frames_sourcer::FramesSourcer;
use crate::implementations::{
    BlockTextRenderer, DictionaryTextTranslator, FixtureFramesOCR, SyntheticFramesSourcer,
};
use crate::text_renderer::TextRenderer;
use crate::text_translator::TextTranslator;

pub fn create_frames_sourcer(config: &Config) -> Result<Box<dyn FramesSourcer>, FactoryError> {
    match config.require("FRAMES_SOURCER")? {
        "synthetic" => Ok(Box::new(SyntheticFramesSourcer::from_config(config)?)),
        name => Err(FactoryError::UnknownImplementation {
            variable: "FRAMES_SOURCER",
            name: name.to_owned(),
        }),
    }
}

pub fn create_frames_ocr(config: &Config) -> Result<Box<dyn FramesOCR>, FactoryError> {
    match config.require("FRAMES_OCR")? {
        "fixture" => Ok(Box::new(FixtureFramesOCR::from_config(config)?)),
        name => Err(FactoryError::UnknownImplementation {
            variable: "FRAMES_OCR",
            name: name.to_owned(),
        }),
    }
}

pub fn create_text_translator(config: &Config) -> Result<Box<dyn TextTranslator>, FactoryError> {
    match config.require("TEXT_TRANSLATOR")? {
        "dictionary" => Ok(Box::new(DictionaryTextTranslator::from_config(config)?)),
        name => Err(FactoryError::UnknownImplementation {
            variable: "TEXT_TRANSLATOR",
            name: name.to_owned(),
        }),
    }
}

pub fn create_text_renderer(config: &Config) -> Result<Box<dyn TextRenderer>, FactoryError> {
    match config.require("TEXT_RENDERER")? {
        "block" => Ok(Box::new(BlockTextRenderer::from_config(config)?)),
        name => Err(FactoryError::UnknownImplementation {
            variable: "TEXT_RENDERER",
            name: name.to_owned(),
        }),
    }
}

#[derive(Debug)]
pub enum FactoryError {
    Config(ConfigError),
    UnknownImplementation {
        variable: &'static str,
        name: String,
    },
}

impl Display for FactoryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "{error}"),
            Self::UnknownImplementation { variable, name } => {
                write!(formatter, "unknown implementation {name:?} for {variable}")
            }
        }
    }
}

impl Error for FactoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Config(error) => Some(error),
            Self::UnknownImplementation { .. } => None,
        }
    }
}

impl From<ConfigError> for FactoryError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}
