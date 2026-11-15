use std::error::Error;
use std::fs;
use std::path::PathBuf;

use frames_translator::{
    ApplicationConfig, create_frames_ocr, create_frames_sourcer, create_text_renderer,
    create_text_translator,
};

const APPLICATION_CONFIG_PATH: &str = "test-case/tests-application-config.jsonc";

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(APPLICATION_CONFIG_PATH);
    let config_source = fs::read_to_string(config_path)?;
    let application_config = ApplicationConfig::from_jsonc(&config_source)?;
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let frames_ocr = create_frames_ocr(&application_config)?;
    let mut text_translator = create_text_translator(&application_config)?;
    let text_renderer = create_text_renderer(&application_config)?;
    while let Some(frame) = frames_sourcer.next_frame()? {
        let regions = frames_ocr.recognize(&frame)?;
        let translated_texts = text_translator.translate_all(&regions)?;
        text_renderer.render(&frame, &translated_texts)?;
    }
    Ok(())
}
