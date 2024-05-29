#!/bin/bash
set -e

# no abi
#RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
# with abi
unc dev-tool build
cp ../target/wasm32-unknown-unknown/release/airdrop.wasm ../res/

