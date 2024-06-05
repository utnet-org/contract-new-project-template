#!/bin/bash
ACCOUNT="${CONTRACT_ACCOUNT_ID:-drop}"
set -e

### Requirements:
##1.$ unc account create-account fund-later use-auto-generation save-to-folder /home/ubuntu/.unc-credentials/implicit
##2.$ cat /home/ubuntu/.unc-credentials/implicit/4b10d0953864750bc352df92d657413fbe4c014dd0ef5bcbeb7c8cec9d359acc.json

## {"account_id":"4b10d0953864750bc352df92d657413fbe4c014dd0ef5bcbeb7c8cec9d359acc",
## "master_seed_phrase":"catch vintage vivid matter luggage actual jungle victory kick edit acoustic used",
## "private_key":"ed25519:3p7cBxkDPJHygm5miYZZw4HGFnTZ8zEeLggBmH5BL8JCQ7ze4NcUkDgopzMfjyhCdkfSX5e3jiFj6hXEjvJ4aoEj",
## "public_key":"ed25519:642T1SQC4N1Qp9mhSD37832kDXHaK7WERptNGvFCzwvT","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:3p7cBxkDPJHygm5miYZZw4HGFnTZ8zEeLggBmH5BL8JCQ7ze4NcUkDgopzMfjyhCdkfSX5e3jiFj6hXEjvJ4aoEj network-config testnet
## > Enter account ID: 4b10d0953864750bc352df92d657413fbe4c014dd0ef5bcbeb7c8cec9d359acc

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc 4b10d0953864750bc352df92d657413fbe4c014dd0ef5bcbeb7c8cec9d359acc '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=4b10d0953864750bc352df92d657413fbe4c014dd0ef5bcbeb7c8cec9d359acc

./build.sh
unc dev-tool deploy $ACCOUNT with-init-call new json-args {} prepaid-gas '100.0 Tgas' attached-deposit '0 unc' network-config testnet sign-with-keychain send

