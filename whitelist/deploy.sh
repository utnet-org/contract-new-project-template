#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-whitelist}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT

