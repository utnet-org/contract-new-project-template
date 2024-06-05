#!/bin/sh
ACCOUNT="${CONTRACT_ACCOUNT_ID:-unc}"
./build.sh

echo ">> Deploying contract"
#unc contract deploy unc use-file ./target/wasm32-unknown-unknown/release/contract.wasm without-init-call network-config testnet sign-with-keychain send
unc dev-tool deploy $ACCOUNT with-init-call new json-args '{"beneficiary": "unc"}' prepaid-gas '100.0 Tgas' attached-deposit '0 unc' network-config testnet sign-with-keychain send