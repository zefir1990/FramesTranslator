# Architecture

## Purpose

FramesTranslator is structured as a four-stage image translation pipeline:

1. `FramesSourcer` provides RGBA frames.
2. `FramesOCR` finds text regions in a frame.
3. `TextTranslator` translates recognized text.
4. `TextRenderer` draws translated regions onto a new frame.

The current repository provides the contracts, domain models, configuration, implementation factory, deterministic development backends, and integration tests. It does not yet provide production OCR, machine translation, or font rendering.

## Architectural style

The project separates four concerns:

- **Domain models** describe frames and text regions without depending on any backend.
- **Pipeline contracts** are Rust traits with typed errors.
- **Concrete implementations** adapt local resources to those contracts.
- **Composition** loads class names from `ApplicationConfig` and constructs the selected implementations through a factory.

The pipeline depends on traits. Concrete types are known only to the factory, `src/lib.rs`, and tests that intentionally exercise a specific implementation.

```mermaid
flowchart LR
    Main[src/main.rs] --> Library[src/lib.rs]
    Library --> Contracts[Pipeline traits]
    Library --> Models[Domain models]
    Library --> Config[src/application_config.rs]
    Library --> Factory[src/factory.rs]
    Factory --> Contracts
    Factory --> Implementations[src/implementations]
    Implementations --> Contracts
    Implementations --> Models
```

## Module layout

| Path | Responsibility |
| --- | --- |
| `src/frame.rs` | Named RGBA `Frame` model and `FrameError` |
| `src/frames_config.rs` | `FramesConfig` and named frame rectangles |
| `src/tests_config.rs` | `TestsConfig` and per-test configuration structs |
| `src/application_config.rs` | `ApplicationConfig` implementation selection and JSONC methods |
| `src/text_region.rs` | `Bounds`, `TextRegion`, and `TranslatedText` |
| `src/frames_sourcer.rs` | `FramesSourcer` contract and `FrameSourceError` |
| `src/frames_ocr.rs` | `FramesOCR` contract and `RecognitionError` |
| `src/text_translator.rs` | `TextTranslator` contract and `TranslationError` |
| `src/text_renderer.rs` | `TextRenderer` contract and `RenderingError` |
| `src/factory.rs` | Class-name based composition and `FactoryError` |
| `src/implementations/` | Concrete backends for the four contracts |
| `src/lib.rs` | Public module and re-export surface |
| `src/main.rs` | Binary entry point, currently empty |

## Domain models

### Frame

`Frame` owns a name and a validated RGBA8 pixel buffer.

Invariants:

- Pixel data length equals `width * height * 4`.
- Multiplication overflow is rejected through `FrameError::DimensionsOverflow`.
- A buffer length mismatch is rejected through `FrameError::PixelBufferSizeMismatch`.

The type is immutable through its public read-only accessors. Renderers copy the buffer and return a new `Frame` instead of mutating the input.

### FramesConfig

`FramesConfig` holds a `frames` list of `NamedRect` values. Each `NamedRect` pairs a name with a `Bounds` rectangle. The configuration preserves order and does not perform additional rectangle validation.

### TestsConfig

`TestsConfig` groups a separate configuration struct for each test case. It currently exposes `ShadowrunCdTestConfig`, which owns a `FramesConfig`. Additional test cases should add their own configuration struct rather than sharing one generic shape.

### Bounds

`Bounds` stores `x`, `y`, `width`, and `height`. `right()` and `bottom()` use saturating addition, so coordinate overflow cannot wrap. Bounds are not clamped to a frame; renderers validate containment before drawing.

### TextRegion and TranslatedText

`TextRegion` pairs recognized text with bounds. `TranslatedText` retains the original region and adds translated text. Retaining the source region lets the renderer preserve position and gives callers access to both the original and translated forms.

## Pipeline contracts

| Contract | Method | Ownership | Termination |
| --- | --- | --- | --- |
| `FramesSourcer` | `next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError>` | Mutable source, owned frame | `None` |
| `FramesOCR` | `recognize(&self, &Frame) -> Result<Vec<TextRegion>, RecognitionError>` | Shared frame | Empty region list |
| `TextTranslator` | `translate(&mut self, &str) -> Result<String, TranslationError>` | Mutable translator | Error for unknown text |
| `TextRenderer` | `render(&self, &Frame, &[TranslatedText]) -> Result<Frame, RenderingError>` | Shared input, new frame | Returns a new frame |

