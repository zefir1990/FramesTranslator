use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::thread;
use std::time::Duration;

use image::RgbaImage;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{Canvas, FRect};
use sdl3::video::Window;

use frames_translator::application_config::ApplicationConfig;
use frames_translator::frame_translation::translate_frame_regions;
use frames_translator::frames_ocr::{FramesOCR, RecognitionError};
use frames_translator::frames_sourcer::FrameSourceError;
use frames_translator::implementations::FramesDetectorSourcer;
use frames_translator::text_region::{Bounds, TranslatedText};
use frames_translator::text_translator::{TextTranslator, TranslationError};

const WINDOW_TITLE: &str = "FramesTranslator";
const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 600;
const MAXIMUM_WINDOW_WIDTH: u32 = 1280;
const MAXIMUM_WINDOW_HEIGHT: u32 = 900;
const MAXIMUM_WINDOW_SCALE: u32 = 4;
const TEXT_PADDING: f32 = 2.0;
const EMPTY_STATE_MESSAGE: &str = "Drag and drop an image here";
const VERBOSE_TEXT_POSITION: (f32, f32) = (2.0, 2.0);
const VERBOSE_TEXT_COLOR: Color = Color::RGB(0, 255, 0);
const EVENT_POLL_DELAY: Duration = Duration::from_millis(16);

pub fn run(
    application_config: &ApplicationConfig,
    frames_ocr: &dyn FramesOCR,
    text_translator: &mut dyn TextTranslator,
    initial_image: Option<&str>,
) -> Result<(), ApplicationError> {
    let sdl_context = sdl3::init().map_err(sdl_error)?;
    let video_subsystem = sdl_context.video().map_err(sdl_error)?;
    let window = video_subsystem
        .window(WINDOW_TITLE, INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .map_err(sdl_error)?;
    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump().map_err(sdl_error)?;
    let mut scene = None;
    if let Some(image_path) = initial_image {
        let loaded_scene = load_scene(image_path, application_config, frames_ocr, text_translator)?;
        canvas
            .window_mut()
            .set_title(&format!("{WINDOW_TITLE} - {image_path}"))
            .map_err(sdl_error)?;
        show_scene(
            &mut canvas,
            &loaded_scene,
            application_config.verbose_mode(),
        )?;
        scene = Some(loaded_scene);
    } else {
        draw_empty_state(&mut canvas, application_config.verbose_mode())?;
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
                    canvas
                        .window_mut()
                        .set_title(&format!("{WINDOW_TITLE} - loading {filename}"))
                        .map_err(sdl_error)?;
                    match load_scene(&filename, application_config, frames_ocr, text_translator) {
                        Ok(loaded_scene) => {
                            canvas
                                .window_mut()
                                .set_title(&format!("{WINDOW_TITLE} - {filename}"))
                                .map_err(sdl_error)?;
                            scene = Some(loaded_scene);
                            redraw = true;
                        }
                        Err(error) => {
                            eprintln!("failed to process {filename}: {error}");
                            canvas
                                .window_mut()
                                .set_title(&format!(
                                    "{WINDOW_TITLE} - failed to process {filename}"
                                ))
                                .map_err(sdl_error)?;
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
                Some(loaded_scene) => {
                    show_scene(&mut canvas, loaded_scene, application_config.verbose_mode())?
                }
                None => draw_empty_state(&mut canvas, application_config.verbose_mode())?,
            }
        }
        thread::sleep(EVENT_POLL_DELAY);
    }
}

struct Scene {
    image: RgbaImage,
    overlays: Vec<FrameOverlay>,
    scale: u32,
}

struct FrameOverlay {
    bounds: Bounds,
    translated_texts: Vec<TranslatedText>,
}

fn load_scene(
    image_path: &str,
    application_config: &ApplicationConfig,
    frames_ocr: &dyn FramesOCR,
    text_translator: &mut dyn TextTranslator,
) -> Result<Scene, ApplicationError> {
    let image = image::open(image_path)?.to_rgba8();
    let detected_frames = FramesDetectorSourcer::detect(
        &image,
        application_config.frames_detection_background_colors(),
        application_config.frames_detection_text_colors(),
    )?;
    let mut overlays = Vec::with_capacity(detected_frames.len());
    for detected_frame in detected_frames {
        let regions = frames_ocr.recognize(detected_frame.frame())?;
        let translated_texts =
            translate_frame_regions(detected_frame.frame(), &regions, text_translator)?;
        overlays.push(FrameOverlay {
            bounds: detected_frame.bounds(),
            translated_texts,
        });
    }
    let scale = display_scale(image.width(), image.height());
    Ok(Scene {
        image,
        overlays,
        scale,
    })
}

fn show_scene(
    canvas: &mut Canvas<Window>,
    scene: &Scene,
    verbose_mode: bool,
) -> Result<(), ApplicationError> {
    let (image_width, image_height) = scene.image.dimensions();
    canvas
        .window_mut()
        .set_size(
            image_width.saturating_mul(scene.scale),
            image_height.saturating_mul(scene.scale),
        )
        .map_err(sdl_error)?;
    canvas
        .set_scale(scene.scale as f32, scene.scale as f32)
        .map_err(sdl_error)?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormat::RGBA32, image_width, image_height)
        .map_err(sdl_error)?;
    texture
        .update(None, scene.image.as_raw(), image_width as usize * 4)
        .map_err(sdl_error)?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.copy(&texture, None, None).map_err(sdl_error)?;
    draw_overlays(canvas, &scene.overlays)?;
    if verbose_mode {
        draw_frame_count(canvas, scene.overlays.len())?;
    }
    canvas.present();
    Ok(())
}

