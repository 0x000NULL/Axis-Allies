#!/usr/bin/env python3
"""Simple HTTP server for the Axis & Allies web client.
Run from the web/ directory: python3 serve.py
Then open http://localhost:8080
"""
import http.server
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        '.wasm': 'application/wasm',
        '.js': 'application/javascript',
    }

print("Serving at http://localhost:8080")
http.server.HTTPServer(('', 8080), Handler).serve_forever()
