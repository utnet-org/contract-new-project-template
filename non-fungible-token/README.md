Non-fungible Token (NFT)
===================

Example implementation of a [non-fungible token] contract which uses [unc-contract-standards].

  [non-fungible token]: https://xxx.io/Standards/NonFungibleToken/README.html
  [unc-contract-standards]: https://github.com/utnet-org/utility-sdk-rs/tree/master/unc-contract-standards

NOTES:

- The maximum balance value is limited by U128 (2**128 - 1).
- JSON calls should pass [U128] or [U64] as a base-10 string. E.g. "100".
- The core NFT standard does not include escrow/approval functionality, as `nft_transfer_call` provides a superior approach. An escrow system can, of course, be added as a separate contract or additional functionality within this contract.

## Building

To build run:

```bash
./build.sh
```

## Testing

To test run:

```bash
cargo test --workspace --package non-fungible-token -- --nocapture
```

## Deploy

This smart contract will get deployed to your Utility Net account. For this example, please create a new Utility Net account. Because Utility Net allows the ability to upgrade contracts on the same account, initialization functions must be cleared. If you'd like to run this example on a Utility Net account that has had prior contracts deployed, please use the `unc-cli` command `unc delete`, and then recreate it in Wallet. To create (or recreate) an account, please follow the directions in [Utility Net Wallet](https://chromewebstore.google.com/detail/mywallets-v1/poljcmobchfooceghefdokchdkfmlcbk) or [Cli wallet](https://github.com/utnet-org/utility-cli-rs/releases/tag/v0.15.0).

In the project root, create account and import account (a.k.a log in) to your newly created account with `unc-cli` by following the instructions after this command.

  unc account import-account using-private-key ed25519:5DwmxYXw3Dy6fwF7Ty9eiDyGGui72xY1bFT7KyzxhckNZ988JA2wPEMZxCKsVUvSmwHm2fmNUra7QozwhHE73HMw network-config testnet

To make this tutorial easier to copy/paste, we're going to set an environment variable for our account id. In the below command, replace `MY_ACCOUNT_NAME` with the account name we just logged in with, including the `.testnet`:

  export NFT_ID=ce8f91bc4fe16c1ecae01276a19a8b582135629845ac32e7bf980324d5318ace

We can tell if the environment variable is set correctly if our command line prints the account name after this command:

  echo $NFT_ID

Now we can deploy the compiled contract in this example to your account:

  unc contract deploy $NFT_ID use-file res/non_fungible_token.wasm without-init-call network-config testnet sign-with-legacy-keychain send

NFT contract should be initialized before usage. More info about the metadata at [xxx.io](https://xxx.io/Standards/NonFungibleToken/Metadata.html). But for now, we'll initialize with the default metadata.

  unc contract call-function as-transaction $NFT_ID new_default_meta json-args '{"owner_id": "'$NFT_ID'"}' prepaid-gas '30 TeraGas' attached-deposit '0 unc' sign-as $NFT_ID network-config testnet sign-with-keychain send

We'll be able to view our metadata right after:

  unc contract call-function as-read-only $NFT_ID nft_metadata text-args '' network-config testnet now

Then, let's mint our first token. This will create a NFT based on Olympus Mons where only one copy exists:

  unc contract call-function as-transaction $NFT_ID nft_mint json-args '{"token_id": "0", "token_owner_id": "'$NFT_ID'", "token_metadata": { "title": "Olympus Mons", "description": "Tallest mountain in charted solar system", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 1}}' prepaid-gas '300 TeraGas' attached-deposit '0.1 unc' sign-as $NFT_ID network-config testnet sign-with-keychain send

  unc tokens $NFT_ID view-nft-assets $NFT_ID network-config testnet now

## Transferring our NFT

Let's set up an account to transfer our freshly minted token to. This account will be a account of the Utility account we logged in with originally via `unc account import`.

```sh
  # create account and import account in testnet
unc account create-account fund-later use-auto-generation save-to-folder /home/ubuntu/.unc-credentials/implicit
#such as follows: 
# cat /home/ubuntu/.unc-credentials/implicit/b44f5b891131042322601250fd9dd951f9b5ee34d828c8fb30bfb2e856a101c4.json
# {"account_id":"b44f5b891131042322601250fd9dd951f9b5ee34d828c8fb30bfb2e856a101c4","master_seed_phrase":"chimney digital indoor wealth ridge item puzzle slice will cabin panel wine","private_key":"ed25519:4Crf6QpcR6UrakrxEpNjEQ5bF84oCUrW8tDAE5PLyevqJKv4xs1SjARVyLYkV2MXjcsjiAtYkFsLsTgUrUsxfhUf","public_key":"ed25519:D8ra3tox4a5dx6PxQxWJ7xGmASRxzqiWfrhwuNyjC2Td","seed_phrase_hd_path":"m/44'/397'/0'"}

# seed phrase use above terminal output, do not directly use here
unc account import-account using-seed-phrase 'chimney digital indoor wealth ridge item puzzle slice will cabin panel wine' --seed-phrase-hd-path 'm/44'\''/397'\''/0'\''' network-config testnet
# faucet fund account 039b331bede0513d50e3edd083800fc9057da0a519aaf5a26efa2ef2e3c236a4
# Processing transaction...
# Please wait for 6 blocks to confirm, use command: unc transaction view-status <tx_hash>
curl -X POST -H "Content-Type: application/json" -d '{"amount":"10000000000000000000000000", "receiverId":"b44f5b891131042322601250fd9dd951f9b5ee34d828c8fb30bfb2e856a101c4", "contractId":"4e0375672ec30f2efe3a6c5a14ff81d37f1271c439501eac2fb445df262b2c32"}' https://unc-faucet.xyz666.org/api/faucet/tokens

export BOB_ACCOUNT=b44f5b891131042322601250fd9dd951f9b5ee34d828c8fb30bfb2e856a101c4
```

Checking Bob's account for tokens:

  unc contract call-function as-read-only $NFT_ID nft_tokens_for_owner text-args '{"account_id": "'$BOB_ACCOUNT'"}' network-config testnet now

Then we'll transfer over the NFT into Bob's account. Exactly 1 attounc of deposit should be attached:

  unc contract call-function as-transaction $NFT_ID nft_transfer json-args '{"token_id": "0", "receiver_id": "'$BOB_ACCOUNT'", "memo": "transfer ownership"}' prepaid-gas '30 TeraGas' attached-deposit '1 attounc' sign-as $NFT_ID network-config testnet sign-with-keychain send

Checking Bob's account again shows us that she has the Olympus Mons token.

  unc tokens $BOB_ACCOUNT view-nft-assets $NFT_ID network-config testnet now
