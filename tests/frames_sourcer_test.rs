mod support;

use std::error::Error;

use frames_translator::factory::create_frames_sourcer;

#[test]
fn frames_sourcer_yields_configured_frames() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let expected_frame_count = config.require_usize("FRAMES_SOURCER_FRAME_COUNT")?;
    let expected_width = config.require_u32("FRAMES_SOURCER_FRAME_WIDTH")?;
    let expected_height = config.require_u32("FRAMES_SOURCER_FRAME_HEIGHT")?;
    let expected_fill_color = config.require_color("FRAMES_SOURCER_FILL_COLOR")?;
    let mut frames_sourcer = create_frames_sourcer(&config)?;
    let mut actual_frame_count = 0;
    while let Some(frame) = frames_sourcer.next_frame()? {
        assert_eq!(frame.width(), expected_width);
        assert_eq!(frame.height(), expected_height);
        assert_eq!(&frame.rgba_pixels()[..4], expected_fill_color.as_slice());
        actual_frame_count += 1;
    }
    assert_eq!(actual_frame_count, expected_frame_count);
    assert!(frames_sourcer.next_frame()?.is_none());
    Ok(())
}
