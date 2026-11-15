mod support;

use std::error::Error;

use frames_translator::factory::create_frames_sourcer;

const EXPECTED_FRAME_COUNT: usize = 2;
const EXPECTED_FRAME_WIDTH: u32 = 32;
const EXPECTED_FRAME_HEIGHT: u32 = 24;
const EXPECTED_FILL_COLOR: [u8; 4] = [12, 34, 56, 255];

#[test]
fn frames_sourcer_yields_expected_frames() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let mut frames_sourcer = create_frames_sourcer(&config)?;
    let mut actual_frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        assert_eq!(frame.width(), EXPECTED_FRAME_WIDTH);
        assert_eq!(frame.height(), EXPECTED_FRAME_HEIGHT);
        assert_eq!(&frame.rgba_pixels()[..4], EXPECTED_FILL_COLOR.as_slice());
        actual_frame_count += 1;
    }
    assert_eq!(actual_frame_count, EXPECTED_FRAME_COUNT);
    assert!(frames_sourcer.next_frame()?.is_none());
    Ok(())
}
