#!/bin/sh

./build.sh

echo ">> Deploying contract"

unc contract deploy unc use-file ./target/wasm32-unknown-unknown/release/contract.wasm without-init-call network-config testnet sign-with-keychain send