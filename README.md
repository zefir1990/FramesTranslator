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

- `Frame` stores validated RGBA pixel data and dimensions.
- `Bounds` describes a rectangular region.
- `TextRegion` pairs recognized text with its bounds.
- `TranslatedText` pairs a source region with translated text.

## Implementations

| Interface | Concrete type | Behavior |
| --- | --- | --- |
| `FramesSourcer` | `ImageFramesSourcer` | Decodes `test-case/Shadowrun_02.png` into one RGBA frame. |
| `FramesSourcer` | `SyntheticFramesSourcer` | Produces fixed in-memory frames. |
| `FramesOCR` | `FixtureFramesOCR` | Returns a fixed set of text regions. |
| `TextTranslator` | `DictionaryTextTranslator` | Translates text through a fixed dictionary. |
| `TextRenderer` | `BlockTextRenderer` | Paints translated regions with a fixed color. |

`FixtureFramesOCR`, `DictionaryTextTranslator`, and `BlockTextRenderer` are deterministic test backends, not production OCR, machine translation, or font rendering.

## Configuration

Copy the example configuration:

```powershell
Copy-Item .env.example .env
```

`.env` contains class names only:

```env
FRAMES_SOURCER=ImageFramesSourcer
FRAMES_OCR=FixtureFramesOCR
TEXT_TRANSLATOR=DictionaryTextTranslator
TEXT_RENDERER=BlockTextRenderer
```

`Config` loads this file and `factory` creates the selected implementation. To add a backend, implement the relevant trait and add its class name to the factory and `.env`.

## Library usage

```rust
use frames_translator::{
    Config,
    create_frames_ocr,
    create_frames_sourcer,
    create_text_renderer,
    create_text_translator,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(".env")?;
    let mut frames_sourcer = create_frames_sourcer(&config)?;
    let frames_ocr = create_frames_ocr(&config)?;
    let mut text_translator = create_text_translator(&config)?;
    let text_renderer = create_text_renderer(&config)?;

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
  config.rs                    Typed .env loading
  factory.rs                   Class-name based implementation selection
  frame.rs                     RGBA frame model
  frames_ocr.rs                OCR trait and error
  frames_sourcer.rs            Frame source trait and error
  text_region.rs               Bounds, TextRegion, TranslatedText
  text_renderer.rs             Renderer trait and error
  text_translator.rs           Translator trait and error
  implementations/             Concrete implementations
tests/
  frames_ocr_test.rs
  frames_sourcer_test.rs
  text_renderer_test.rs
  text_translator_test.rs
test-case/
  Shadowrun_02.png             Sample input image
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
