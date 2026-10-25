# v0.4.3 visual restoration

The source baseline is `a3e0614` (v0.4.2). Earlier `057aa52` artwork/orb rendering
was also reviewed. The lilac silhouette and eye geometry are retained; the
explicit mint lower-eye highlight was the facial colour defect. It is replaced
with the existing near-white eye colour, rather than removing legitimate mint
body accents. The orb now has a dark teal boundary, saturated turquoise fill,
cyan highlight and near-white core.

[Terminal activity frames](terminal-clips.png) show Idle, Curious, Bored, Jump
(the small hop), Sit, Sleep, Wake, Think, Talk and Play. The individual PNGs retain
the full 110×30 terminal output. [Reference sheet](references.png) shows six
fixed-time samples per activity. These were visually inspected before accepting
`tests/visual/v043`. Earlier v0.4.2 references remain as historical artifacts.

The PNGs reconstruct the actual ANSI/Unicode PTY buffers using `pyte` and Pillow;
half-block character colours are exact, labels use a substitute font. They are
not generated illustrations or GUI-emulator screenshots. Crops use nearest-neighbour
sampling. Each activity was recorded for roughly three seconds; `autonomous.cast`
records 22 seconds of the actual `cargo run --release --example terri-demo -- --auto`
command. `.cast` recordings remain locally available and ignored under the existing
repository policy. Terminal restoration is verified by the recording harness.

## Reproduce

```bash
cargo build --release --examples
python3 tests/record_terminal.py /tmp/terri-auto.cast 22 \
  target/release/examples/terri-demo --auto
python3 tests/record_terminal.py /tmp/terri-play.cast 4 \
  target/release/examples/terri-demo --clip Play
# Optional development dependencies: pyte and Pillow.
python3 tests/render_terminal_capture.py /tmp/terri-play.cast 1 /tmp/terri-play.png
TERRABOT_REVIEW_REFERENCES=/tmp/terri-candidates cargo test --test visual_snapshots
python3 tests/review_visual.py /tmp/terri-candidates /tmp/terri-review.png
```

Review candidates and actual terminal frames before copying accepted `.cells`
files into the versioned reference directory. Normal tests never regenerate them.
Preview controls remain G (guides), Space (pause), . (step), , (slow) and V (facing).
The fixed 36×36 model, composed purple outline, grounding and connected-limb
checks remain. New tests cover every expression's eye palette, orb/body occlusion,
20 simulated minutes of bounded home movement, old-position migration and in-place
rest/orb interactions. Chase must leave its activity within a bounded interval;
the test allows later legitimate reselection rather than assuming a particular
activity at one arbitrary future instant.

## Results

All required commands passed: formatting, strict Clippy (`--all-targets -- -D
warnings`), 43 unit tests, the 60-reference integration test, release application
and example builds, and all 13 PTY smoke tests. The PTY suite uses a local mock
Ollama server and checks streaming, discovery/selection, history, cancellation,
F4 controls, persistence, resizing and terminal cleanup. No live model benchmark
is implied.

The ten activity captures and [four autonomous samples](autonomous.png) were
visually reviewed. They show the furniture-free scene, clean eye highlights,
bright side orb, hand-held orb and contained central movement. The full
[chat layout](chat-layout.png) was inspected separately.

| Target FPS | Observed FPS, three PTY runs | Mean composition |
| --- | --- | --- |
| 20 | 19.79–19.80 | 62.66 µs |
| 30 | 29.75–29.79 | 61.42 µs |
| 60 | 57.18–57.21 | 61.32 µs |

Raw [terminal results](terminal-benchmark.json) and
[composition results](composition-benchmark.txt) are saved. Composition p99 was
97–118 µs. These measure the local process and drained PTY, not a GUI emulator's
refresh rate. No framework changes, commits or pushes were made.
