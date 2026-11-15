use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use image::RgbaImage;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{BlendMode, Canvas, FRect};
use sdl3::video::Window;

use crate::debug_text::DebugTextLayout;
use crate::text_region::{Bounds, TranslatedText};

const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 600;
const MAXIMUM_WINDOW_WIDTH: u32 = 1280;
const MAXIMUM_WINDOW_HEIGHT: u32 = 900;
const MAXIMUM_WINDOW_SCALE: u32 = 4;
const EMPTY_STATE_MESSAGE: &str = "Drag and drop an image here";
const VERBOSE_TEXT_POSITION: (f32, f32) = (2.0, 2.0);
const VERBOSE_TEXT_COLOR: Color = Color::RGB(0, 255, 0);
const TRANSLATED_TEXT_COLOR: Color = Color::RGB(255, 255, 255);
const FRAME_FILL_COLOR: Color = Color::RGB(0, 0, 0);
const OVERLAY_ALPHA: u8 = 204;
const MINIMUM_OVERLAY_CHANNEL: u8 = 80;
const OVERLAY_CHANNEL_RANGE: u64 = 176;
const DEFAULT_RANDOM_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
const EVENT_POLL_DELAY: Duration = Duration::from_millis(16);

pub fn run<ApplicationError, BuildScene>(
    title: &str,
    verbose_mode: bool,
    initial_image: Option<&str>,
    mut build_scene: BuildScene,
) -> Result<(), ApplicationError>
where
    ApplicationError: From<FrameViewerError> + Display,
    BuildScene: FnMut(&str) -> Result<Scene, ApplicationError>,
{
    let sdl_context = sdl3::init().map_err(|error| ApplicationError::from(sdl_failure(error)))?;
    let video_subsystem = sdl_context
        .video()
        .map_err(|error| ApplicationError::from(sdl_failure(error)))?;
    let window = video_subsystem
        .window(title, INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .map_err(|error| ApplicationError::from(sdl_failure(error)))?;
    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|error| ApplicationError::from(sdl_failure(error)))?;
    let mut scene = None;
    if let Some(image_path) = initial_image {
        let loaded_scene = build_scene(image_path)?;
        set_window_title(&mut canvas, &format!("{title} - {image_path}"))?;
        show_scene(&mut canvas, &loaded_scene, verbose_mode).map_err(ApplicationError::from)?;
        scene = Some(loaded_scene);
    } else {
        draw_empty_state(&mut canvas, verbose_mode).map_err(ApplicationError::from)?;
    }
    loop {
        let mut redraw = false;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                Event::DropFile { filename, .. } => {
                    set_window_title(&mut canvas, &format!("{title} - loading {filename}"))?;
                    match build_scene(&filename) {
                        Ok(loaded_scene) => {
                            set_window_title(&mut canvas, &format!("{title} - {filename}"))?;
                            scene = Some(loaded_scene);
                            redraw = true;
                        }
                        Err(error) => {
                            eprintln!("failed to process {filename}: {error}");
                            set_window_title(
                                &mut canvas,
                                &format!("{title} - failed to process {filename}"),
                            )?;
                        }
                    }
                }
                Event::Window {
                    win_event: WindowEvent::Exposed,
                    ..
                } => {
                    redraw |= scene.is_some();
                }
                _ => {}
            }
        }
        if redraw {
            match &scene {
                Some(loaded_scene) => show_scene(&mut canvas, loaded_scene, verbose_mode)
                    .map_err(ApplicationError::from)?,
                None => {
                    draw_empty_state(&mut canvas, verbose_mode).map_err(ApplicationError::from)?
                }
            }
        }
        thread::sleep(EVENT_POLL_DELAY);
    }
}

#[derive(Debug, Clone)]
pub struct FrameOverlay {
    bounds: Bounds,
    layout: Option<DebugTextLayout>,
    color: Color,
}

impl FrameOverlay {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            layout: None,
            color: FRAME_FILL_COLOR,
        }
    }

    pub fn with_translated_texts(bounds: Bounds, translated_texts: &[TranslatedText]) -> Self {
        Self {
            bounds,
            layout: Some(DebugTextLayout::for_translated_texts(
                translated_texts,
                bounds,
            )),
            color: FRAME_FILL_COLOR,
        }
    }

    pub const fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub const fn color(&self) -> Color {
        self.color
    }

    pub fn layout(&self) -> Option<&DebugTextLayout> {
        self.layout.as_ref()
    }
}

pub struct Scene {
    image: RgbaImage,
    overlays: Vec<FrameOverlay>,
    scale: u32,
}

impl Scene {
    pub fn new(image: RgbaImage, mut overlays: Vec<FrameOverlay>) -> Self {
        let mut color_generator = RandomColorGenerator::new(random_seed());
        for overlay in &mut overlays {
            overlay.color = color_generator.next_color();
        }
        let scale = display_scale(image.width(), image.height());
        Self {
            image,
            overlays,
            scale,
        }
    }

    pub fn image(&self) -> &RgbaImage {
        &self.image
    }

    pub fn overlays(&self) -> &[FrameOverlay] {
        &self.overlays
    }

    pub const fn scale(&self) -> u32 {
        self.scale
    }
}

fn show_scene(
    canvas: &mut Canvas<Window>,
    scene: &Scene,
    verbose_mode: bool,
) -> Result<(), FrameViewerError> {
    let (image_width, image_height) = scene.image.dimensions();
    canvas
        .window_mut()
        .set_size(
            image_width.saturating_mul(scene.scale),
            image_height.saturating_mul(scene.scale),
        )
        .map_err(sdl_failure)?;
    canvas
        .set_scale(scene.scale as f32, scene.scale as f32)
        .map_err(sdl_failure)?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormat::RGBA32, image_width, image_height)
        .map_err(sdl_failure)?;
    texture
        .update(None, scene.image.as_raw(), image_width as usize * 4)
        .map_err(sdl_failure)?;
    canvas.set_draw_color(FRAME_FILL_COLOR);
    canvas.clear();
    canvas.copy(&texture, None, None).map_err(sdl_failure)?;
    draw_overlays(canvas, &scene.overlays, scene.scale, verbose_mode)?;
    if verbose_mode {
        draw_frame_count(canvas, scene.overlays.len())?;
    }
    canvas.present();
    Ok(())
}

fn draw_overlays(
    canvas: &mut Canvas<Window>,
    overlays: &[FrameOverlay],
    scene_scale: u32,
    verbose_mode: bool,
) -> Result<(), FrameViewerError> {
    canvas.set_blend_mode(if verbose_mode {
        BlendMode::Blend
    } else {
        BlendMode::None
    });
    for overlay in overlays {
        let bounds = overlay.bounds();
        canvas.set_draw_color(if verbose_mode {
            overlay.color()
        } else {
            FRAME_FILL_COLOR
        });
        canvas
            .fill_rect(FRect::new(
                bounds.x() as f32,
                bounds.y() as f32,
                bounds.width() as f32,
                bounds.height() as f32,
            ))
            .map_err(sdl_failure)?;
    }
    canvas.set_blend_mode(BlendMode::None);
    for overlay in overlays {
        if let Some(layout) = overlay.layout() {
            layout
                .draw(
                    canvas,
                    (overlay.bounds().x() as f32, overlay.bounds().y() as f32),
                    scene_scale as f32,
                    TRANSLATED_TEXT_COLOR,
                )
                .map_err(sdl_failure)?;
        }
    }
    Ok(())
}

