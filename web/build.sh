#!/bin/bash
# Build the WASM package for the web client
set -e
cd "$(dirname "$0")/.."
wasm-pack build crates/aa-wasm --target web --out-dir ../../web/pkg
echo "✅ Build complete. Run 'python3 web/serve.py' then open http://localhost:8080"
