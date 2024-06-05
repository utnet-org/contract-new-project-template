#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-unc}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT

