#!/usr/bin/env python3
"""Record actual PTY output in asciicast v2 format; no generated graphics."""
import codecs
import json
import os
from pathlib import Path
import select
import sys
import time
from pty_smoke import Terminal

class Recording(Terminal):
    def __init__(self, path, command):
        self.started = time.monotonic()
        self.record = Path(path).open('w')
        self.decoder = codecs.getincrementaldecoder('utf-8')('replace')
        self.record.write(json.dumps({'version': 2, 'width': 110, 'height': 30,
            'title': 'Terri — actual terminal output', 'env': {'TERM': 'xterm-256color'}}) + '\n')
        super().__init__('http://127.0.0.1:1', command=command)
    def pump(self, duration=0.1):
        until = time.monotonic() + duration
        while time.monotonic() < until:
            if select.select([self.master], [], [], 0.02)[0]:
                try:
                    raw = os.read(self.master, 65536)
                except OSError:
                    break
                self.output.extend(raw)
                decoded = self.decoder.decode(raw)
                if decoded:
                    self.record.write(json.dumps([round(time.monotonic()-self.started, 6), 'o', decoded], ensure_ascii=False) + '\n')
    def finish(self):
        try:
            self.close(b'q')
        finally:
            self.record.close()

if __name__ == '__main__':
    if len(sys.argv) < 4:
        raise SystemExit('Usage: record_terminal.py OUTPUT.cast SECONDS COMMAND [ARGS...]')
    path, seconds, *command = sys.argv[1:]
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    t = Recording(path, command)
    try:
        end = time.monotonic() + float(seconds)
        while time.monotonic() < end and t.process.poll() is None:
            t.pump()
        assert b'Terri paused' not in t.output, 'rendering fault'
    finally:
        t.finish()
    print(f'Recorded {path}; terminal restoration verified')
