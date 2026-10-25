# Terrabot v0.5.0 — Floating Trixie

<div align="center">
  <img src="docs/captures/v050/trixie-preview.gif" alt="Trixie floating animation" width="360">
</div>

*Animated preview assembled from v0.5.0's handcrafted sprite poses. For real terminal output, see the [v0.5.0 captures](docs/captures/v050/README.md).*

Trixie is a curious, technically capable AI companion who lives in your terminal.
Her handcrafted lilac body, large rounded arms, expressive eyes and turquoise orb are drawn
entirely with terminal cells. She has no legs or feet and floats near her home position.
She explores, investigates, plays, rests and reacts
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

## Try Trixie without Ollama

```bash
cargo run --release --example trixie-demo
cargo run --release --example trixie-demo -- --auto
```

| Demo key | Action |
| --- | --- |
| M | Cycle the eleven moods |
| N | Trigger the next of seventeen activities |
| B / E | Increase boredom / reduce energy |
| S / W | Sleep / wake and attend |
| F | Simulate speaking for three seconds |
| D | Simulate a security denial; executes nothing |
| R | Reset the simulation |
| A | Toggle autonomous life |
| I | Toggle diagnostic overlay |
| O / C / H | Offer orb / chase / attention |
| U / L | Raise curiosity / lower comfort |
| T | Toggle day/night input for testing |
| P | Toggle animation |
| − / + | Reduce / increase motion intensity |
| Q / Escape / Ctrl+C | Quit |

The demo does not load or save your personality file. Optional arguments:
`--fps 20`, `--seconds 10`, `--indexed`, `--ascii`. Timed runs print observed
render pacing after restoring the terminal. Autonomous mode is enabled by default and chooses activities from needs and context.
Use `--manual` to pause autonomous selection, `--diagnostics` for the overlay, and
`--seed 7` for reproducible initial traits. N cycles all activities for inspection.

## Chat controls

| Key | Action |
| --- | --- |
| Enter | Send; reconnect if disconnected |
| Left / Right, Home / End | Move through the draft by grapheme |
| Backspace / Delete | Edit; bracketed paste is supported |
| Page Up / Page Down | Scroll conversation |
| Escape | Cancel the active request |
| F2 / F3 | Resize Trixie's panel on larger terminals |
| F4 | Pet controls: attention, orb, play, rest, wake, inspect, autonomy, reset |
| F5 | Recheck server and model |
| F6 | Open model picker; arrows navigate, Enter selects, Escape closes |
| F7 | Toggle normal / reduced motion |
| F8 | Hide / show Trixie |
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
must remain false. `show_trixie` is the current visibility key; `show_terri` and
`show_terry` remain aliases. Explicit old default names `Terri` and `Terry` are
read as `Trixie`; other custom names are preserved.

Traits, energy, mood, preferred activities, RNG state and interaction counters
are saved on clean shutdown to `~/.local/state/terrabot/trixie.json`, or beneath
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

A concise system profile makes Trixie helpful, curious and thoughtful, without
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
cargo run --release --example trixie-benchmark
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
asciinema rec -c './target/release/examples/trixie-demo --auto' trixie.cast
# Quit with Q when finished.
agg trixie.cast trixie.gif

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

## Life in the habitat

Trixie now chooses activities from seven bounded needs: energy, boredom, curiosity,
playfulness, comfort, sociability and restfulness. Stable boldness, liveliness and
social inclination influence utility scores. Preferences change a little after
completed interactions; cooldowns, bounded travel and minimum durations prevent
rapid switching or endless chasing. This is a cosmetic virtual creature, with no
illness, death, care obligations or claims of consciousness.

She walks toward her cushion, orb, plant or chat side before interacting. Orb
bounces use fixed-step gravity and damping; balancing catches it above her head.
Walking, sitting, napping, stretching, reaching and play share the original
handcrafted character art and smooth pose transitions. A small viewport uses the
same art at half size, with all drawing clipped to the panel.

F4 opens a keyboard-operated pet menu. These actions never send Ollama requests.
Typing and generation take priority over autonomous behavior; completed replies
release that priority. Serious conversation cues suppress playful activities.
The diagnostic overlay shows needs, travel/interaction phase, position, ticks and
the strongest scores at the most recent decision. Toggle it through F4 or set
`[pet] diagnostics = true`.

Optional `daily_routine` uses the local clock to favor rest at night (22:00–07:00),
without forcing sleep or delaying replies. The clock is sampled once a minute on
Unix; platforms without that clock adapter use neutral behavior. Set it false
for time-independent behavior. No unattended elapsed time is simulated on startup.

