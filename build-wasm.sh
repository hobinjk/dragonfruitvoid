#!/usr/bin/bash
set -ex

cargo build --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen --out-name dragonfruitvoid --out-dir dist --target web target/wasm32-unknown-unknown/wasm-release/dragonfruitvoid.wasm
rm -r dist/assets && cp -r assets dist/assets
cp index.html dist/
