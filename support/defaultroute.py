# Python 3 server example
from http.server import BaseHTTPRequestHandler, HTTPServer
from json import JSONDecoder

hostName = "defaultroute"
serverPort = 8080

class MyServer(BaseHTTPRequestHandler):
    def do_GET(self):
        print([x for x in self.headers.raw_items()], flush=True)
        body = JSONDecoder().decode(self.rfile.read(int(self.headers['Content-Length'])).decode('UTF-8'))
        print(body, flush=True)
        self.send_response(200)
        self.send_header("Content-type", "text/plain")
        self.end_headers()

if __name__ == "__main__":        
    webServer = HTTPServer((hostName, serverPort), MyServer)
    print("Server started http://%s:%s" % (hostName, serverPort), flush=True)

    try:
        webServer.serve_forever()
    except KeyboardInterrupt:
        pass

    webServer.server_close()
    print("Server stopped.", flush=True)
