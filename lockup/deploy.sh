#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-lookup}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT

