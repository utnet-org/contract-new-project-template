#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-../target}"
set -e

#RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
unc dev-tool build
cp $TARGET/wasm32-unknown-unknown/release/staking_pool_factory.wasm ../res/
