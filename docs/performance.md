# Performance — 2026-10-25

Linux x86_64, release build, Rust 1.99.1. Default remains 20 FPS.
Three four-second samples at each requested rate used the real TerraTUI demo in
a drained 110×30 PTY. Each process restored its terminal. Rendering and waiting
were included; GUI emulator painting and GPU/compositor display latency were not.
This was a shared desktop, including a real Ollama check during measurement.

| Requested FPS | Observed paced redraws/second | Median CPU fraction of one core |
| --- | --- | --- |
| 20 | 19.76–19.78 | 1.02% |
| 30 | 29.76–29.78 | 1.57% |
| 60 | 57.06–57.18 | 2.95% |

The frame counter counts paced redraw requests. TerraBox only emits changed rows;
identical cells are not evidence of additional physical display frames. Sixty FPS
was measured, but its observed pacing falls below the target here. These results
do not guarantee smooth 60 FPS on a different terminal, SSH connection or font.

Headless simulation and 80×30 character composition (excluding terminal I/O):

```text
target_fps=20 composition_mean_us=76.68 p99_us=108.33 simulation_ticks=400
target_fps=30 composition_mean_us=77.07 p99_us=112.97 simulation_ticks=399
target_fps=60 composition_mean_us=75.78 p99_us=107.42 simulation_ticks=400

```

The fixed simulation is 20 Hz regardless of render rate. Fractional nanosecond
rounding in the benchmark's per-frame Duration can yield 399 instead of 400 ticks
at 30 FPS; the equal-total-time partition test verifies simulation independence.
Rendering interpolates the visual phase without advancing simulation RNG.

Reproduce with:

```bash
cargo build --release --examples
python3 tests/benchmark_terminal.py
cargo run --release --example terri-benchmark
```

Raw measurements: [terminal samples](benchmark-terminal.json) and
[composition samples](benchmark-composition.txt). Character grids are capped at
160×80 logical pixels (the widget uses at most 100×44), particles at six, worker
channels at 32 messages, and state files at 16 KiB. Disabled/hidden chat animation
uses one simulation wake per second with no idle redraws. The PTY suite checks
that disabled animation produces no idle output.
