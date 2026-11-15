use std::error::Error;
use std::fmt::{self, Display, Formatter};

use frames_translator::application_config::ApplicationConfig;
use frames_translator::frame_translation::translate_frame_regions;
use frames_translator::frames_detector_sourcer::FramesDetectorSourcer;
use frames_translator::frames_ocr::{FramesOCR, RecognitionError};
use frames_translator::frames_sourcer::FrameSourceError;
use frames_translator::sdl_frame_viewer::{
    FrameOverlay, FrameViewerError, Scene, run as run_frame_viewer,
};
use frames_translator::text_translator::{TextTranslator, TranslationError};

const WINDOW_TITLE: &str = "FramesTranslator";

pub fn run(
    application_config: &ApplicationConfig,
    frames_detector: &dyn FramesDetectorSourcer,
    frames_ocr: &dyn FramesOCR,
    text_translator: &mut dyn TextTranslator,
    initial_image: Option<&str>,
) -> Result<(), ApplicationError> {
    let mut build_scene = |image_path: &str| -> Result<Scene, ApplicationError> {
        load_scene(image_path, frames_detector, frames_ocr, text_translator)
    };
    run_frame_viewer(
        WINDOW_TITLE,
        application_config.verbose_mode(),
        initial_image,
        &mut build_scene,
    )
}

fn load_scene(
    image_path: &str,
    frames_detector: &dyn FramesDetectorSourcer,
    frames_ocr: &dyn FramesOCR,
    text_translator: &mut dyn TextTranslator,
) -> Result<Scene, ApplicationError> {
    let image = image::open(image_path)?.to_rgba8();
    let detected_frames = frames_detector.detect(&image)?;
    let mut overlays = Vec::with_capacity(detected_frames.len());
    for detected_frame in detected_frames {
        let regions = frames_ocr.recognize(detected_frame.frame())?;
        let translated_texts =
            translate_frame_regions(detected_frame.frame(), &regions, text_translator)?;
        overlays.push(FrameOverlay::with_translated_texts(
            detected_frame.bounds(),
            &translated_texts,
        ));
    }
    Ok(Scene::new(image, overlays))
}

#[derive(Debug)]
pub enum ApplicationError {
    FrameSource(FrameSourceError),
    Recognition(RecognitionError),
    Translation(TranslationError),
    Image(image::ImageError),
    Viewer(FrameViewerError),
}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameSource(error) => write!(formatter, "{error}"),
            Self::Recognition(error) => write!(formatter, "{error}"),
            Self::Translation(error) => write!(formatter, "{error}"),
            Self::Image(error) => write!(formatter, "failed to load image: {error}"),
            Self::Viewer(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FrameSource(error) => Some(error),
            Self::Recognition(error) => Some(error),
            Self::Translation(error) => Some(error),
            Self::Image(error) => Some(error),
            Self::Viewer(error) => Some(error),
        }
    }
}

impl From<FrameSourceError> for ApplicationError {
    fn from(error: FrameSourceError) -> Self {
        Self::FrameSource(error)
    }
}

impl From<RecognitionError> for ApplicationError {
    fn from(error: RecognitionError) -> Self {
        Self::Recognition(error)
    }
}

impl From<TranslationError> for ApplicationError {
    fn from(error: TranslationError) -> Self {
        Self::Translation(error)
    }
}

impl From<image::ImageError> for ApplicationError {
    fn from(error: image::ImageError) -> Self {
        Self::Image(error)
    }
}

impl From<FrameViewerError> for ApplicationError {
    fn from(error: FrameViewerError) -> Self {
        Self::Viewer(error)
    }
}