fn draw_overlays(
    canvas: &mut Canvas<Window>,
    overlays: &[FrameOverlay],
) -> Result<(), ApplicationError> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    for overlay in overlays {
        let bounds = overlay.bounds;
        canvas
            .fill_rect(FRect::new(
                bounds.x() as f32,
                bounds.y() as f32,
                bounds.width() as f32,
                bounds.height() as f32,
            ))
            .map_err(sdl_error)?;
    }
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for overlay in overlays {
        let overlay_bounds = overlay.bounds;
        for translated_text in &overlay.translated_texts {
            let bounds = translated_text.source_region().bounds();
            canvas
                .draw_debug_text(
                    translated_text.text(),
                    (
                        overlay_bounds.x() as f32 + bounds.x() as f32 + TEXT_PADDING,
                        overlay_bounds.y() as f32 + bounds.y() as f32 + TEXT_PADDING,
                    ),
                )
                .map_err(sdl_error)?;
        }
    }
    Ok(())
}

fn draw_empty_state(
    canvas: &mut Canvas<Window>,
    verbose_mode: bool,
) -> Result<(), ApplicationError> {
    canvas
        .window_mut()
        .set_size(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
        .map_err(sdl_error)?;
    canvas.set_scale(1.0, 1.0).map_err(sdl_error)?;
    canvas.set_draw_color(Color::RGB(24, 24, 24));
    canvas.clear();
    if verbose_mode {
        draw_frame_count(canvas, 0)?;
    }
    canvas.set_draw_color(Color::RGB(230, 230, 230));
    canvas
        .draw_debug_text(EMPTY_STATE_MESSAGE, (20.0, 20.0))
        .map_err(sdl_error)?;
    canvas.present();
    Ok(())
}

fn draw_frame_count(
    canvas: &mut Canvas<Window>,
    frame_count: usize,
) -> Result<(), ApplicationError> {
    canvas.set_scale(1.0, 1.0).map_err(sdl_error)?;
    canvas.set_draw_color(VERBOSE_TEXT_COLOR);
    canvas
        .draw_debug_text(
            &format!("Frames detected: {frame_count}"),
            VERBOSE_TEXT_POSITION,
        )
        .map_err(sdl_error)?;
    Ok(())
}

fn display_scale(image_width: u32, image_height: u32) -> u32 {
    (MAXIMUM_WINDOW_WIDTH / image_width.max(1))
        .min(MAXIMUM_WINDOW_HEIGHT / image_height.max(1))
        .clamp(1, MAXIMUM_WINDOW_SCALE)
}

fn sdl_error(error: impl ToString) -> ApplicationError {
    ApplicationError::Sdl(error.to_string())
}

#[derive(Debug)]
pub enum ApplicationError {
    FrameSource(FrameSourceError),
    Recognition(RecognitionError),
    Translation(TranslationError),
    Image(image::ImageError),
    Sdl(String),
}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameSource(error) => write!(formatter, "{error}"),
            Self::Recognition(error) => write!(formatter, "{error}"),
            Self::Translation(error) => write!(formatter, "{error}"),
            Self::Image(error) => write!(formatter, "failed to load image: {error}"),
            Self::Sdl(message) => write!(formatter, "SDL error: {message}"),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FrameSource(error) => Some(error),
            Self::Recognition(error) => Some(error),
            Self::Translation(error) => Some(error),
            Self::Image(error) => Some(error),
            Self::Sdl(_) => None,
        }
    }
}

impl From<FrameSourceError> for ApplicationError {
    fn from(error: FrameSourceError) -> Self {
        Self::FrameSource(error)
    }
}

impl From<RecognitionError> for ApplicationError {
    fn from(error: RecognitionError) -> Self {
        Self::Recognition(error)
    }
}

impl From<TranslationError> for ApplicationError {
    fn from(error: TranslationError) -> Self {
        Self::Translation(error)
    }
}

impl From<image::ImageError> for ApplicationError {
    fn from(error: image::ImageError) -> Self {
        Self::Image(error)
    }
}

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
}
