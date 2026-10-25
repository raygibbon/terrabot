# Terrabot v0.2 — Terri

Terri is a curious, technically capable AI companion who lives in your terminal.
Her handcrafted lilac coat, mint orb, expressive eyes and tiny feet are drawn
entirely with terminal cells. She explores, investigates, plays, rests and reacts
to conversation while your local Ollama model supplies her replies.

Her idle activities are cosmetic. She never demands care, suffers for being left
alone, or triggers tools, shell commands, file access or model requests herself.

## Start chatting

Use stable Rust and these sibling repositories:

```text
work/
  terrabox/crates/{terrabox,terratui}
  terragame/crates/terragame-core
  terrabot/
```

```bash
# Start Ollama separately if it is not running (use an installed chat model):
ollama serve

# From terrabot:
cargo run --release
```

Use a UTF-8 terminal with a monospace font and true colour for the best result.
The executable is `target/release/terrabot`. Terrabot checks the server and model
asynchronously, reports failures in a replaceable status area, and refreshes discovery with F5. It does not
download models. You can edit the next draft while a reply streams; Enter during
generation leaves that draft intact.

## Try Terri without Ollama

```bash
cargo run --release --example terri-demo
cargo run --release --example terri-demo -- --auto
```

| Demo key | Action |
| --- | --- |
| M | Cycle the eleven moods |
| N | Trigger the next of ten activities |
| B / E | Increase boredom / reduce energy |
| S / W | Sleep / wake and attend |
| F | Simulate speaking for three seconds |
| D | Simulate a security denial; executes nothing |
| R | Reset the simulation |
| A | Toggle automatic activity cycle |
| P | Toggle animation |
| − / + | Reduce / increase motion intensity |
| Q / Escape / Ctrl+C | Quit |

The demo does not load or save your personality file. Optional arguments:
`--fps 20`, `--seconds 10`, `--indexed`, `--ascii`. Timed runs print observed
render pacing after restoring the terminal. The automatic cycle changes activity
every four seconds; leave it running for forty seconds to see all ten.

## Chat controls

| Key | Action |
| --- | --- |
| Enter | Send; reconnect if disconnected |
| Left / Right, Home / End | Move through the draft by grapheme |
| Backspace / Delete | Edit; bracketed paste is supported |
| Page Up / Page Down | Scroll conversation |
| Escape | Cancel the active request |
| F2 / F3 | Resize Terri's panel on larger terminals |
| F5 | Recheck server and model |
| F6 | Open model picker; arrows navigate, Enter selects, Escape closes |
| F7 | Toggle normal / reduced motion |
| F8 | Hide / show Terri |
| F9 | Toggle animation |
| Ctrl+Q / Ctrl+C | Quit and restore terminal |

Small panels use a half-size character. Terminals below 52 columns or 14 rows
hide her to preserve chat space. Her clipped viewport cannot overwrite input or
conversation. A character-rendering error pauses her display while chat continues.

## Configuration and saved personality

```bash
mkdir -p ~/.config/terrabot
cp config.example.toml ~/.config/terrabot/config.toml
```

Defaults work without a file. Use `--config PATH` for another file. See the
[complete example](config.example.toml) for model, context/output limits, frame
rate, intensity, palette, persistence and behaviour weights. Unknown keys and
invalid values are rejected before entering terminal mode. Execution enable flags
must remain false. Existing `show_terry` settings remain valid; change an explicit
old `agent.name = "Terry"` to `"Terri"` to adopt her new name.

Traits, energy, mood, preferred activities, RNG state and interaction counters
are saved on clean shutdown to `~/.local/state/terrabot/terri.json`, or beneath
`$XDG_STATE_HOME` when set to an absolute path. Writes use a versioned JSON format,
a private temporary file and atomic replacement. Invalid or future-version files
are preserved; that session runs without saving. No conversation text is stored
there. Closed time is not simulated. `pet.persist = false` disables state I/O.

