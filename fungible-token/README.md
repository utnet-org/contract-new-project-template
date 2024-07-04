Fungible Token (FT)
===================

Example implementation of a [Fungible Token] contract which uses [unc-contract-standards].

  [Fungible Token]: https://xxx.io/Standards/Tokens/FungibleTokenCore.html
  [unc-contract-standards]: https://github.com/utnet-org/utility-sdk-rs/tree/master/unc-contract-standards

NOTES:

- The maximum balance value is limited by U128 (2**128 - 1).
- JSON calls should pass U128 as a base-10 string. E.g. "100".
- This does not include escrow functionality, as `ft_transfer_call` provides a superior approach. An escrow system can, of course, be added as a separate contract.

## Building

To build run:

```bash
./build.sh
```

## Testing

To test run:

```bash
cargo test --package fungible-token -- --nocapture
```

## Deploy

This smart contract will get deployed to your Utility Net account. For this example, please create a new Utility Net account. Because Utility Net allows the ability to upgrade contracts on the same account, initialization functions must be cleared. If you'd like to run this example on a Utility Net account that has had prior contracts deployed, please use the `unc-cli` command `unc delete`, and then recreate it in Wallet. To create (or recreate) an account, please follow the directions in [Utility Net Wallet](https://chromewebstore.google.com/detail/mywallets-v1/poljcmobchfooceghefdokchdkfmlcbk) or [Cli wallet](https://github.com/utnet-org/utility-cli-rs/releases/tag/v0.15.0).

```sh
# create account and import account in testnet
unc account create-account fund-later use-auto-generation save-to-folder /home/ubuntu/.unc-credentials/implicit
#such as follows: 
# cat /home/ubuntu/.unc-credentials/implicit/039b331bede0513d50e3edd083800fc9057da0a519aaf5a26efa2ef2e3c236a4.json
# {"account_id":"039b331bede0513d50e3edd083800fc9057da0a519aaf5a26efa2ef2e3c236a4","master_seed_phrase":"west business coin live small thing lounge own orient artwork cousin rubber","private_key":"ed25519:2QcdBynQVfjGXujiq2jGwobwUkqx9933d8UYnZwk7WTAZkjV5wAkwVGi2j6rAYwrmKuCmTNKjWS4xw6pDMDmvLLb","public_key":"ed25519:F5Uyx1P13aKwXRSdfxTjNe13H2ZgPryJGESF6CV7arP","seed_phrase_hd_path":"m/44'/397'/0'"}

# seed phrase use above terminal output, do not directly use here
unc account import-account using-seed-phrase 'west business coin live small thing lounge own orient artwork cousin rubber' --seed-phrase-hd-path 'm/44'\''/397'\''/0'\''' network-config testnet

# faucet fund account 039b331bede0513d50e3edd083800fc9057da0a519aaf5a26efa2ef2e3c236a4
# Processing transaction...
# Please wait for 6 blocks to confirm, use command: unc transaction view-status <tx_hash>
curl -X POST -H "Content-Type: application/json" -d '{"amount":"10000000000000000000000000", "receiverId":"93c6113b5ec60eb0cf9fc8556397bf9dd56ae1792df61ae3520193b07898fbf0", "contractId":"4e0375672ec30f2efe3a6c5a14ff81d37f1271c439501eac2fb445df262b2c32"}' https://unc-faucet.xyz666.org/api/faucet/tokens

# set environment variable `contract account`
export CONTRACT_ID=039b331bede0513d50e3edd083800fc9057da0a519aaf5a26efa2ef2e3c236a4
# deploy contract, Processing transaction...
# Please wait for 6 blocks to confirm, use command: unc transaction view-status <tx_hash>
unc contract deploy $CONTRACT_ID use-file res/fungible_token.wasm without-init-call network-config testnet sign-with-legacy-keychain send
```

FT contract should be initialized before usage.
The next command will initialize the contract using the `new` method:

