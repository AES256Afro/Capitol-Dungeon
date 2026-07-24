#!/usr/bin/env bash
# Build the browser version of Capitol Dungeon into web/.
# Serve the web/ directory with any static file server, e.g.:
#   python3 -m http.server 8080 --directory web
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/capitol-dungeon.wasm web/
# ship the data files so the wasm build can load (and players can mod) them
rm -rf web/data
cp -R data web/data
echo "Done. web/ is ready to serve or deploy to any static host."
