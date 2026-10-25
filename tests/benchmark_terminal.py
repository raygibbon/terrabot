#!/usr/bin/env python3
"""Paced real-terminal output, drained PTY; does not measure a GUI emulator."""
import json
import re
import resource
from pty_smoke import Terminal, ROOT

def cpu():
    r = resource.getrusage(resource.RUSAGE_CHILDREN)
    return r.ru_utime + r.ru_stime

results = []
for fps in (20, 30, 60):
    for sample in range(3):
        before = cpu()
        t = Terminal("http://127.0.0.1:1", command=[str(ROOT / "target/release/examples/terri-demo"),
            "--auto", "--fps", str(fps), "--seconds", "4"])
        try:
            t.wait_for("observed_fps=", timeout=8)
            output = bytes(t.output)
        finally:
            t.close()
        match = re.search(rb"frames=(\d+) elapsed=([\d.]+)s observed_fps=([\d.]+)", output)
        if not match:
            raise RuntimeError("Missing paced-render measurements")
        results.append(dict(target_fps=fps, sample=sample + 1, frames=int(match[1]),
            elapsed_seconds=float(match[2]), observed_fps=float(match[3]),
            cpu_seconds=round(cpu() - before, 5), output_bytes=len(output)))
print(json.dumps(results, indent=2))
