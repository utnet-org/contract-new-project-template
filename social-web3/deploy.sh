#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-social}"
set -e

./build.sh

### Requirements:
##1.$ unc account create-account fund-later use-auto-generation save-to-folder /home/ubuntu/.unc-credentials/implicit
##2.$ cat /home/ubuntu/.unc-credentials/implicit/3a562cc6f1f6e4f4acfbf29a04f7b46e70decaedc11e704ce0ed7c9467c78d9a.json

## {"account_id":"3a562cc6f1f6e4f4acfbf29a04f7b46e70decaedc11e704ce0ed7c9467c78d9a",
## "master_seed_phrase":"sure truck endorse ready unhappy theory poverty angle remove leaf birth impulse",
## "private_key":"ed25519:2LrTMyBTcqxJP5t56XFAt2p8GVKWs5rKcgR9E7ZWFBTsEGQwZHzVCxC7FjPyAdHt4uNyS8m4VRoW4JzBrHf3B2Ly",
## "public_key":"ed25519:4virt2vstZSRRdLoQuZD4kGY7Urv96HjyB74GHs8cLFT","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:2LrTMyBTcqxJP5t56XFAt2p8GVKWs5rKcgR9E7ZWFBTsEGQwZHzVCxC7FjPyAdHt4uNyS8m4VRoW4JzBrHf3B2Ly network-config testnet
## > Enter account ID: 3a562cc6f1f6e4f4acfbf29a04f7b46e70decaedc11e704ce0ed7c9467c78d9a

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc 3a562cc6f1f6e4f4acfbf29a04f7b46e70decaedc11e704ce0ed7c9467c78d9a '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=3a562cc6f1f6e4f4acfbf29a04f7b46e70decaedc11e704ce0ed7c9467c78d9a
echo ">> Deploying social contract"
#unc contract deploy unc use-file ./target/wasm32-unknown-unknown/release/contract.wasm without-init-call network-config testnet sign-with-keychain send

cd "$(dirname $0)"
pushd contract

#unc dev-tool deploy $ACCOUNT
unc contract deploy ${ACCOUNT} use-file ../../res/social_web3.wasm with-init-call new json-args {} prepaid-gas '100.0 Tgas' attached-deposit '0 unc' network-config testnet sign-with-keychain send
popd