
#!/usr/bin/env bash

# Remove this script once the following issue is addressed:
# https://github.com/rustwasm/wasm-pack/issues/313.

set -e

# Check if 'rollup' is installed.
if ! [ -x "$(command -v rollup)" ]; then
    echo "parcel is not installed" >& 2
    exit 1
fi

# Clean old builds
if [ -d pkg ]; then rm -rf pkg; fi
if [ -d dist ]; then rm -rf dist; fi

# PKG_NAME="vector-merkle-tree"

# Build WASM module
wasm-pack build -t nodejs -d dist/node --out-name index
wasm-pack build -t bundler -d dist/browser --out-name index

# Build browser and server bundle
# parcel build pkg/index.js --dist-dir dist/browser --target browser
# parcel build pkg/index.js --dist-dir dist/node --target node
# ./node_modules/.bin/rollup pkg/index.js --file dist/browser/index.js --plugin wasm --format iife
# ./node_modules/.bin/rollup pkg/index.js --file dist/node/index.js --plugin wasm --format cjs

# # Copy the module into the distributed files
# cp -R pkg dist/module

# Copy package.json as well
cp package.json dist/package.json