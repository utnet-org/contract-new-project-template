#!/bin/sh

echo ">> Building contract"

rustup target add wasm32-unknown-unknown
# no abi
#cargo build --all --target wasm32-unknown-unknown --release
# with abi
unc dev-tool build
cp ../target/wasm32-unknown-unknown/release/hello.wasm ../res/