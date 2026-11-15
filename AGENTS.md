# AGENTS.md

## Repository

FramesTranslator is a Rust image translation pipeline. The current codebase defines the pipeline contracts, domain models, configuration, implementation factory, deterministic local backends, and integration tests. It is not yet an end-to-end application: `src/main.rs` is empty, OCR and translation are fixtures, and rendering draws colored blocks instead of glyphs.

## Stack

- Rust, edition 2024
- Binary target: `FramesTranslator`
- Library target: `frames_translator`
- Dependency: `image` with only the `png` feature enabled
- Serialization: `serde`, `serde_json`, and `json5`
- Frame detection backend: PyTorch in `.venv` with `scripts/neural_frame_detector_worker.py`
- OCR backend: `manga-ocr` in `.venv` with Python 3.13 and `scripts/manga_ocr_worker.py`
- Translation backend: `argostranslate` in `.venv` with Japanese packages and `scripts/argos_translate_worker.py`
- Windowing: `sdl3` with vendored SDL3 3.4.20 under `third-party/sdl3`
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
- `src/frame_translation.rs`: context-aware dialog translation and distribution across original lines.
- `src/frames_config.rs`: `FramesConfig` and `NamedRect` for named frame rectangles.
- `src/tests_config.rs`: `TestsConfig` with a separate config struct per test.
- `src/application_config.rs`: `ApplicationConfig` implementation selection with JSONC loading.
- `src/text_region.rs`: `Bounds`, `TextRegion`, and `TranslatedText`.
- `src/factory.rs`: composition root; maps class names from `ApplicationConfig` to concrete implementations.
- `src/main.rs`: interactive SDL application that processes dropped images.
- `src/sdl_image_translator.rs`: drag-and-drop event loop and in-place translation overlays.
- `src/frames_detector_sourcer.rs`: abstract `FramesDetectorSourcer` trait and `DetectedFrame`.
- `src/implementations/procedural_frames_detector_sourcer.rs`: background-color and text-color frame detection.
- `src/implementations/neural_network_frames_detector_sourcer.rs`: local PyTorch frame detector adapter.
- `src/implementations/manga_frames_ocr.rs`: persistent Manga OCR Python worker adapter.
- `src/implementations/sdl_three_text_renderer.rs`: SDL3 window renderer.
- `build.rs`: copies the vendored SDL3 DLL into target directories.
- `third-party/sdl3`: vendored SDL3 3.4.20 import library, DLL, and license.
- `src/implementations/`: concrete backends.

Detailed design documentation is in `docs/ARCHITECTURE.md`. User-facing documentation is in `README.md`. Keep both accurate when architecture or behavior changes.

## Configuration

`ApplicationConfig` contains implementation class names and the detector colors. The test application configuration is `test-case/tests-application-config.jsonc`:

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

`framesDetectionBackgroundColors` and `framesDetectionTextColors` are arrays of 24-bit RGB values used by `ProceduralFramesDetectorSourcer`. `FramesDetectorSourcer` is the abstract detector contract. The procedural detector matches pixels within a small channel tolerance of the configured colors, which handles anti-aliased text from scaled screenshots. `NeuralNetworkFramesDetectorSourcer` ignores configured colors. `verboseMode` defaults to `true`; when enabled, the interactive application prints the detected frame count in green at the top-left corner. The expected detected frames and their dimensions are listed in `tests-config.jsonc`.

Do not add paths, colors, sizes, credentials, or feature flags to `ApplicationConfig` without an explicit requirement to change the configuration model. Backend-specific values currently live as private constants in the implementation files. If configurable settings become necessary, extend the configuration structs and the factory deliberately and update JSONC files, tests, README, and architecture docs.

The selected detector uses `test-case/Shadowrun_02.png` when constructed through `create_frames_sourcer`. `ProceduralFramesDetectorSourcer` matches background and text colors with fixed tolerances. `NeuralNetworkFramesDetectorSourcer` starts `.venv/Scripts/python.exe scripts/neural_frame_detector_worker.py`, proposes regions from dominant image colors, and validates them with a locally trained PyTorch classifier over structural features: normalized size, aspect ratio, fill share, contrast density, and area share. The neural detector never receives configured colors. Both detect the two menu panels in reading order and name them `Detected Frame 1` and `Detected Frame 2`.

`ImageFramesSourcer` remains available and extracts the named frames defined in `test-case/tests-config.jsonc` when selected. Its public `open(path, frames_config)` constructor accepts explicit paths and frame definitions.

`ApplicationConfig` is deserialized with `ApplicationConfig::from_jsonc` and serialized with `ApplicationConfig::to_jsonc`. JSONC comments are supported through `json5`.

`test-case/tests-config.jsonc` is deserialized into `TestsConfig` with `TestsConfig::from_jsonc` and serialized with `TestsConfig::to_jsonc`. `TestsConfig` exposes `shadowrun_cd`, `shadowrun_dialog`, `shadowrun_street`, and `shadowrun_hospital` cases. Each case's `frames` lists the expected detected frames with their names, positions, and dimensions. `expectedResultOCR` and `expectedTranslationResult` are dictionaries keyed by expected frame name. `timeoutSeconds` sets the SDL preview duration per frame.

