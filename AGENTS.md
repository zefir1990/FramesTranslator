# AGENTS.md

## Repository

FramesTranslator is a Rust image translation pipeline. The current codebase defines the pipeline contracts, domain models, configuration, implementation factory, deterministic local backends, and integration tests. It is not yet an end-to-end application: `src/main.rs` is empty, OCR and translation are fixtures, and rendering draws colored blocks instead of glyphs.

## Stack

- Rust, edition 2024
- Binary target: `FramesTranslator`
- Library target: `frames_translator`
- Dependency: `image` with only the `png` feature enabled
- Serialization: `serde`, `serde_json`, and `json5`
- OCR backend: `manga-ocr` in `.venv` with Python 3.13 and `scripts/manga_ocr_worker.py`
- Translation backend: `argostranslate` in `.venv` with Japanese packages and `scripts/argos_translate_worker.py`
- No async runtime, database, network client, or dotenv dependency

## Commands

Run these before finishing any change:

```powershell
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

Use `cargo check --all-targets` for a faster compile check when iterating. The clippy command must pass with warnings denied.

## Architecture

The pipeline has four object-safe traits:

| Trait | File | Contract |
| --- | --- | --- |
| `FramesSourcer` | `src/frames_sourcer.rs` | `next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError>` |
| `FramesOCR` | `src/frames_ocr.rs` | `recognize(&self, &Frame) -> Result<Vec<TextRegion>, RecognitionError>` |
| `TextTranslator` | `src/text_translator.rs` | `translate(&mut self, &str) -> Result<String, TranslationError>` and default `translate_all` |
| `TextRenderer` | `src/text_renderer.rs` | `render(&self, &Frame, &[TranslatedText]) -> Result<Frame, RenderingError>` |

Supporting modules:

- `src/frame.rs`: named, validated RGBA8 `Frame`; buffer length must be `width * height * 4`.
- `src/frames_config.rs`: `FramesConfig` and `NamedRect` for named frame rectangles.
- `src/tests_config.rs`: `TestsConfig` with a separate config struct per test.
- `src/application_config.rs`: `ApplicationConfig` implementation selection with JSONC loading.
- `src/text_region.rs`: `Bounds`, `TextRegion`, and `TranslatedText`.
- `src/factory.rs`: composition root; maps class names from `ApplicationConfig` to concrete implementations.
- `src/implementations/manga_frames_ocr.rs`: persistent Manga OCR Python worker adapter.
- `src/implementations/`: concrete backends.

Detailed design documentation is in `docs/ARCHITECTURE.md`. User-facing documentation is in `README.md`. Keep both accurate when architecture or behavior changes.

## Configuration

`ApplicationConfig` contains implementation class names only. The test application configuration is `test-case/tests-application-config.jsonc`:

```jsonc
{
  "framesSourcer": "ImageFramesSourcer",
  "framesOcr": "MangaFramesOCR",
  "textTranslator": "ArgosTextTranslator",
  "textRenderer": "BlockTextRenderer"
}
```

Do not add paths, colors, sizes, credentials, or feature flags to `ApplicationConfig` without an explicit requirement to change the configuration model. Backend-specific values currently live as private constants in the implementation files. If configurable settings become necessary, extend the configuration structs and the factory deliberately and update JSONC files, tests, README, and architecture docs.

`ImageFramesSourcer` uses `test-case/Shadowrun_02.png` and extracts the named frames defined in `test-case/tests-config.jsonc` when constructed through the factory. Its public `open(path, frames_config)` constructor accepts explicit paths and frame definitions.

`ApplicationConfig` is deserialized with `ApplicationConfig::from_jsonc` and serialized with `ApplicationConfig::to_jsonc`. JSONC comments are supported through `json5`.

`test-case/tests-config.jsonc` is deserialized into `TestsConfig` with `TestsConfig::from_jsonc` and serialized with `TestsConfig::to_jsonc`. It includes the expected OCR lines under `expectedResultOCR` and the expected translations under `expectedTranslationResult`.

## Concrete implementations

| Type | Behavior |
| --- | --- |
| `ImageFramesSourcer` | Extracts the named frames configured in `tests-config.jsonc` from one PNG |
| `SyntheticFramesSourcer` | Yields two fixed `32x24` frames, then `None` |
| `FixtureFramesOCR` | Returns two fixed text regions without inspecting the frame |
| `MangaFramesOCR` | Splits frames into text lines and runs Manga OCR per line through a persistent Python worker in `.venv` |
| `DictionaryTextTranslator` | Exact-match lookup from a fixed two-entry dictionary |
| `ArgosTextTranslator` | Japanese to English translation through a persistent Argos Translate worker |
| `BlockTextRenderer` | Fills translated region bounds with a fixed orange color |

These are development and test backends, not production implementations.

## Tests

Integration tests are in `tests/`:

- `tests/support/mod.rs` loads `test-case/tests-application-config.jsonc` from `CARGO_MANIFEST_DIR`.
- `tests/frames_ocr_test.rs` extracts the `"Actions Menu"` frame through the source factory, recognizes four text lines through `MangaFramesOCR`, and asserts the exact line texts.
- `tests/text_translator_test.rs` translates Japanese text through `ArgosTextTranslator` and verifies the batch `translate_all` method.
- `tests/shadowrun_cd_pipeline_test.rs` runs the full source → OCR → translation pipeline and requires four recognized and translated Shadowrun CD menu lines.
- `tests/frames_sourcer_test.rs` constructs the configured source through the factory and verifies the default `test-case/Shadowrun_02.png` image.
- `tests/application_config_test.rs` loads and validates the application configuration and round-trips it through JSONC.
- `tests/tests_config_test.rs` loads `test-case/tests-config.jsonc`, validates it, and round-trips `TestsConfig` through JSONC.

The sample image is `320x224`, and the configured `"Actions Menu"` frame is `91x68` at `(18, 86)`. Tests assert PNG decoding, configured frame extraction and crop pixels, Manga OCR text extraction, frame exhaustion, batch translation, rendering output, and out-of-bounds rejection.

When adding an implementation:

1. Implement the relevant trait under `src/implementations/`.
2. Export the type from `src/implementations/mod.rs`.
3. Add a factory match arm in `src/factory.rs`.
4. Select the class name in `test-case/tests-application-config.jsonc`.
5. Add or update an integration test.
6. Update `README.md`, `docs/ARCHITECTURE.md`, and this file if behavior or conventions change.

## Code conventions

- Do not write code comments. Names and structure must be self-documenting.
- Prefer clean code, decomposition, DRY, SOLID, single responsibility, GoF patterns where appropriate, and strict typing.
- Use full names for variables and constants.
- Avoid unnecessary blank lines.
- Prefer standard-library and native ecosystem libraries over custom replacements.
- Keep each pipeline stage independent; stages must not call each other.
- Preserve typed errors per stage. Do not replace them with strings or a shared catch-all error.
- Keep traits object-safe because the factory returns `Box<dyn Trait>`.
- Preserve `Frame` invariants: RGBA8 data only, validated dimensions, no unchecked buffer access.
- Renderers must not mutate the input frame; return a new `Frame`.
- Validate region containment before drawing. `Bounds::right()` and `Bounds::bottom()` are saturating and do not guarantee containment.
- Keep `test-case/tests-application-config.jsonc` and `test-case/tests-config.jsonc` synchronized with their configuration structs.

## Current limitations

- No CLI or end-to-end application flow.
- No real OCR engine or translation service.
- No glyph or font rendering.
- PNG only.
- Synchronous, single-threaded processing.
- All frames are processed fully in memory.

## License

MIT. See `LICENSE`.