Chat history remains in memory and goes only to your configured Ollama endpoint.
A remote endpoint receives your conversation. Input is capped at 8 KiB and a
response at 128 KiB. The transcript retains up to 128 entries / 512 KiB plus an
in-flight response. Model history trims complete oldest turns before requests at
32 turns / a 256 KiB byte budget. Failed or cancelled turns are visible but excluded
from future model context. These byte limits do not estimate tokens; Ollama also
receives `num_ctx` and `num_predict`.

## Personality and optional AI cues

A concise system profile makes Terri helpful, curious and thoughtful, without
pretending she has human consciousness or narrating every movement. Local events
control attention, thinking and speaking. Conservative text heuristics recognize
success and distress; serious conversations suppress playful cues and celebration.
They are cosmetic heuristics, not an assessment of a person's emotional state.

`ollama.emotion_cues = true` enables a separate structured inference request after
a reply completes. It never delays that reply's first token. Only known mood and
gesture enums and finite intensity values from 0 to 1 are accepted. Local priority
and tone rules can reject even valid suggestions. Typing or sending a new prompt
cancels a pending cue. Unsupported models, malformed cues and timeouts fall back
to local behaviour. Cue JSON is never mixed into the conversation. See
[Ollama's structured-output API](https://docs.ollama.com/capabilities/structured-outputs).

## Development and measurements

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
cargo build --release --examples
python3 tests/pty_smoke.py
cargo run --release --example terri-benchmark
python3 tests/benchmark_terminal.py
```

The PTY tests run the real app and demo, using a local mock Ollama server for
streaming and cues. They isolate saved state in temporary directories. Read the
[architecture](docs/architecture.md), [validation](docs/validation.md), and
[performance measurements](docs/performance.md). Twenty FPS is the default;
higher settings are measured options, not guarantees for every terminal emulator.

## Record authentic terminal animation

Install [asciinema](https://docs.asciinema.org/manual/cli/quick-start/) and optionally
[agg](https://docs.asciinema.org/manual/agg/usage/), then record the actual program:

```bash
cargo build --release --examples
asciinema rec -c './target/release/examples/terri-demo --auto' terri.cast
# Quit with Q when finished.
agg terri.cast terri.gif

# Or record a real conversation:
asciinema rec -c './target/release/terrabot' terrabot.cast
```

Review recordings for private text before sharing. A 110×30 terminal shows the
full character. Add your actual recording/GIF to the README if desired. No generated
image is presented as a screenshot, and the application uses no image assets,
graphics protocols, graphical window or second terminal backend.

## Ollama model discovery and selection

`[ollama] model = "auto"` is the default. Startup and F5 query `/api/tags`.
Selection prefers an installed explicit configuration, then the saved selection
for this endpoint, then a suitable chat model. Automatic selection checks completion
capabilities when supplied; older servers use a name heuristic to exclude embedding
and reranking models. General conversation names precede code/agent names, with
lexical name ordering as the deterministic tie-breaker. Size does not influence ranking.
Capability discovery is imperfect on older Ollama servers; F6 allows manual selection.

A missing explicit model (including old `qwen3:8b` configurations) produces a visible
notice naming the fallback. F6 displays installed alternatives and their sizes.
No model is downloaded and your configuration is preserved. If no chat candidate
exists, Terrabot explains the problem instead of sending a request.

F6 selections take effect immediately, cancel any unfinished request, and keep
completed conversation history. The endpoint-scoped choice is saved atomically in
`$XDG_STATE_HOME/terrabot/models.json` (default `~/.local/state/terrabot/models.json`),
separate from personality and configuration. An installed explicit configuration
still takes priority on startup/F5; use `model = "auto"` to prefer the saved choice.
Errors saving a selection are shown in the status area.

Connection failures/timeouts get at most two automatic discovery retries, after
2 and 4 seconds. Generation is never automatically replayed. F5 starts a fresh
check and cancels an active request. Partial or cancelled responses remain visible
but are excluded from model history. Loading/thinking, generation, timeout and
interrupted-stream states are reported separately. Esc cancels an active request.
