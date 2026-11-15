use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{Config, ConfigError};
use crate::text_translator::{TextTranslator, TranslationError};

#[derive(Debug, Clone)]
pub struct DictionaryTextTranslator {
    translations: HashMap<String, String>,
}

impl DictionaryTextTranslator {
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        let path = config.resolve_path("TEXT_TRANSLATOR_DICTIONARY_PATH")?;
        let contents = fs::read_to_string(&path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        Ok(Self {
            translations: parse_translations(&path, &contents)?,
        })
    }
}

impl TextTranslator for DictionaryTextTranslator {
    fn translate(&mut self, text: &str) -> Result<String, TranslationError> {
        self.translations.get(text).cloned().ok_or_else(|| {
            TranslationError::Unavailable(format!("no dictionary entry for {text:?}"))
        })
    }
}

fn parse_translations(path: &Path, contents: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut translations = HashMap::new();
    for (line_index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (source, target) =
            line.split_once('=')
                .ok_or_else(|| ConfigError::MalformedFileEntry {
                    path: path.to_path_buf(),
                    line: line_index + 1,
                })?;
        let source = source.trim();
        let target = target.trim();
        if source.is_empty() || target.is_empty() {
            return Err(ConfigError::MalformedFileEntry {
                path: path.to_path_buf(),
                line: line_index + 1,
            });
        }
        translations.insert(source.to_owned(), target.to_owned());
    }
    Ok(translations)
}
