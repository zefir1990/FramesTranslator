use std::path::PathBuf;

use frames_translator::config::{Config, ConfigError};

pub fn load_config() -> Result<Config, ConfigError> {
    Config::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env"))
}
