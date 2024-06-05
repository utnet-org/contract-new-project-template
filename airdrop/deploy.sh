#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-unc}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT with-init-call new json-args {} prepaid-gas '100.0 Tgas' attached-deposit '0 unc' network-config testnet sign-with-keychain send

