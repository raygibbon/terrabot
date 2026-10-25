# v0.5.0 floating character review

The starting point was the user's staged v0.4.4 implementation. Its release demo
was preserved in `/tmp/trixie-v044-baseline`. The existing index was left untouched;
all v0.5.0 edits are working-tree changes. No framework repositories were changed.

[Terminal activity/depth sheet](terminal-clips.png) shows Idle, behind/front orb
passes, Happy, Bored, Think, Talk, Sleep, Wake, Held, Toss and Catch. Each source
PNG is a full actual 110×30 terminal buffer. The raw local `.cast` recordings
contain timed ANSI/Unicode output and remain ignored under the existing policy.
These images reconstruct the PTY buffer with pyte/Pillow: exact half-block pixel
colours, substitute label font, nearest-neighbour crops. They are not generated
illustrations or screenshots of a GUI terminal emulator.

The reviewed [72-frame reference sheet](references.png) samples 0, 520, 1200,
2000, 3200 and 4300ms. Later samples deliberately include the orbit's front half
and the play hold/release/catch sequence. Initial visual review caught cheek
pixels overlapping eye anchors; the final artwork places cheeks beneath the
unchanged normal eyes. References were accepted only after review.

## Artwork and compositing

Eight complete text sprites plus one foreground hand mask live in `src/sprites`.
Every frame is 36×36, uses the same origin and has a single tapering lower body
with no legs or feet. Arms are authored into each full frame, not assembled by
line/circle drawing. Text is validated and parsed once into a shared cache.
The opacity boundary is normalized once to one dark-purple logical pixel.

Faces remain anchored overlays. The orb is drawn exactly once per frame: before
the body when Behind, after it when Front/Side/Free, and beneath the foreground
hand mask when Held. Orbit depth changes occur at the lateral extremes outside
the body. Physics and presentation share the held anchor/root convention. Root
hover is one logical pixel around a fixed airborne base; reduced motion is still.

The reference implementation inspected was [Gostty's main.go](https://github.com/ashish0kumar/gostty/blob/main/main.go):
embedded frame data, preprocessing once, then elapsed-time frame selection.
No Gostty artwork or rendering framework was imported. Existing motion timelines,
Terrabox rendering, TerraGame simulation, persistence compatibility and UI controls
remain in place.

## Reproduce

```bash
cargo build --release --examples
python3 tests/record_terminal.py /tmp/trixie-showcase.cast 74 \
  target/release/examples/trixie-demo --showcase
python3 tests/record_terminal.py /tmp/trixie-orbit.cast 7 \
  target/release/examples/trixie-demo --clip Curious
python3 tests/record_terminal.py /tmp/trixie-play.cast 7 \
  target/release/examples/trixie-demo --clip Play
# Optional development dependencies: pyte and Pillow.
python3 tests/render_terminal_capture.py /tmp/trixie-orbit.cast 1.4 /tmp/behind.png
python3 tests/render_terminal_capture.py /tmp/trixie-orbit.cast 4.3 /tmp/front.png
python3 tests/render_terminal_capture.py /tmp/trixie-play.cast 2 /tmp/held.png
TERRABOT_REVIEW_REFERENCES=/tmp/trixie-candidates cargo test --test visual_snapshots
python3 tests/review_visual.py /tmp/trixie-candidates /tmp/trixie-review.png
```

G shows frame/total duration, origin, face/arm anchors, orb location/depth and the
last animation event. Space pauses; . steps; , slows playback; V reverses facing.
The showcase includes the legacy clips as floating behaviors, with longer holds
for Curious and Play so depth changes and object interactions can be inspected.

## Validation

All required commands passed: `cargo fmt --all --check`, strict Clippy with
`--all-targets -- -D warnings`, 51 unit tests, the 72-reference visual integration
test, release application/example builds and all 13 PTY smoke tests. The PTY suite
uses a mock Ollama server, covering discovery/selection, streaming, history,
cancellation, persistence, F4/preview controls, resizing and terminal restoration.
No live model performance claim is made.

The actual clips and [74-second showcase samples](showcase.png) were visually
inspected. Separate final-build captures check an interrupted orbit
[remaining behind the body](interruption-behind.png) and then
[clearing its side](interruption-side.png). Tests specifically cover dimensions/palette, original open-eye pixels,
cheek anchors, connected outlines, unsplit legless lower silhouettes, all three
occlusion cases (including mirrored held arms), bounded orbit continuity (including interruptions while behind the body), event
order, interruption priority, fixed-step movement and reduced-motion altitude.
AI gesture cues select authored clips, while thinking/speaking retain priority.

The final source preserves the staged baseline and its state-migration behavior.
No commits, pushes or index edits were made.


| Target FPS | Observed FPS, three PTY runs |
| --- | --- |
| 20 | 19.79–19.79 |
| 30 | 29.79–29.81 |
| 60 | 57.18–57.20 |

Raw [terminal data](terminal-benchmark.json) and [composition timings](composition-benchmark.txt) are saved. Composition averaged 57–59 µs; p99 was 69–99 µs. Measurements describe the local process and drained PTY, not GUI-emulator refresh rate.
