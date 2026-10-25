# Trixie v0.4.4 visual review

Baseline: `1b4e4c6` (v0.4.3). Its release demo was saved before editing.
[Before/after comparison](comparison.png) shows Idle, Sleep, Happy and Play at
one second. [Twelve terminal activity frames](terminal-clips.png) also cover
Curious, Bored, Jump, Sit, Wake, Think, Talk and Celebrate.

These are reconstructed actual 110×30 PTY ANSI/Unicode buffers, not generated
illustrations or GUI-terminal screenshots. Half-block pixel colours are exact;
text uses a substitute font. Crops use nearest-neighbour sampling. The full PNGs
are available by clip name. Each clip has a local asciicast recording; the
20-second autonomous recording is `autonomous.cast`. `.cast` files remain ignored
under the existing repository policy. The recorder verifies terminal restoration.

[The reference sheet](references.png) contains six fixed-time samples for each
of twelve clips. It was visually reviewed before accepting `tests/visual/v044`.
The review specifically checks the removed torso marking, unchanged open eyes,
straight connected lids, rounded attached paws/feet and visible hand-held orb.
Earlier reference sets remain as historical artifacts.

## Reproduce

```bash
cargo build --release --examples
python3 tests/record_terminal.py /tmp/trixie.cast 6 \
  target/release/examples/trixie-demo --clip Play
# Optional development dependencies: pyte and Pillow.
python3 tests/render_terminal_capture.py /tmp/trixie.cast 1 /tmp/trixie.png
TERRABOT_REVIEW_REFERENCES=/tmp/trixie-candidates cargo test --test visual_snapshots
python3 tests/review_visual.py /tmp/trixie-candidates /tmp/trixie-review.png
```

Inspect candidates and actual terminal output before copying accepted `.cells`
files into the versioned reference directory. Normal tests never regenerate them.
Use G for anchor/home/scale diagnostics, Space to pause, . to step, , to slow
playback and V to reverse facing. Legacy `terri-demo` remains functional.

## Compatibility and test scope

The new state path is `terrabot/trixie.json`. If absent, the old `terri.json` is
read and validated. Atomic saving writes the new name without deleting the old
file. Existing new state takes precedence. A malformed legacy file is reported
and preserved; the UI disables saving for that session. Tests compare complete
snapshots across migration, including learned preferences and counters.

`show_terri` and `show_terry` remain config aliases; `show_trixie` is canonical.
Old default agent names become Trixie, while custom names remain unchanged.
Normal open-eye pixel geometry is explicitly tested. All-frame body palette,
outline connectivity, bounds, orb attachment and reduced-motion checks remain.
The PTY suite uses a local mock Ollama server to test streaming, model selection,
history, persistence, pet/preview controls, cancellation and resizing.

## Validation results

All required commands passed:

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`: 45 unit tests plus the integration test comparing 72 references
- `cargo build --release`
- `cargo build --release --examples` (both canonical and legacy commands)
- `python3 tests/pty_smoke.py`: all 13 tests passed
- `git diff --check`

The actual clip comparisons, [autonomous samples](autonomous.png) and
[chat layout](chat-layout.png) were visually inspected. The app header and
welcome dialogue identify Trixie while the application remains Terrabot.
The source tree contains no changes to Terrabox or TerraGame. No commits or
pushes were made.


| Target FPS | Observed FPS (three PTY runs) |
| --- | --- |
| 20 | 19.80–19.81 |
| 30 | 29.73–29.76 |
| 60 | 57.12–57.20 |

Raw [terminal measurements](terminal-benchmark.json) and [composition timings](composition-benchmark.txt) are saved. These measure CPU and drained PTY output, not GUI-emulator refresh rate.
