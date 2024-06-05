# Setup scripts

## Rebuild all contracts

```bash
./build_all_docker.sh
```

## Deploy core contracts using master account

### Set master account

```bash
export MASTER_ACCOUNT_ID=7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376
```

### Set network environment

```bash
export CHAIN_ID=testnet
```

### Deploy

This will deploy the following contracts:

- Voting contract at `VOTING_ACCOUNT_ID` with `100` UNC tokens
- Whitelist contract at `WHITELIST_ACCOUNT_ID` with `100` UNC tokens
- Staking pool factory contract at `POOL_ACCOUNT_ID` with `100` UNC tokens

It will whitelist the staking pool factory account.

It requires total `80` UNC tokens + gas fees.

```bash
./deploy_core.sh
```

## Deploying lockup contract

NOTE: This flow is mostly for testnet and is not recommended for production use.

### Set lockup root account

This account will be used as a suffix to deploy lockup contracts.
Also this account will fund the newly created lockup contracts.

```bash
export LOCKUP_ACCOUNT_ID=lockup
```

### Deploying

To deploy a lockup call the script. It has interactive interface to provide details.

```bash
./deploy_lockup.sh
```

Once the amount (in UNC) is provided, the lockup contract will be deployed.

## Notes

For rebuilding contracts, make sure you have `rust` with `wasm32` target installed.

For deploying, you need to have [`unc-cli`](https://github.com/utnet-org/utility-cli-rs) installed and be logged in with the master account ID.
