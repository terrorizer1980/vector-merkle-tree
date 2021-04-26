#!/usr/bin/env bash

# Remove this script once the following issue is addressed:
# https://github.com/rustwasm/wasm-pack/issues/313.

set -e

# Clean old builds
if [ -d pkg ]; then rm -rf pkg; fi
if [ -d dist ]; then rm -rf dist; fi

echo "========================================================================"
echo "Build WASM module for Node.js"
echo

# Build WASM module for Node.js
wasm-pack build -t nodejs -d dist/node --out-name index

echo
echo "========================================================================"
echo "Build WASM module for browsers"
echo

# Build WASM module for the browser
wasm-pack build -t bundler -d dist/browser --out-name index