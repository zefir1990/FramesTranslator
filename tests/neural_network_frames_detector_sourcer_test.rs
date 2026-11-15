mod support;

use std::error::Error;
use std::path::PathBuf;

use frames_translator::frames_detector_sourcer::FramesDetectorSourcer;
use frames_translator::implementations::NeuralNetworkFramesDetectorSourcer;
use frames_translator::text_region::Bounds;

#[test]
fn neural_network_detector_finds_expected_frames_in_sample_images() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let frames_detector = NeuralNetworkFramesDetectorSourcer::new(
        application_config.frames_detection_background_colors(),
        application_config.frames_detection_text_colors(),
    )?;
    let expected_cases = [
        (
            "test-case/Shadowrun_01.png",
            vec![Bounds::new(26, 158, 267, 52)],
        ),
        (
            "test-case/Shadowrun_02.png",
            vec![Bounds::new(18, 86, 91, 68), Bounds::new(146, 86, 75, 68)],
        ),
        (
            "test-case/Shadowrun_03.png",
            vec![Bounds::new(26, 158, 267, 52)],
        ),
        (
            "test-case/Shadowrun_04.png",
            vec![Bounds::new(39, 261, 411, 84)],
        ),
    ];
    for (image_relative_path, expected_bounds) in expected_cases {
        let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(image_relative_path);
        let image = image::open(image_path)?.to_rgba8();
        let detected_bounds = frames_detector
            .detect(&image)?
            .iter()
            .map(|detected_frame| detected_frame.bounds())
            .collect::<Vec<_>>();
        assert_eq!(detected_bounds, expected_bounds);
    }
    Ok(())
}
