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

## Implementations

| Interface | Concrete type | Behavior |
| --- | --- | --- |
| `FramesSourcer` | `FramesDetectorSourcer` | Detects frames as connected background-color regions that contain text-color pixels in the Shadowrun sample images. |
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
  "framesSourcer": "FramesDetectorSourcer",
  "framesOcr": "MangaFramesOCR",
  "textTranslator": "ArgosTextTranslator",
  "textRenderer": "SdlThreeTextRenderer",
  "framesDetectionBackgroundColors": [0x000000],
  "framesDetectionTextColors": [0xfcfcfc, 0xffffff, 0xfcfcb4]
}
```

`ApplicationConfig::from_jsonc` loads this file and `factory` creates the selected implementation. `framesDetectionBackgroundColors` and `framesDetectionTextColors` are arrays of 24-bit RGB values used by `FramesDetectorSourcer`; JSONC hexadecimal literals are supported. A frame matches when its background pixels use any configured background color and its text pixels use any configured text color. Detected frames are ordered top-to-bottom and left-to-right and are named `Detected Frame 1`, `Detected Frame 2`, and so on. To add a backend, implement the relevant trait, add its class name to the factory, and select it in the application configuration.

Test-case expectations are loaded from `test-case/tests-config.jsonc` through `TestsConfig::from_jsonc`. It contains a `shadowrun_cd` case for the menu panels, a `shadowrun_dialog` case for the intro dialog box, and a `shadowrun_street` case for the Shibuya street dialog box. Each case lists the expected detected frames with their positions and dimensions, and `expectedResultOCR` and `expectedTranslationResult` dictionaries keyed by the expected frame names, so each detected frame is validated against its own OCR lines and translations. The parser accepts JSONC comments, and `TestsConfig::to_jsonc` serializes the configuration back to JSON.

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
  factory.rs                   Class-name based implementation selection
  frame.rs                     Named RGBA frame model
  frames_config.rs             Named frame rectangles
  frames_ocr.rs                OCR trait and error
  frames_sourcer.rs            Frame source trait and error
  tests_config.rs              Per-test configuration structs
  text_region.rs               Bounds, TextRegion, TranslatedText
  text_renderer.rs             Renderer trait and error
  text_translator.rs           Translator trait and error
  implementations/             Concrete implementations
scripts/
  manga_ocr_worker.py           Persistent Manga OCR worker
  argos_translate_worker.py     Persistent Argos Translate worker
tests/
  application_config_test.rs
  frames_ocr_test.rs
  frames_detector_sourcer_test.rs
  frames_sourcer_test.rs
  shadowrun_cd_pipeline_test.rs
  shadowrun_dialog_pipeline_test.rs
  shadowrun_street_pipeline_test.rs
  tests_config_test.rs
  text_renderer_test.rs
  text_translator_test.rs
test-case/
  Shadowrun_01.png             Intro dialog sample image
  Shadowrun_02.png             Gameplay menu sample image
  Shadowrun_03.png             Street dialog sample image
  tests-application-config.jsonc
  tests-config.jsonc           Serialized TestsConfig
third-party/
  sdl3/                        Vendored SDL3 import library, DLL, and license
```

## Build and test

```powershell
cargo build
cargo run
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The test suite covers image decoding, color-based detection of both Shadowrun CD panels and the intro and street dialog boxes, expected dimensions and crop pixels, frame exhaustion, per-frame OCR region bounds and line extraction, per-frame batch translation, rendering output, out-of-bounds rejection, and the end-to-end SDL preview flow with a configured timeout.

## Status

The pipeline interfaces, configuration, factory, tests, and SDL3 preview flow are in place. Run `cargo run` to process the configured Shadowrun CD frame and display the translated result; the window closes after the configured timeout or when the user closes it.

## License

MIT License. See `LICENSE`.
