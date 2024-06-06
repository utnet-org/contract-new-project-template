#!/bin/bash
set -e

# start 5 unc-node instances
cat << EOF > docker-compose.yml
version: "3.8"
services:
  unc-node1:
    image: ghcr.io/utnet-org/utility:latest
    environment:
      - UNC_HOME=$HOME/.unc
      - CHAIN_ID=localnet
      - INIT=true
    volumes:
      - ${HOME}//.unc/localnet/node1:$HOME/.unc
    ports:
      - 3031:3030
      - 12346:12345
  unc-node2:
    image: ghcr.io/utnet-org/utility:latest
    environment:
      - UNC_HOME=$HOME/.unc
      - CHAIN_ID=localnet
      - INIT=true
    volumes:
      - ${HOME}/.unc/localnet/node2:$HOME/.unc
    ports:
      - 3032:3030
      - 12347:12345
  unc-node3:
    image: ghcr.io/utnet-org/utility:latest
    environment:
      - UNC_HOME=$HOME/.unc
      - CHAIN_ID=localnet
      - INIT=true
    volumes:
      - ${HOME}/.unc/localnet/node3:$HOME/.unc
    ports:
      - 3033:3030
      - 12348:12345
  unc-node4:
    image: ghcr.io/utnet-org/utility:latest
    environment:
      - UNC_HOME=$HOME/.unc
      - CHAIN_ID=localnet
      - INIT=true
    volumes:
      - ${HOME}/.unc/localnet/node4:$HOME/.unc
    ports:
      - 3034:3030
      - 12349:12345
  unc-node5:
    image: ghcr.io/utnet-org/utility:latest
    environment:
      - UNC_HOME=$HOME/.unc
      - CHAIN_ID=localnet
      - INIT=true
    volumes:
      - ${HOME}/.unc/localnet/node5:$HOME/.unc
    ports:
      - 3035:3030
      - 12350:12345
EOF

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null
then
    echo "docker-compose could not be found"
    exit
fi

# Run the Docker containers
sudo docker-compose up -d

export MASTER_ACCOUNT_ID=node0
export CHAIN_ID=localnet

stop_nodes() {
  echo "STOOOP THE NODES!"
  # Stop the Docker containers
  sudo docker-compose down

  # Remove Docker networks and volumes
  sudo docker network prune -f
  sudo docker volume prune -f
}

trap "stop_nodes" ERR

LAST_NODE=4
NODES_TO_VOTE=3

echo "Awaiting for network to start"
sleep 3

echo "Current validator should be the $LAST_NODE + 1 nodes"
unc pledging validators network-config $CHAIN_ID now

for (( i=0; i<=$LAST_NODE; i++ )); do
  cp ~/.unc/localnet/node$i/node_key.json ~/.unc-credentials/local/node$i.json
done;

