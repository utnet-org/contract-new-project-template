#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-mulisig}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT

