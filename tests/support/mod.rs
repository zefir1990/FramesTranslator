#![allow(dead_code)]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use frames_translator::application_config::ApplicationConfig;
use frames_translator::tests_config::TestsConfig;

const APPLICATION_CONFIG_PATH: &str = "test-case/tests-application-config.jsonc";
const TESTS_CONFIG_PATH: &str = "test-case/tests-config.jsonc";

pub fn load_application_config() -> Result<ApplicationConfig, Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(APPLICATION_CONFIG_PATH);
    let contents = fs::read_to_string(config_path)?;
    Ok(ApplicationConfig::from_jsonc(&contents)?)
}

pub fn load_tests_config() -> Result<TestsConfig, Box<dyn Error>> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TESTS_CONFIG_PATH);
    let contents = fs::read_to_string(config_path)?;
    Ok(TestsConfig::from_jsonc(&contents)?)
}
