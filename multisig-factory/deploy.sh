#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-mutisig-factory}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT

