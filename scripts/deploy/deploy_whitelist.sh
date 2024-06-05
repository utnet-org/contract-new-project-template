#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
FOUNDATION_ACCOUNT_ID="${FOUNDATION_ACCOUNT_ID:-unc}"

set -e

CONTRACT_ACCOUNT_ID="${WHITELIST_ACCOUNT_ID:-e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773}"

# Verifying contract account exist
AMOUNT=$(unc account view-account-summary $CONTRACT_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for master account ${CONTRACT_ACCOUNT_ID}. Maybe the account doesn't exist."
  cat << EOF
#1. create account and transfer funds if account not exist on-chain
### if you want to use a different account, follow the steps below
### or use the default account, it has some tokens, execute the command step 3 #3 to import the account
## account whitelist:
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773.json

## {"account_id":"e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773",
## "master_seed_phrase":"degree crisp fat noodle clog word globe filter problem rice swear priority",
## "private_key":"ed25519:FfnmBekG6NYcrqsgk3JNoLr9qbc6T3cqY6YjoYFYKGHuELswSyjxRZzZAoDc4rweuByHpqCQDrQnLV1Excm2W2W",
## "public_key":"ed25519:GDHGgfGte8prwJ4dEJcCB8SZhKWf7RWSXxTHg4fzY62W","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:FfnmBekG6NYcrqsgk3JNoLr9qbc6T3cqY6YjoYFYKGHuELswSyjxRZzZAoDc4rweuByHpqCQDrQnLV1Excm2W2W network-config testnet
## > Enter account ID: e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773 '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773
## 6. wait for the account to be on-chain, 6 blocks time
EOF
  exit 1
fi

echo "Deploying whitelist contract to $CONTRACT_ACCOUNT_ID with 100 unc"

#2. deploy contract and call new method initializing the contract
unc contract deploy $CONTRACT_ACCOUNT_ID \
    use-file ../../res/whitelist.wasm \
    with-init-call new json-args '{"foundation_account_id": "'$FOUNDATION_ACCOUNT_ID'"}' \
    prepaid-gas '300.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:GDHGgfGte8prwJ4dEJcCB8SZhKWf7RWSXxTHg4fzY62W" \
        --signer-private-key "ed25519:FfnmBekG6NYcrqsgk3JNoLr9qbc6T3cqY6YjoYFYKGHuELswSyjxRZzZAoDc4rweuByHpqCQDrQnLV1Excm2W2W" \
    send
