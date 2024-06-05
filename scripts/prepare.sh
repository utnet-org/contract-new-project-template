#!/bin/bash
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
set -e

#1 create account
unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/whitelist
unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/voting
unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/stake-pool-factory

#2 import account
./target/debug/unc account import-account \
    using-private-key ed25519:5oMR6XaRmiFGDuvqyez5gN4oozCush1ocKgAVCZqmMHRjJNx7ZNn2tk678fDFXykFgU6hzyPZbbLpK7TNWBwMouG \
    network-config testnet
#> ? Enter account ID: a6aed866de268cfd8c0b559ad2cdd691d9d5a75d35cc7f707fb0a663c72196de

#3 transfer funds
unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 \
    send-unc a6aed866de268cfd8c0b559ad2cdd691d9d5a75d35cc7f707fb0a663c72196de '100 unc' \
    network-config testnet sign-with-keychain send
