# Actual terminal recordings

These local artifacts contain real ANSI/Unicode output from a 110×30 PTY, with
monotonic timestamps in asciicast v2 format. They are not generated illustrations
or headless render substitutes. Both runs verified termios and alternate-screen
restoration. `.cast` files remain ignored by the existing repository policy.

- `terri-v03-before.cast`: twelve seconds of the staged v0.3 baseline demo's
  autonomous life, built in a temporary checkout without changing the index.
- `terri-v04-showcase.cast`: sixty-five seconds covering all sixteen v0.4 clips.

Play with an asciicast-compatible player, for example:

```bash
asciinema play docs/captures/terri-v03-before.cast
asciinema play docs/captures/terri-v04-showcase.cast
```

The recorder uses Python's standard library; asciinema/agg were not installed on
this machine. Recreate a recording without installing recording software:

```bash
cargo build --release --examples
python3 tests/record_terminal.py docs/captures/terri-v04-showcase.cast 65 \
  target/release/examples/terri-demo --showcase
```

The source version and capture duration differ deliberately: v0.3 demonstrates
its existing autonomous presentation; v0.4 exposes each named clip for inspection.
Use the interactive preview to inspect individual contacts, faces and transitions.
