mod support;

use std::error::Error;
use std::path::PathBuf;

use frames_translator::factory::create_frames_sourcer;

const EXPECTED_FRAME_COUNT: usize = 1;
const EXPECTED_FRAME_NAME: &str = "Actions Menu";
const EXPECTED_FRAME_WIDTH: u32 = 91;
const EXPECTED_FRAME_HEIGHT: u32 = 68;
const EXPECTED_FRAME_X: u32 = 18;
const EXPECTED_FRAME_Y: u32 = 86;
const IMAGE_PATH: &str = "test-case/Shadowrun_02.png";

#[test]
fn frames_sourcer_extracts_configured_frames() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(IMAGE_PATH);
    let source_image = image::open(image_path)?.to_rgba8();
    let expected_pixels = image::imageops::crop_imm(
        &source_image,
        EXPECTED_FRAME_X,
        EXPECTED_FRAME_Y,
        EXPECTED_FRAME_WIDTH,
        EXPECTED_FRAME_HEIGHT,
    )
    .to_image()
    .into_raw();
    let mut actual_frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        assert_eq!(frame.name(), EXPECTED_FRAME_NAME);
        assert_eq!(frame.width(), EXPECTED_FRAME_WIDTH);
        assert_eq!(frame.height(), EXPECTED_FRAME_HEIGHT);
        assert_eq!(
            frame.rgba_pixels().len(),
            EXPECTED_FRAME_WIDTH as usize * EXPECTED_FRAME_HEIGHT as usize * 4
        );
        assert_eq!(frame.rgba_pixels(), expected_pixels.as_slice());
        actual_frame_count += 1;
    }
    assert_eq!(actual_frame_count, EXPECTED_FRAME_COUNT);
    assert!(frames_sourcer.next_frame()?.is_none());
    Ok(())
}