State format 2 extends the existing private JSON file with needs, stable traits,
activity preferences, favorite locations, interaction counters, familiarity and
current routine. Version-1 files migrate in memory and save atomically on clean
exit; corrupt or unsupported files stay untouched. Chat transcripts and credentials
are never included. `develop_preferences = false` freezes preference learning;
F4 can reset learned traits, or `reset_personality = true` resets them each launch.
Autonomy and overlay switches in F4 apply to the current session; config sets defaults.

For an accelerated, deterministic 24-hour life run without a terminal or Ollama:

```bash
cargo run --release --example trixie-life -- 86400 7
```

It checks bounds every tick and reports time in each activity, transitions, longest
activity and final state as JSON. The injected day/night schedule starts at noon.

## v0.4 animation studio

The presentation layer now has sixteen named, hand-authored clips: Idle, Walk,
Run, Jump, Sit, Sleep, Wake, Think, Talk, Happy, Sad, Curious, Bored, Play,
Celebrate and Stumble. Distinct standing, seated, curled and stretched artwork
shares Trixie's lilac shading, mint light, feet and facial proportions. Authored
foot and hand contacts, timed anticipation/recovery, independent face motion and
small secondary effects replace the previous pose-only system.

```bash
cargo run --release --example trixie-demo -- --showcase
cargo run --release --example trixie-demo -- --clip Jump
```

| Preview control | Action |
| --- | --- |
| `[` / `]` | Previous / next animation |
| Space | Pause / resume playback |
| `.` | Pause and advance exactly one authored frame |
| `,` / `;` | Slower / faster playback, 0.1×–3× |
| X | Cycle facial expressions independently |
| V | Reverse facing |
| G | Toggle bounds, origin, face/arm anchors, orb depth, events and timing |
| Enter | Replay selected clip from the start |
| Z | Return to autonomous life and clear preview overrides |

Existing demo controls remain available, including P for animation visibility,
+/− for motion intensity and M for simulation mood. The preview status shows
observed FPS; the character label shows clip, frame, frame time, speed and pause
state. One-shot clips hold their final pose until replayed or replaced. Showcase
mode displays each clip for four seconds. This is separate from `--auto`, which
runs the life simulation rather than forcing animations.

Loops can retain an introductory sequence: sleep sits and curls before its
breathing loop, while sitting holds a seated pose. Thinking and curiosity use
ping-pong timing. Jump/stumble/wake/celebration protect their short sequences from
lower-priority animation changes; thinking and speaking can interrupt them.
Presentation never blocks provider events or changes saved pet needs.

Play clips reach, lift, throw, catch and place the orb. Bounded catch/release/landing
and foot-contact events drive presentation effects. The face keeps its own clock
for blinking, gaze and speech, preserving expression continuity across body clips.
Reduced motion removes large lifts and secondary effects; narrow viewports retain
clipped, half-size logical-pixel rendering.

Real before/after terminal recordings and capture instructions are described in
[the recording notes](docs/captures/README.md). No graphical assets or alternative
rendering backends are used.

## v0.4.2 pixel-art repair

Trixie now uses a fixed 36×36 logical canvas and authored standing, lowering,
seated, curled, stretched and forward-leaning run silhouettes. The old inverse
sampling, vertical scaling, compensating width changes and shear are removed.
Only the root translates during a jump. Shoulder/hip attachments keep limbs
connected, with back limbs, torso, face and foreground hands composed before a
single one-pixel dark-purple silhouette boundary is applied.

Walking and running advance by distance travelled, retaining phase when changing
between gaits. Rest and wake sequences use intermediate artwork. The orb has
explicit pickup, held, thrown, bouncing and return states; held coordinates use
the same hand anchors and root as the character, including reversed facing.
The G overlay also reports velocity, clip/frame, shape transition and orb state.
Character proportions remain 1:1; compact terminals retain the existing reduced
output scale. No chat, life simulation, configuration or persistence format changed.

Regression coverage includes all-frame silhouette connectivity, outline and
ground bounds, mirrored orb pickup/release continuity, and 48 reviewed full-colour
terminal-cell references across eight clips. References are never automatically
updated. See [the repair review and actual terminal comparisons](docs/captures/v042/README.md)
for captures, reproduction steps and measured validation results.


## v0.4.3: Trixie and the orb

The scene is now Trixie, her bright turquoise orb and a small ground shadow.
Furniture, the full-width floor and idle ambient particles are gone. Eye shapes,
expressions and gaze are preserved; the accidental mint eye pixels now use the
existing light eye colour. Mint body accents remain below the face.

