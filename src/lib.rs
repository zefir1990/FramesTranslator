pub mod application_config;
pub mod factory;
pub mod frame;
pub mod frame_translation;
pub mod frames_config;
pub mod frames_ocr;
pub mod frames_sourcer;
pub mod implementations;
pub mod tests_config;
pub mod text_region;
pub mod text_renderer;
pub mod text_translator;

pub use application_config::ApplicationConfig;
pub use factory::{
    FactoryError, create_frames_ocr, create_frames_sourcer, create_text_renderer,
    create_text_translator,
};
pub use frame::{Frame, FrameError};
pub use frame_translation::translate_frame_regions;
pub use frames_config::{FramesConfig, NamedRect};
pub use frames_ocr::{FramesOCR, RecognitionError};
pub use frames_sourcer::{FrameSourceError, FramesSourcer};
pub use tests_config::{ShadowrunCdTestConfig, TestsConfig};
pub use text_region::{Bounds, TextRegion, TranslatedText};
pub use text_renderer::{RenderingError, TextRenderer};
pub use text_translator::{TextTranslator, TranslationError};
