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
- `TestsConfig` groups separate configuration structs for each test case.
- `TextRegion` pairs recognized text with its bounds.
- `TranslatedText` pairs a source region with translated text.

## Implementations

| Interface | Concrete type | Behavior |
| --- | --- | --- |
| `FramesSourcer` | `ImageFramesSourcer` | Extracts named frames from `test-case/Shadowrun_02.png` using `test-case/tests-config.jsonc`. |
| `FramesSourcer` | `SyntheticFramesSourcer` | Produces fixed in-memory frames. |
| `FramesOCR` | `FixtureFramesOCR` | Returns a fixed set of text regions. |
| `FramesOCR` | `MangaFramesOCR` | Runs Manga OCR through a persistent Python worker. |
| `TextTranslator` | `DictionaryTextTranslator` | Translates text through a fixed dictionary. |
| `TextRenderer` | `BlockTextRenderer` | Paints translated regions with a fixed color. |

`FixtureFramesOCR`, `DictionaryTextTranslator`, and `BlockTextRenderer` are deterministic test backends, not production OCR, machine translation, or font rendering. `MangaFramesOCR` runs the Python `manga-ocr` package through `.venv` and requires the model to be downloaded on first use.

## Configuration

Implementation selection is defined by `ApplicationConfig`. The current test configuration is `test-case/tests-application-config.jsonc`:

```jsonc
{
  "framesSourcer": "ImageFramesSourcer",
  "framesOcr": "MangaFramesOCR",
  "textTranslator": "DictionaryTextTranslator",
  "textRenderer": "BlockTextRenderer"
}
```

`ApplicationConfig::from_jsonc` loads this file and `factory` creates the selected implementation. To add a backend, implement the relevant trait, add its class name to the factory, and select it in the application configuration.

Test-case frame layouts are loaded from `test-case/tests-config.jsonc` through `TestsConfig::from_jsonc`. The parser accepts JSONC comments, and `TestsConfig::to_jsonc` serializes the configuration back to JSON.

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
tests/
  application_config_test.rs
  frames_ocr_test.rs
  frames_sourcer_test.rs
  tests_config_test.rs
  text_renderer_test.rs
  text_translator_test.rs
test-case/
  Shadowrun_02.png             Sample input image
  tests-application-config.jsonc
  tests-config.jsonc           Serialized TestsConfig
```

## Build and test

```powershell
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The test suite covers image decoding, frame exhaustion, OCR region bounds, single and batch translation, rendering output, and out-of-bounds rejection.

## Status

The pipeline interfaces, configuration, factory, and tests are in place. Production OCR, translation, and font rendering backends still need to be implemented.

## License

MIT License. See `LICENSE`.
