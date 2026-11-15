mod support;

use std::error::Error;
use std::path::PathBuf;

use frames_translator::factory::create_frames_sourcer;

const IMAGE_PATH: &str = "test-case/Shadowrun_02.png";

#[test]
fn frames_sourcer_detects_configured_frames() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let tests_config = support::load_tests_config()?;
    let expected_frames = tests_config.shadowrun_cd().frames().frames();
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(IMAGE_PATH);
    let source_image = image::open(image_path)?.to_rgba8();
    let mut actual_frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        let expected_frame = expected_frames
            .get(actual_frame_count)
            .expect("the detector must not produce more frames than expected");
        let expected_bounds = expected_frame.rect();
        assert_eq!(
            frame.name(),
            format!("Detected Frame {}", actual_frame_count + 1)
        );
        assert_eq!(frame.width(), expected_bounds.width());
        assert_eq!(frame.height(), expected_bounds.height());
        let expected_pixels = image::imageops::crop_imm(
            &source_image,
            expected_bounds.x(),
            expected_bounds.y(),
            expected_bounds.width(),
            expected_bounds.height(),
        )
        .to_image()
        .into_raw();
        assert_eq!(frame.rgba_pixels(), expected_pixels.as_slice());
        actual_frame_count += 1;
    }
    assert_eq!(actual_frame_count, expected_frames.len());
    assert!(frames_sourcer.next_frame()?.is_none());
    Ok(())
}
