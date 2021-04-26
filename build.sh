#!/usr/bin/env bash

# Remove this script once the following issue is addressed:
# https://github.com/rustwasm/wasm-pack/issues/313.

set -e

# Clean old builds
if [ -d pkg ]; then rm -rf pkg; fi
if [ -d dist ]; then rm -rf dist; fi

# Build WASM module
wasm-pack build -t nodejs -d dist/node --out-name index
wasm-pack build -t bundler -d dist/browser --out-name index

# Copy package.json as well
cp package.json dist/package.json