The simulation constrains ordinary travel to the central home area. Rest, sleep,
attention and exploration stay in place; wander and chase use brief local shuffles.
Explore/search/investigate map to Curious, while play/balance/chase share the orb
sequence after any shuffle. Needs, learned preferences and saved-state fields are
retained. Old saved positions are clamped into the home area on load. The renderer
uses the simulation position directly, with no recurring position overrides.

The orb floats beside Trixie, moves lower during curiosity/boredom, pulses during
thinking, flashes during celebration and dims/settles during sleep. Pickup uses
an outer hand anchor, away from the face; tosses and returns retain bounded
physics. Orb pixels and trails cannot overwrite the character. Hops use at most
three logical pixels of root lift. The compact scene and wider default chat-side
panel make the artwork more prominent without stretching it; small terminals
retain the existing half-resolution mode.

F4 interactions, F6 model selection, streaming, cues and persistence remain in
place. [Visual review, terminal captures and validation](docs/captures/v043/README.md)
include 60 accepted terminal-cell references for the ten requested activities.


## v0.4.4: Trixie character polish

Terrabot remains the app; Trixie is the character. The UI, default personality
name, F4 menu and demo use the new name. `trixie-demo`, `trixie-benchmark` and
`trixie-life` are the current example commands; the `terri-*` examples and public
`Terri`/`default_terri` Rust aliases remain available.

When `trixie.json` is absent, the app reads the sibling `terri.json`. Clean shutdown
atomically saves the new filename and leaves the old file untouched as a backup.
An existing new file takes precedence; unreadable or invalid state disables saving
for that session, including invalid legacy state. No schema or personality fields
were removed. Disabling persistence still disables state I/O.

All embedded turquoise torso pixels are replaced with lilac shading in standing,
lowering, seated, curled, stretched and moving artwork. Normal open eyes retain
the exact geometry and highlights. Blink, sleep and relaxed happy lids are clean,
continuous horizontal strokes. Shorter arms, rounded highlighted paws, thicker
ankle connections and tucked rounded feet create a more cohesive silhouette.
A single composed outer outline remains; body parts do not receive extra outlines.

The Happy clip now has anticipation, one small bounce and a longer settle; orb play
has a longer visible hold and recovery. The compact home area, minimal scene,
bright orb, F4/F6 controls and chat behaviour remain in place. G now includes the
local home offset and actual output scale alongside the existing anchor guides.

[Reviewed terminal captures and validation](docs/captures/v044/README.md) include
before/after frames and 72 accepted full-colour references. Earlier release
sections above describe historical stages; this pass removes the remaining mint
body markings that were deliberately retained in v0.4.3.


## v0.5.0: handcrafted floating frames

The current renderer selects complete 36×36 sprite frames from `src/sprites`.
Each editable text file contains the body, large integrated arms, shading and
silhouette; dots are transparent. `src/art.rs` defines the stable origin, face,
arm and orb anchors. Frames are parsed and their single silhouette boundary is
normalized once in a shared cache. Only the face overlays and floating root
translation change during drawing; no procedural limbs, feet, body resampling or
per-frame text parsing remain.

Eight authored poses form the existing named clips: relaxed, drooping, peeking,
raised arms, reaching, holding, stretching and sleeping. The normal open-eye pixels
are unchanged; cheeks now sit consistently below them. Wake transitions from
closed to sleepy to open eyes. The retained Walk/Run clip names are local glides.
Simulation movement accelerates and decelerates within the existing home bounds,
while root hover adds a slow one-pixel oscillation (slower during sleep). Reduced
motion retains a fixed floating altitude.

The orb has one position and explicit Behind, Side, Front, Held and Free depth
states. Curious follows a local orbit whose depth changes at the outer sides.
Behind draws before the character; Front draws after it. Held adds an authored
foreground hand over the orb's lower rim. Catch/release events remain controlled
by the existing timeline. Orb colour pulse and bounded physics stay procedural.

`--showcase` gives Curious eight seconds and Play seven seconds so both orbit
halves, holding, tossing and catching are visible. G reports frame duration,
anchors, depth and the latest event; pause, frame stepping and slow playback remain.
See [actual terminal captures and validation](docs/captures/v050/README.md).

The embedded-frame/cache/playback approach was informed by
[Gostty's source](https://github.com/ashish0kumar/gostty/blob/main/main.go).
Trixie's artwork is original. Terrabox's changed-row renderer and TerraGame's
fixed-step simulation remain in use. Earlier release notes above describe the
historical grounded artwork; v0.5.0 replaces it without changing saved data or
chat, model-selection and pet-menu behavior.
