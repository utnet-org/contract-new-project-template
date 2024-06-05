#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
FOUNDATION_ACCOUNT_ID="${FOUNDATION_ACCOUNT_ID:-unc}"
set -e

CONTRACT_ACCOUNT_ID="lockup-whitelist-${MASTER_ACCOUNT_ID}"

echo "Deploying whitelist contract to $CONTRACT_ACCOUNT_ID with 20 unc"

#1. create account and transfer funds
(
unc account create-account fund-myself $CONTRACT_ACCOUNT_ID '20 unc' \
    use-manually-provided-public-key "ed25519:5FF38DhwzfavJxR4FULScKMZ3qn9rFeeTcDPYbyW8egN" \
    sign-as $MASTER_ACCOUNT_ID \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:9DbmnSYXws5hB7KHBLD6YwuDYCxCTX9b4MSEQhZzgTp1" \
        --signer-private-key "ed25519:4JoG9dVMwPp869VXPaWYwAfT7cLYDoZifk48FwK7gVCWXrpytrT4uyQcLQNS6vGNQZVfAHWUGTeos6fhHTsWskv9" \
    send
) &
wait
# Wait for the account to be created
while [ true ]
do
    (
    AMOUNT=$(unc account view-account-summary $CONTRACT_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
    ) &
    wait
    if [ -z "$AMOUNT" ]; then
        echo "Failed to get account summary for $CONTRACT_ACCOUNT_ID"
        sleep 30
    else
        echo "Contract account ${CONTRACT_ACCOUNT_ID} has been created with balance $AMOUNT"
        break
    fi
done
#2. deploy contract and call new method initializing the contract
unc contract deploy $CONTRACT_ACCOUNT_ID \
    use-file ../../res/whitelist.wasm \
    with-init-call new json-args '{"foundation_account_id": "'$FOUNDATION_ACCOUNT_ID'"}' \
    prepaid-gas '100.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:5FF38DhwzfavJxR4FULScKMZ3qn9rFeeTcDPYbyW8egN" \
        --signer-private-key "ed25519:UcMrCTarWPc4Sc3XLL8LPvHAPrZqFVYJqA5mSNaMo1P46ncoycRhwc4RRb7RhCiac1HKwTKDwCoZc6cy6tK28H4" \
    send
