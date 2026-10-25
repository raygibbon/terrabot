# Validation — 2026-10-25

Executed on Linux x86_64 with Rust 1.99.1. Local TerraBox/TerraTUI source revision:
`775f46f`. Terrabox's working tree remained unchanged.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --all-targets -- -D warnings` | Passed |
| `cargo test` | Passed, 10 tests |
| `cargo build --release` | Passed; executable at `target/release/terrabot` |
| `python3 tests/pty_smoke.py` | Passed, 6 scenarios |

Unit coverage includes configuration defaults and rejection, fragmented Unicode
NDJSON, server errors and frame size limits, animation transitions and independent
eye/orb motion, Unicode/word wrapping, transcript bounds, input limits without
swallowing quit, and all eight expressions at zero/tiny/normal canvas sizes.

The PTY suite drives the release executable through real terminal descriptors
against a controlled local Ollama-compatible HTTP fixture. It verifies:

- Unreachable server and missing model messages.
- Incremental Unicode output and ordered user/assistant/user request history.
- Editing the next draft while the first response and Terry animate.
- Resizing through 110×30, 65×20, 28×8, 5×3 and 1×1 and restoring input focus.
- Page navigation, request cancellation and quitting during a stalled request.
- Premature stream failure and disabled-animation idle without redraws.
- Ctrl+C/Ctrl+Q exits, restored termios attributes and alternate-screen exit.
- Invalid security configuration rejected before terminal initialization.

The initial resize scenario exposed lost input focus after returning from 1×1.
Using a TerraTUI Group with input as its sole focusable child fixed it; the final
suite passes. A captured terminal screen was also visually reviewed for the
character, layout and text flow; word wrapping was improved following that check.

No Ollama server was listening at the default local endpoint in this environment.
Real-model inference, GPU behavior and a human-operated GUI terminal session were
not verified. Streaming protocol and terminal behavior were exercised using the
fixture, not represented as a real-model test. Windows/macOS were not exercised.

For a real-model check, start Ollama, install the configured model, run
`cargo run --release`, send two related questions, edit a draft while generating,
resize the terminal and try Escape and Ctrl+Q. Recording instructions are in the
README. No model downloads or existing Ollama container changes were performed.
