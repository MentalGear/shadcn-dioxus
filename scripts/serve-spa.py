#!/usr/bin/env python3
"""Static server for a prerendered SSG site with GitHub Pages' unknown-URL behaviour (backlog row 152a).

    scripts/serve-spa.py <dir> <port>        (binds 127.0.0.1)

`python3 -m http.server` answers an unknown URL with its own bare 404 page. The deployed site is served by GitHub
Pages, which answers it with the site's `404.html` (status 404, scripts/emit-404.sh): a real prerendered page that
boots the app, whose router then renders the not-found page. So a spec of a not-found route (theme-picker-phone's
`/component/no-such-component/`, `/charts/no-such-chart/`) timed out at 90 s on the bare server, four tests = 6 of the
121 minutes of a full debug run, and was a red that said nothing about the site. This server does what Pages does:
status 404 with `<dir>/404.html` as the body when that file exists, the stock 404 otherwise.
"""
import functools
import http.server
import os
import sys


class Handler(http.server.SimpleHTTPRequestHandler):
    def send_error(self, code, message=None, explain=None):
        page = os.path.join(self.directory, "404.html")
        if code == 404 and os.path.isfile(page):
            body = open(page, "rb").read()
            self.send_response(404)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            if self.command != "HEAD":
                self.wfile.write(body)
        else:
            super().send_error(code, message, explain)

    def log_message(self, *args):
        pass


if __name__ == "__main__":
    directory, port = os.path.abspath(sys.argv[1]), int(sys.argv[2])
    handler = functools.partial(Handler, directory=directory)
    http.server.ThreadingHTTPServer(("127.0.0.1", port), handler).serve_forever()
