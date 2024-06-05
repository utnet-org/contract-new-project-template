#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-social}"
set -e

./build.sh

cd "$(dirname $0)"
pushd contract

#unc dev-tool deploy $ACCOUNT
unc contract deploy ${ACCOUNT} use-file ../../res/social_web3.wasm with-init-call new json-args {} prepaid-gas '100.0 Tgas' attached-deposit '0 unc' network-config testnet sign-with-keychain send
popd