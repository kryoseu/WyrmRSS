"""Serves a tiny RSS feed and prints each request's headers.

python3 -u feed_server.py                         # no auth
python3 -u feed_server.py --basic reader:hunter2  # require basic auth
python3 -u feed_server.py --header X-Api-Key:abc  # require a custom header
"""

import argparse
import base64
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, HTTPServer

FEED = """<?xml version="1.0"?>
<rss version="2.0"><channel>
  <title>Auth test feed</title>
  <link>http://localhost/</link>
  <item>
    <title>Hello from the test server ({now})</title>
    <link>http://localhost/hello</link>
    <guid>auth-test-{now}</guid>
  </item>
</channel></rss>
"""


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--basic", metavar="USER:PASS")
    parser.add_argument("--header", metavar="NAME:VALUE")
    args = parser.parse_args()

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            print(f"\n--- {self.command} {self.path}")
            for name, value in self.headers.items():
                print(f"{name}: {value}")

            if not self.authorized():
                print(">>> 401 (credentials missing or wrong)")
                self.send_response(401)
                if args.basic:
                    self.send_header("WWW-Authenticate", 'Basic realm="test"')
                self.end_headers()
                return

            body = FEED.format(now=datetime.now(timezone.utc).isoformat()).encode()
            print(">>> 200")
            self.send_response(200)
            self.send_header("Content-Type", "application/rss+xml")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def authorized(self):
            if args.basic:
                expected = "Basic " + base64.b64encode(args.basic.encode()).decode()
                if self.headers.get("Authorization") != expected:
                    return False
                print(f">>> basic auth decoded: {args.basic}")
            if args.header:
                name, _, value = args.header.partition(":")
                if self.headers.get(name) != value:
                    return False
            return True

        def log_message(self, *_):
            pass  # headers are printed above; skip the default access log

    print(f"Serving on http://localhost:{args.port}/feed.xml", flush=True)
    HTTPServer(("127.0.0.1", args.port), Handler).serve_forever()


if __name__ == "__main__":
    main()
