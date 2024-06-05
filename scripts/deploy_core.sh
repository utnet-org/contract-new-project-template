#!/bin/bash
CHAIN_ID="${CHAIN_ID:-testnet}"
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
FOUNDATION_ACCOUNT_ID="${FOUNDATION_ACCOUNT_ID:-unc}"
set -e

if [ -z "${CHAIN_ID}" ]; then
  echo "CHAIN_ID is required, e.g. \`export CHAIN_ID=testnet\`"
  exit 1
fi

if [ -z "${MASTER_ACCOUNT_ID}" ]; then
  echo "MASTER_ACCOUNT_ID is required, e.g. \`export MASTER_ACCOUNT_ID=unc\`"
  exit 1
fi

if [ -z "${FOUNDATION_ACCOUNT_ID}" ]; then
  echo "FOUNDATION_ACCOUNT_ID is required, e.g. \`export FOUNDATION_ACCOUNT_ID=foundation\`"
fi

echo "Using CHAIN_ID=${CHAIN_ID}"
echo "Using MASTER_ACCOUNT_ID=${MASTER_ACCOUNT_ID}"
echo "Using FOUNDATION_ACCOUNT_ID=${FOUNDATION_ACCOUNT_ID}"

# Verifying master account exist
AMOUNT=$(unc account view-account-summary $MASTER_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for master account ${MASTER_ACCOUNT_ID}. Maybe the account doesn't exist."
  exit 1
fi

# Verifying foundation account exist
AMOUNT=$(unc account view-account-summary $FOUNDATION_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for foundation account ${FOUNDATION_ACCOUNT_ID}. Maybe the account doesn't exist."
  exit 1
fi

./prepare.sh

pushd deploy
./deploy_voting.sh
./deploy_whitelist.sh
./deploy_staking_pool_factory.sh

popd
