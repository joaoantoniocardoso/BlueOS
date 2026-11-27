"""Registry proxy for the pull tests. Runs inside blueos-core: `python3 proxy.py <listen> <upstream> <bytes/s>`.

Blobs of the repository `stall` never arrive. Blobs of the repository `hung` never arrive while the file
/tmp/hung exists. Blobs of the repository `slow` arrive at the given rate. All other blobs arrive at full speed.
"""

import http.server
import os
import socketserver
import sys
import threading
import time
import urllib.error
import urllib.request
from typing import Any

LISTEN, UPSTREAM, RATE = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
CHUNK = 4096


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self) -> None:  # pylint: disable=invalid-name
        is_blob = "/blobs/" in self.path
        if is_blob and (
            self.path.startswith("/v2/stall/") or self.path.startswith("/v2/hung/") and os.path.exists("/tmp/hung")
        ):
            threading.Event().wait()
        # Docker sends one Accept header per media type, and a dict would keep only one of them
        headers = {
            key: ", ".join(self.headers.get_all(key) or []) for key in set(self.headers) if key.lower() != "host"
        }
        upstream_request = urllib.request.Request(f"http://127.0.0.1:{UPSTREAM}{self.path}", headers=headers)
        try:
            upstream = urllib.request.urlopen(upstream_request)  # pylint: disable=consider-using-with
        except urllib.error.HTTPError as error:
            upstream = error
        with upstream:
            self.send_response(upstream.status)
            for key, value in upstream.headers.items():
                if key.lower() not in ("transfer-encoding", "connection"):
                    self.send_header(key, value)
            self.end_headers()
            while chunk := upstream.read(CHUNK):
                self.wfile.write(chunk)
                if is_blob and self.path.startswith("/v2/slow/"):
                    time.sleep(len(chunk) / RATE)

    do_HEAD = do_GET

    def log_message(self, *_args: Any) -> None:
        pass


class Server(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True


Server(("0.0.0.0", LISTEN), Handler).serve_forever()
