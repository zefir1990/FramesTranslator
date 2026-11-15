# FramesTranslator

FramesTranslator is a Rust workspace for building an image translation pipeline:

1. Read frames from a source image.
2. Detect text regions with OCR.
3. Translate recognized text.
4. Render translated text back onto frames.

The pipeline is defined through four traits and currently ships with deterministic local implementations for development and testing.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed design and extension guidance.

## Pipeline

| Stage | Trait | Responsibility |
| --- | --- | --- |
| Source | `FramesSourcer` | Produces RGBA `Frame` values and reports when the source is exhausted. |
| Recognition | `FramesOCR` | Extracts `TextRegion` values from a frame. |
| Translation | `TextTranslator` | Translates text and supports batch translation through `translate_all`. |
| Rendering | `TextRenderer` | Produces a new frame with translated text regions rendered. |

Core models:

- `Frame` stores a name, validated RGBA pixel data, and dimensions.
- `Bounds` describes a rectangular region.
- `FramesConfig` holds named frame rectangles.
- `TestsConfig` groups separate configuration structs for each test case, including expected OCR and translation results.
- `TextRegion` pairs recognized text with its bounds.
- `TranslatedText` pairs a source region with translated text.
- `frame_translation` translates wide multi-line dialogs as one block and splits the result back across the original lines.

## Implementations

| Interface | Concrete type | Behavior |
| --- | --- | --- |
| `FramesSourcer` | `ProceduralFramesDetectorSourcer` | Detects frames as connected background-color regions that contain text-color pixels, with tolerance for scaled and anti-aliased pixels. |
| `FramesSourcer` | `NeuralNetworkFramesDetectorSourcer` | Proposes regions from dominant image colors and validates them with a locally trained PyTorch structural classifier in `scripts/neural_frame_detector_worker.py`. |
| `FramesSourcer` | `ImageFramesSourcer` | Extracts named frames from `test-case/Shadowrun_02.png` using `test-case/tests-config.jsonc`. |
| `FramesSourcer` | `SyntheticFramesSourcer` | Produces fixed in-memory frames. |
| `FramesOCR` | `FixtureFramesOCR` | Returns a fixed set of text regions. |
| `FramesOCR` | `MangaFramesOCR` | Splits frames into text lines and runs Manga OCR per line. |
| `TextTranslator` | `DictionaryTextTranslator` | Translates text through a fixed dictionary. |
| `TextTranslator` | `ArgosTextTranslator` | Translates Japanese to English through Argos Translate. |
| `TextRenderer` | `BlockTextRenderer` | Paints translated regions with a fixed color. |
| `TextRenderer` | `SdlThreeTextRenderer` | Displays the Shadowrun image with translated text in an SDL3 window. |

`FixtureFramesOCR`, `DictionaryTextTranslator`, and `BlockTextRenderer` are deterministic test backends, not production OCR, machine translation, or font rendering. `MangaFramesOCR` and `ArgosTextTranslator` run Python packages through `.venv`; their models are loaded on first use. `SdlThreeTextRenderer` displays results in an SDL3 window until the user closes it.

## Configuration

Implementation selection is defined by `ApplicationConfig`. The current test configuration is `test-case/tests-application-config.jsonc`:

```jsonc
{
  "framesSourcer": "NeuralNetworkFramesDetectorSourcer",
  "framesOcr": "MangaFramesOCR",
  "textTranslator": "ArgosTextTranslator",
  "textRenderer": "SdlThreeTextRenderer",
  "framesDetectionBackgroundColors": [0x000000],
  "framesDetectionTextColors": [0xfcfcfc, 0xffffff, 0xfcfcb4],
  "verboseMode": true
}
```

`ApplicationConfig::from_jsonc` loads this file and `factory` creates the selected implementation. `framesDetectionBackgroundColors` and `framesDetectionTextColors` are arrays of 24-bit RGB values used by `ProceduralFramesDetectorSourcer`; JSONC hexadecimal literals are supported. `FramesDetectorSourcer` is the abstract trait; `ProceduralFramesDetectorSourcer` and `NeuralNetworkFramesDetectorSourcer` are concrete implementations. The procedural detector matches background and text pixels against the configured colors with channel tolerance. The neural detector does not use configured colors: it proposes regions from dominant image colors and validates them with a structural neural classifier. Detected frames are ordered top-to-bottom and left-to-right and are named `Detected Frame 1`, `Detected Frame 2`, and so on. `verboseMode` defaults to `true`; when enabled, the interactive window tints each detected frame with a translucent random color, draws the translation on top, and prints the detected frame count in green at the top-left corner. To add a backend, implement the relevant trait, add its class name to the factory, and select it in the application configuration.

Test-case expectations are loaded from `test-case/tests-config.jsonc` through `TestsConfig::from_jsonc`. It contains a `shadowrun_cd` case for the menu panels, a `shadowrun_dialog` case for the intro dialog box, a `shadowrun_street` case for the Shibuya street dialog box, and a `shadowrun_hospital` case for the upscaled hospital dialog. Each case lists the expected detected frames with their positions and dimensions, and `expectedResultOCR` and `expectedTranslationResult` dictionaries keyed by the expected frame names, so each detected frame is validated against its own OCR lines and translations. The parser accepts JSONC comments, and `TestsConfig::to_jsonc` serializes the configuration back to JSON.

## Interactive application

The `FramesTranslator` executable opens an SDL3 window that accepts image files by drag and drop. After a file is dropped, it:

1. Loads the image.
2. Detects text frames with the configured detector.
3. Recognizes and translates each detected frame. Wide dialog frames with three or more lines are translated as one block so the lines keep their context, then the translation is distributed back across the original lines.
4. Replaces each frame in place with its translations, wrapping and scaling long text so it stays inside the frame.
5. In verbose mode, tints each detected frame with a translucent random color, draws translations on top, and shows the detected frame count in green at the top-left corner.

Drop another image to process it again; press `Escape` or close the window to exit. A file path may also be passed as the first command-line argument to open an image immediately:

```powershell
cargo run -- test-case/Shadowrun_03.png
```

## Detector test application

`NeuralNetworkFramesDetectorSourcerTestApplication` is a detector-only variant of the interactive application. It uses `NeuralNetworkFramesDetectorSourcer` without OCR or translation, tints each detected frame with a random color at alpha `0.8`, and shows the detected frame count in green at the top-left corner. It accepts dropped images and an optional image path:

```powershell
cargo run --bin NeuralNetworkFramesDetectorSourcerTestApplication -- test-case/Shadowrun_02.png
```

## Library usage

```rust
use std::fs;

use frames_translator::{
    ApplicationConfig,
    create_frames_ocr,
    create_frames_sourcer,
    create_text_renderer,
    create_text_translator,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_source = fs::read_to_string("test-case/tests-application-config.jsonc")?;
    let application_config = ApplicationConfig::from_jsonc(&config_source)?;
    let mut frames_sourcer = create_frames_sourcer(&application_config)?;
    let frames_ocr = create_frames_ocr(&application_config)?;
    let mut text_translator = create_text_translator(&application_config)?;
    let text_renderer = create_text_renderer(&application_config)?;

    if let Some(frame) = frames_sourcer.next_frame()? {
        let regions = frames_ocr.recognize(&frame)?;
        let translated_texts = text_translator.translate_all(&regions)?;
        let rendered_frame = text_renderer.render(&frame, &translated_texts)?;
        let _ = rendered_frame;
    }

    Ok(())
}
```

## Project structure

```text
build.rs                       Copies the vendored SDL3 DLL into target
src/
  application_config.rs        ApplicationConfig JSONC loading
  debug_text.rs                Debug-font text wrapping, fitting, and drawing
  factory.rs                   Class-name based implementation selection
  frame.rs                     Named RGBA frame model
  frame_translation.rs         Context-aware dialog translation and line distribution
  frames_config.rs             Named frame rectangles
  frames_detector_sourcer.rs   Abstract frame detector trait and DetectedFrame
  frames_ocr.rs                OCR trait and error
  frames_sourcer.rs            Frame source trait and error
  main.rs                      Drag-and-drop SDL application entry point
  sdl_frame_viewer.rs          Shared SDL window, frame tinting, and counter
  sdl_image_translator.rs      FramesTranslator scene builder and error type
  bin/
    neural_network_frames_detector_sourcer_test_application.rs  Detector-only SDL application
  tests_config.rs              Per-test configuration structs
  text_region.rs               Bounds, TextRegion, TranslatedText
  text_renderer.rs             Renderer trait and error
  text_translator.rs           Translator trait and error
  implementations/             Concrete implementations
scripts/
  manga_ocr_worker.py           Persistent Manga OCR worker
  argos_translate_worker.py     Persistent Argos Translate worker
  neural_frame_detector_worker.py  Local PyTorch frame detector worker
tests/
  application_config_test.rs
  frames_ocr_test.rs
  frames_detector_sourcer_test.rs
  frames_sourcer_test.rs
  shadowrun_cd_pipeline_test.rs
  shadowrun_dialog_pipeline_test.rs
  shadowrun_hospital_pipeline_test.rs
  shadowrun_street_pipeline_test.rs
  tests_config_test.rs
  text_renderer_test.rs
  text_translator_test.rs
test-case/
  Shadowrun_01.png             Intro dialog sample image
  Shadowrun_02.png             Gameplay menu sample image
  Shadowrun_03.png             Street dialog sample image
  Shadowrun_04.png             Upscaled hospital dialog sample image
  Banana_01.png                Game-over dialog sample image
  tests-application-config.jsonc
  tests-config.jsonc           Serialized TestsConfig
third-party/
  sdl3/                        Vendored SDL3 import library, DLL, and license
```

## Build and test

```powershell
cargo build
cargo run
cargo run -- test-case/Shadowrun_03.png
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The test suite covers image decoding, procedural and neural detection of both Shadowrun CD panels and the intro, street, hospital, and Banana dialog boxes, color-free neural detection of the beige `Banana_01.png` dialog, tolerant matching of scaled gray text, expected dimensions and crop pixels, frame exhaustion, per-frame OCR region bounds and line extraction on dark and light backgrounds, per-frame batch translation, debug-font text wrapping and fitting, rendering output, out-of-bounds rejection, and the end-to-end SDL preview flow with a configured timeout.

## Status

The pipeline interfaces, configuration, factory, tests, and interactive SDL3 application are in place. Run `cargo run` and drop an image into the window, or pass an image path to translate it immediately. The configured timeout-based SDL preview remains available for the automated pipeline tests.

## License

MIT License. See `LICENSE`.