fn draw_empty_state(
    canvas: &mut Canvas<Window>,
    verbose_mode: bool,
) -> Result<(), FrameViewerError> {
    canvas
        .window_mut()
        .set_size(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
        .map_err(sdl_failure)?;
    canvas.set_scale(1.0, 1.0).map_err(sdl_failure)?;
    canvas.set_draw_color(Color::RGB(24, 24, 24));
    canvas.clear();
    if verbose_mode {
        draw_frame_count(canvas, 0)?;
    }
    canvas.set_draw_color(Color::RGB(230, 230, 230));
    canvas
        .draw_debug_text(EMPTY_STATE_MESSAGE, (20.0, 20.0))
        .map_err(sdl_failure)?;
    canvas.present();
    Ok(())
}

fn draw_frame_count(
    canvas: &mut Canvas<Window>,
    frame_count: usize,
) -> Result<(), FrameViewerError> {
    canvas.set_scale(1.0, 1.0).map_err(sdl_failure)?;
    canvas.set_draw_color(VERBOSE_TEXT_COLOR);
    canvas
        .draw_debug_text(
            &format!("Frames detected: {frame_count}"),
            VERBOSE_TEXT_POSITION,
        )
        .map_err(sdl_failure)?;
    Ok(())
}

fn set_window_title<ApplicationError>(
    canvas: &mut Canvas<Window>,
    title: &str,
) -> Result<(), ApplicationError>
where
    ApplicationError: From<FrameViewerError>,
{
    canvas
        .window_mut()
        .set_title(title)
        .map_err(|error| ApplicationError::from(sdl_failure(error)))
}

fn display_scale(image_width: u32, image_height: u32) -> u32 {
    (MAXIMUM_WINDOW_WIDTH / image_width.max(1))
        .min(MAXIMUM_WINDOW_HEIGHT / image_height.max(1))
        .clamp(1, MAXIMUM_WINDOW_SCALE)
}

struct RandomColorGenerator {
    state: u64,
}

impl RandomColorGenerator {
    const fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    fn next_color(&mut self) -> Color {
        let bits = self.next_bits();
        Color::RGBA(
            overlay_channel(bits),
            overlay_channel(bits >> 21),
            overlay_channel(bits >> 42),
            OVERLAY_ALPHA,
        )
    }

    fn next_bits(&mut self) -> u64 {
        let mut state = self.state;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.state = state;
        state
    }
}

fn overlay_channel(bits: u64) -> u8 {
    MINIMUM_OVERLAY_CHANNEL + (bits % OVERLAY_CHANNEL_RANGE) as u8
}

fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(DEFAULT_RANDOM_SEED)
}

fn sdl_failure(error: impl ToString) -> FrameViewerError {
    FrameViewerError::Sdl(error.to_string())
}

#[derive(Debug)]
pub enum FrameViewerError {
    Sdl(String),
}

impl Display for FrameViewerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sdl(message) => write!(formatter, "SDL error: {message}"),
        }
    }
}

impl Error for FrameViewerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_small_images_up_to_the_maximum_scale() {
        assert_eq!(display_scale(320, 224), MAXIMUM_WINDOW_SCALE);
    }

    #[test]
    fn scales_large_images_down_to_one() {
        assert_eq!(display_scale(1920, 1080), 1);
    }

    #[test]
    fn preserves_aspect_ratio_when_choosing_a_scale() {
        assert_eq!(display_scale(640, 224), 2);
    }

    #[test]
    fn generates_translucent_overlay_colors() {
        let mut color_generator = RandomColorGenerator::new(DEFAULT_RANDOM_SEED);
        for _ in 0..8 {
            let color = color_generator.next_color();
            assert_eq!(color.a, OVERLAY_ALPHA);
            for channel in [color.r, color.g, color.b] {
                assert!(channel >= MINIMUM_OVERLAY_CHANNEL);
                assert!(
                    u64::from(channel) < u64::from(MINIMUM_OVERLAY_CHANNEL) + OVERLAY_CHANNEL_RANGE
                );
            }
        }
    }

    #[test]
    fn generates_distinct_colors_for_adjacent_frames() {
        let mut color_generator = RandomColorGenerator::new(DEFAULT_RANDOM_SEED);
        let first_color = color_generator.next_color();
        let second_color = color_generator.next_color();
        assert_ne!(first_color, second_color);
    }
}
