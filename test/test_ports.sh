#!/usr/bin/bash
# Smoke-test `quickbridge ports` against a short-lived local HTTP server.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/target/debug/quickbridge"
if [[ ! -x $BIN ]]; then
  echo "missing debug helper; run cargo test first" >&2
  exit 1
fi

PORT=18765
python3 - "$PORT" <<'PY' &
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer
port = int(sys.argv[1])
class H(BaseHTTPRequestHandler):
    def do_GET(self):
        body = b"<html><title>Quick Bridge Probe</title></html>"
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def log_message(self, *args):
        pass
HTTPServer(("127.0.0.1", port), H).serve_forever()
PY
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 20); do
  if bash -c "echo >/dev/tcp/127.0.0.1/$PORT" 2>/dev/null; then
    break
  fi
  sleep 0.05
done
out=$("$BIN" ports)
echo "$out"
echo "$out" | grep -q "\"event\":\"ports\""
echo "$out" | grep -q "\"port\":$PORT"
echo "$out" | grep -q "Quick Bridge Probe"
kill "$pid"
trap - EXIT
echo ok