```bash
unc contract call-function as-transaction $CONTRACT_ID new json-args '{"owner_id": "'$CONTRACT_ID'", "total_supply": "1000000000000000", "metadata": { "spec": "ft-1.0.0", "name": "Example Token Name", "symbol": "AI", "decimals": 8 }}' prepaid-gas '300 TeraGas' attached-deposit '0 unc' sign-as $CONTRACT_ID network-config testnet sign-with-legacy-keychain send
```

To get the fungible token metadata:

```bash
unc contract call-function as-read-only $CONTRACT_ID ft_metadata text-args '' network-config testnet now
```

## Transfer

Let's set up an account to transfer some tokens to. These account will be a sub-account of the Utility account you logged in with.

```sh
# create another account and import account in testnet
unc account create-account fund-later use-auto-generation save-to-folder /home/ubuntu/.unc-credentials/implicit
#such as follows: 
# cat /home/ubuntu/.unc-credentials/implicit/603fa784a93d77f0064550a39063d578c603d64b4d08f132bf12c5fd4fef4a5b.json
# {"account_id":"603fa784a93d77f0064550a39063d578c603d64b4d08f132bf12c5fd4fef4a5b","master_seed_phrase":"scheme endless person public tiger uncover inside quantum naive unit organ harbor","private_key":"ed25519:EAuK9PALxv3kwe24PvA68unp5EU8dLTuYPreXeq1gnVjgcya4B2dd8qPRkn4KM9qhCY7zL5QQXoCRJZMe6HvsWS","public_key":"ed25519:7UiSF7ktiRgR2tuHMwGrCBdmyDDFdFbs5xnyxsLmnNHY","seed_phrase_hd_path":"m/44'/397'/0'"}

# seed phrase use above terminal output, do not directly use here
unc account import-account using-seed-phrase 'scheme endless person public tiger uncover inside quantum naive unit organ harbor' --seed-phrase-hd-path 'm/44'\''/397'\''/0'\''' network-config testnet

# faucet fund account 603fa784a93d77f0064550a39063d578c603d64b4d08f132bf12c5fd4fef4a5b `10 unc`
# Processing transaction...
# Please wait for 6 blocks to confirm, use command: unc transaction view-status <tx_hash>
curl -X POST -H "Content-Type: application/json" -d '{"amount":"10000000000000000000000000", "receiverId":"603fa784a93d77f0064550a39063d578c603d64b4d08f132bf12c5fd4fef4a5b", "contractId":"4e0375672ec30f2efe3a6c5a14ff81d37f1271c439501eac2fb445df262b2c32"}' https://unc-faucet.xyz666.org/api/faucet/tokens

# set environment variable `contract account`
export BOB_ACCOUNT=603fa784a93d77f0064550a39063d578c603d64b4d08f132bf12c5fd4fef4a5b
```

Add storage deposit for Bob's account (optional):

```sh
unc contract call-function as-transaction $CONTRACT_ID storage_deposit json-args '{}' prepaid-gas '300 TeraGas' attached-deposit '1 unc' sign-as $BOB_ACCOUNT network-config testnet sign-with-legacy-keychain send
```

Check balance of Bob's account, it should be `0` for now:

```sh
unc contract call-function as-read-only $CONTRACT_ID ft_balance_of text-args '{"account_id": "'$BOB_ACCOUNT'"}' network-config testnet now
```

Transfer tokens to Bob from the contract that minted these fungible tokens, exactly 1 attoUNC of deposit should be attached:

```sh
unc contract call-function as-transaction $CONTRACT_ID ft_transfer json-args '{"receiver_id": "'$BOB_ACCOUNT'", "amount": "19"}' prepaid-gas '300 TeraGas' attached-deposit '1 attounc' sign-as $CONTRACT_ID network-config testnet sign-with-legacy-keychain send

```

Check the balance of Bob again with the command from before and it will now return `19`.

## Changelog

### `0.2.0`

- Switched form using [UIP-1](https://github.com/utnet-org/UIPs/pull/1) to [UIP-1](https://github.com/utnet-org/UIPs/issues/141).

### `0.1.0`

#### Breaking storage change

- Switching `UnorderedMap` to `LookupMap`. It makes it cheaper and faster due to decreased storage access.
