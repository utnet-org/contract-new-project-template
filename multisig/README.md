# Multisig contract

*Please do your own due diligence before using this contract. There is no guarantee that this contract doesn't have issues.*

This contract provides:

- Set K out of N multi sig scheme
- Request to sign transfers, function calls, adding and removing keys.
- Any of the access keys or set of specified accounts can confirm, until the required number of confirmation achieved.

## Multisig implementation details

Multisig uses set of `FunctionCall` `AccessKey`s and account ids as a set of allowed N members.
When contract is being setup, it should be initialized with set of members that will be initially managing this account.
All operations going forward will require `K` members to call `confirm` to be executed.

### Initialization

### Request

There are number of different request types that multisig can confirm and execute:

```rs
/// Lowest level action that can be performed by the multisig contract.
pub enum MultiSigRequestAction {
    /// Transfers given amount to receiver.
    Transfer {
        amount: U128,
    },
    /// Create a new account.
    CreateAccount,
    /// Deploys contract to receiver's account. Can upgrade given contract as well.
    DeployContract { code: Base64VecU8 },
    /// Adds new member to multisig, either public key or account.
    AddMember {
        member: MultisigMember,
    },
    /// Delete existing member from multisig, either public key or account.
    DeleteMember {
        member: MultisigMember,
    },
    /// Adds key, either new key for multisig or full access key to another account.
    AddKey {
        public_key: Base58PublicKey,
        #[serde(skip_serializing_if = "Option::is_none")]
        permission: Option<FunctionCallPermission>,
    },
    /// Call function on behalf of this contract.
    FunctionCall {
        method_name: String,
        args: Base64VecU8,
        deposit: U128,
        gas: U64,
    },
    /// Sets number of confirmations required to authorize requests.
    /// Can not be bundled with any other actions or transactions.
    SetNumConfirmations {
        num_confirmations: u32,
    },
    /// Sets number of active requests (unconfirmed requests) per access key
    /// Default is 12 unconfirmed requests at a time
    /// The REQUEST_COOLDOWN for requests is 15min
    /// Worst gas attack a malicious keyholder could do is 12 requests every 15min
    SetActiveRequestsLimit {
        active_requests_limit: u32,
    },
}

/// Permission for an access key, scoped to receiving account and method names with allowance to add when key is added to accoount
pub struct FunctionCallPermission {
    allowance: Option<U128>,
    receiver_id: AccountId,
    method_names: Vec<String>,
}

/// The request the user makes specifying the receiving account and actions they want to execute (1 tx)
pub struct MultiSigRequest {
    receiver_id: AccountId,
    actions: Vec<MultiSigRequestAction>,
}

/// An internal request wrapped with the signer_pk and added timestamp to determine num_requests_pk and prevent against malicious key holder gas attacks
pub struct MultiSigRequestWithSigner {
    request: MultiSigRequest,
    member: MultisigMember,
    added_timestamp: u64,
}

/// Represents member of the multsig: either account or access key to given account.
pub enum MultisigMember {
    AccessKey { public_key: Base58PublicKey },
    Account { account_id: AccountId },
}
```

### Methods

```rust
/// Add request for multisig.
pub fn add_request(&mut self, request: MultiSigRequest) -> RequestId {

/// Add request for multisig and confirm right away with the key that is adding the request.
pub fn add_request_and_confirm(&mut self, request: MultiSigRequest) -> RequestId {

/// Remove given request and associated confirmations.
pub fn delete_request(&mut self, request_id: RequestId) {

/// Confirm given request with given signing key.
/// If with this, there has been enough confirmation, a promise with request will be scheduled.
pub fn confirm(&mut self, request_id: RequestId) -> PromiseOrValue<bool> {
```

### View Methods

