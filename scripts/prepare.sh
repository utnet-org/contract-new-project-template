#!/bin/bash
MASTER_ACCOUNT_ID="${MASTER_ACCOUNT_ID:-7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376}"
set -e

### Requirements:  create account, import account, transfer funds, export CONTRACT_ACCOUNT_ID
## whitelist
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773.json

## {"account_id":"e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773",
## "master_seed_phrase":"degree crisp fat noodle clog word globe filter problem rice swear priority",
## "private_key":"ed25519:FfnmBekG6NYcrqsgk3JNoLr9qbc6T3cqY6YjoYFYKGHuELswSyjxRZzZAoDc4rweuByHpqCQDrQnLV1Excm2W2W",
## "public_key":"ed25519:GDHGgfGte8prwJ4dEJcCB8SZhKWf7RWSXxTHg4fzY62W","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:FfnmBekG6NYcrqsgk3JNoLr9qbc6T3cqY6YjoYFYKGHuELswSyjxRZzZAoDc4rweuByHpqCQDrQnLV1Excm2W2W network-config testnet
## > Enter account ID: e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773 '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=e204abad77845ac1d756d580480a463d3a5efd7bb039a12293ca15ebb1878773


## voting
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee.json

##{"account_id":"0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee",
## "master_seed_phrase":"crunch coach section hospital disagree denial hospital suspect view cycle brand please",
## "private_key":"ed25519:v9zXRShtYhyEDEjBeNDjU4fnjghiCwSVm4qwA5kBA17fXT4y66S7YvYjYEdYaRiT8xnvPEErEgegeTpYxPaiZ5F",
## "public_key":"ed25519:mz4koCMGRmbEDW6GCgferaVP5Upq9tozgaz3gnXZSp5","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:v9zXRShtYhyEDEjBeNDjU4fnjghiCwSVm4qwA5kBA17fXT4y66S7YvYjYEdYaRiT8xnvPEErEgegeTpYxPaiZ5F network-config testnet
## > Enter account ID: 0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc 0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=0b861433767ace72eeace6cd636feec7e44c82ff4e25d048e09d0460f748acee



## stake-pool-factory
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737.json

## {"account_id":"81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737",
## "master_seed_phrase":"luggage into fall pill wine repeat undo salon index plate until matter",
## "private_key":"ed25519:2wJFRRVYadDwQT3svS81vCGdFqgX8ZMeLuNPqUejg5wNKWgQ9Crh5uhmGMRvB3NkBjGZ73Bnr5L694nkZ8qB8NWz",
## "public_key":"ed25519:9jYETemz2TFrXfmy72kRqpgWkCjiZn1BBRcYfY8ZMyPU","seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:2wJFRRVYadDwQT3svS81vCGdFqgX8ZMeLuNPqUejg5wNKWgQ9Crh5uhmGMRvB3NkBjGZ73Bnr5L694nkZ8qB8NWz network-config testnet
## > Enter account ID: 81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc 81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737 '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=81c3341ed21f7f39f9507a5953c81da6a1db46fee08e3a9d508ce7adc2e87737


## lockup
##1.$ unc account create-account fund-later use-auto-generation save-to-folder $HOME/.unc-credentials/implicit
##2.$ cat $HOME/.unc-credentials/implicit/ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846.json

## {"account_id":"ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846",
## "master_seed_phrase":"glimpse card pride element local monkey company puppy stock fashion giraffe salt",
## "private_key":"ed25519:4mwryZ4GXVJieS9ccaGBhH3BnyNV9ovNonWrrxr9nfpu9wybPYUQsY1DdxQfVZ6ZJdvj61WYgsLoWMVPxSqYv2ms",
## "public_key":"ed25519:H6Gx6FSEFJVPGXQkhX6kcPpB9NyXM2SNp8weFgb1A9fb",
## "seed_phrase_hd_path":"m/44'/397'/0'"
## }

## 3.$ unc account import-account using-private-key ed25519:4mwryZ4GXVJieS9ccaGBhH3BnyNV9ovNonWrrxr9nfpu9wybPYUQsY1DdxQfVZ6ZJdvj61WYgsLoWMVPxSqYv2ms network-config testnet
## > Enter account ID: ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846

## 4.$ unc tokens 7a17c8371a5a511fc92bc61e2b4c068e7546a3cd5d6c0bbdef1b8132c8b30376 send-unc ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846 '100 unc' network-config testnet sign-with-keychain send

## 5.$ export CONTRACT_ACCOUNT_ID=ef14eded70222383b8aed8a999879e06f28d86557b087db6d98d5d37ee198846