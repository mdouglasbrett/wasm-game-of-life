#!/bin/bash

set -euo pipefail

TARGET=wasm32-unknown-unknown
BINARY=target/$TARGET/release/wasm_game_of_life.wasm
OUT=client/wasm

# TODO: get the binary size down
cargo build --target $TARGET --release
wasm-bindgen --target web --out-dir $OUT $BINARY
ls -lh $OUT/wasm_game_of_life.wasm
