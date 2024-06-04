#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-unc}"
set -e

WHITELIST_ACCOUNT_ID="lockup-whitelist-${MASTER_ACCOUNT_ID}"
CONTRACT_ACCOUNT_ID="poolv1.${MASTER_ACCOUNT_ID}"

echo "Deploying staking pool factory contract to $CONTRACT_ACCOUNT_ID with 50 unc"


#1. create account and transfer funds
unc account create-account fund-myself $CONTRACT_ACCOUNT_ID '20 unc' \
    use-manually-provided-public-key "ed25519:AYU8UsZZJM5pXpYafzpwvizJk3rZitsVTyK4nqhMfvXx" \
    sign-as $MASTER_ACCOUNT_ID \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:2yMvZrTtjgFMtcpE12G3tdt7KsYKdKE6jufRnz4Yyxw3" \
        --signer-private-key "ed25519:3NVx4sHxBJciEH2wZoMig8YiMx1Q84Ur2RWTd2GQ7JNfWdyDxwwYrUR6XtJR3YcYeWh9NzVEmsnYe2keB97mVExZ" \
    send

#sleep 180
#2. deploy contract and call new method initializing the contract
unc contract deploy $CONTRACT_ACCOUNT_ID \
    use-file ../../res/staking_pool_factory.wasm \
    with-init-call new json-args '{"staking_pool_whitelist_account_id": "'$WHITELIST_ACCOUNT_ID'"}' \
    prepaid-gas '100.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:AYU8UsZZJM5pXpYafzpwvizJk3rZitsVTyK4nqhMfvXx" \
        --signer-private-key "ed25519:3NZU7esBCfejMa556Sp4DATuadrcUSQmrQwZUb32b2ehN4tyMkzcydwAcQ41ukeEn3hkoNVTax8GusceRf2RVVFC" \
    send


echo "Whitelisting staking pool factory $CONTRACT_ACCOUNT_ID on whitelist contract $WHITELIST_ACCOUNT_ID"

#3. call add_factory
unc contract call-function \
    as-transaction $WHITELIST_ACCOUNT_ID add_factory json-args '{"factory_account_id": "'$CONTRACT_ACCOUNT_ID'"}' \
    prepaid-gas '100.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:AYU8UsZZJM5pXpYafzpwvizJk3rZitsVTyK4nqhMfvXx" \
        --signer-private-key "ed25519:3NZU7esBCfejMa556Sp4DATuadrcUSQmrQwZUb32b2ehN4tyMkzcydwAcQ41ukeEn3hkoNVTax8GusceRf2RVVFC" \
    send
