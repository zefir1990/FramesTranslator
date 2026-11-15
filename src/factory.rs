use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::application_config::ApplicationConfig;
use crate::frames_ocr::{FramesOCR, RecognitionError};
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};
use crate::implementations::{
    ArgosTextTranslator, BlockTextRenderer, DictionaryTextTranslator, FixtureFramesOCR,
    ImageFramesSourcer, MangaFramesOCR, SdlThreeTextRenderer, SyntheticFramesSourcer,
};
use crate::tests_config::TestsConfig;
use crate::text_renderer::TextRenderer;
use crate::text_translator::{TextTranslator, TranslationError};

const DEFAULT_IMAGE_PATH: &str = "test-case/Shadowrun_02.png";
const DEFAULT_TESTS_CONFIG_PATH: &str = "test-case/tests-config.jsonc";

pub fn create_frames_sourcer(
    config: &ApplicationConfig,
) -> Result<Box<dyn FramesSourcer>, FactoryError> {
    match config.frames_sourcer() {
        "SyntheticFramesSourcer" => Ok(Box::new(SyntheticFramesSourcer::new())),
        "ImageFramesSourcer" => {
            let tests_config = load_tests_config(manifest_path(DEFAULT_TESTS_CONFIG_PATH))?;
            let image_path = manifest_path(DEFAULT_IMAGE_PATH);
            Ok(Box::new(ImageFramesSourcer::open(
                image_path,
                tests_config.shadowrun_cd().frames(),
            )?))
        }
        name => Err(FactoryError::UnknownImplementation {
            variable: "framesSourcer",
            name: name.to_owned(),
        }),
    }
}

pub fn create_frames_ocr(config: &ApplicationConfig) -> Result<Box<dyn FramesOCR>, FactoryError> {
    match config.frames_ocr() {
        "FixtureFramesOCR" => Ok(Box::new(FixtureFramesOCR::new())),
        "MangaFramesOCR" => Ok(Box::new(MangaFramesOCR::new()?)),
        name => Err(FactoryError::UnknownImplementation {
            variable: "framesOcr",
            name: name.to_owned(),
        }),
    }
}

pub fn create_text_translator(
    config: &ApplicationConfig,
) -> Result<Box<dyn TextTranslator>, FactoryError> {
    match config.text_translator() {
        "DictionaryTextTranslator" => Ok(Box::new(DictionaryTextTranslator::new())),
        "ArgosTextTranslator" => Ok(Box::new(ArgosTextTranslator::new()?)),
        name => Err(FactoryError::UnknownImplementation {
            variable: "textTranslator",
            name: name.to_owned(),
        }),
    }
}

pub fn create_text_renderer(
    config: &ApplicationConfig,
) -> Result<Box<dyn TextRenderer>, FactoryError> {
    match config.text_renderer() {
        "BlockTextRenderer" => Ok(Box::new(BlockTextRenderer::new())),
        "SdlThreeTextRenderer" => Ok(Box::new(SdlThreeTextRenderer::new())),
        name => Err(FactoryError::UnknownImplementation {
            variable: "textRenderer",
            name: name.to_owned(),
        }),
    }
}

fn load_tests_config(path: PathBuf) -> Result<TestsConfig, FactoryError> {
    let contents = fs::read_to_string(&path).map_err(|source| FactoryError::Io {
        path: path.clone(),
        source,
    })?;
    TestsConfig::from_jsonc(&contents).map_err(|source| FactoryError::TestsConfig { path, source })
}

fn manifest_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

#[derive(Debug)]
pub enum FactoryError {
    FrameSource(FrameSourceError),
    Recognition(RecognitionError),
    Translation(TranslationError),
    Io {
        path: PathBuf,
        source: io::Error,
    },
    TestsConfig {
        path: PathBuf,
        source: json5::Error,
    },
    UnknownImplementation {
        variable: &'static str,
        name: String,
    },
}

impl Display for FactoryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameSource(error) => write!(formatter, "{error}"),
            Self::Recognition(error) => write!(formatter, "{error}"),
            Self::Translation(error) => write!(formatter, "{error}"),
            Self::Io { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            Self::TestsConfig { path, source } => {
                write!(formatter, "failed to parse {}: {source}", path.display())
            }
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
            Self::Recognition(error) => Some(error),
            Self::Translation(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            Self::TestsConfig { source, .. } => Some(source),
            Self::UnknownImplementation { .. } => None,
        }
    }
}

impl From<FrameSourceError> for FactoryError {
    fn from(error: FrameSourceError) -> Self {
        Self::FrameSource(error)
    }
}

impl From<RecognitionError> for FactoryError {
    fn from(error: RecognitionError) -> Self {
        Self::Recognition(error)
    }
}

impl From<TranslationError> for FactoryError {
    fn from(error: TranslationError) -> Self {
        Self::Translation(error)
    }
}
