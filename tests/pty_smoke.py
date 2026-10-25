#!/usr/bin/env python3
"""Exercise the real terminal executable with a controllable Ollama HTTP fixture."""
import fcntl
import http.server
import json
import os
from pathlib import Path
import select
import signal
import struct
import subprocess
import tempfile
import termios
import threading
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target/release/terrabot"

class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    def log_message(self, *_):
        pass
    def do_GET(self):
        body = json.dumps({"models": [{"name": "test:latest"}]}).encode()
        self.send_response(200)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        self.server.requests.append(request)
        self.send_response(200)
        self.send_header("Content-Type", "application/x-ndjson")
        self.send_header("Connection", "close")
        self.end_headers()
        prompt = request["messages"][-1]["content"]
        if prompt == "stall":
            self.server.stop_event.wait(10)
            return
        records = [{"message": {"content": text}, "done": False}
                   for text in ["Hello ", "世界", " — streamed reply."]]
        if prompt != "truncate":
            records.append({"done": True})
        try:
            for record in records:
                raw = (json.dumps(record, ensure_ascii=False) + "\n").encode()
                for i in range(0, len(raw), 3):
                    self.wfile.write(raw[i:i+3])
                    self.wfile.flush()
                    time.sleep(0.002)
        except (BrokenPipeError, ConnectionResetError):
            pass

class Terminal:
    def __init__(self, endpoint, model="test", animation=True):
        self.directory = tempfile.TemporaryDirectory()
        config = Path(self.directory.name) / "config.toml"
        config.write_text(f'[ollama]\nendpoint="{endpoint}"\nmodel="{model}"\n'
                          f'[interface]\nanimation={str(animation).lower()}\n')
        self.master, self.slave = os.openpty()
        self.resize(110, 30, notify=False)
        self.original = termios.tcgetattr(self.slave)
        def attach():
            os.setsid()
            fcntl.ioctl(self.slave, termios.TIOCSCTTY, 0)
        self.process = subprocess.Popen([str(BINARY), "--config", str(config)],
            stdin=self.slave, stdout=self.slave, stderr=self.slave,
            preexec_fn=attach, env={**os.environ, "TERM": "xterm-256color"})
        self.output = bytearray()
    def resize(self, width, height, notify=True):
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", height, width, 0, 0))
        if notify:
            os.kill(self.process.pid, signal.SIGWINCH)
    def send(self, text):
        os.write(self.master, text if isinstance(text, bytes) else text.encode())
    def pump(self, duration=0.1):
        until = time.monotonic() + duration
        while time.monotonic() < until:
            if select.select([self.master], [], [], 0.02)[0]:
                try:
                    self.output.extend(os.read(self.master, 65536))
                except OSError:
                    break
    def wait_for(self, text, timeout=5):
        target = text.encode()
        until = time.monotonic() + timeout
        while target not in self.output and time.monotonic() < until:
            self.pump()
        if target not in self.output:
            raise AssertionError(f"Missing {text!r}; terminal tail: {self.output[-3000:]!r}")
    def close(self, key=b"\x11"):
        self.send(key)
        until = time.monotonic() + 3
        while self.process.poll() is None and time.monotonic() < until:
            self.pump()
        try:
            assert self.process.poll() == 0, f"exit: {self.process.poll()}"
            assert termios.tcgetattr(self.slave) == self.original, "terminal attributes not restored"
            self.pump()
            assert b"\x1b[?1049l" in self.output, "alternate screen not restored"
        finally:
            if self.process.poll() is None:
                self.process.kill()
            self.process.wait()
            os.close(self.master)
            os.close(self.slave)
            self.directory.cleanup()

class Smoke(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        cls.server.daemon_threads = True
        cls.server.requests = []
        cls.server.stop_event = threading.Event()
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.endpoint = f"http://127.0.0.1:{cls.server.server_port}"
    @classmethod
    def tearDownClass(cls):
        cls.server.stop_event.set()
        cls.server.shutdown()
        cls.server.server_close()
    def test_stream_history_resize_typing_and_normal_exit(self):
        self.server.requests.clear()
        t = Terminal(self.endpoint)
        try:
            t.wait_for("Connected")
            t.send("first\r")
            t.wait_for("Hello")
            # Edit a second draft while Terry and the first response animate.
            t.send("seconx\x7fd")
            t.wait_for("streamed reply.")
            t.pump(0.3)
            for width, height in [(65, 20), (28, 8), (5, 3), (1, 1), (110, 30)]:
                t.resize(width, height)
                t.pump(0.12)
            t.send("\r")
            deadline = time.monotonic() + 4
            while len(self.server.requests) < 2 and time.monotonic() < deadline:
                t.pump()
            self.assertEqual(len(self.server.requests), 2)
            messages = self.server.requests[1]["messages"]
            self.assertEqual([m["role"] for m in messages], ["user", "assistant", "user"])
            self.assertEqual(messages[-1]["content"], "second")
            self.assertEqual(messages[1]["content"], "Hello 世界 — streamed reply.")
            self.assertEqual(self.server.requests[1]["options"]["num_ctx"], 8192)
            t.send(b"\x1b[5~\x1b[6~")
        finally:
            t.close()
    def test_static_idle_and_invalid_config(self):
        t = Terminal(self.endpoint, animation=False)
        try:
            t.wait_for("Connected")
            t.pump(0.2)
            previous = len(t.output)
            t.pump(0.3)
            self.assertEqual(len(t.output), previous, "static idle must not redraw")
        finally:
            t.close()
        with tempfile.TemporaryDirectory() as d:
            config = Path(d) / "bad.toml"
            config.write_text("[security]\nshell_enabled=true\n")
            result = subprocess.run([str(BINARY), "--config", str(config)], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b"all security enable flags must remain false", result.stderr)
            self.assertNotIn(b"\x1b[?1049h", result.stdout)
    def test_unavailable_model(self):
        t = Terminal(self.endpoint, "absent")
        try:
            t.wait_for("unavailable")
            t.wait_for("ollama pull absent")
        finally:
            t.close(b"\x03")
    def test_unavailable_server(self):
        t = Terminal("http://127.0.0.1:1", animation=False)
        try:
            t.wait_for("Ollama unavailable")
        finally:
            t.close()
    def test_cancel_stalled_request_and_exit(self):
        t = Terminal(self.endpoint)
        try:
            t.wait_for("Connected")
            t.send("stall\r")
            t.pump(0.3)
            start = time.monotonic()
            t.send(b"\x1b")
            t.wait_for("Cancelled", timeout=2)
            self.assertLess(time.monotonic() - start, 2)
            t.send("stall\r")
            t.pump(0.2)
        finally:
            t.close(b"\x03")
    def test_premature_stream_restoration(self):
        t = Terminal(self.endpoint)
        try:
            t.wait_for("Connected")
            t.send("truncate\r")
            t.wait_for("ended without completion")
        finally:
            t.close()

if __name__ == "__main__":
    unittest.main(verbosity=2)
