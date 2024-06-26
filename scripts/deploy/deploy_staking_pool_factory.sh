#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
WHITELIST_ACCOUNT_ID="${WHITELIST_ACCOUNT_ID:-e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773}"

set -e

CONTRACT_ACCOUNT_ID="${POOL_ACCOUNT_ID:-81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737}"

echo "Deploying staking pool factory contract to $CONTRACT_ACCOUNT_ID with 100 unc"

# Verifying contract account exist
AMOUNT=$(unc account view-account-summary $CONTRACT_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for master account ${CONTRACT_ACCOUNT_ID}. Maybe the account doesn't exist."
cat << EOF
#1. create account and transfer funds if account not exist on-chain
### if you want to use a different account, follow the steps below
### or use the default account, it has some tokens, execute the command step 3 #3 to import the account
### account stake-pool-factory:
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
## 6. wait for the account to be on-chain, 6 blocks time
EOF
  exit 1
fi


#2. deploy contract and call new method initializing the contract
unc contract deploy $CONTRACT_ACCOUNT_ID \
    use-file ../../res/staking_pool_factory.wasm \
    with-init-call new json-args '{"staking_pool_whitelist_account_id": "'$WHITELIST_ACCOUNT_ID'"}' \
    prepaid-gas '300.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:9jYETemz2TFrXfmy72kRqpgWkCjiZn1BBRcYfY8ZMyPU" \
        --signer-private-key "ed25519:2wJFRRVYadDwQT3svS81vCGdFqgX8ZMeLuNPqUejg5wNKWgQ9Crh5uhmGMRvB3NkBjGZ73Bnr5L694nkZ8qB8NWz" \
    send


echo "Whitelisting staking pool factory $CONTRACT_ACCOUNT_ID on whitelist contract $WHITELIST_ACCOUNT_ID"

# wait for the contract to be deployed, 6 blocks time
sleep 180
#3. call add_factory
unc contract call-function \
    as-transaction $WHITELIST_ACCOUNT_ID add_factory json-args '{"factory_account_id": "'$CONTRACT_ACCOUNT_ID'"}' \
    prepaid-gas '300.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:9jYETemz2TFrXfmy72kRqpgWkCjiZn1BBRcYfY8ZMyPU" \
        --signer-private-key "ed25519:2wJFRRVYadDwQT3svS81vCGdFqgX8ZMeLuNPqUejg5wNKWgQ9Crh5uhmGMRvB3NkBjGZ73Bnr5L694nkZ8qB8NWz" \
    send