`TextTranslator` also provides a default `translate_all` method that maps each `TextRegion` through `translate` and returns `Vec<TranslatedText>`.

The translator takes `&mut self` to allow stateful implementations such as caches or sessions. OCR and rendering take `&self` because they are modeled as read-only operations over their inputs.

All four traits are object-safe, allowing the factory and application to hold `Box<dyn Trait>` values.

## Data flow

```mermaid
sequenceDiagram
    participant App
    participant Source as FramesSourcer
    participant OCR as FramesOCR
    participant Translator as TextTranslator
    participant Renderer as TextRenderer

    App->>Source: next_frame()
    Source-->>App: Option<Frame>
    App->>OCR: recognize(&frame)
    OCR-->>App: Vec<TextRegion>
    App->>Translator: translate_all(&regions)
    Translator-->>App: Vec<TranslatedText>
    App->>Renderer: render(&frame, &translated_texts)
    Renderer-->>App: Frame
```

The application owns orchestration. Individual stages do not call each other, which keeps them independently replaceable and testable.

## Configuration and composition

Implementation selection is defined by `ApplicationConfig`. The current test configuration is `test-case/tests-application-config.jsonc`:

```jsonc
{
  "framesSourcer": "ImageFramesSourcer",
  "framesOcr": "FixtureFramesOCR",
  "textTranslator": "DictionaryTextTranslator",
  "textRenderer": "BlockTextRenderer"
}
```

`ApplicationConfig` uses Serde with camelCase field names. `ApplicationConfig::from_jsonc` parses JSONC through `json5`; `ApplicationConfig::to_jsonc` emits pretty JSON through `serde_json`.

`src/factory.rs` is the composition root. Each factory function reads one implementation name from `ApplicationConfig` and constructs the matching implementation:

| Factory function | Configuration getter | Configuration key |
| --- | --- | --- |
| `create_frames_sourcer` | `frames_sourcer()` | `framesSourcer` |
| `create_frames_ocr` | `frames_ocr()` | `framesOcr` |
| `create_text_translator` | `text_translator()` | `textTranslator` |
| `create_text_renderer` | `text_renderer()` | `textRenderer` |

`ImageFramesSourcer` uses the fixed image `test-case/Shadowrun_02.png` and the frame definitions in `test-case/tests-config.jsonc` when created through the factory. The constructor `ImageFramesSourcer::open(path, frames_config)` remains available for explicit image paths and frame definitions. Because the application configuration is restricted to class names, backend-specific settings are currently constants inside implementations rather than configuration values.

### Configuration serialization

`ApplicationConfig`, `TestsConfig`, and their nested configuration types derive Serde serialization and deserialization. All JSONC entry points use `json5`, which supports comments and trailing commas. Serialization uses `serde_json` and emits pretty JSON. `test-case/tests-application-config.jsonc` and `test-case/tests-config.jsonc` are the serialized representations of the current configurations.

`ShadowrunCdTestConfig` flattens its `FramesConfig`, so the JSONC shape is:

```jsonc
{
  "shadowrun_cd": {
    "frames": [
      {
        "name": "Actions Menu",
        "rect": { "x": 18, "y": 86, "width": 91, "height": 68 }
      }
    ]
  }
}
```

## Concrete implementations

| Type | Contract | Behavior | Limitations |
| --- | --- | --- | --- |
| `ImageFramesSourcer` | `FramesSourcer` | Decodes a PNG and extracts the named frame rectangles from `TestsConfig` | Single image, no video or directory input |
| `SyntheticFramesSourcer` | `FramesSourcer` | Yields two fixed `32x24` frames and then `None` | In-memory test data |
| `FixtureFramesOCR` | `FramesOCR` | Returns two fixed text regions | Does not inspect the frame |
| `DictionaryTextTranslator` | `TextTranslator` | Maps two fixed source strings to translated strings | Exact-match dictionary only |
| `BlockTextRenderer` | `TextRenderer` | Fills each translated region with a fixed orange color | Draws blocks, not glyphs |

