#!/bin/bash
set -e

# start 5 unc-node instances
```yaml
version: "3.8"
services:
  unc-node1:
    image: ghcr.io/utnet-org/utility:latest
    environment:
      - UNC_HOME=$HOME/.unc
      - CHAIN_ID=local
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
      - CHAIN_ID=local
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
      - CHAIN_ID=local
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
      - CHAIN_ID=local
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
      - CHAIN_ID=local
      - INIT=true
    volumes:
      - ${HOME}/.unc/localnet/node5:$HOME/.unc
    ports:
      - 3035:3030
      - 12350:12345
```

export MASTER_ACCOUNT_ID=node0
export CHAIN_ID=local

stop_nodes() {
  echo "STOOOP THE NODES!"
  uncup stop
}

trap "stop_nodes" ERR

LAST_NODE=4
NODES_TO_VOTE=3

echo "Awaiting for network to start"
sleep 3

echo "Current validator should be the $LAST_NODE + 1 nodes"
unc validators current

for (( i=0; i<=$LAST_NODE; i++ )); do
  cp ~/.unc/localnet/node$i/node_key.json ~/.unc-credentials/local/node$i.json
done;

OWNER_ACCOUNT_ID="owner.$MASTER_ACCOUNT_ID"
unc create-account $OWNER_ACCOUNT_ID --masterAccount=$MASTER_ACCOUNT_ID --initialBalance=10000

echo "Deploying core accounts/"
(cd .. && ./deploy_core.sh)

for (( i=1; i<=$LAST_NODE; i++ )); do
  ACCOUNT_ID="node${i}"
  unc stake $ACCOUNT_ID "ed25519:7PGseFbWxvYVgZ89K1uTJKYoKetWs7BJtbyXDzfbAcqX" 0
done;

NODE0_PUBLIC_KEY=$(grep -oE 'ed25519:[^"]+' ~/.unc/localnet/node0/validator_key.json | head -1)
echo "Staking close to 1B UNC by node0, to avoid it being kicked out too fast."
unc stake node0 "$NODE0_PUBLIC_KEY" 999000000

echo "Sleeping 3+ minutes (for 3+ epochs)"
sleep 200

echo "The only current validator should be the node0"
unc validators current

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
unc stake node0 "$NODE0_PUBLIC_KEY" 0

echo "Sleeping 3+ minutes (for 3+ epochs)"
sleep 200

echo "Current validators should be the $LAST_NODE nodes with the staking pools only"
unc validators current
unc validators current | grep "Validators (total: $LAST_NODE,"

VOTE_ACCOUNT_ID="vote.$MASTER_ACCOUNT_ID"

check_votes() {
  echo "Checking votes"
  unc view $VOTE_ACCOUNT_ID get_total_voted_stake
  unc view $VOTE_ACCOUNT_ID get_votes
  echo "Checking result"
  unc view $VOTE_ACCOUNT_ID get_result

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
unc stake node0 "$NODE0_PUBLIC_KEY" 999000000

echo "Sleeping 3+ minutes (for 3+ epochs)"
sleep 200

echo "Current validators should be the 3 nodes with the staking pools and node0"
unc validators current
unc validators current | grep "Validators (total: 4,"

check_votes

vote 3
vote 4

stop_nodes
