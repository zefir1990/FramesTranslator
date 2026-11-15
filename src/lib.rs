pub mod config;
pub mod factory;
pub mod frame;
pub mod frames_ocr;
pub mod frames_sourcer;
pub mod implementations;
pub mod text_region;
pub mod text_renderer;
pub mod text_translator;

pub use config::{Config, ConfigError};
pub use factory::{
    FactoryError, create_frames_ocr, create_frames_sourcer, create_text_renderer,
    create_text_translator,
};
pub use frame::{Frame, FrameError};
pub use frames_ocr::{FramesOCR, RecognitionError};
pub use frames_sourcer::{FrameSourceError, FramesSourcer};
pub use text_region::{Bounds, TextRegion, TranslatedText};
pub use text_renderer::{RenderingError, TextRenderer};
pub use text_translator::{TextTranslator, TranslationError};
