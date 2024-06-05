#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
set -e

WHITELIST_ACCOUNT_ID="lockup-whitelist-${MASTER_ACCOUNT_ID}"
CONTRACT_ACCOUNT_ID="poolv1.${MASTER_ACCOUNT_ID}"

echo "Deploying staking pool factory contract to $CONTRACT_ACCOUNT_ID with 50 unc"


#1. create account and transfer funds
## stake-pool-factory
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737.json

## {"account_id":"81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737",
## "master_seed_phrase":"luggage into fall pill wine repeat undo salon index plate until matter",
## "private_key":"ed25519:2wJFRRVYadDwQT3svS81vCGdFqgX8ZMeLuNPqUejg5wNKWgQ9Crh5uhmGMRvB3NkBjGZ73Bnr5L694nkZ8qB8NWz",
## "public_key":"ed25519:9jYETemz2TFrXfmy72kRqpgWkCjiZn1BBRcYfY8ZMyPU","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:2wJFRRVYadDwQT3svS81vCGdFqgX8ZMeLuNPqUejg5wNKWgQ9Crh5uhmGMRvB3NkBjGZ73Bnr5L694nkZ8qB8NWz network-config testnet
## > Enter account ID: 81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc 81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737 '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737


#2. deploy contract and call new method initializing the contract
unc contract deploy $CONTRACT_ACCOUNT_ID \
    use-file ../../res/staking_pool_factory.wasm \
    with-init-call new json-args '{"staking_pool_whitelist_account_id": "'$WHITELIST_ACCOUNT_ID'"}' \
    prepaid-gas '100.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:5FF38DhwzfavJxR4FULScKMZ3qn9rFeeTcDPYbyW8egN" \
        --signer-private-key "ed25519:UcMrCTarWPc4Sc3XLL8LPvHAPrZqFVYJqA5mSNaMo1P46ncoycRhwc4RRb7RhCiac1HKwTKDwCoZc6cy6tK28H4" \
    send


echo "Whitelisting staking pool factory $CONTRACT_ACCOUNT_ID on whitelist contract $WHITELIST_ACCOUNT_ID"

#3. call add_factory
unc contract call-function \
    as-transaction $WHITELIST_ACCOUNT_ID add_factory json-args '{"factory_account_id": "'$CONTRACT_ACCOUNT_ID'"}' \
    prepaid-gas '100.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:5FF38DhwzfavJxR4FULScKMZ3qn9rFeeTcDPYbyW8egN" \
        --signer-private-key "ed25519:UcMrCTarWPc4Sc3XLL8LPvHAPrZqFVYJqA5mSNaMo1P46ncoycRhwc4RRb7RhCiac1HKwTKDwCoZc6cy6tK28H4" \
    send
