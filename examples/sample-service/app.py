#!/usr/bin/env python3
"""
Sample X-Road Information System / Provider Adapter Service
Demonstrating end-to-end SOAP and REST data exchange through X-Road Security Servers.
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import os
import time

PORT = int(os.environ.get("PORT", 5000))

SAMPLE_PERSONS = [
    {"id": "38001010001", "name": "Mati Tamm", "status": "ACTIVE", "city": "Tallinn"},
    {"id": "48002020002", "name": "Mari Kallas", "status": "ACTIVE", "city": "Tartu"},
    {"id": "39003030003", "name": "Jaan Sepp", "status": "ACTIVE", "city": "Pärnu"},
]

class XRoadSampleHandler(BaseHTTPRequestHandler):
    def _send_json(self, status, payload):
        data = json.dumps(payload, indent=2).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def _extract_xroad_headers(self):
        return {
            k: v for k, v in self.headers.items()
            if k.lower().startswith("x-road-") or k.lower() in ("x-request-id", "authorization")
        }

    def do_GET(self):
        xroad_headers = self._extract_xroad_headers()
        print(f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] GET {self.path} | X-Road Headers: {xroad_headers}")

        if self.path in ("/", "/health", "/status"):
            self._send_json(200, {
                "status": "UP",
                "service": "sample-xroad-provider-service",
                "timestamp": int(time.time()),
            })
        elif self.path.startswith("/hello"):
            self._send_json(200, {
                "message": "Hello from X-Road Provider Adapter Service!",
                "service_code": "sampleService",
                "received_headers": xroad_headers,
                "server_time": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            })
        elif self.path.startswith("/api/persons"):
            self._send_json(200, {
                "count": len(SAMPLE_PERSONS),
                "persons": SAMPLE_PERSONS,
                "caller": xroad_headers.get("x-road-client", "UNKNOWN_CLIENT"),
            })
        else:
            self._send_json(200, {
                "path": self.path,
                "message": "Processed successfully by provider service adapter",
                "received_headers": xroad_headers,
            })

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8", errors="replace") if length > 0 else ""
        xroad_headers = self._extract_xroad_headers()
        print(f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] POST {self.path} (Length: {length}) | X-Road Headers: {xroad_headers}")

        content_type = self.headers.get("Content-Type", "")

        # If incoming request is a SOAP XML Envelope
        if "xml" in content_type:
            soap_resp = (
                '<?xml version="1.0" encoding="utf-8"?>\n'
                '<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" '
                'xmlns:xrd="http://x-road.eu/xsd/xroad.xsd">\n'
                '  <soapenv:Header/>\n'
                '  <soapenv:Body>\n'
                '    <sampleServiceResponse>\n'
                '      <response>\n'
                '        <message>SOAP query executed successfully by sample provider</message>\n'
                '        <status>OK</status>\n'
                '      </response>\n'
                '    </sampleServiceResponse>\n'
                '  </soapenv:Body>\n'
                '</soapenv:Envelope>\n'
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/xml; charset=utf-8")
            self.send_header("Content-Length", str(len(soap_resp)))
            self.end_headers()
            self.wfile.write(soap_resp)
            return

        # Default JSON response
        try:
            parsed_json = json.loads(body) if body else {}
        except Exception:
            parsed_json = {"raw_body": body}

        self._send_json(200, {
            "status": "SUCCESS",
            "received_payload": parsed_json,
            "received_headers": xroad_headers,
        })

    def log_message(self, format, *args):
        # Clean logging output
        return

def run():
    server_address = ("0.0.0.0", PORT)
    httpd = HTTPServer(server_address, XRoadSampleHandler)
    print(f"Sample X-Road Provider Adapter Service listening on http://0.0.0.0:{PORT}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        httpd.server_close()

if __name__ == "__main__":
    run()
