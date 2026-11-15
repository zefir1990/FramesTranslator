use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::frame::{CHANNELS_PER_PIXEL, Frame};
use crate::frames_ocr::{FramesOCR, RecognitionError};
use crate::text_region::{Bounds, TextRegion};

use super::python_worker::{PythonWorker, manifest_path};

const PYTHON_EXECUTABLE_PATH: &str = ".venv/Scripts/python.exe";
const OCR_SCRIPT_PATH: &str = "scripts/manga_ocr_worker.py";
const READY_RESPONSE: &str = "READY";
const INK_LUMINANCE_THRESHOLD: u32 = 160;
const LINE_VERTICAL_PADDING: u32 = 1;
const LINE_HORIZONTAL_PADDING: u32 = 6;
const MINIMUM_LINE_HEIGHT: u32 = 4;

static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct MangaFramesOCR {
    worker: Mutex<PythonWorker>,
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
        let mut worker = PythonWorker::spawn(python_executable.as_ref(), script_path.as_ref(), &[])
            .map_err(RecognitionError::RecognitionFailed)?;
        let readiness = worker
            .read_line()
            .map_err(RecognitionError::RecognitionFailed)?;
        match readiness {
            Some(response) if response.trim() == READY_RESPONSE => Ok(Self {
                worker: Mutex::new(worker),
            }),
            Some(response) => Err(RecognitionError::RecognitionFailed(format!(
                "unexpected Manga OCR readiness response: {response:?}"
            ))),
            None => Err(RecognitionError::RecognitionFailed(
                "Manga OCR worker exited before becoming ready".to_owned(),
            )),
        }
    }
}

impl FramesOCR for MangaFramesOCR {
    fn recognize(&self, frame: &Frame) -> Result<Vec<TextRegion>, RecognitionError> {
        let mut worker = self.worker.lock().map_err(|_| {
            RecognitionError::RecognitionFailed("Manga OCR worker lock is poisoned".to_owned())
        })?;
        let mut regions = Vec::new();
        for bounds in detect_text_lines(frame) {
            let line_frame = crop_frame(frame, bounds)?;
            let temporary_image = save_frame(&line_frame)?;
            worker
                .send_line(&temporary_image.path.to_string_lossy())
                .map_err(RecognitionError::RecognitionFailed)?;
            let response = worker
                .read_line()
                .map_err(RecognitionError::RecognitionFailed)?
                .ok_or_else(|| {
                    RecognitionError::RecognitionFailed(
                        "Manga OCR worker stopped unexpectedly".to_owned(),
                    )
                })?;
            if let Some(text) = parse_response_text(&response)? {
                regions.push(TextRegion::new(bounds, text));
            }
        }
        Ok(regions)
    }
}

fn detect_text_lines(frame: &Frame) -> Vec<Bounds> {
    let inked_rows: Vec<bool> = (0..frame.height()).map(|y| row_has_ink(frame, y)).collect();
    let mut lines = Vec::new();
    let mut line_start = None;
    for y in 0..frame.height() {
        if inked_rows[y as usize] {
            line_start.get_or_insert(y);
        } else if let Some(start) = line_start.take() {
            push_line(&mut lines, frame, start, y);
        }
    }
    if let Some(start) = line_start {
        push_line(&mut lines, frame, start, frame.height());
    }
    if lines.is_empty() {
        return vec![Bounds::new(0, 0, frame.width(), frame.height())];
    }
    lines
}

fn row_has_ink(frame: &Frame, y: u32) -> bool {
    let row_length = frame.width() as usize * CHANNELS_PER_PIXEL;
    let row_start = y as usize * row_length;
    frame.rgba_pixels()[row_start..row_start + row_length]
        .as_chunks::<CHANNELS_PER_PIXEL>()
        .0
        .iter()
        .any(|pixel| pixel_luminance(pixel) >= INK_LUMINANCE_THRESHOLD)
}

fn pixel_luminance(pixel: &[u8]) -> u32 {
    (pixel[0] as u32 * 299 + pixel[1] as u32 * 587 + pixel[2] as u32 * 114) / 1000
}

