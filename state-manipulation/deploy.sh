#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-state-manipulation}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT

