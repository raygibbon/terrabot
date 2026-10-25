# Terrabot

A local-first Ollama chatbot with Terry, a shaded, animated terminal-cell robot.
Built directly on TerraBox and TerraTUI. No image assets or graphics protocols.

## Run

Use modern stable Rust (edition 2024) and keep the repositories beside each other:

```text
work/
  terrabox/crates/{terrabox,terratui}
  terrabot/
```

```bash
# In another terminal, if Ollama isn't already running:
ollama serve
# Install the default model explicitly (Terrabot never downloads models):
ollama pull qwen3:8b

# In terrabot:
cargo run --release
```

Use a UTF-8 terminal with true-colour support and a monospace font containing
Unicode block characters. An attached controlling terminal is required. The
executable is `target/release/terrabot`. No external terminal backend is used.

Terrabot checks the configured server and model asynchronously. Failures appear
in the conversation; fix the issue and press F5. Your input remains editable
while a request runs. Enter during generation leaves the draft in place.

## Controls

| Key | Action |
| --- | --- |
| Enter | Send a message (or reconnect when disconnected) |
| Left / Right, Home / End | Move through the input by grapheme |
| Backspace / Delete | Edit input |
| Page Up / Page Down | Scroll conversation / return toward live output |
| F2 / F3 | Shrink / grow Terry's panel on larger terminals |
| Escape | Cancel connection check or generation |
| F5 | Recheck server and installed model when idle |
| Ctrl+Q / Ctrl+C | Quit and restore terminal |

Large terminals show a 36-column character, compact layouts show an 18-column
version, and terminals below 52 columns or 14 rows hide Terry. Layouts resize
immediately. Input remains at the bottom. Transcript text is plain text, not
Markdown; model-supplied control characters are filtered for display.

## Configuration

Defaults work without a file. To customize:

```bash
mkdir -p ~/.config/terrabot
cp config.example.toml ~/.config/terrabot/config.toml
```

Set `ollama.model` to an installed model, or change the endpoint.
`terrabot --config PATH` uses an alternate file. Both dark and light themes are
supported. `animation = false` keeps a static character whose expression still
reflects the current state. `show_terry = false` gives the space to conversation.
Unknown keys, invalid values and enabled security flags are rejected before
entering terminal mode. See [config.example.toml](config.example.toml).

Conversation is held only in memory. Requests go to the configured endpoint;
using a remote endpoint sends your conversation there. No tools, shell commands,
MCP servers, credentials or autonomous actions are implemented. A model's text
never triggers execution.

Input is capped at 8 KiB, each response at 128 KiB, and the transcript at 128
entries / 512 KiB (plus at most one in-flight response). Provider history retains
up to 32 completed turns, trimming oldest complete pairs before the next request
at a 256 KiB budget. Ollama applies `num_ctx` and `num_predict`; the byte budget
is not a token estimator. Failed/cancelled turns remain visible but are excluded
from subsequent model history. Retrying after failure requires F5 and resending.
Requests have a configurable total timeout; a slow cold model may need more time.

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
python3 tests/pty_smoke.py
```

The PTY smoke suite runs the real executable against a local mock Ollama server,
checking stream fragments, multi-turn role ordering, missing server/model,
cancellation, resizing, keyboard editing, clean exits and terminal restoration.
It does not replace testing against an installed model or visual review in your
own terminal. See [architecture](docs/architecture.md) and
[validation](docs/validation.md) for implementation details and actual results.

## Capture a real terminal recording

Install [asciinema](https://docs.asciinema.org/manual/cli/quick-start/) and
optionally [agg](https://docs.asciinema.org/manual/agg/usage/), then record the
actual release executable:

```bash
asciinema rec -c './target/release/terrabot' terrabot.cast
# Chat with Terry, resize the terminal if desired, then Ctrl+Q.
agg terrabot.cast terrabot.gif
```

Review the recording for private prompts before publishing. Add the resulting
GIF to the repository and link it in this README. Use a roughly 110×30 terminal
for the detailed character. Recording tools are optional development utilities;
Terrabot itself never renders generated artwork or external images.
