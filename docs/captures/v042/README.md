# v0.4.2 repair review

The baseline is commit `7b9cd9391c2074344483bc8d217fcb5467ace8c7`.
Its release `terri-demo` binary was saved before editing. Both versions were run
in real 110×30 PTYs using the same explicit clip and capture time. No sibling
framework changes were required.

[Before/after comparison](comparison.png) shows Idle, Walk, Run, Jump, Sit, Sleep,
Wake and Play. Left is baseline; right is repaired. The full terminal images are
`before-CLIP.png` and `after-CLIP.png`. Crops use nearest-neighbour sampling.
These PNGs reconstruct the actual ANSI terminal buffers, including their exact
half-block pixel colours. Text uses a substitute font; these are not screenshots
of a GUI terminal emulator. The character pixels are neither generated nor
retouched. Both columns and the [48-frame reference sheet](references.png) were
visually inspected before the references were accepted.

The review found stable rounded proportions, attached arms/feet, distinct run
and rest silhouettes, root-based jump height and a correctly anchored held orb.
The original long standing-to-wake stretch and detached hands are visible in the
baseline. Compact layouts still use the existing reduced-resolution rendering;
the full pixel-art detail requires a viewport at least 68 columns by 21 rows.

Raw timed recordings remain local ignored artifacts, following the existing
`.cast` policy: `before-CLIP.cast`, `after-CLIP.cast`, and `showcase.cast` (65 seconds,
all sixteen clips). Terminal cleanup is verified by the recorder. Replay with an
asciicast v2 player. The earlier baseline showcase is
`../terri-v04-repair-baseline.cast`.

## Reproduce and review

```bash
cargo build --release --examples
python3 tests/record_terminal.py docs/captures/v042/showcase.cast 65 \
  target/release/examples/terri-demo --showcase
# Optional development-only dependencies for PNG reconstruction: pyte, Pillow.
python3 tests/render_terminal_capture.py docs/captures/v042/showcase.cast 13 \
  /tmp/terri-terminal.png
```

For matched captures, invoke `--clip Idle` (or another clip) on each release binary.
Capture times in the comparison are 0.6s for Idle/Walk/Run, 0.55s Jump, 0.8s Sit,
1.3s Sleep, 0.9s Wake and 1.05s Play. Pacing is real time, so captures can differ
by one terminal frame. The exact automated references instead use fixed 20ms
presentation steps and the actual TerraTUI Widget/Surface renderer.

```bash
TERRABOT_REVIEW_REFERENCES=/tmp/terri-candidates cargo test --test visual_snapshots
python3 tests/review_visual.py /tmp/terri-candidates /tmp/terri-review.png
```

Inspect candidate frames and actual terminal output before deliberately copying
accepted `.cells` files to `tests/visual/v042`. Normal tests compare exact palette
and pixel data and never regenerate references. The reference sheet samples
0, 240, 520, 800, 1200 and 1800ms for all eight required clips.

For interactive review use `--clip Jump`, G for bounds/anchors, Space to pause,
`.` to step authored frames, `,` to slow playback and V to reverse facing.

## Validation

All required commands passed on 2026-10-25:

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`: 41 unit tests plus the integration test comparing 48 references
- `cargo build --release`
- `cargo build --release --examples`
- `python3 tests/pty_smoke.py`: 13 tests, including streaming, discovery/selection,
  persistence, interaction controls, preview controls, resizing and terminal cleanup
- `git diff --check`

The PTY suite uses a local mock Ollama server; it does not require or benchmark a
live model. The actual 65-second showcase covers all sixteen clips; its
[sampled terminal frames](showcase.png) were also visually inspected.

| Target FPS | Observed FPS (three paced PTY runs) | Mean composition time |
| --- | --- | --- |
| 20 | 19.77–19.78 | 75.33 µs |
| 30 | 29.76–29.79 | 76.31 µs |
| 60 | 56.95–57.06 | 77.21 µs |

Raw results are in [terminal-benchmark.json](terminal-benchmark.json) and
[composition-benchmark.txt](composition-benchmark.txt). Composition p99 was
129–140 µs. These are local CPU/PTY measurements, not GUI-emulator refresh-rate
measurements. The higher-rate result is reported separately from the 20–30 FPS
target. Commands: `python3 tests/benchmark_terminal.py` and
`target/release/examples/terri-benchmark`.
