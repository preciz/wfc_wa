#!/bin/bash
set -e

echo "Downloading wasm-pack..."
# Download wasm-pack binary locally to avoid read-only file system errors
# Using a fixed version to ensure stability
WASM_PACK_VERSION="v0.12.1"
curl -L "https://github.com/rustwasm/wasm-pack/releases/download/${WASM_PACK_VERSION}/wasm-pack-${WASM_PACK_VERSION}-x86_64-unknown-linux-musl.tar.gz" | tar -xz

# The tarball extracts to a directory named after the release
# We'll run the binary directly from there
WASM_PACK_BIN="./wasm-pack-${WASM_PACK_VERSION}-x86_64-unknown-linux-musl/wasm-pack"

echo "Building WASM..."
$WASM_PACK_BIN build --target web --out-dir www/pkg
