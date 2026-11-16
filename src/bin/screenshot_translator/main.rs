mod imgui_renderer;
mod sdl3_platform;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use frames_translator::FactoryError;
use frames_translator::application_config::ApplicationConfig;
use frames_translator::factory::{
    create_frames_detector_sourcer, create_frames_ocr, create_text_translator,
};
use frames_translator::frame::Frame;
use frames_translator::frame_translation::translate_frame_regions;
use frames_translator::frames_detector_sourcer::FramesDetectorSourcer;
use frames_translator::frames_ocr::{FramesOCR, RecognitionError};
use frames_translator::frames_sourcer::FrameSourceError;
use frames_translator::implementations::{SoftwareTextRenderer, WindowsScreenshoter};
use frames_translator::screenshoter::{ScreenshotError, Screenshoter};
use frames_translator::text_renderer::{RenderingError, TextRenderer};
use frames_translator::text_translator::{TextTranslator, TranslationError};
use image::RgbaImage;
use imgui::{Condition, TextureId, Ui};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;

use crate::imgui_renderer::{Renderer, RendererError};
use crate::sdl3_platform::Platform;

const APPLICATION_CONFIG_PATH: &str = "test-case/tests-application-config.jsonc";
const WINDOW_TITLE: &str = "Screenshot Translator";
const WINDOW_WIDTH: u32 = 1440;
const WINDOW_HEIGHT: u32 = 900;
const CLEAR_COLOR: Color = Color::RGB(24, 24, 24);
const EVENT_POLL_DELAY: Duration = Duration::from_millis(16);
const WORKER_POLL_DELAY: Duration = Duration::from_millis(50);
const MAXIMUM_IMAGE_SCALE: f32 = 4.0;
const CAPTURE_INTERVAL: Duration = Duration::from_secs(5);
const SPINNER_INTERVAL: Duration = Duration::from_millis(120);
const SPINNER_FRAMES: [&str; 4] = ["|", "/", "-", "\\"];
const CAPTURE_INDICATOR_COLOR: [f32; 4] = [0.4, 1.0, 0.4, 1.0];

fn main() -> Result<(), ApplicationError> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(APPLICATION_CONFIG_PATH);
    let config_source =
        fs::read_to_string(&config_path).map_err(|source| ApplicationError::Io {
            path: config_path.clone(),
            source,
        })?;
    let application_config = ApplicationConfig::from_jsonc(&config_source)
        .map_err(|error| ApplicationError::Configuration(error.to_string()))?;
    let initial_source = std::env::args().nth(1);
    run(application_config, initial_source)
}

fn run(
    application_config: ApplicationConfig,
    initial_source: Option<String>,
) -> Result<(), ApplicationError> {
    let sdl_context = sdl3::init().map_err(sdl_failure)?;
    let video_subsystem = sdl_context.video().map_err(sdl_failure)?;
    let window = video_subsystem
        .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .map_err(sdl_failure)?;
    let mut canvas = window.clone().into_canvas();
    let mut event_pump = sdl_context.event_pump().map_err(sdl_failure)?;
    let texture_creator = canvas.texture_creator();
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(None);
    imgui_context.set_log_filename(None);
    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    let mut renderer = Renderer::new(&texture_creator, &mut imgui_context)?;
    let mut platform = Platform::new(&mut imgui_context);
    let mut application = ScreenshotTranslatorApplication::new(application_config, initial_source);
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                _ => platform.handle_event(&mut imgui_context, &event),
            }
        }
        application.poll_worker();
        platform.prepare_frame(&mut imgui_context, &window);
        let ui = imgui_context.frame();
        application.draw_ui(ui);
        canvas.set_draw_color(CLEAR_COLOR);
        canvas.clear();
        renderer.render(imgui_context.render(), &mut canvas)?;
        canvas.present();
        if let Some(result) = application.take_completed_result() {
            match renderer.set_image(&result.image) {
                Ok(texture_identifier) => application.set_result(result, texture_identifier),
                Err(error) => {
                    application.apply_error(format!("Failed to show the result: {error}"))
                }
            }
        }
        thread::sleep(EVENT_POLL_DELAY);
    }
}

struct TranslationPipeline {
    frames_detector: Box<dyn FramesDetectorSourcer>,
    frames_ocr: Box<dyn FramesOCR>,
    text_translator: Box<dyn TextTranslator>,
    text_renderer: SoftwareTextRenderer,
}