`ImageFramesSourcer` depends on the `image` crate with only the PNG feature enabled.

## Error strategy

Each layer has a typed error enum implementing `std::error::Error`:

| Error | Variants |
| --- | --- |
| `FrameError` | `DimensionsOverflow`, `PixelBufferSizeMismatch` |
| `FrameSourceError` | `Io`, `Decoding`, `RegionOutOfBounds` |
| `RecognitionError` | `UnsupportedFrame`, `RecognitionFailed` |
| `TranslationError` | `Unavailable`, `InvalidResponse` |
| `RenderingError` | `RegionOutOfBounds`, `UnsupportedFrame`, `InvalidFrame` |
| `FactoryError` | `FrameSource`, `Io`, `TestsConfig`, `UnknownImplementation` |

Errors are propagated with `?`. `FactoryError` preserves source errors where a dedicated variant exists. No stage panics on malformed external input; invalid dimensions, missing configuration, unknown class names, and out-of-bounds rendering are represented as errors.

## Testing architecture

Integration tests live in `tests/`:

- `tests/support/mod.rs` loads `test-case/tests-application-config.jsonc` from `CARGO_MANIFEST_DIR`.
- `frames_sourcer_test.rs`, `frames_ocr_test.rs`, `text_translator_test.rs`, and `text_renderer_test.rs` obtain concrete types through the factory.
- `application_config_test.rs` loads and validates the application configuration and round-trips it through JSONC.
- `tests_config_test.rs` loads `test-case/tests-config.jsonc`, validates its contents, and round-trips it through JSONC serialization and deserialization.

Coverage includes:

- PNG decoding, configured frame extraction, crop dimensions and pixel fidelity,
- source termination,
- exact OCR regions and frame containment,
- single and batch translation,
- unknown dictionary entries,
- rendered region color, unchanged frame dimensions,
- out-of-bounds region rejection,
- application configuration loading and round-trip serialization,
- test configuration loading and round-trip serialization.

Tests run with `cargo test`. Linting uses `cargo clippy --all-targets -- -D warnings`; formatting uses `cargo fmt --check`.

## Design decisions

### Trait objects at composition boundaries

Traits keep the pipeline independent of backends. Factory functions return `Box<dyn Trait>` because `ApplicationConfig` selects implementations at runtime and each stage has a single active backend. Static generics would require the selection to be known at compile time.

### Synchronous contracts

The current contracts are synchronous and single-threaded. This keeps the pipeline simple while backend behavior is still being established. Async OCR or network translation can be added later through new traits or a deliberate contract change.

### Owned frames

`Frame` owns its pixel buffer. Sources transfer ownership to callers, and renderers return new frames. This avoids hidden aliasing and keeps frame history straightforward at the cost of copying the buffer during rendering.

### Typed errors per stage

Each stage has an error type specific to its failure modes. A shared pipeline error would reduce type precision and make backend-specific failures harder to distinguish.

### Class names only in application configuration

Configuration is limited to selecting implementations. This keeps configuration files stable and free of secrets or backend-specific options. The tradeoff is that concrete backends currently need constructor defaults for settings such as file paths, colors, and fixture data.

## Extension guide

To add a backend:

1. Implement the relevant trait under `src/implementations/`.
2. Export the type from `src/implementations/mod.rs`.
3. Add a match arm in the corresponding factory function in `src/factory.rs`.
4. Select the class name in `test-case/tests-application-config.jsonc`.
5. Add an integration test that exercises the contract.

Production backends will likely need additional design work:

- OCR: a local OCR engine or an external service adapter.
- Translation: a service adapter with credentials and rate-limit handling.
- Rendering: glyph layout, font loading, text fitting, and background preservation.
- Source: video decoding, directory traversal, and frame timestamps.

If backends require configurable paths, credentials, or tuning parameters, the configuration model should be extended deliberately rather than adding ad hoc file reads inside implementations.

## Current limitations

- `src/main.rs` is empty; there is no CLI or end-to-end application yet.
- OCR and translation implementations are deterministic fixtures, not real services.
- Rendering produces colored blocks rather than text glyphs.
- Only PNG input is supported.
- Frames are processed synchronously and fully in memory.
- `ApplicationConfig` supports implementation selection only.
