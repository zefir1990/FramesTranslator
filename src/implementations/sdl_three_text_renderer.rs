use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::FRect;

use crate::frame::Frame;
use crate::frames_config::NamedRect;
use crate::tests_config::TestsConfig;
use crate::text_region::{Bounds, TranslatedText};
use crate::text_renderer::{RenderingError, TextRenderer};

const DEFAULT_IMAGE_PATH: &str = "test-case/Shadowrun_02.png";
const DEFAULT_TESTS_CONFIG_PATH: &str = "test-case/tests-config.jsonc";
const DEFAULT_WINDOW_SCALE: u32 = 4;
const TEXT_PADDING: f32 = 2.0;
const FRAME_DELAY: Duration = Duration::from_millis(16);

pub struct SdlThreeTextRenderer {
    image_path: PathBuf,
    tests_config_path: PathBuf,
    window_scale: u32,
}

impl SdlThreeTextRenderer {
    pub fn new() -> Self {
        Self {
            image_path: manifest_path(DEFAULT_IMAGE_PATH),
            tests_config_path: manifest_path(DEFAULT_TESTS_CONFIG_PATH),
            window_scale: DEFAULT_WINDOW_SCALE,
        }
    }

    pub fn with_paths(
        image_path: impl Into<PathBuf>,
        tests_config_path: impl Into<PathBuf>,
        window_scale: u32,
    ) -> Self {
        Self {
            image_path: image_path.into(),
            tests_config_path: tests_config_path.into(),
            window_scale: window_scale.max(1),
        }
    }
}

impl Default for SdlThreeTextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextRenderer for SdlThreeTextRenderer {
    fn render(
        &self,
        frame: &Frame,
        translated_texts: &[TranslatedText],
    ) -> Result<Frame, RenderingError> {
        let background = image::open(&self.image_path)
            .map_err(|error| rendering_failed(&self.image_path, error))?
            .to_rgba8();
        let (image_width, image_height) = background.dimensions();
        let tests_config = load_tests_config(&self.tests_config_path)?;
        let placement = find_frame_placement(&tests_config, frame)
            .unwrap_or_else(|| Bounds::new(0, 0, frame.width(), frame.height()));
        let sdl_context = sdl3::init().map_err(|error| {
            RenderingError::RenderingFailed(format!("failed to initialize SDL: {error}"))
        })?;
        let video_subsystem = sdl_context.video().map_err(|error| {
            RenderingError::RenderingFailed(format!("failed to initialize SDL video: {error}"))
        })?;
        let window = video_subsystem
            .window(
                "FramesTranslator",
                image_width.saturating_mul(self.window_scale),
                image_height.saturating_mul(self.window_scale),
            )
            .position_centered()
            .build()
            .map_err(|error| {
                RenderingError::RenderingFailed(format!("failed to create SDL window: {error}"))
            })?;
        let mut canvas = window.into_canvas();
        canvas
            .set_scale(self.window_scale as f32, self.window_scale as f32)
            .map_err(|error| {
                RenderingError::RenderingFailed(format!("failed to set SDL scale: {error}"))
            })?;
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormat::RGBA32, image_width, image_height)
            .map_err(|error| {
                RenderingError::RenderingFailed(format!("failed to create SDL texture: {error}"))
            })?;
        texture
            .update(None, background.as_raw(), image_width as usize * 4)
            .map_err(|error| {
                RenderingError::RenderingFailed(format!("failed to update SDL texture: {error}"))
            })?;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy(&texture, None, None).map_err(|error| {
            RenderingError::RenderingFailed(format!("failed to draw background image: {error}"))
        })?;
        canvas
            .fill_rect(FRect::new(
                placement.x() as f32,
                placement.y() as f32,
                placement.width() as f32,
                placement.height() as f32,
            ))
            .map_err(|error| {
                RenderingError::RenderingFailed(format!(
                    "failed to draw black frame background: {error}"
                ))
            })?;
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for translated_text in translated_texts {
            let bounds = translated_text.source_region().bounds();
            canvas
                .draw_debug_text(
                    translated_text.text(),
                    (
                        placement.x() as f32 + bounds.x() as f32 + TEXT_PADDING,
                        placement.y() as f32 + bounds.y() as f32 + TEXT_PADDING,
                    ),
                )
                .map_err(|error| {
                    RenderingError::RenderingFailed(format!(
                        "failed to draw translated text: {error}"
                    ))
                })?;
        }
        canvas.present();
        let timeout = Duration::from_secs(tests_config.shadowrun_cd().timeout_seconds());
        wait_for_window_close(&sdl_context, timeout)?;
        Ok(frame.clone())
    }
}

