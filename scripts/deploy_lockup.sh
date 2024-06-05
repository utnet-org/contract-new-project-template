#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
VOTE_ACCOUNT_ID="${VOTE_ACCOUNT_ID:-0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee}"
WHITELIST_ACCOUNT_ID="${WHITELIST_ACCOUNT_ID:-e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773}"

set -e

./prepare.sh

if [ -z "${CHAIN_ID}" ]; then
  echo "CHAIN_ID is required, e.g. \`export CHAIN_ID=testnet\`"
  exit 1
fi

if [ -z "${MASTER_ACCOUNT_ID}" ]; then
  echo "MASTER_ACCOUNT_ID is required, e.g. \`export MASTER_ACCOUNT_ID=master\`"
  exit 1
fi


if [ -z "${MASTER_ACCOUNT_ID}" ]; then
  echo "MASTER_ACCOUNT_ID is required, e.g. \`export MASTER_ACCOUNT_ID=lockup\`"
  exit 1
fi

echo "Using CHAIN_ID=${CHAIN_ID}"
echo "Using MASTER_ACCOUNT_ID=${MASTER_ACCOUNT_ID}"

# Verifying master account exist
RES=$(unc account view-account-summary $MASTER_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance" && echo "OK" || echo "BAD")
if [ "$RES" = "BAD" ]; then
  echo "Can't get state for ${MASTER_ACCOUNT_ID}. Maybe the account doesn't exist."
  exit 1
fi

read -p "Enter account ID: " ACCOUNT_PREFIX

PREFIX_RE=$(grep -qE '^([a-z0-9]+[-_])*[a-z0-9]+$' <<< "$ACCOUNT_PREFIX" && echo "OK" || echo "BAD")

if [ "$PREFIX_RE" = "OK" ]; then
  ACCOUNT_ID="$ACCOUNT_PREFIX"
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


# Verifying master account exist
AMOUNT=$(unc account view-account-summary $CONTRACT_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for master account ${CONTRACT_ACCOUNT_ID}. Maybe the account doesn't exist."
cat << EOF
#1. create account and transfer funds if account not exist on-chain
### if you want to use a different account, follow the steps below
### or use the default account, it has some tokens, execute the command step 3 #3 to import the account
## account lockup:
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846.json

## {"account_id":"ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846",
## "master_seed_phrase":"glimpse card pride element local monkey company puppy stock fashion giraffe salt",
## "private_key":"ed25519:4mwryZ4GXVJieS9ccaGBhH3BnyNV9ovNonWrrxr9nfpu9wybPYUQsY1DdxQfVZ6ZJdvj61WYgsLoWMVPxSqYv2ms",
## "public_key":"ed25519:H6Gx6FSEFJVPGXQkhX6kcPpB9NyXM2SNp8weFgb1A9fb",
## "seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:4mwryZ4GXVJieS9ccaGBhH3BnyNV9ovNonWrrxr9nfpu9wybPYUQsY1DdxQfVZ6ZJdvj61WYgsLoWMVPxSqYv2ms network-config testnet
## > Enter account ID: ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846 '${LOCKUP_BALANCE}000000000000000000000000 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846
## 6. wait for the account to be on-chain, 6 blocks time
EOF
  exit 1
fi


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
        --signer-public-key "ed25519:H6Gx6FSEFJVPGXQkhX6kcPpB9NyXM2SNp8weFgb1A9fb" \
        --signer-private-key "ed25519:4mwryZ4GXVJieS9ccaGBhH3BnyNV9ovNonWrrxr9nfpu9wybPYUQsY1DdxQfVZ6ZJdvj61WYgsLoWMVPxSqYv2ms" \
    send
