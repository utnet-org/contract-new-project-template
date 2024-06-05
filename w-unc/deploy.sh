#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-w-unc}"
set -e

./build.sh

cd "`dirname $0`"
pushd contract

unc dev-tool deploy $ACCOUNT

popd
