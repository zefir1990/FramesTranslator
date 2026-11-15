mod support;

use std::error::Error;
use std::path::PathBuf;

use frames_translator::application_config::ApplicationConfig;
use frames_translator::factory::create_frames_sourcer;
use frames_translator::frames_sourcer::FramesSourcer;
use frames_translator::implementations::FramesDetectorSourcer;
use frames_translator::text_region::Bounds;

const SHADOWRUN_CD_IMAGE_PATH: &str = "test-case/Shadowrun_02.png";
const SHADOWRUN_DIALOG_IMAGE_PATH: &str = "test-case/Shadowrun_01.png";
const SHADOWRUN_STREET_IMAGE_PATH: &str = "test-case/Shadowrun_03.png";
const SHADOWRUN_HOSPITAL_IMAGE_PATH: &str = "test-case/Shadowrun_04.png";
const BACKGROUND_COLOR: u32 = 0x000000;
const TEXT_COLOR: u32 = 0xFCFCFC;
const SHADOWRUN_CD_BOUNDS: [Bounds; 2] =
    [Bounds::new(18, 86, 91, 68), Bounds::new(146, 86, 75, 68)];
const SHADOWRUN_DIALOG_BOUNDS: [Bounds; 1] = [Bounds::new(26, 158, 267, 52)];
const SHADOWRUN_STREET_BOUNDS: [Bounds; 1] = [Bounds::new(26, 158, 267, 52)];
const SHADOWRUN_HOSPITAL_BOUNDS: [Bounds; 1] = [Bounds::new(39, 261, 411, 84)];

#[test]
fn frames_detector_sourcer_detects_shadowrun_menu_panels() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    assert_detected_frames(
        &application_config,
        SHADOWRUN_CD_IMAGE_PATH,
        &SHADOWRUN_CD_BOUNDS,
    )
}

#[test]
fn frames_detector_sourcer_detects_shadowrun_dialog_frame() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    assert_detected_frames(
        &application_config,
        SHADOWRUN_DIALOG_IMAGE_PATH,
        &SHADOWRUN_DIALOG_BOUNDS,
    )
}

#[test]
fn frames_detector_sourcer_detects_shadowrun_street_dialog_frame() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    assert_detected_frames(
        &application_config,
        SHADOWRUN_STREET_IMAGE_PATH,
        &SHADOWRUN_STREET_BOUNDS,
    )
}

#[test]
fn frames_detector_sourcer_detects_shadowrun_hospital_dialog_frame() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    assert_detected_frames(
        &application_config,
        SHADOWRUN_HOSPITAL_IMAGE_PATH,
        &SHADOWRUN_HOSPITAL_BOUNDS,
    )
}

#[test]
fn detect_reports_frame_bounds_for_drag_and_drop() -> Result<(), Box<dyn Error>> {
    let application_config = support::load_application_config()?;
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SHADOWRUN_STREET_IMAGE_PATH);
    let image = image::open(image_path)?.to_rgba8();
    let detected_frames = FramesDetectorSourcer::detect(
        &image,
        application_config.frames_detection_background_colors(),
        application_config.frames_detection_text_colors(),
    )?;
    assert_eq!(detected_frames.len(), 1);
    assert_eq!(detected_frames[0].bounds(), Bounds::new(26, 158, 267, 52));
    assert_eq!(detected_frames[0].frame().width(), 267);
    assert_eq!(detected_frames[0].frame().height(), 52);
    Ok(())
}

#[test]
fn factory_creates_frames_detector_sourcer_from_application_config() -> Result<(), Box<dyn Error>> {
    let application_config = ApplicationConfig::new(
        "FramesDetectorSourcer",
        "MangaFramesOCR",
        "ArgosTextTranslator",
        "SdlThreeTextRenderer",
        vec![BACKGROUND_COLOR],
        vec![TEXT_COLOR],
        true,
    );
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let mut frame_count = 0;
    while frames_sourcer.next_frame()?.is_some() {
        frame_count += 1;
    }
    assert_eq!(frame_count, SHADOWRUN_CD_BOUNDS.len());
    Ok(())
}

fn assert_detected_frames(
    application_config: &ApplicationConfig,
    image_relative_path: &str,
    expected_bounds: &[Bounds],
) -> Result<(), Box<dyn Error>> {
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(image_relative_path);
    let mut frames_sourcer = FramesDetectorSourcer::open(
        &image_path,
        application_config.frames_detection_background_colors(),
        application_config.frames_detection_text_colors(),
    )?;
    let source_image = image::open(&image_path)?.to_rgba8();
    for (index, expected_bounds) in expected_bounds.iter().enumerate() {
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
