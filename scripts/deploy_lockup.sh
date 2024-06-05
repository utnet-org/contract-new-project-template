#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
LOCKUP_MASTER_ACCOUNT_ID="${LOCKUP_MASTER_ACCOUNT_ID:-lockup}"
set -e

if [ -z "${CHAIN_ID}" ]; then
  echo "CHAIN_ID is required, e.g. \`export CHAIN_ID=testnet\`"
  exit 1
fi

if [ -z "${MASTER_ACCOUNT_ID}" ]; then
  echo "MASTER_ACCOUNT_ID is required, e.g. \`export MASTER_ACCOUNT_ID=master\`"
  exit 1
fi


if [ -z "${LOCKUP_MASTER_ACCOUNT_ID}" ]; then
  echo "LOCKUP_MASTER_ACCOUNT_ID is required, e.g. \`export LOCKUP_MASTER_ACCOUNT_ID=lockup\`"
  exit 1
fi

echo "Using CHAIN_ID=${CHAIN_ID}"
echo "Using MASTER_ACCOUNT_ID=${MASTER_ACCOUNT_ID}"
echo "Using LOCKUP_MASTER_ACCOUNT_ID=${LOCKUP_MASTER_ACCOUNT_ID}"

# Verifying master account exist
RES=$(unc account view-account-summary $LOCKUP_MASTER_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance" && echo "OK" || echo "BAD")
if [ "$RES" = "BAD" ]; then
  echo "Can't get state for ${LOCKUP_MASTER_ACCOUNT_ID}. Maybe the account doesn't exist."
  exit 1
fi

read -p "Enter account ID: " ACCOUNT_PREFIX

PREFIX_RE=$(grep -qE '^([a-z0-9]+[-_])*[a-z0-9]+$' <<< "$ACCOUNT_PREFIX" && echo "OK" || echo "BAD")

if [ "$PREFIX_RE" = "OK" ]; then
  ACCOUNT_ID="$ACCOUNT_PREFIX.${LOCKUP_MASTER_ACCOUNT_ID}"
else
  echo "Invalid new account prefix."
  exit 1
fi

LOCKUP_ACCOUNT_ID=$ACCOUNT_ID

echo "Lockup account ID is $LOCKUP_ACCOUNT_ID"

if [ ${#LOCKUP_ACCOUNT_ID} -gt "64" ]; then
  echo "The legnth of the lockup account is longer than 64 characters"
  exit 1
fi

# Verifying the new account doesn't exist
RES=$(unc account view-account-summary $ACCOUNT_ID network-config $CHAIN_ID now | grep "balance" && echo "BAD" || echo "OK")
if [ "$RES" = "BAD" ]; then
  echo "The account ${ACCOUNT_ID} already exist."
  exit 1
fi

while true; do
  read -p "Enter OWNER_ACCOUNT_ID: " OWNER_ACCOUNT_ID

  # Verifying master account exist
  RES=$(unc account view-account-summary $OWNER_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance" && echo "OK" || echo "BAD")
  if [ "$RES" = "BAD" ]; then
    echo "Can't get state for ${OWNER_ACCOUNT_ID}. Maybe the account doesn't exist."
  else
    echo "Using owner's account ID $OWNER_ACCOUNT_ID"
    break;
  fi
done

MINIMUM_BALANCE="35"
while true; do
  read -p "Enter the amount in unc tokens (not atto) to deposit on lockup contract (min $MINIMUM_BALANCE): " LOCKUP_BALANCE
  if [ "$LOCKUP_BALANCE" -ge "$MINIMUM_BALANCE" ]; then
    echo "Going to deposit $LOCKUP_BALANCE tokens or ${LOCKUP_BALANCE}000000000000000000000000 atto UNC"
    break;
  else
    echo "The lockup balance has to be at least $MINIMUM_BALANCE UNC tokens. Try again."
  fi
done

VOTE_ACCOUNT_ID="vote-${MASTER_ACCOUNT_ID}"
WHITELIST_ACCOUNT_ID="whitelist-${MASTER_ACCOUNT_ID}"


#1. create account and transfer funds
unc account create-account fund-myself $CONTRACT_ACCOUNT_ID '${LOCKUP_BALANCE}000000000000000000000000 unc' \
    use-manually-provided-public-key "ed25519:AYU8UsZZJM5pXpYafzpwvizJk3rZitsVTyK4nqhMfvXx" \
    sign-as $LOCKUP_MASTER_ACCOUNT_ID \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:2yMvZrTtjgFMtcpE12G3tdt7KsYKdKE6jufRnz4Yyxw3" \
        --signer-private-key "ed25519:3NVx4sHxBJciEH2wZoMig8YiMx1Q84Ur2RWTd2GQ7JNfWdyDxwwYrUR6XtJR3YcYeWh9NzVEmsnYe2keB97mVExZ" \
    send

#sleep 180
#2. deploy contract and call new method initializing the contract
unc contract deploy $CONTRACT_ACCOUNT_ID \
    use-file ../../res/lockup_contract.wasm \
    with-init-call new json-args \
    '{ \
      "owner_account_id": "'$OWNER_ACCOUNT_ID'", \
      "lockup_duration": "259200000000000", \
      "transfers_information": { \
          "TransfersDisabled": { \
              "transfer_poll_account_id": "'$VOTE_ACCOUNT_ID'" \
          } \
      }, \
      "release_duration": "2592000000000000", \
      "staking_pool_whitelist_account_id": "'$WHITELIST_ACCOUNT_ID'" \
    }' \
    prepaid-gas '100.0 Tgas' \
    attached-deposit '0 unc' \
    network-config $CHAIN_ID \
    sign-with-plaintext-private-key \
        --signer-public-key "ed25519:AYU8UsZZJM5pXpYafzpwvizJk3rZitsVTyK4nqhMfvXx" \
        --signer-private-key "ed25519:3NZU7esBCfejMa556Sp4DATuadrcUSQmrQwZUb32b2ehN4tyMkzcydwAcQ41ukeEn3hkoNVTax8GusceRf2RVVFC" \
    send
