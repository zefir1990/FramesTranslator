use std::error::Error;
use std::path::PathBuf;

use frames_translator::frames_detector_sourcer::FramesDetectorSourcer;
use frames_translator::implementations::NeuralNetworkFramesDetectorSourcer;
use frames_translator::text_region::Bounds;

#[test]
fn neural_network_detector_finds_banana_dialog_frame() -> Result<(), Box<dyn Error>> {
    let frames_detector = NeuralNetworkFramesDetectorSourcer::new()?;
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-case/Banana_01.png");
    let image = image::open(image_path)?.to_rgba8();
    let detected_frames = frames_detector.detect(&image)?;
    assert_eq!(detected_frames.len(), 1);
    assert_eq!(detected_frames[0].bounds(), Bounds::new(24, 199, 460, 246));
    assert_eq!(detected_frames[0].frame().width(), 460);
    assert_eq!(detected_frames[0].frame().height(), 246);
    Ok(())
}

#[test]
fn neural_network_detector_finds_expected_frames_in_sample_images() -> Result<(), Box<dyn Error>> {
    let frames_detector = NeuralNetworkFramesDetectorSourcer::new()?;
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
