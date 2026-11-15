mod sdl_image_translator;

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use frames_translator::ApplicationConfig;
use frames_translator::factory::{
    create_frames_detector_sourcer, create_frames_ocr, create_text_translator,
};

const APPLICATION_CONFIG_PATH: &str = "test-case/tests-application-config.jsonc";

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(APPLICATION_CONFIG_PATH);
    let config_source = fs::read_to_string(config_path)?;
    let application_config = ApplicationConfig::from_jsonc(&config_source)?;
    let frames_detector = create_frames_detector_sourcer(&application_config)?;
    let frames_ocr = create_frames_ocr(&application_config)?;
    let mut text_translator = create_text_translator(&application_config)?;
    let initial_image = std::env::args().nth(1);
    sdl_image_translator::run(
        &application_config,
        frames_detector.as_ref(),
        frames_ocr.as_ref(),
        text_translator.as_mut(),
        initial_image.as_deref(),
    )?;
    Ok(())
}
