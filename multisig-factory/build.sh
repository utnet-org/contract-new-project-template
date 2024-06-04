#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-../target}"
set -e

#RUSTFLAGS='-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
unc dev-tool build
cp $TARGET/wasm32-unknown-unknown/release/multisig_factory.wasm ../res/
