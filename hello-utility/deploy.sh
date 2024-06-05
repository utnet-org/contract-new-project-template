#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-hello}"
./build.sh

### Requirements:
##1.$ unc account create-account fund-later use-auto-generation save-to-folder /home/ubuntu/.unc-credentials/implicit
##2.$ cat /home/ubuntu/.unc-credentials/implicit/baeca15656753d36b5f5c903ed5a977fed292af19a4698c3660045d1fd8d0b6a.json

##  {"account_id":"baeca15656753d36b5f5c903ed5a977fed292af19a4698c3660045d1fd8d0b6a",
##  "master_seed_phrase":"mutual combine knife table cross army naive undo quick final joke reopen",
##  "private_key":"ed25519:F6dzasVpq9jizeV99ATgLmHfiExwhRwpqfk35BYQyDdnXa4f4BJP4hwPhpdcVPD4mTVy4opsHTYempkcbLqRjNZ",
##  "public_key":"ed25519:Dag7Jr1VW5c5TsfaBp58z5NW2Yda2YC7jfo14JRQnYkR","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:F6dzasVpq9jizeV99ATgLmHfiExwhRwpqfk35BYQyDdnXa4f4BJP4hwPhpdcVPD4mTVy4opsHTYempkcbLqRjNZ network-config testnet
## > Enter account ID: baeca15656753d36b5f5c903ed5a977fed292af19a4698c3660045d1fd8d0b6a

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc baeca15656753d36b5f5c903ed5a977fed292af19a4698c3660045d1fd8d0b6a '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=baeca15656753d36b5f5c903ed5a977fed292af19a4698c3660045d1fd8d0b6a
echo ">> Deploying hello contract"
#unc contract deploy unc use-file ./target/wasm32-unknown-unknown/release/contract.wasm without-init-call network-config testnet sign-with-keychain send
unc dev-tool deploy $ACCOUNT with-init-call new json-args '{"beneficiary": "7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376"}' prepaid-gas '300.0 Tgas' attached-deposit '0 unc' network-config testnet sign-with-keychain send