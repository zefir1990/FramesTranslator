use std::error::Error;

use frames_translator::frames_detector_sourcer::FramesDetectorSourcer;
use frames_translator::implementations::NeuralNetworkFramesDetectorSourcer;
use frames_translator::sdl_frame_viewer::{FrameOverlay, Scene, run};

const WINDOW_TITLE: &str = "NeuralNetworkFramesDetectorSourcerTestApplication";

fn main() -> Result<(), Box<dyn Error>> {
    let frames_detector = NeuralNetworkFramesDetectorSourcer::new()?;
    let initial_image = std::env::args().nth(1);
    run(WINDOW_TITLE, true, initial_image.as_deref(), |image_path| {
        build_scene(image_path, &frames_detector)
    })
}

fn build_scene(
    image_path: &str,
    frames_detector: &dyn FramesDetectorSourcer,
) -> Result<Scene, Box<dyn Error>> {
    let image = image::open(image_path)?.to_rgba8();
    let overlays = frames_detector
        .detect(&image)?
        .into_iter()
        .map(|detected_frame| FrameOverlay::new(detected_frame.bounds()))
        .collect();
    Ok(Scene::new(image, overlays))
}