OWNER_ACCOUNT_ID="8613616183c9f381fcfa79852235f818d058958bc03ec0b8ca3bc9ca289392b4"
# Verifying owner account exist
AMOUNT=$(unc account view-account-summary $OWNER_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for master account ${OWNER_ACCOUNT_ID}. Maybe the account doesn't exist."
cat << EOF
#1. create account and transfer funds if account not exist on-chain
### if you want to use a different account, follow the steps below
### or use the default account, it has some tokens, execute the command step 3 #3 to import the account
### account stake-pool-factory:
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/8613616183c9f381fcfa79852235f818d058958bc03ec0b8ca3bc9ca289392b4.json

## { "account_id":"8613616183c9f381fcfa79852235f818d058958bc03ec0b8ca3bc9ca289392b4",
## "master_seed_phrase":"praise vacant sample tent casual devote sniff barely ship daughter vapor angry",
## "private_key":"ed25519:5sxzpVrPJxj5BiphCUd41PAv8gdMWsPVhAgVcuezLwEHFUnG1f8ix6g5xhDLKKV2HfDoE4KQxbWxKw2rLyhGeswy",
## "public_key":"ed25519:A2Nmkj76YzeBD2WcmowJvV6cuqgGNxPo2JPDLdgndcR9",
## "seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:2wJFRRVYadDwQT3svS81vCGdFqgX8ZMeLuNPqUejg5wNKWgQ9Crh5uhmGMRvB3NkBjGZ73Bnr5L694nkZ8qB8NWz network-config $CHAIN_ID
## > Enter account ID: 8613616183c9f381fcfa79852235f818d058958bc03ec0b8ca3bc9ca289392b4

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc 8613616183c9f381fcfa79852235f818d058958bc03ec0b8ca3bc9ca289392b4 '100 unc' network-config $CHAIN_ID sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=8613616183c9f381fcfa79852235f818d058958bc03ec0b8ca3bc9ca289392b4
## 6. wait for the account to be on-chain, 6 blocks time
EOF
  exit 1
fi

echo "Deploying core accounts/"
(cd .. && ./deploy_core.sh)

for (( i=1; i<=$LAST_NODE; i++ )); do
  ACCOUNT_ID="node${i}"
  unc validator pledging pledge-proposal $ACCOUNT_ID ed25519:7PGseFbWxvYVgZ89K1uTJKYoKetWs7BJtbyXDzfbAcqX '0 unc' network-config $CHAIN_ID sign-with-keychain send
done;

NODE0_PUBLIC_KEY=$(grep -oE 'ed25519:[^"]+' ~/.unc/localnet/node0/validator_key.json | head -1)
echo "Staking close to 1B UNC by node0, to avoid it being kicked out too fast."
unc validator pledging pledge-proposal node0 "$NODE0_PUBLIC_KEY" '999000000 unc' network-config $CHAIN_ID sign-with-keychain send

echo "Sleeping 3+ minutes (for 3+ epochs)"
sleep 200

echo "The only current validator should be the node0"
unc pledging validators network-config $CHAIN_ID now

for (( i=1; i<=$LAST_NODE; i++ )); do
  ACCOUNT_ID="node${i}"
  unc deploy --wasmFile="../../res/staking_pool.wasm" --accountId=$ACCOUNT_ID
  PUBLIC_KEY=$(grep -oE 'ed25519:[^"]+' ~/.unc/localnet/node$i/validator_key.json | head -1)
  unc call $ACCOUNT_ID new "{\"owner_id\": \"$OWNER_ACCOUNT_ID\", \"stake_public_key\": \"$PUBLIC_KEY\", \"reward_fee_fraction\": {\"numerator\": 10, \"denominator\": 100}}" --accountId=$OWNER_ACCOUNT_ID
  sleep 1
done;

echo "Deployed pools and staked a lot. Sleep for 1 minute."
sleep 70

echo "Going to ping pools in case the stake was lost due to seat assignment"

for (( i=1; i<=$LAST_NODE; i++ )); do
  ACCOUNT_ID="node${i}"
  unc call $ACCOUNT_ID ping "{}" --accountId=$OWNER_ACCOUNT_ID
  sleep 1
done;

echo "Unstaking for node0"
unc validator pledging pledge-proposal node0 "$NODE0_PUBLIC_KEY" '0 unc' network-config $CHAIN_ID sign-with-keychain send

echo "Sleeping 3+ minutes (for 3+ epochs)"
sleep 200

echo "Current validators should be the $LAST_NODE nodes with the staking pools only"
unc pledging validators network-config $CHAIN_ID now
unc pledging validators network-config $CHAIN_ID now | grep "Validators (total: $LAST_NODE,"

VOTE_ACCOUNT_ID="0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee"
AMOUNT=$(unc account view-account-summary $VOTE_ACCOUNT_ID network-config $CHAIN_ID now | grep "balance")
if [ -z "$AMOUNT" ]; then
  echo "Can't get state for master account ${VOTE_ACCOUNT_ID}. Maybe the account doesn't exist."
  cat << EOF
#1. create account and transfer funds if account not exist on-chain
### if you want to use a different account, follow the steps below
### or use the default account, it has some tokens, execute the command step 3 #3 to import the account
## account voting:
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee.json

##{"account_id":"0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee",
## "master_seed_phrase":"crunch coach section hospital disagree denial hospital suspect view cycle brand please",
## "private_key":"ed25519:v9zXRShtYhyEDEjBeNDjU4fnjghiCwSVm4qwA5kBA17fXT4y66S7YvYjYEdYaRiT8xnvPEErEgegeTpYxPaiZ5F",
## "public_key":"ed25519:mz4koCMGRmbEDW6GCgferaVP5Upq9tozgaz3gnXZSp5","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:v9zXRShtYhyEDEjBeNDjU4fnjghiCwSVm4qwA5kBA17fXT4y66S7YvYjYEdYaRiT8xnvPEErEgegeTpYxPaiZ5F network-config $CHAIN_ID
## > Enter account ID: 0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc 0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee '100 unc' network-config $CHAIN_ID sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee

## 6. wait for the account to be on-chain, 6 blocks time
EOF
  exit 1
fi

check_votes() {
  echo "Checking votes"
  unc contract call-function as-read-only $VOTE_ACCOUNT_ID get_total_voted_stake text-args '' network-config $CHAIN_ID now
  unc contract call-function as-read-only $VOTE_ACCOUNT_ID get_votes text-args '' network-config $CHAIN_ID now
  echo "Checking result"
  unc contract call-function as-read-only $VOTE_ACCOUNT_ID get_result text-args '' network-config $CHAIN_ID now

}

vote() {
  ACCOUNT_ID="node${1}"
  echo "Voting through the pool to node $ACCOUNT_ID"
  unc call $ACCOUNT_ID vote "{\"voting_account_id\": \"$VOTE_ACCOUNT_ID\", \"is_vote\": true}" --accountId=$OWNER_ACCOUNT_ID --gas=200000000000000

  check_votes
}

vote 1
vote 2

echo "Going to kick out node1. And restake with node0"
unc call node1 pause_staking --accountId=$OWNER_ACCOUNT_ID
sleep 1
unc validator pledging pledge-proposal node0 "$NODE0_PUBLIC_KEY" 999000000

echo "Sleeping 3+ minutes (for 3+ epochs)"
sleep 200

echo "Current validators should be the 3 nodes with the staking pools and node0"
unc pledging validators network-config $CHAIN_ID now
unc pledging validators network-config $CHAIN_ID now | grep "Validators (total: 4,"

check_votes

vote 3
vote 4

stop_nodes
