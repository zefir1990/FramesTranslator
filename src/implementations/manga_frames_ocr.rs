use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::frame::Frame;
use crate::frames_ocr::{FramesOCR, RecognitionError};
use crate::text_region::{Bounds, TextRegion};

const PYTHON_EXECUTABLE_PATH: &str = ".venv/Scripts/python.exe";
const OCR_SCRIPT_PATH: &str = "scripts/manga_ocr_worker.py";
const READY_RESPONSE: &str = "READY";

static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct MangaFramesOCR {
    worker: Mutex<MangaOcrWorker>,
}

impl MangaFramesOCR {
    pub fn new() -> Result<Self, RecognitionError> {
        Self::with_paths(
            manifest_path(PYTHON_EXECUTABLE_PATH),
            manifest_path(OCR_SCRIPT_PATH),
        )
    }

    pub fn with_paths(
        python_executable: impl AsRef<Path>,
        script_path: impl AsRef<Path>,
    ) -> Result<Self, RecognitionError> {
        Ok(Self {
            worker: Mutex::new(MangaOcrWorker::spawn(
                python_executable.as_ref(),
                script_path.as_ref(),
            )?),
        })
    }
}

impl FramesOCR for MangaFramesOCR {
    fn recognize(&self, frame: &Frame) -> Result<Vec<TextRegion>, RecognitionError> {
        let temporary_image = save_frame(frame)?;
        let mut worker = self.worker.lock().map_err(|_| {
            RecognitionError::RecognitionFailed("Manga OCR worker lock is poisoned".to_owned())
        })?;
        writeln!(worker.input, "{}", temporary_image.path.display()).map_err(|error| {
            RecognitionError::RecognitionFailed(format!(
                "failed to send frame to Manga OCR: {error}"
            ))
        })?;
        worker.input.flush().map_err(|error| {
            RecognitionError::RecognitionFailed(format!(
                "failed to flush frame to Manga OCR: {error}"
            ))
        })?;
        let mut response = String::new();
        let bytes_read = worker.output.read_line(&mut response).map_err(|error| {
            RecognitionError::RecognitionFailed(format!(
                "failed to read Manga OCR response: {error}"
            ))
        })?;
        if bytes_read == 0 {
            return Err(RecognitionError::RecognitionFailed(
                "Manga OCR worker stopped unexpectedly".to_owned(),
            ));
        }
        parse_response(&response, frame)
    }
}

struct MangaOcrWorker {
    process: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl MangaOcrWorker {
    fn spawn(python_executable: &Path, script_path: &Path) -> Result<Self, RecognitionError> {
        let mut process = Command::new(python_executable)
            .arg(script_path)
            .env("HF_HUB_DISABLE_SYMLINKS_WARNING", "1")
            .env("HF_HUB_DISABLE_PROGRESS_BARS", "1")
            .env("TRANSFORMERS_VERBOSITY", "error")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| {
                RecognitionError::RecognitionFailed(format!(
                    "failed to start Manga OCR with {}: {error}",
                    python_executable.display()
                ))
            })?;
        let Some(input) = process.stdin.take() else {
            stop_process(process);
            return Err(RecognitionError::RecognitionFailed(
                "Manga OCR stdin is unavailable".to_owned(),
            ));
        };
        let Some(output) = process.stdout.take() else {
            stop_process(process);
            return Err(RecognitionError::RecognitionFailed(
                "Manga OCR stdout is unavailable".to_owned(),
            ));
        };
        let mut output = BufReader::new(output);
        let mut readiness = String::new();
        match output.read_line(&mut readiness) {
            Ok(0) => {
                stop_process(process);
                return Err(RecognitionError::RecognitionFailed(
                    "Manga OCR worker exited before becoming ready".to_owned(),
                ));
            }
            Ok(_) if readiness.trim() == READY_RESPONSE => {}
            Ok(_) => {
                stop_process(process);
                return Err(RecognitionError::RecognitionFailed(format!(
                    "unexpected Manga OCR readiness response: {readiness:?}"
                )));
            }
            Err(error) => {
                stop_process(process);
                return Err(RecognitionError::RecognitionFailed(format!(
                    "failed to read Manga OCR readiness response: {error}"
                )));
            }
        }
        Ok(Self {
            process,
            input,
            output,
        })
    }
}

impl Drop for MangaOcrWorker {
    fn drop(&mut self) {
        stop_process_ref(&mut self.process);
    }
}

fn stop_process(mut process: Child) {
    stop_process_ref(&mut process);
}

fn stop_process_ref(process: &mut Child) {
    let _ = process.kill();
    let _ = process.wait();
}

fn parse_response(response: &str, frame: &Frame) -> Result<Vec<TextRegion>, RecognitionError> {
    let response = response.trim_end_matches(&['\r', '\n'][..]);
    if let Some(text) = response.strip_prefix("OK\t") {
        if text.is_empty() {
            return Ok(Vec::new());
        }
        return Ok(vec![TextRegion::new(
            Bounds::new(0, 0, frame.width(), frame.height()),
            text,
        )]);
    }
    if let Some(message) = response.strip_prefix("ERR\t") {
        return Err(RecognitionError::RecognitionFailed(message.to_owned()));
    }
    Err(RecognitionError::RecognitionFailed(format!(
        "unexpected Manga OCR response: {response:?}"
    )))
}

fn save_frame(frame: &Frame) -> Result<TemporaryImage, RecognitionError> {
    let temporary_image = TemporaryImage::new();
    image::save_buffer_with_format(
        &temporary_image.path,
        frame.rgba_pixels(),
        frame.width(),
        frame.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .map_err(|error| {
        RecognitionError::RecognitionFailed(format!(
            "failed to encode frame for Manga OCR: {error}"
        ))
    })?;
    Ok(temporary_image)
}

struct TemporaryImage {
    path: PathBuf,
}

impl TemporaryImage {
    fn new() -> Self {
        let file_counter = TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let file_name = format!(
            "frames_translator_{}_{}.png",
            std::process::id(),
            file_counter
        );
        Self {
            path: std::env::temp_dir().join(file_name),
        }
    }
}

impl Drop for TemporaryImage {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn manifest_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recognized_text_response() {
        let frame = Frame::new("test", 2, 2, vec![0; 2 * 2 * 4]).unwrap();
        let regions = parse_response("OK\ttranslated text\n", &frame).unwrap();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].text(), "translated text");
        assert_eq!(regions[0].bounds(), Bounds::new(0, 0, 2, 2));
    }

    #[test]
    fn parses_empty_text_response() {
        let frame = Frame::new("test", 2, 2, vec![0; 2 * 2 * 4]).unwrap();
        assert!(parse_response("OK\t\n", &frame).unwrap().is_empty());
    }

    #[test]
    fn parses_error_response() {
        let frame = Frame::new("test", 2, 2, vec![0; 2 * 2 * 4]).unwrap();
        let result = parse_response("ERR\tfailed\n", &frame);
        assert!(matches!(
            result,
            Err(RecognitionError::RecognitionFailed(_))
        ));
    }
}
