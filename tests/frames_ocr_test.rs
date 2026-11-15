mod support;

use std::error::Error;

use frames_translator::factory::create_frames_ocr;
use frames_translator::frame::Frame;

const FRAME_WIDTH: u32 = 64;
const FRAME_HEIGHT: u32 = 48;
const EXPECTED_REGIONS: [(&str, u32, u32, u32, u32); 2] = [
    ("Shadowrun", 10, 8, 40, 12),
    ("Second Edition", 10, 28, 40, 12),
];

#[test]
fn frames_ocr_recognizes_regions_inside_the_frame() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let frame = Frame::new(
        FRAME_WIDTH,
        FRAME_HEIGHT,
        vec![0; FRAME_WIDTH as usize * FRAME_HEIGHT as usize * 4],
    )?;
    let frames_ocr = create_frames_ocr(&config)?;
    let regions = frames_ocr.recognize(&frame)?;
    assert_eq!(regions.len(), EXPECTED_REGIONS.len());
    for (region, (text, x, y, width, height)) in regions.iter().zip(EXPECTED_REGIONS) {
        assert_eq!(region.text(), text);
        assert_eq!(region.bounds().x(), x);
        assert_eq!(region.bounds().y(), y);
        assert_eq!(region.bounds().width(), width);
        assert_eq!(region.bounds().height(), height);
        assert!(region.bounds().right() <= frame.width());
        assert!(region.bounds().bottom() <= frame.height());
    }
    Ok(())
}