fn push_line(lines: &mut Vec<Bounds>, frame: &Frame, start: u32, end: u32) {
    if end - start < MINIMUM_LINE_HEIGHT {
        return;
    }
    let Some((ink_start, ink_end)) = horizontal_ink_extent(frame, start, end) else {
        return;
    };
    let padded_start_y = start.saturating_sub(LINE_VERTICAL_PADDING);
    let padded_end_y = (end + LINE_VERTICAL_PADDING).min(frame.height());
    let padded_start_x = ink_start.saturating_sub(LINE_HORIZONTAL_PADDING);
    let padded_end_x = ink_end
        .saturating_add(LINE_HORIZONTAL_PADDING)
        .saturating_add(1)
        .min(frame.width());
    lines.push(Bounds::new(
        padded_start_x,
        padded_start_y,
        padded_end_x - padded_start_x,
        padded_end_y - padded_start_y,
    ));
}

fn horizontal_ink_extent(frame: &Frame, start_y: u32, end_y: u32) -> Option<(u32, u32)> {
    let row_length = frame.width() as usize * CHANNELS_PER_PIXEL;
    let mut minimum_x = frame.width();
    let mut maximum_x = 0;
    let mut ink_found = false;
    for y in start_y..end_y {
        let row_start = y as usize * row_length;
        let row = &frame.rgba_pixels()[row_start..row_start + row_length];
        for (x, pixel) in row.as_chunks::<CHANNELS_PER_PIXEL>().0.iter().enumerate() {
            if pixel_luminance(pixel) >= INK_LUMINANCE_THRESHOLD {
                minimum_x = minimum_x.min(x as u32);
                maximum_x = maximum_x.max(x as u32);
                ink_found = true;
            }
        }
    }
    ink_found.then_some((minimum_x, maximum_x))
}

fn crop_frame(frame: &Frame, bounds: Bounds) -> Result<Frame, RecognitionError> {
    let source_width = frame.width() as usize;
    let row_length = bounds.width() as usize * CHANNELS_PER_PIXEL;
    let mut rgba_pixels = Vec::with_capacity(row_length * bounds.height() as usize);
    for y in bounds.y()..bounds.bottom() {
        let row_start = (y as usize * source_width + bounds.x() as usize) * CHANNELS_PER_PIXEL;
        rgba_pixels.extend_from_slice(&frame.rgba_pixels()[row_start..row_start + row_length]);
    }
    Frame::new(frame.name(), bounds.width(), bounds.height(), rgba_pixels)
        .map_err(|error| RecognitionError::RecognitionFailed(error.to_string()))
}

fn parse_response_text(response: &str) -> Result<Option<String>, RecognitionError> {
    let response = response.trim_end_matches(&['\r', '\n'][..]);
    if let Some(text) = response.strip_prefix("OK\t") {
        if text.is_empty() {
            return Ok(None);
        }
        return Ok(Some(text.to_owned()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recognized_text_response() {
        assert_eq!(
            parse_response_text("OK\ttranslated text\n").unwrap(),
            Some("translated text".to_owned())
        );
    }

    #[test]
    fn parses_empty_text_response() {
        assert_eq!(parse_response_text("OK\t\n").unwrap(), None);
    }

    #[test]
    fn parses_error_response() {
        let result = parse_response_text("ERR\tfailed\n");
        assert!(matches!(
            result,
            Err(RecognitionError::RecognitionFailed(_))
        ));
    }

    #[test]
    fn detects_four_text_lines() {
        let width = 4;
        let height = 24;
        let mut rgba_pixels = vec![0; width as usize * height as usize * CHANNELS_PER_PIXEL];
        for y in [1, 2, 3, 4, 7, 8, 9, 10, 13, 14, 15, 16, 19, 20, 21, 22] {
            for x in 0..width {
                let offset = (y as usize * width as usize + x as usize) * CHANNELS_PER_PIXEL;
                rgba_pixels[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        let frame = Frame::new("test", width, height, rgba_pixels).unwrap();
        let lines = detect_text_lines(&frame);
        assert_eq!(lines.len(), 4);
    }
}
