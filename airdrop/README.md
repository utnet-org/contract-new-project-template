# AirDrop contract

AirDrop contract allows any user to create a link that their friends can use to claim tokens even if they don't have an account yet.

The way it works:

Sender, that has UNC Tokens:

- Creates a new key pair `(pk1, privkey1)`.
- Calls `linkdrop.send(pk1)` with attached balance of UNC Tokens that they want to send.
- Sends a link to any supported wallet app with `privkey1` as part of URL.

Receiver, that doesn't have UNC Tokens:

- Receives link to the wallet with `privkey1`.
- Wallet creates new key pair for this user (or they generate it via HSM) `(pk2, privkey2)`.
- Enters the `new_account_id` receiver want for their new account.
- Wallet creates a transaction to `linkdrop.create_account_and_claim(new_account_id, pk2)`.
- Contract creates new account with `new_account_id` name and `pk2` as full access key and transfers UNC Tokens that Sender sent.

If Receiver already has account (or Sender wants to get back the money):

- Sign tx with `privkey1` to call `linkdrop.claim()`, which transfers money to signer's account.
