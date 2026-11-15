use std::error::Error;
use std::path::PathBuf;

use frames_translator::frames_sourcer::FramesSourcer;
use frames_translator::implementations::ImageFramesSourcer;

const EXPECTED_FRAME_COUNT: usize = 1;
const EXPECTED_FRAME_WIDTH: u32 = 320;
const EXPECTED_FRAME_HEIGHT: u32 = 224;
const IMAGE_PATH: &str = "test-case/Shadowrun_02.png";

#[test]
fn frames_sourcer_reads_the_shadowrun_image() -> Result<(), Box<dyn Error>> {
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(IMAGE_PATH);
    let mut frames_sourcer = ImageFramesSourcer::open(image_path)?;
    let mut actual_frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        assert_eq!(frame.width(), EXPECTED_FRAME_WIDTH);
        assert_eq!(frame.height(), EXPECTED_FRAME_HEIGHT);
        assert_eq!(
            frame.rgba_pixels().len(),
            EXPECTED_FRAME_WIDTH as usize * EXPECTED_FRAME_HEIGHT as usize * 4
        );
        assert!(frame.rgba_pixels().iter().any(|&channel| channel != 0));
        actual_frame_count += 1;
    }
    assert_eq!(actual_frame_count, EXPECTED_FRAME_COUNT);
    assert!(frames_sourcer.next_frame()?.is_none());
    Ok(())
}
