#!/usr/bin/env bash
# The server a spec run talks to answers an unknown URL the way the deployed site does (backlog row 407).
#
#   source tools/site-server.sh; site_server_ready <runner-name>
#
# GitHub Pages answers a URL the SSG did not prerender with the site's `404.html` (status 404, scripts/emit-404.sh), a
# real page that boots the app, whose router renders the not-found page. `python3 -m http.server` answers it with its
# own bare 404, so every spec that visits a not-found route (theme-picker-phone's `/charts/no-such-chart/` and
# `/component/no-such-component/`) waits for a hydration that never comes and fails at its timeout, a red that says
# nothing about the site. So a runner never trusts the server it is handed:
#
# - `PLAYWRIGHT_BASE_URL` unset and `SSG_SITE_DIR` set: the runner serves that dir itself, through
#   `scripts/emit-404.sh` + `scripts/serve-spa.py` on a free port, exports `PLAYWRIGHT_BASE_URL`, and stops the server
#   when the shell exits.
# - Otherwise it probes `${PLAYWRIGHT_BASE_URL:-http://127.0.0.1:8080}/__dx-unknown-route__/` and refuses (exit 2)
#   unless the answer is status 404 with the app shell in the body (`404.html`'s `dx-404-pending` marker).
# Callers: tools/test-changed.sh and tools/run-suite.sh.
site_server_ready() {
  local who="$1" repo base status body
  repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  if [ -z "${PLAYWRIGHT_BASE_URL:-}" ] && [ -n "${SSG_SITE_DIR:-}" ]; then
    "$repo/scripts/emit-404.sh" "$SSG_SITE_DIR" >/dev/null || return 2
    local port
    port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')"
    python3 "$repo/scripts/serve-spa.py" "$SSG_SITE_DIR" "$port" >/dev/null 2>&1 </dev/null &
    SITE_SERVER_PID=$!
    trap 'kill "$SITE_SERVER_PID" 2>/dev/null' EXIT
    export PLAYWRIGHT_BASE_URL="http://127.0.0.1:$port"
    local i
    for i in $(seq 1 50); do
      curl -s -o /dev/null "$PLAYWRIGHT_BASE_URL/" && break
      sleep 0.1
    done
    echo "$who: serving $SSG_SITE_DIR at $PLAYWRIGHT_BASE_URL (serve-spa.py)" >&2
  fi
  base="${PLAYWRIGHT_BASE_URL:-http://127.0.0.1:8080}"
  base="${base%/}"
  body="$(mktemp)"
  status="$(curl -s -o "$body" -w '%{http_code}' "$base/__dx-unknown-route__/" || true)"
  if [ "$status" = 404 ] && grep -q 'id="dx-404-pending"' "$body"; then
    rm -f "$body"
    return 0
  fi
  rm -f "$body"
  if [ "$status" = 000 ]; then
    echo "$who: nothing answers at $base -- serve the build under test first (scripts/lane-ssg.sh), or set SSG_SITE_DIR=<public dir> and leave PLAYWRIGHT_BASE_URL unset" >&2
  else
    echo "$who: $base answers an unknown URL with status $status and no app shell, so the not-found-route specs would time out." >&2
    echo "$who: serve the build the way the site is served: scripts/emit-404.sh <public> && python3 scripts/serve-spa.py <public> <port> (scripts/lane-ssg.sh does both), or set SSG_SITE_DIR=<public> and leave PLAYWRIGHT_BASE_URL unset" >&2
  fi
  return 2
}
