#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-unc}"
set -e

pushd $(dirname ${BASH_SOURCE[0]})

echo ">> Deploying contract"
for wasm_file in $(find res -name "*.wasm"); do
    unc contract deploy $ACCOUNT use-file "$wasm_file" without-init-call network-config testnet sign-with-keychain send
    # Sleep a bit to let the previous contract upload to blockchain. Otherwise we fail publishing checks.
    echo "sleeping for wait for 6 blocks to confirm..."
    sleep 120
done

popd