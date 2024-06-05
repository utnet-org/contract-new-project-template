#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-social}"
set -e

./build.sh

cd "$(dirname $0)"
pushd contract

#unc dev-tool deploy $ACCOUNT
unc contract deploy unc use-file ../../res/social_web3.wasm without-init-call network-config testnet sign-with-keychain send

popd