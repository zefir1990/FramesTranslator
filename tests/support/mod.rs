use std::error::Error;
use std::fs;
use std::path::PathBuf;

use frames_translator::application_config::ApplicationConfig;

const CONFIG_PATH: &str = "test-case/tests-application-config.jsonc";

pub fn load_application_config() -> Result<ApplicationConfig, Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CONFIG_PATH);
    let contents = fs::read_to_string(config_path)?;
    Ok(ApplicationConfig::from_jsonc(&contents)?)
}
