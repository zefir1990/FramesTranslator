mod support;

use std::error::Error;
use std::path::PathBuf;

use frames_translator::application_config::ApplicationConfig;
use frames_translator::factory::create_frames_sourcer;
use frames_translator::frames_sourcer::FramesSourcer;
use frames_translator::implementations::FramesDetectorSourcer;
use frames_translator::text_region::Bounds;

const IMAGE_PATH: &str = "test-case/Shadowrun_02.png";
const BACKGROUND_COLOR: u32 = 0x000000;
const TEXT_COLOR: u32 = 0xFCFCFC;
const EXPECTED_BOUNDS: [Bounds; 2] = [Bounds::new(18, 86, 91, 68), Bounds::new(146, 86, 75, 68)];

#[test]
fn frames_detector_sourcer_detects_shadowrun_menu_panels() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(IMAGE_PATH);
    let mut frames_sourcer = FramesDetectorSourcer::open(
        &image_path,
        application_config.frames_detection_background_color(),
        application_config.frames_detection_text_color(),
    )?;
    let source_image = image::open(&image_path)?.to_rgba8();
    for (index, expected_bounds) in EXPECTED_BOUNDS.iter().enumerate() {
        let frame = frames_sourcer
            .next_frame()?
            .expect("the detector must produce every expected frame");
        assert_eq!(frame.name(), format!("Detected Frame {}", index + 1));
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
    }
    assert!(frames_sourcer.next_frame()?.is_none());
    Ok(())
}

#[test]
fn factory_creates_frames_detector_sourcer_from_application_config() -> Result<(), Box<dyn Error>> {
    let application_config = ApplicationConfig::new(
        "FramesDetectorSourcer",
        "MangaFramesOCR",
        "ArgosTextTranslator",
        "SdlThreeTextRenderer",
        BACKGROUND_COLOR,
        TEXT_COLOR,
    );
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let mut frame_count = 0;
    while frames_sourcer.next_frame()?.is_some() {
        frame_count += 1;
    }
    assert_eq!(frame_count, EXPECTED_BOUNDS.len());
    Ok(())
}
