mod support;

use std::error::Error;

use frames_translator::factory::create_frames_sourcer;

const EXPECTED_FRAME_COUNT: usize = 1;
const EXPECTED_FRAME_NAME: &str = "Shadowrun_02.png";
const EXPECTED_FRAME_WIDTH: u32 = 320;
const EXPECTED_FRAME_HEIGHT: u32 = 224;

#[test]
fn frames_sourcer_yields_expected_frames() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let mut actual_frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        assert_eq!(frame.name(), EXPECTED_FRAME_NAME);
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