fn wait_for_window_close(sdl_context: &sdl3::Sdl, timeout: Duration) -> Result<(), RenderingError> {
    let mut event_pump = sdl_context.event_pump().map_err(|error| {
        RenderingError::RenderingFailed(format!("failed to create SDL event pump: {error}"))
    })?;
    let started_at = Instant::now();
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                _ => {}
            }
        }
        if started_at.elapsed() >= timeout {
            return Ok(());
        }
        thread::sleep(FRAME_DELAY);
    }
}

fn load_tests_config(path: &Path) -> Result<TestsConfig, RenderingError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        RenderingError::RenderingFailed(format!("failed to read {}: {error}", path.display()))
    })?;
    TestsConfig::from_jsonc(&contents).map_err(|error| {
        RenderingError::RenderingFailed(format!("failed to parse {}: {error}", path.display()))
    })
}

fn find_frame_placement(tests_config: &TestsConfig, frame: &Frame) -> Option<Bounds> {
    test_case_frames(tests_config)
        .into_iter()
        .find_map(|named_rects| find_frame_in_frames(named_rects, frame))
}

fn test_case_frames(tests_config: &TestsConfig) -> [&[NamedRect]; 3] {
    [
        tests_config.shadowrun_cd().frames().frames(),
        tests_config.shadowrun_dialog().frames().frames(),
        tests_config.shadowrun_street().frames().frames(),
    ]
}

fn find_frame_in_frames(named_rects: &[NamedRect], frame: &Frame) -> Option<Bounds> {
    named_rects
        .iter()
        .find(|named_rect| named_rect.name() == frame.name())
        .or_else(|| {
            named_rects
                .iter()
                .find(|named_rect| matches_frame_dimensions(named_rect.rect(), frame))
        })
        .map(NamedRect::rect)
}

fn matches_frame_dimensions(bounds: Bounds, frame: &Frame) -> bool {
    bounds.width() == frame.width() && bounds.height() == frame.height()
}

fn rendering_failed(path: &Path, error: image::ImageError) -> RenderingError {
    RenderingError::RenderingFailed(format!("failed to load {}: {error}", path.display()))
}

fn manifest_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::frames_config::{FramesConfig, NamedRect};
    use crate::tests_config::{
        ShadowrunCdTestConfig, ShadowrunDialogTestConfig, ShadowrunStreetTestConfig,
    };

    #[test]
    fn finds_frame_placement_by_frame_name() {
        let frame = frame("Actions Menu", 91, 68);
        assert_eq!(
            find_frame_placement(&tests_config(), &frame),
            Some(Bounds::new(18, 86, 91, 68))
        );
    }

    #[test]
    fn finds_frame_placement_by_frame_dimensions() {
        let frame = frame("Detected Frame 2", 75, 68);
        assert_eq!(
            find_frame_placement(&tests_config(), &frame),
            Some(Bounds::new(146, 86, 75, 68))
        );
    }

    #[test]
    fn finds_dialog_frame_placement_by_frame_dimensions() {
        let frame = frame("Detected Frame 1", 267, 52);
        assert_eq!(
            find_frame_placement(&tests_config(), &frame),
            Some(Bounds::new(26, 158, 267, 52))
        );
    }

    fn tests_config() -> TestsConfig {
        TestsConfig::new(
            ShadowrunCdTestConfig::new(
                FramesConfig::new(vec![
                    NamedRect::new("Actions Menu", Bounds::new(18, 86, 91, 68)),
                    NamedRect::new("Contacts", Bounds::new(146, 86, 75, 68)),
                ]),
                BTreeMap::new(),
                BTreeMap::new(),
                5,
            ),
            ShadowrunDialogTestConfig::new(
                FramesConfig::new(vec![NamedRect::new(
                    "Dialog",
                    Bounds::new(26, 158, 267, 52),
                )]),
                BTreeMap::new(),
                BTreeMap::new(),
                5,
            ),
            ShadowrunStreetTestConfig::new(
                FramesConfig::new(vec![NamedRect::new(
                    "Street Dialog",
                    Bounds::new(26, 158, 267, 52),
                )]),
                BTreeMap::new(),
                BTreeMap::new(),
                5,
            ),
        )
    }

    fn frame(name: &str, width: u32, height: u32) -> Frame {
        Frame::new(
            name,
            width,
            height,
            vec![0; width as usize * height as usize * 4],
        )
        .expect("the frame buffer must match its dimensions")
    }
}