impl TranslationPipeline {
    fn open(application_config: &ApplicationConfig) -> Result<Self, ApplicationError> {
        Ok(Self {
            frames_detector: create_frames_detector_sourcer(application_config)?,
            frames_ocr: create_frames_ocr(application_config)?,
            text_translator: create_text_translator(application_config)?,
            text_renderer: SoftwareTextRenderer::new(),
        })
    }
}

struct TranslationResult {
    image: RgbaImage,
    detected_frame_count: usize,
}

struct CaptureSchedule {
    interval: Duration,
    next_capture: Option<Instant>,
}

impl CaptureSchedule {
    const fn new(interval: Duration) -> Self {
        Self {
            interval,
            next_capture: None,
        }
    }

    fn start(&mut self, now: Instant) {
        self.next_capture = Some(now);
    }

    fn is_due(&self, now: Instant) -> bool {
        self.next_capture
            .is_some_and(|next_capture| now >= next_capture)
    }

    fn postpone(&mut self, now: Instant) {
        self.next_capture = Some(now + self.interval);
    }

    const fn next_capture(&self) -> Option<Instant> {
        self.next_capture
    }
}

enum WorkerMessage {
    CycleStarted,
    CycleFinished(Result<TranslationResult, String>),
    Waiting { next_capture: Instant },
    Failed(String),
}

struct CaptureWorker {
    cancellation: Arc<AtomicBool>,
    receiver: Receiver<WorkerMessage>,
}

impl Drop for CaptureWorker {
    fn drop(&mut self) {
        self.cancellation.store(true, Ordering::Relaxed);
    }
}

struct ScreenshotTranslatorApplication {
    application_config: ApplicationConfig,
    sources: Vec<String>,
    selected_source: Option<String>,
    worker: Option<CaptureWorker>,
    processing: bool,
    spinner_started: Instant,
    next_capture: Option<Instant>,
    status: String,
    completed_result: Option<TranslationResult>,
    result: Option<TranslationResult>,
    result_texture: Option<TextureId>,
}

impl ScreenshotTranslatorApplication {
    fn new(application_config: ApplicationConfig, initial_source: Option<String>) -> Self {
        let mut application = Self {
            application_config,
            sources: Vec::new(),
            selected_source: None,
            worker: None,
            processing: false,
            spinner_started: Instant::now(),
            next_capture: None,
            status: "Select a window and press Start Capture.".to_owned(),
            completed_result: None,
            result: None,
            result_texture: None,
        };
        application.refresh_sources();
        if let Some(source) = initial_source
            && application.sources.contains(&source)
        {
            application.selected_source = Some(source);
            application.start_capture();
        }
        application
    }

    fn draw_ui(&mut self, ui: &Ui) {
        let display_size = ui.io().display_size;
        let panel_height = (display_size[1] - 20.0).max(200.0);
        let result_width = (display_size[0] - 450.0).max(300.0);
        let capture_active = self.worker.is_some();
        ui.window("Screenshot sources")
            .position([10.0, 10.0], Condition::FirstUseEver)
            .size([420.0, panel_height], Condition::FirstUseEver)
            .build(|| {
                if ui.button("Refresh windows") {
                    self.refresh_sources();
                }
                ui.same_line();
                ui.text(format!("{} found", self.sources.len()));
                ui.separator();
                ui.disabled(capture_active, || {
                    for source in &self.sources {
                        let is_selected = self.selected_source.as_deref() == Some(source.as_str());
                        if ui
                            .selectable_config(source.as_str())
                            .selected(is_selected)
                            .build()
                        {
                            self.selected_source = Some(source.clone());
                        }
                    }
                });
                if self.sources.is_empty() {
                    ui.text_disabled("No screenshot sources found.");
                }
                ui.separator();
                if capture_active {
                    if ui.button("Stop Capture") {
                        self.stop_capture();
                    }
                } else {
                    let can_start = self.selected_source.is_some();
                    ui.disabled(!can_start, || {
                        if ui.button("Start Capture") {
                            self.start_capture();
                        }
                    });
                }
                ui.separator();
                self.draw_capture_status(ui);
            });
        ui.window("Translation result")
            .position([440.0, 10.0], Condition::FirstUseEver)
            .size([result_width, panel_height], Condition::FirstUseEver)
            .build(|| self.draw_result(ui));
    }

    fn draw_capture_status(&self, ui: &Ui) {
        if self.worker.is_none() {
            ui.text_wrapped(&self.status);
            return;
        }
        let spinner = spinner_frame(self.spinner_started);
        if self.processing {
            ui.text_colored(
                CAPTURE_INDICATOR_COLOR,
                format!("{spinner} Capturing and translating..."),
            );
            return;
        }
        ui.text_wrapped(&self.status);
        if let Some(next_capture) = self.next_capture {
            ui.text_colored(
                CAPTURE_INDICATOR_COLOR,
                format!(
                    "{spinner} Capture active: next capture in {}s.",
                    seconds_until(next_capture, Instant::now())
                ),
            );
        }
    }

    fn draw_result(&self, ui: &Ui) {
        let (Some(result), Some(texture_identifier)) = (&self.result, self.result_texture) else {
            ui.text("No translation result yet.");
            return;
        };
        let available_size = ui.content_region_avail();
        let (image_width, image_height) = result.image.dimensions();
        let scale = (available_size[0] / image_width.max(1) as f32)
            .min(available_size[1] / image_height.max(1) as f32)
            .clamp(0.01, MAXIMUM_IMAGE_SCALE);
        let display_size = [image_width as f32 * scale, image_height as f32 * scale];
        let origin = ui.cursor_screen_pos();
        let maximum = [origin[0] + display_size[0], origin[1] + display_size[1]];
        ui.get_window_draw_list()
            .add_image(texture_identifier, origin, maximum)
            .build();
        ui.dummy(display_size);
    }

    fn refresh_sources(&mut self) {
        match WindowsScreenshoter::new().sources() {
            Ok(sources) => {
                if let Some(selected_source) = &self.selected_source
                    && !sources.contains(selected_source)
                {
                    self.selected_source = None;
                }
                if self.selected_source.is_none() {
                    self.selected_source = sources.first().cloned();
                }
                self.sources = sources;
                self.status = format!("Found {} screenshot sources.", self.sources.len());
            }
            Err(error) => {
                self.sources.clear();
                self.selected_source = None;
                self.status = format!("Failed to list windows: {error}");
            }
        }
    }

    fn start_capture(&mut self) {
        self.stop_capture();
        let Some(source) = self.selected_source.clone() else {
            return;
        };
        let (sender, receiver) = mpsc::channel();
        let cancellation = Arc::new(AtomicBool::new(false));
        let worker_cancellation = Arc::clone(&cancellation);
        let application_config = self.application_config.clone();
        thread::spawn(move || {
            run_capture_worker(application_config, source, sender, worker_cancellation);
        });
        self.worker = Some(CaptureWorker {
            cancellation,
            receiver,
        });
        self.processing = true;
        self.spinner_started = Instant::now();
        self.next_capture = None;
        self.status = "Starting capture...".to_owned();
    }

    fn stop_capture(&mut self) {
        self.worker = None;
        self.processing = false;
        self.next_capture = None;
        self.status = "Capture stopped.".to_owned();
    }

    fn poll_worker(&mut self) {
        let messages = match self.worker.as_ref() {
            Some(worker) => worker.receiver.try_iter().collect::<Vec<_>>(),
            None => return,
        };
        let mut worker_failed = false;
        for message in messages {
            match message {
                WorkerMessage::CycleStarted => {
                    self.processing = true;
                    self.spinner_started = Instant::now();
                    self.next_capture = None;
                    self.status = "Capturing and translating...".to_owned();
                }
                WorkerMessage::CycleFinished(Ok(result)) => {
                    self.processing = false;
                    self.status = format!(
                        "Detected {} frame(s) in the selected window.",
                        result.detected_frame_count
                    );
                    self.completed_result = Some(result);
                }
                WorkerMessage::CycleFinished(Err(message)) => {
                    self.processing = false;
                    self.status = format!("Translation failed: {message}");
                }
                WorkerMessage::Waiting { next_capture } => {
                    self.next_capture = Some(next_capture);
                }
                WorkerMessage::Failed(message) => {
                    self.processing = false;
                    self.next_capture = None;
                    self.status = format!("Capture failed: {message}");
                    worker_failed = true;
                }
            }
        }
        if worker_failed {
            self.worker = None;
        }
    }

    fn take_completed_result(&mut self) -> Option<TranslationResult> {
        self.completed_result.take()
    }

    fn set_result(&mut self, result: TranslationResult, texture_identifier: TextureId) {
        self.result = Some(result);
        self.result_texture = Some(texture_identifier);
    }

    fn apply_error(&mut self, message: String) {
        self.status = message;
    }
}

fn run_capture_worker(
    application_config: ApplicationConfig,
    source: String,
    sender: Sender<WorkerMessage>,
    cancellation: Arc<AtomicBool>,
) {
    if cancellation.load(Ordering::Relaxed) {
        return;
    }
    let mut pipeline = match TranslationPipeline::open(&application_config) {
        Ok(pipeline) => pipeline,
        Err(error) => {
            let _ = sender.send(WorkerMessage::Failed(error.to_string()));
            return;
        }
    };
    let mut schedule = CaptureSchedule::new(CAPTURE_INTERVAL);
    schedule.start(Instant::now());
    loop {
        if !wait_for_capture(&schedule, &cancellation) {
            return;
        }
        if sender.send(WorkerMessage::CycleStarted).is_err() {
            return;
        }
        match translate_capture(&mut pipeline, &source, &cancellation) {
            Ok(result) => {
                if sender
                    .send(WorkerMessage::CycleFinished(Ok(result)))
                    .is_err()
                {
                    return;
                }
            }
            Err(ApplicationError::Cancelled) => return,
            Err(error) => {
                if sender
                    .send(WorkerMessage::CycleFinished(Err(error.to_string())))
                    .is_err()
                {
                    return;
                }
            }
        }
        if cancellation.load(Ordering::Relaxed) {
            return;
        }
        schedule.postpone(Instant::now());
        if let Some(next_capture) = schedule.next_capture()
            && sender
                .send(WorkerMessage::Waiting { next_capture })
                .is_err()
        {
            return;
        }
    }
}

fn wait_for_capture(schedule: &CaptureSchedule, cancellation: &AtomicBool) -> bool {
    loop {
        if cancellation.load(Ordering::Relaxed) {
            return false;
        }
        if schedule.is_due(Instant::now()) {
            return true;
        }
        thread::sleep(WORKER_POLL_DELAY);
    }
}

fn translate_capture(
    pipeline: &mut TranslationPipeline,
    source: &str,
    cancellation: &AtomicBool,
) -> Result<TranslationResult, ApplicationError> {
    ensure_not_cancelled(cancellation)?;
    let captured_frame = WindowsScreenshoter::new().screenshot(source)?;
    ensure_not_cancelled(cancellation)?;
    let mut image = frame_to_image(&captured_frame)?;
    let detected_frames = pipeline.frames_detector.detect(&image)?;
    ensure_not_cancelled(cancellation)?;
    let detected_frame_count = detected_frames.len();
    for detected_frame in detected_frames {
        ensure_not_cancelled(cancellation)?;
        let regions = pipeline.frames_ocr.recognize(detected_frame.frame())?;
        ensure_not_cancelled(cancellation)?;
        let translated_texts = translate_frame_regions(
            detected_frame.frame(),
            &regions,
            pipeline.text_translator.as_mut(),
        )?;
        ensure_not_cancelled(cancellation)?;
        let rendered_frame = pipeline
            .text_renderer
            .render(detected_frame.frame(), &translated_texts)?;
        ensure_not_cancelled(cancellation)?;
        let rendered_image = frame_to_image(&rendered_frame)?;
        let bounds = detected_frame.bounds();
        image::imageops::overlay(
            &mut image,
            &rendered_image,
            i64::from(bounds.x()),
            i64::from(bounds.y()),
        );
    }
    Ok(TranslationResult {
        image,
        detected_frame_count,
    })
}

fn ensure_not_cancelled(cancellation: &AtomicBool) -> Result<(), ApplicationError> {
    if cancellation.load(Ordering::Relaxed) {
        Err(ApplicationError::Cancelled)
    } else {
        Ok(())
    }
}

