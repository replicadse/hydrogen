# Python 3 server example
from http.server import BaseHTTPRequestHandler, HTTPServer

hostName = "connect"
serverPort = 8080

class MyServer(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.headers.get('Authorization') != 'sugarhowyougetsofly':
            self.send_response(500)
            self.end_headers()
            return
        self.send_response(200)
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
