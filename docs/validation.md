# Validation — Terrabot v0.2, 2026-10-25

Linux x86_64, Rust 1.99.1. Local framework revisions: TerraBox `775f46f`,
TerraGame `c7ff1fe`. Both framework working trees remained unchanged. No commits
or pushes were made.

| Check | Actual result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --all-targets -- -D warnings` | Passed |
| `cargo test` | Passed: 21 unit tests |
| `cargo build --release` | Passed |
| `cargo build --release --examples` | Passed: demo and benchmark |
| `python3 tests/pty_smoke.py` | Passed: 8 scenarios |
| `python3 tests/benchmark_terminal.py` | Passed: nine timed demo runs |
| `terri-benchmark` | Completed composition measurements at 20/30/60 FPS |
| Actual local Ollama conversation | Passed with installed `qwen2.5-coder:14b` |

Unit coverage includes mood transitions, event priorities, configured selection,
cooldowns/deterministic simulation, rest recovery, render-independent timing,
strict cue validation, atomic persistence and invalid snapshots, frame clips,
pose transitions, sprite bounds/clipping, tiny-resize focus restoration, safe
input limits, configuration rejection and fragmented Unicode streaming.

The PTY suite runs the real release executable against a controlled local HTTP
fixture. It covers unavailable server/model, multi-turn ordering with the character
system profile, typing during streaming, scrolling, 1×1 resize recovery, request
cancellation, premature stream failure, static idle, Ctrl+C/Ctrl+Q and terminal
restoration. Additional scenarios exercise separate structured cues (including
rejection of an unknown gesture), metadata-only persistence, and the standalone
demo's moods, sleep/wake, fake speaking, denial, reset, animation controls and
resizing. Every scenario isolates its state directory.

The final real-model check ran Terrabot in a PTY against Ollama on port 11434.
The installed `qwen2.5-coder:14b` completed a streamed introduction identifying
itself as Terri; the program then exited with restored terminal attributes and
alternate screen. No model was downloaded or user configuration changed. The
default `qwen3:8b` is not installed on this machine, so select an installed model
in the TOML file or install the default explicitly.

The character was visually reviewed from a capture of actual terminal output.
No generated artwork is presented as a screenshot. Performance observations and
raw samples are in [performance.md](performance.md).

Remaining verification limits: optional emotion inference was tested through the
HTTP fixture, not a real model; a human-operated GUI terminal session, Windows and
macOS were not tested. PTY tests exercise real terminal I/O and restoration but do
not measure a GUI terminal emulator's paint latency.

## Ollama discovery and model selection fix — 2026-10-25

- `cargo fmt --all --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: 24 unit tests passed; binary/doc-test targets also passed.
- `cargo build --release`: passed; release examples also built.
- `python3 tests/pty_smoke.py`: all 11 terminal integration tests passed.
- `git diff --check`: passed. Terrabox and TerraGame working trees remain clean.

New coverage includes typed tags parsing, empty lists, missing explicit models,
deterministic fallback, endpoint-scoped saved preferences, duplicate diagnostic
suppression, HTTP failure recovery, and switching models during a stalled request.
The switch test verifies the new exact model name, persistence on disk, retained
completed history and exclusion of the cancelled turn. Existing tests exercise
fragmented Unicode streaming, draft input, resize, cancellation, optional emotion
cues, interrupted streams and terminal restoration. Quit while the selector is
open is covered by the missing-model test.

Live validation queried `http://127.0.0.1:11434/api/tags`: it returned
`gemma4-agent:latest`, `gemma4:26b`, `qwen2.5-coder:14b` and `devstral:24b`.
A release-binary PTY session configured with `model = "auto"` selected
`devstral:24b` by the documented deterministic ordering. F6 displayed all four
names and sizes; Ctrl+Q from the modal restored terminal attributes and the
alternate screen. This check did not load or benchmark those models; generation
and switching were tested with the controllable HTTP fixture.

Limitations: tags without capabilities require a name heuristic. F6 permits a
manual override, including a model that may reject chat requests. Preference
writes are atomic but concurrent sessions use last-writer-wins semantics.

## Terri v0.3 life simulation — 2026-10-25

All requested checks passed: `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test` (31 unit tests),
`cargo build --release`, `cargo build --release --examples`, and
`python3 tests/pty_smoke.py` (12 tests).

New tests cover navigation before interaction, gradual needs and preferences,
night utility bias, disabling learning, cooldowns, travel time limits,
conversation priority, autonomy pause and version-1 migration with original-file
preservation. The PTY pet-control test verifies no model request is made and that
the draft survives the modal before being sent normally. Existing rendering tests
now iterate all 17 activities across tiny and large viewports.

Accelerated runs simulated 24 hours each for seeds 7 and 99, checking finite,
bounded state every 50-ms tick. A repeated seed-7 run was byte-identical. Both
visited 14 activities; longest uninterrupted activities were 46.6 and 47.75 seconds.
Celebration requires a success event, while stumbling and searching did not win
utility selection in these runs; they remain available in the developer demo.
Results, including activity distributions and evolved preferences, are in
[life-soak-v0.3.json](life-soak-v0.3.json). The soak exposed an indefinitely extended
chase during development; travel now has a ten-second budget and a regression test.

A rendered four-pose inspection checked exploration, balance, sleep and investigation
using the actual sprite composer. The original body artwork remains intact; the
cushion, plant and orb occupy consistent normalized habitat positions.

These tests use the controllable HTTP fixture for generation and do not benchmark
local LLM inference. Time-of-day input is injected for deterministic tests. The
local clock adapter is Unix-specific; other platforms use a neutral time bias.

A 90-second real-terminal autonomous demo (`--auto --diagnostics --seed 7`) also
completed without a rendering fault, restoring terminal attributes and the alternate
screen. Terrabox and TerraGame remained unchanged. No commits or pushes were made.

## Terri v0.4 animation overhaul — 2026-10-25

Passed: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test` (38 unit tests), `cargo build --release`,
`cargo build --release --examples`, and `python3 tests/pty_smoke.py` (13 tests).
New coverage includes exact frame boundaries and fractional-time accumulation,
ping-pong and introductory loops, one-shot completion, interruption priority,
bounded events, authored contact/silhouette differences, all face/body combinations,
orb catches/releases, reduced motion, pause and repeatable layered composition.
The preview PTY test exercises stepping, speed, expression, facing, guides, resize
and return to life; all prior chat, model discovery, cancellation and persistence
checks continue to pass.

A rendered contact-sheet inspection of all sixteen clips caught an undesirable
speckled blend during development. The final transition uses grounded extent
interpolation instead. Real PTY recordings were then captured: twelve seconds of
the staged v0.3 baseline and sixty-five seconds of the full v0.4 showcase. Both
restored terminal attributes and the alternate screen. See `docs/captures/README.md`.
The final cue/gaze refinements were validated with the unit and PTY suites.

The implementation preserves the v0.3 life/persistence schema and Ollama transport.
No framework changes, commits or pushes were made. Rendering quality remains
terminal/font dependent; palette and compact-layout fallbacks remain supported.
