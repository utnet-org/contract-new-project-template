#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"

set -e

CONTRACT_ACCOUNT_ID="${VOTING_ACCOUNT_ID:-0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee}"

echo "Deploying voting contract to $CONTRACT_ACCOUNT_ID with 100 unc"

# Verifying contract account exist
AMOUNT=$(unc account view-account-summary $CONTRACT_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for master account ${CONTRACT_ACCOUNT_ID}. Maybe the account doesn't exist."
  cat << EOF
#1. create account and transfer funds if account not exist on-chain
### if you want to use a different account, follow the steps below
### or use the default account, it has some tokens, execute the command step 3 #3 to import the account
## account voting:
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee.json

##{"account_id":"0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee",
## "master_seed_phrase":"crunch coach section hospital disagree denial hospital suspect view cycle brand please",
## "private_key":"ed25519:v9zXRShtYhyEDEjBeNDjU4fnjghiCwSVm4qwA5kBA17fXT4y66S7YvYjYEdYaRiT8xnvPEErEgegeTpYxPaiZ5F",
## "public_key":"ed25519:mz4koCMGRmbEDW6GCgferaVP5Upq9tozgaz3gnXZSp5","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:v9zXRShtYhyEDEjBeNDjU4fnjghiCwSVm4qwA5kBA17fXT4y66S7YvYjYEdYaRiT8xnvPEErEgegeTpYxPaiZ5F network-config testnet
## > Enter account ID: 0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc 0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee

## 6. wait for the account to be on-chain, 6 blocks time
EOF
  exit 1
fi

#2. deploy contract and call new method initializing the contract
unc contract deploy $CONTRACT_ACCOUNT_ID \
    use-file ../../res/voting_contract.wasm \
    with-init-call new json-args '{}' \
    prepaid-gas '300.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:mz4koCMGRmbEDW6GCgferaVP5Upq9tozgaz3gnXZSp5" \
        --signer-private-key "ed25519:v9zXRShtYhyEDEjBeNDjU4fnjghiCwSVm4qwA5kBA17fXT4y66S7YvYjYEdYaRiT8xnvPEErEgegeTpYxPaiZ5F" \
    send