```rust
pub fn get_request(&self, request_id: RequestId) -> MultiSigRequest
pub fn get_num_requests_per_member(&self, member: MultisigMember) -> u32
pub fn list_request_ids(&self) -> Vec<RequestId>
pub fn get_confirmations(&self, request_id: RequestId) -> Vec<MultisigMember>
pub fn get_num_confirmations(&self) -> u32
pub fn get_request_nonce(&self) -> u32
```

### State machine

Per each request, multisig maintains next state machine:

- `add_request` adds new request with empty list of confirmations.
- `add_request_and_confirm` adds new request with 1 confirmation from the adding key.
- `delete_request` deletes request and ends state machine.
- `confirm` either adds new confirmation to list of confirmations or if there is more than `num_confirmations` confirmations with given call - switches to execution of request. `confirm` fails if request is already has been confirmed and already is executing which is determined if `confirmations` contain given `request_id`.
- each step of execution, schedules a promise of given set of actions on `receiver_id` and puts a callback.
- when callback executes, it checks if promise executed successfully: if no - stops executing the request and return failure. If yes - execute next transaction in the request if present.
- when all transactions are executed, remove request from `requests` and with that finish the execution of the request.

## Pre-requisites

To develop Rust contracts you would need to:

- Install [Rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- Add wasm target to your toolchain:

```bash
rustup target add wasm32-unknown-unknown
```

## Building the contract

```bash
./build.sh
```

## Usage

Before deploying the contract, you need to collect all public keys that it will be initialized with.

### Create request

To create request for transfer funds:

```bash
unc call multisig.unc add_request '{"request": {"receiver_id": "unc", "actions": [{"type": "Transfer", "amount": "1000000000000000000000"}]}}' --accountId multisig.unc
```

Add another key to multisig:

```bash
unc call multisig.unc add_request '{"request": {"receiver_id": "multisig.unc", "actions": [{"type": "AddMember", "member": {"public_key": "ed25519:<base58 of the key>"}}]}}' --accountId multisig.unc
```

Change number of confirmations required to approve multisig:

```bash
unc call multisig.unc add_request '{"request": {"receiver_id": "multisig.unc", "actions": [{"type": "SetNumConfirmations", "num_confirmations": 2}]}}' --accountId multisig.unc
```

Returns the `request_id` of this request that can be used to confirm or see details.

As a side note, for this to work one of the keys from multisig should be available in your `~/.unc-credentials/<network>/<multisig-name>.json` or use `--useLedgerKey` to sign with Ledger.

You can also create a way more complex call that chains calling multiple different contracts:

### Confirm request

To confirm a specific request:

```bash
unc call multisig.unc confirm '{"request_id": 0}' --accountId multisig.unc
```

### View requests

To list all requests ids:

```bash
unc view multisig.unc list_request_ids
```

To see information about specific request:

```bash
unc view multisig.unc get_request '{"request_id": 0}'
```

To see confirmations for specific request:

```bash
unc view multisig.unc get_confirmations '{"request_id": 0}'
```

Total confirmations required for any request:

```bash
unc view multisig.unc get_num_confirmations
```

### Upgrade given multisig with new code

Create a request that deploys new contract code on the given account.
Be careful about data and requiring migrations (contract updates should include data migrations going forward).

After this, still will need to confirm this with `num_confirmations` you have setup for given contract.

### Common commands for multisig

__Create an account__

```bash
unc call unc add_request '{"request": {"receiver_id": "new_account.unc", "actions": [{"type", "CreateAccount"}, {"type": "Transfer", "amount": "1000000000000000000000"}, {"type": "AddKey", "public_key": "<public key for the new account>"}]}}' --accountId unc
unc call unc confirm '{"request_id": <request number from previous line>}'
```

## Deploying and building

This repository has simulation tests, which can offer more advanced testing functionality than unit tests. However, this means the manifest (`Cargo.toml`) should be modified to reduce the contract size when building for production.

Remove the `rlib` like shown here:

```diff
[lib]
# Below is used for production
+crate-type = ["cdylib"]
# Below used when running simulation tests
+# crate-type = ["cdylib", "rlib"]
```
