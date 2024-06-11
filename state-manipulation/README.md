# State Manipulation contract

This contract has been designed to put key value pairs into storage with `replace` and clear key/value pairs with `clean` from an account's storage.

Deploy this contract into the account that already has another contract deployed to it.
This contract on call `clean` will remove any items of the state specified (keys should be specified in base64). When compiled with `replace` feature, `replace` method can be called with an array of key/value tuple pairs to insert into state.

## Parameters

JSON format for `clean`:

```json
{"keys":["<base64 encoded key byte string>", "...", "..."]}
```

JSON format for `replace`:

```json
{"entries":[["<base64 key byte string>", "<base64 value byte string>"], ["...", "..."]]}
```

## With CLI

Usage example to put and remove only the "STATE" item using [unc-cli](https://github.com/utnet-org/utility-cli-rs):

```bash
# Build the contracts will all feature combinations
./build.sh

# Deploy built code on chain
unc contract deploy <CONTRACT_ID> use-file ../res/state_manipulation.wasm without-init-call network-config testnet sign-with-keychain send
# Add state item for "STATE" key
unc contract call-function as-transaction <CONTRACT_ID> replace json-args '{"entries":[["U1RBVEU=", "dGVzdA=="]]}' prepaid-gas '100.0 Tgas' attached-deposit '0 unc' sign-as <SIGNER_ID> network-config testnet sign-with-keychain send

# View Added state item
unc contract call-function as-read-only contract-state contract-state text-args '' network-config testnet now

# Clear added state item
unc contract call-function as-transaction <CONTRACT_ID> clean json-args '{"keys":["U1RBVEU="]}' prepaid-gas '100.0 Tgas' attached-deposit '0 unc' sign-as <SIGNER_ID> network-config testnet sign-with-keychain send

# View that item was removed
unc contract call-function as-read-only contract-state contract-state text-args '' network-config testnet now
```

## Features

`clean`: Enables `clean` method to remove keys
`replace`: Enables `replace` method to add key/value pairs to storage
