# AGENTS.md

## Repository

FramesTranslator is a Rust image translation pipeline. The current codebase defines the pipeline contracts, domain models, configuration, implementation factory, deterministic local backends, and integration tests. It is not yet an end-to-end application: `src/main.rs` is empty, OCR and translation are fixtures, and rendering draws colored blocks instead of glyphs.

## Stack

- Rust, edition 2024
- Binary target: `FramesTranslator`
- Library target: `frames_translator`
- Dependency: `image` with only the `png` feature enabled
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

- `src/frame.rs`: validated RGBA8 `Frame`; buffer length must be `width * height * 4`.
- `src/text_region.rs`: `Bounds`, `TextRegion`, and `TranslatedText`.
- `src/config.rs`: typed `.env` parser; only string lookup is supported.
- `src/factory.rs`: composition root; maps class names from `.env` to concrete implementations.
- `src/implementations/`: concrete backends.

Detailed design documentation is in `docs/ARCHITECTURE.md`. User-facing documentation is in `README.md`. Keep both accurate when architecture or behavior changes.

## Configuration

`.env` and `.env.example` contain class names only:

```env
FRAMES_SOURCER=ImageFramesSourcer
FRAMES_OCR=FixtureFramesOCR
TEXT_TRANSLATOR=DictionaryTextTranslator
TEXT_RENDERER=BlockTextRenderer
```

Do not add paths, colors, sizes, credentials, or feature flags to `.env` without an explicit requirement to change the configuration model. Backend-specific values currently live as private constants in the implementation files. If configurable settings become necessary, extend `Config` and the factory deliberately and update both `.env` files, tests, README, and architecture docs.

`ImageFramesSourcer` uses `test-case/Shadowrun_02.png` when constructed through the factory. Its public `open(path)` constructor accepts an explicit path.

## Concrete implementations

| Type | Behavior |
| --- | --- |
| `ImageFramesSourcer` | Decodes one PNG to RGBA and yields one frame |
| `SyntheticFramesSourcer` | Yields two fixed `32x24` frames, then `None` |
| `FixtureFramesOCR` | Returns two fixed text regions without inspecting the frame |
| `DictionaryTextTranslator` | Exact-match lookup from a fixed two-entry dictionary |
| `BlockTextRenderer` | Fills translated region bounds with a fixed orange color |

These are development and test backends, not production implementations.

## Tests

Integration tests are in `tests/`:

- `tests/support/mod.rs` loads `.env` from `CARGO_MANIFEST_DIR`.
- OCR, translator, and renderer tests construct implementations through the factory.
- `tests/frames_sourcer_test.rs` opens `test-case/Shadowrun_02.png` directly with `ImageFramesSourcer`.

The sample image is `320x224`. Tests assert PNG decoding, frame exhaustion, exact OCR regions, batch translation, rendering output, and out-of-bounds rejection.

When adding an implementation:

1. Implement the relevant trait under `src/implementations/`.
2. Export the type from `src/implementations/mod.rs`.
3. Add a factory match arm in `src/factory.rs`.
4. Select the class name in `.env` and `.env.example`.
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
- Keep `.env` and `.env.example` synchronized.

## Current limitations

- No CLI or end-to-end application flow.
- No real OCR engine or translation service.
- No glyph or font rendering.
- PNG only.
- Synchronous, single-threaded processing.
- All frames are processed fully in memory.

## License

MIT. See `LICENSE`.