## Concrete implementations

| Type | Behavior |
| --- | --- |
| `ImageFramesSourcer` | Extracts the named frames configured in `tests-config.jsonc` from one PNG |
| `ProceduralFramesDetectorSourcer` | Detects connected background-color regions containing text-color pixels with fixed tolerances |
| `NeuralNetworkFramesDetectorSourcer` | Proposes dominant-color regions and validates them with a local PyTorch structural classifier |
| `SyntheticFramesSourcer` | Yields two fixed `32x24` frames, then `None` |
| `FixtureFramesOCR` | Returns two fixed text regions without inspecting the frame |
| `MangaFramesOCR` | Splits frames into text lines with horizontal ink padding and runs Manga OCR per line through a persistent Python worker in `.venv` |
| `DictionaryTextTranslator` | Exact-match lookup from a fixed two-entry dictionary |
| `ArgosTextTranslator` | Japanese to English translation through a persistent Argos Translate worker |
| `BlockTextRenderer` | Fills translated region bounds with a fixed orange color |
| `SdlThreeTextRenderer` | Shows the Shadowrun image and translated text in an SDL3 window until closed |

These are development and test backends, not production implementations.

## Tests

Integration tests are in `tests/`:

- `tests/support/mod.rs` loads `test-case/tests-application-config.jsonc` from `CARGO_MANIFEST_DIR`.
- `tests/frames_detector_sourcer_test.rs` detects the two Shadowrun CD menu panels and the intro, street, and hospital dialog boxes with `ProceduralFramesDetectorSourcer`, asserts their order, dimensions, and pixels, and verifies factory selection.
- `tests/neural_network_frames_detector_sourcer_test.rs` verifies that `NeuralNetworkFramesDetectorSourcer` finds the expected bounds in all four Shadowrun sample images and in `Banana_01.png` without receiving detection colors.
- `tests/frames_ocr_test.rs` detects both Shadowrun CD frames through the source factory, recognizes four text lines per frame through `MangaFramesOCR`, and asserts the exact line texts keyed by expected frame name.
- `tests/text_translator_test.rs` translates Japanese text through `ArgosTextTranslator` and verifies the batch `translate_all` method.
- `tests/shadowrun_cd_pipeline_test.rs` runs the full source → OCR → translation → SDL rendering pipeline for both detected frames and requires four recognized and translated lines per frame. The SDL window closes after `timeoutSeconds`.
- `tests/shadowrun_dialog_pipeline_test.rs` runs the Shadowrun intro dialog through detection, `MangaFramesOCR`, `ArgosTextTranslator`, and `BlockTextRenderer`, asserting two recognized and translated lines and their rendered blocks.
- `tests/shadowrun_street_pipeline_test.rs` runs the Shadowrun Shibuya street dialog through the same pipeline and asserts its two recognized and translated lines and rendered blocks.
- `tests/shadowrun_hospital_pipeline_test.rs` runs the upscaled hospital dialog through the same pipeline and asserts its three recognized and block-translated lines and rendered blocks.
- `tests/frames_sourcer_test.rs` constructs the detector through the factory and verifies both detected frames against the configured positions and dimensions in `test-case/tests-config.jsonc`.
- `tests/application_config_test.rs` loads and validates the application configuration and round-trips it through JSONC.
- `tests/tests_config_test.rs` loads `test-case/tests-config.jsonc`, validates it, and round-trips `TestsConfig` through JSONC.

`Shadowrun_01.png` through `Shadowrun_03.png` are `320x224`; `Shadowrun_04.png` is an upscaled `488x364` screenshot. For `Shadowrun_02.png`, the detector must produce `"Actions Menu"` at `91x68` and `(18, 86)` followed by `"Contacts"` at `75x68` and `(146, 86)`. For `Shadowrun_01.png`, it must produce the `"Dialog"` box at `267x52` and `(26, 158)`. For `Shadowrun_03.png`, it must produce the `"Street Dialog"` box at `267x52` and `(26, 158)`. For the upscaled `Shadowrun_04.png`, it must produce the `"Hospital Dialog"` box at `411x84` and `(39, 261)`. `Banana_01.png` is a `504x467` game-over screen; the neural detector must find its beige dialog at `(24, 199, 460, 246)` without configured detection colors. Tests assert PNG decoding, detected dimensions and crop pixels, per-frame Manga OCR text extraction, frame exhaustion, batch translation, rendering output, and out-of-bounds rejection.

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

- `ProceduralFramesDetectorSourcer` matches background and text colors within fixed channel tolerances and uses fixed minimum size and density thresholds.
- `NeuralNetworkFramesDetectorSourcer` retrains a small PyTorch structural classifier when each worker starts and uses image-derived dominant colors only for region proposals.
- OCR and translation use local Python models with model-dependent quality; wide dialog frames with at least three lines are translated as one block, while menu frames translate each line separately.
- `BlockTextRenderer` draws colored blocks; `SdlThreeTextRenderer` and the interactive application use SDL's ASCII debug font.
- The application processes each dropped image synchronously, so the window does not repaint during detection, OCR, or translation.
- PNG only.
- Synchronous, single-threaded processing.
- All frames are processed fully in memory.

## License

MIT. See `LICENSE`.