fn frame_to_image(frame: &Frame) -> Result<RgbaImage, ApplicationError> {
    RgbaImage::from_raw(frame.width(), frame.height(), frame.rgba_pixels().to_vec()).ok_or(
        ApplicationError::InvalidFrame {
            width: frame.width(),
            height: frame.height(),
        },
    )
}

fn spinner_frame(started_at: Instant) -> &'static str {
    let frame_index =
        usize::try_from(started_at.elapsed().as_millis() / SPINNER_INTERVAL.as_millis())
            .unwrap_or(0);
    SPINNER_FRAMES[frame_index % SPINNER_FRAMES.len()]
}

fn seconds_until(instant: Instant, now: Instant) -> u64 {
    instant.saturating_duration_since(now).as_secs_f32().ceil() as u64
}

fn sdl_failure(error: impl ToString) -> ApplicationError {
    ApplicationError::Sdl(error.to_string())
}

#[derive(Debug)]
enum ApplicationError {
    Io { path: PathBuf, source: io::Error },
    Configuration(String),
    Screenshot(ScreenshotError),
    Factory(FactoryError),
    FrameSource(FrameSourceError),
    Recognition(RecognitionError),
    Translation(TranslationError),
    Rendering(RenderingError),
    Renderer(RendererError),
    InvalidFrame { width: u32, height: u32 },
    Cancelled,
    Sdl(String),
}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            Self::Configuration(message) => {
                write!(
                    formatter,
                    "failed to parse the application configuration: {message}"
                )
            }
            Self::Screenshot(error) => write!(formatter, "{error}"),
            Self::Factory(error) => write!(formatter, "{error}"),
            Self::FrameSource(error) => write!(formatter, "{error}"),
            Self::Recognition(error) => write!(formatter, "{error}"),
            Self::Translation(error) => write!(formatter, "{error}"),
            Self::Rendering(error) => write!(formatter, "{error}"),
            Self::Renderer(error) => write!(formatter, "{error}"),
            Self::InvalidFrame { width, height } => {
                write!(
                    formatter,
                    "captured frame {width}x{height} has an invalid pixel buffer"
                )
            }
            Self::Cancelled => write!(formatter, "capture was cancelled"),
            Self::Sdl(message) => write!(formatter, "SDL error: {message}"),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Screenshot(error) => Some(error),
            Self::Factory(error) => Some(error),
            Self::FrameSource(error) => Some(error),
            Self::Recognition(error) => Some(error),
            Self::Translation(error) => Some(error),
            Self::Rendering(error) => Some(error),
            Self::Renderer(error) => Some(error),
            Self::Configuration(_) | Self::InvalidFrame { .. } | Self::Cancelled | Self::Sdl(_) => {
                None
            }
        }
    }
}

impl From<ScreenshotError> for ApplicationError {
    fn from(error: ScreenshotError) -> Self {
        Self::Screenshot(error)
    }
}

impl From<FactoryError> for ApplicationError {
    fn from(error: FactoryError) -> Self {
        Self::Factory(error)
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

impl From<RenderingError> for ApplicationError {
    fn from(error: RenderingError) -> Self {
        Self::Rendering(error)
    }
}

impl From<RendererError> for ApplicationError {
    fn from(error: RendererError) -> Self {
        Self::Renderer(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedules_the_next_capture_after_the_last_translation() {
        let started_at = Instant::now();
        let mut schedule = CaptureSchedule::new(CAPTURE_INTERVAL);
        schedule.start(started_at);
        assert!(schedule.is_due(started_at));
        let finished_at = started_at + Duration::from_secs(2);
        schedule.postpone(finished_at);
        assert!(!schedule.is_due(finished_at + Duration::from_secs(4)));
        assert!(schedule.is_due(finished_at + CAPTURE_INTERVAL));
        assert_eq!(
            schedule.next_capture(),
            Some(finished_at + CAPTURE_INTERVAL)
        );
        assert_eq!(
            seconds_until(finished_at + CAPTURE_INTERVAL, finished_at),
            5
        );
    }

    #[test]
    fn reports_cancellation() {
        let cancellation = AtomicBool::new(false);
        assert!(ensure_not_cancelled(&cancellation).is_ok());
        cancellation.store(true, Ordering::Relaxed);
        assert!(matches!(
            ensure_not_cancelled(&cancellation),
            Err(ApplicationError::Cancelled)
        ));
    }
}
