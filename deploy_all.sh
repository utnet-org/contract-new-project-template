#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
set -e

pushd $(dirname ${BASH_SOURCE[0]})

./scripts/prepare.sh

# base contracts
echo ">> Deploying contract"
for d in */deploy.sh ; do
    d=$(dirname "$d");
    echo "Deploying $d";
    (cd $d;./deploy.sh)
done

# core contracts
./scripts/deploy_core.sh
./scripts/deploy_lockup.sh
# for wasm_file in $(find res -name "*.wasm"); do
#     unc contract deploy $ACCOUNT use-file "$wasm_file" without-init-call network-config testnet sign-with-keychain send
#     # Sleep a bit to let the previous contract upload to blockchain. Otherwise we fail publishing checks.
#     echo "sleeping for wait for 6 blocks to confirm..."
#     sleep 120
# done

popd