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
# bump the service-worker cache version so installed PWAs pick up this build
STAMP=$(date +%s)
sed -i '' -E "s/capitol-dungeon-v[0-9a-z]+/capitol-dungeon-v${STAMP}/" web/sw.js
echo "Done (cache capitol-dungeon-v${STAMP}). web/ is ready to serve or deploy."
