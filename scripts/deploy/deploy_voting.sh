#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
set -e

CONTRACT_ACCOUNT_ID="transfer-vote-${MASTER_ACCOUNT_ID}"

echo "Deploying voting contract to $CONTRACT_ACCOUNT_ID with 20 unc"

#1. create account and transfer funds
## voting
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


#2. deploy contract and call new method initializing the contract
unc contract deploy $CONTRACT_ACCOUNT_ID \
    use-file ../../res/voting_contract.wasm \
    with-init-call new json-args '{}' \
    prepaid-gas '100.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:5FF38DhwzfavJxR4FULScKMZ3qn9rFeeTcDPYbyW8egN" \
        --signer-private-key "ed25519:UcMrCTarWPc4Sc3XLL8LPvHAPrZqFVYJqA5mSNaMo1P46ncoycRhwc4RRb7RhCiac1HKwTKDwCoZc6cy6tK28H4" \
    send

