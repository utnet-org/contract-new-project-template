#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-unc}"
set -e

pushd $(dirname ${BASH_SOURCE[0]})

echo ">> Deploying contract"
for wasm_file in $(find res -name "*.wasm"); do
    unc contract deploy $ACCOUNT use-file "$wasm_file" without-init-call network-config testnet sign-with-keychain send
done

popd