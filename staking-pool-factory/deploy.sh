#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-staking-pool-factory}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT

