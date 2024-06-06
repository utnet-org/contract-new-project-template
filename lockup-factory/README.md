# Lockup Factory Contract

This contract deploys lockup contracts.
It allows any user to create and fund the lockup contract.
The lockup factory contract packages the binary of the
<a href="https://github.com/utnet-org/contract-new-project-template/tree/master/lockup">lockup
contract</a> within its own binary.

To create a new lockup contract a user should issue a transaction and
attach the required minimum deposit. The entire deposit will be transferred to
the newly created lockup contract including to cover the storage.

The benefits:

1. Lockups can be funded from any account.
2. No need to have access to the foundation keys to create lockup.
3. Auto-generates the lockup from the owner account.
4. Refund deposit on errors.

## Deployment & Usage

## TestNet

## Initialize the factory

unc dev-tool deploy ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846 with-init-call new json-args '{"whitelist_account_id": "e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773", "foundation_account_id": "unc"}' prepaid-gas '100.0 Tgas' attached-deposit '0 unc' network-config testnet  sign-with-keychain send

## Create a new lockup with the given parameters

contract call-function as-transaction <LOCKUP_ACCOUNT_ID> create json-args '{"owner_account_id":"<ONWER_ACCOUNT_ID>","lockup_duration":"63036000000000000"}' --accountId funding_account.testnet --amount 50000

## Create a new lockup with the vesting schedule

contract call-function as-transaction <LOCKUP_ACCOUNT_ID> create json-args '{"owner_account_id":"<ONWER_ACCOUNT_ID>","lockup_duration":"31536000000000000","vesting_schedule": { "VestingSchedule": {"start_timestamp": "1535760000000000000", "cliff_timestamp": "1567296000000000000", "end_timestamp": "1661990400000000000"}}}' --accountId funding_account.testnet --amount 50000 --gas 110000000000000
