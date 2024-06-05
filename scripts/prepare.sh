#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"

set -e

# Verifying master account exist
AMOUNT=$(unc account view-account-summary $MASTER_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for master account ${MASTER_ACCOUNT_ID}. Maybe the account doesn't exist."
  cat << EOF
  ## > Enter account ID: 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376
EOF
  ## use scripts/master-account-id.json.sample, default has some tokens
  unc account import-account using-private-key ed25519:4JoG9dVMwPp869VXPaWYwAfT7cLYDoZifk48FwK7gVCWXrpytrT4uyQcLQNS6vGNQZVfAHWUGTeos6fhHTsWskv9 network-config testnet
  exit 1
fi