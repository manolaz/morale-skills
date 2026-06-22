from __future__ import annotations
import hashlib, threading
from typing import Any
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

class Receiver:
    def __init__(self, run_id: str, host: str = "127.0.0.1", port: int = 0):
        self.run_id = run_id
        self._records: list[dict[str, Any]] = []
        self._lock = threading.Lock()
        outer = self

        class Handler(BaseHTTPRequestHandler):
            def _record(self):
                n = int(self.headers.get("Content-Length", 0) or 0)
                body = self.rfile.read(n) if n else b""
                rec = {"run_id": outer.run_id, "method": self.command, "path": self.path,
                       "headers": dict(self.headers), "body": body,
                       "body_hash": hashlib.sha256(body).hexdigest()}
                with outer._lock:
                    outer._records.append(rec)
                self.send_response(200); self.end_headers(); self.wfile.write(b"ok")
            do_POST = _record
            do_GET = _record
            do_PUT = _record
            def log_message(self, *a): pass

        self._server = ThreadingHTTPServer((host, port), Handler)
        self.port = self._server.server_address[1]
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)

    def start(self) -> None:
        self._thread.start()

    def stop(self) -> None:
        self._server.shutdown()
        self._server.server_close()
        self._thread.join(timeout=5)

    def records(self) -> list[dict[str, Any]]:
        with self._lock:
            return list(self._records)

    def reset(self) -> None:
        with self._lock:
            self._records.clear()
