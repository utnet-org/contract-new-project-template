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
    use-manually-provided-public-key "ed25519:AYU8UsZZJM5pXpYafzpwvizJk3rZitsVTyK4nqhMfvXx" \
    sign-as $MASTER_ACCOUNT_ID \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:2yMvZrTtjgFMtcpE12G3tdt7KsYKdKE6jufRnz4Yyxw3" \
        --signer-private-key "ed25519:3NVx4sHxBJciEH2wZoMig8YiMx1Q84Ur2RWTd2GQ7JNfWdyDxwwYrUR6XtJR3YcYeWh9NzVEmsnYe2keB97mVExZ" \
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
        --signer-public-key "ed25519:AYU8UsZZJM5pXpYafzpwvizJk3rZitsVTyK4nqhMfvXx" \
        --signer-private-key "ed25519:3NZU7esBCfejMa556Sp4DATuadrcUSQmrQwZUb32b2ehN4tyMkzcydwAcQ41ukeEn3hkoNVTax8GusceRf2RVVFC" \
    send
