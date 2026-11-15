use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::text_region::{TextRegion, TranslatedText};

pub trait TextTranslator {
    fn translate(&mut self, text: &str) -> Result<String, TranslationError>;

    fn translate_all(
        &mut self,
        regions: &[TextRegion],
    ) -> Result<Vec<TranslatedText>, TranslationError> {
        regions
            .iter()
            .map(|region| {
                let translated_text = self.translate(region.text())?;
                Ok(TranslatedText::new(region.clone(), translated_text))
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationError {
    Unavailable(String),
    InvalidResponse(String),
}

impl Display for TranslationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => {
                write!(formatter, "translation service is unavailable: {message}")
            }
            Self::InvalidResponse(message) => {
                write!(formatter, "translation response is invalid: {message}")
            }
        }
    }
}

impl Error for TranslationError {}
