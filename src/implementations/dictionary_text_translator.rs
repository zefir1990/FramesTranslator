use std::collections::HashMap;

use crate::text_translator::{TextTranslator, TranslationError};

const TRANSLATIONS: [(&str, &str); 2] = [
    ("Shadowrun", "Shadowrun (translated)"),
    ("Second Edition", "Second Edition (translated)"),
];

#[derive(Debug, Clone)]
pub struct DictionaryTextTranslator {
    translations: HashMap<String, String>,
}

impl DictionaryTextTranslator {
    pub fn new() -> Self {
        Self {
            translations: TRANSLATIONS
                .iter()
                .map(|(source, target)| ((*source).to_owned(), (*target).to_owned()))
                .collect(),
        }
    }
}

impl Default for DictionaryTextTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl TextTranslator for DictionaryTextTranslator {
    fn translate(&mut self, text: &str) -> Result<String, TranslationError> {
        self.translations.get(text).cloned().ok_or_else(|| {
            TranslationError::Unavailable(format!("no dictionary entry for {text:?}"))
        })
    }
}
