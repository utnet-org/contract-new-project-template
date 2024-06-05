#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-voting}"
set -e

./build.sh
unc dev-tool deploy $ACCOUNT

