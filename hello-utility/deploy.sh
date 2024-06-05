#!/bin/sh
ACCOUNT="${CONTRACT_ACCOUNT_ID:-hello}"
./build.sh

echo ">> Deploying hello contract"
#unc contract deploy unc use-file ./target/wasm32-unknown-unknown/release/contract.wasm without-init-call network-config testnet sign-with-keychain send
unc dev-tool deploy $ACCOUNT with-init-call new json-args '{"beneficiary": "7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376"}' prepaid-gas '300.0 Tgas' attached-deposit '0 unc' network-config testnet sign-with-keychain send