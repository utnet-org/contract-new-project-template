#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-../target}"
set -e

cd "$(dirname $0)"
rustup target add wasm32-unknown-unknown
# Build with all features
cargo build --target wasm32-unknown-unknown --release --all-features
cp $TARGET/wasm32-unknown-unknown/release/state_manipulation.wasm ../res/state_manipulation.wasm

# Build with just clean
cargo build --target wasm32-unknown-unknown --release --no-default-features --features clean
cp $TARGET/wasm32-unknown-unknown/release/state_manipulation.wasm ../res/state_cleanup.wasm

# Build with just state replace
cargo build --target wasm32-unknown-unknown --release --no-default-features --features replace
cp $TARGET/wasm32-unknown-unknown/release/state_manipulation.wasm ../res/state_replace.wasm