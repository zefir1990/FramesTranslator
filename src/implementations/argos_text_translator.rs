use serde::{Deserialize, Serialize};

use crate::text_translator::{TextTranslator, TranslationError};

use super::python_worker::{PythonWorker, manifest_path};

const PYTHON_EXECUTABLE_PATH: &str = ".venv/Scripts/python.exe";
const TRANSLATION_SCRIPT_PATH: &str = "scripts/argos_translate_worker.py";
const DEFAULT_FROM_LANGUAGE: &str = "ja";
const DEFAULT_TO_LANGUAGE: &str = "en";

pub struct ArgosTextTranslator {
    worker: PythonWorker,
}

impl ArgosTextTranslator {
    pub fn new() -> Result<Self, TranslationError> {
        Self::with_languages(DEFAULT_FROM_LANGUAGE, DEFAULT_TO_LANGUAGE)
    }

    pub fn with_languages(
        from_language: impl Into<String>,
        to_language: impl Into<String>,
    ) -> Result<Self, TranslationError> {
        let from_language = from_language.into();
        let to_language = to_language.into();
        let mut worker = PythonWorker::spawn(
            &manifest_path(PYTHON_EXECUTABLE_PATH),
            &manifest_path(TRANSLATION_SCRIPT_PATH),
            &[from_language.as_str(), to_language.as_str()],
        )
        .map_err(TranslationError::Unavailable)?;
        let readiness = worker
            .read_line()
            .map_err(TranslationError::Unavailable)?
            .ok_or_else(|| {
                TranslationError::Unavailable(
                    "Argos Translate worker exited before becoming ready".to_owned(),
                )
            })?;
        let response: WorkerResponse = serde_json::from_str(readiness.trim()).map_err(|error| {
            TranslationError::InvalidResponse(format!(
                "invalid Argos Translate readiness response: {error}"
            ))
        })?;
        if !response.ready {
            return Err(TranslationError::Unavailable(
                "Argos Translate worker reported not ready".to_owned(),
            ));
        }
        Ok(Self { worker })
    }
}

impl TextTranslator for ArgosTextTranslator {
    fn translate(&mut self, text: &str) -> Result<String, TranslationError> {
        let request = serde_json::to_string(&TranslationRequest { text }).map_err(|error| {
            TranslationError::InvalidResponse(format!(
                "failed to encode Argos Translate request: {error}"
            ))
        })?;
        self.worker
            .send_line(&request)
            .map_err(TranslationError::Unavailable)?;
        let response = self
            .worker
            .read_line()
            .map_err(TranslationError::Unavailable)?
            .ok_or_else(|| {
                TranslationError::Unavailable(
                    "Argos Translate worker stopped unexpectedly".to_owned(),
                )
            })?;
        parse_translation_response(&response)
    }
}

fn parse_translation_response(response: &str) -> Result<String, TranslationError> {
    let response: WorkerResponse = serde_json::from_str(response.trim()).map_err(|error| {
        TranslationError::InvalidResponse(format!("invalid Argos Translate response: {error}"))
    })?;
    if response.ok {
        return response.text.ok_or_else(|| {
            TranslationError::InvalidResponse("Argos Translate response is missing text".to_owned())
        });
    }
    Err(TranslationError::Unavailable(
        response
            .error
            .unwrap_or_else(|| "Argos Translate failed without an error message".to_owned()),
    ))
}

#[derive(Serialize)]
struct TranslationRequest<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct WorkerResponse {
    #[serde(default)]
    ready: bool,
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_successful_translation_response() {
        let translated_text = parse_translation_response(r#"{"ok":true,"text":"Hello!"}"#).unwrap();
        assert_eq!(translated_text, "Hello!");
    }

    #[test]
    fn parses_failed_translation_response() {
        let result = parse_translation_response(r#"{"ok":false,"error":"missing package"}"#);
        assert!(matches!(result, Err(TranslationError::Unavailable(_))));
    }

    #[test]
    fn rejects_successful_response_without_text() {
        let result = parse_translation_response(r#"{"ok":true}"#);
        assert!(matches!(result, Err(TranslationError::InvalidResponse(_))));
    }
}
