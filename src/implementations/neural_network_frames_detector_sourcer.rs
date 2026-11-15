use std::path::Path;
use std::sync::Mutex;

use image::RgbaImage;
use serde::Deserialize;

use crate::frame::Frame;
use crate::frames_detector_sourcer::{
    DetectedFrame, DetectedFrameQueue, FramesDetectorSourcer, decode_image,
    detected_frames_from_bounds,
};
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};
use crate::text_region::Bounds;

use super::python_worker::{PythonWorker, TemporaryImage, manifest_path};

const PYTHON_EXECUTABLE_PATH: &str = ".venv/Scripts/python.exe";
const DETECTOR_SCRIPT_PATH: &str = "scripts/neural_frame_detector_worker.py";

pub struct NeuralNetworkFramesDetectorSourcer {
    frames: DetectedFrameQueue,
    worker: Mutex<PythonWorker>,
}

impl NeuralNetworkFramesDetectorSourcer {
    pub fn new() -> Result<Self, FrameSourceError> {
        let mut worker = PythonWorker::spawn(
            &manifest_path(PYTHON_EXECUTABLE_PATH),
            &manifest_path(DETECTOR_SCRIPT_PATH),
            &[],
        )
        .map_err(detector_failed)?;
        let readiness = worker
            .read_line()
            .map_err(detector_failed)?
            .ok_or_else(|| {
                detector_failed("neural frame detector worker exited before becoming ready")
            })?;
        let response = parse_worker_response(&readiness)?;
        if !response.ready {
            return Err(detector_failed(response.error.unwrap_or_else(|| {
                "neural frame detector worker is not ready".to_owned()
            })));
        }
        Ok(Self {
            frames: DetectedFrameQueue::default(),
            worker: Mutex::new(worker),
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, FrameSourceError> {
        let image = decode_image(path.as_ref())?;
        let mut sourcer = Self::new()?;
        let detected_frames = sourcer.detect(&image)?;
        sourcer.frames = DetectedFrameQueue::from_detected_frames(detected_frames);
        Ok(sourcer)
    }
}

impl FramesSourcer for NeuralNetworkFramesDetectorSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError> {
        self.frames.next_frame()
    }
}

impl FramesDetectorSourcer for NeuralNetworkFramesDetectorSourcer {
    fn detect(&self, image: &RgbaImage) -> Result<Vec<DetectedFrame>, FrameSourceError> {
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| detector_failed("neural frame detector worker lock is poisoned"))?;
        let temporary_image = save_image(image)?;
        worker
            .send_line(&temporary_image.path().to_string_lossy())
            .map_err(detector_failed)?;
        let response_line = worker
            .read_line()
            .map_err(detector_failed)?
            .ok_or_else(|| detector_failed("neural frame detector worker stopped unexpectedly"))?;
        let response = parse_worker_response(&response_line)?;
        if !response.ok {
            return Err(detector_failed(
                response
                    .error
                    .unwrap_or_else(|| "neural frame detector failed".to_owned()),
            ));
        }
        let detected_bounds = response
            .bounds
            .into_iter()
            .map(|[x, y, width, height]| Bounds::new(x, y, width, height))
            .collect();
        detected_frames_from_bounds(image, detected_bounds)
    }
}

fn save_image(image: &RgbaImage) -> Result<TemporaryImage, FrameSourceError> {
    let temporary_image = TemporaryImage::new();
    image::save_buffer_with_format(
        temporary_image.path(),
        image.as_raw(),
        image.width(),
        image.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .map_err(|error| {
        detector_failed(format!(
            "failed to encode image for neural frame detection: {error}"
        ))
    })?;
    Ok(temporary_image)
}

fn parse_worker_response(response: &str) -> Result<WorkerResponse, FrameSourceError> {
    serde_json::from_str(response.trim()).map_err(|error| {
        detector_failed(format!("invalid neural frame detector response: {error}"))
    })
}

fn detector_failed(message: impl ToString) -> FrameSourceError {
    FrameSourceError::Decoding(message.to_string())
}

#[derive(Deserialize)]
struct WorkerResponse {
    #[serde(default)]
    ready: bool,
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    bounds: Vec<[u32; 4]>,
    #[serde(default)]
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ready_response() {
        let response = parse_worker_response(r#"{"ready":true}"#).unwrap();
        assert!(response.ready);
    }

    #[test]
    fn parses_detected_bounds() {
        let response =
            parse_worker_response(r#"{"ok":true,"bounds":[[1,2,3,4],[5,6,7,8]]}"#).unwrap();
        assert!(response.ok);
        assert_eq!(response.bounds, vec![[1, 2, 3, 4], [5, 6, 7, 8]]);
    }

    #[test]
    fn parses_error_response() {
        let response = parse_worker_response(r#"{"ok":false,"error":"failed"}"#).unwrap();
        assert_eq!(response.error.as_deref(), Some("failed"));
    }
}
