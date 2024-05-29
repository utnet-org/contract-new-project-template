# Hello Utility Contract

The smart contract exposes two methods to enable storing and retrieving a greeting in the Utility network.

```rust
const DEFAULT_MESSAGE: &str = "Hello";

#[unc_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    greeting: String,
}

impl Default for Contract {
    fn default() -> Self {
        Self{greeting: DEFAULT_MESSAGE.to_string()}
    }
}

#[unc_bindgen]
impl Contract {
    // Public: Returns the stored greeting, defaulting to 'Hello'
    pub fn get_greeting(&self) -> String {
        return self.greeting.clone();
    }

    // Public: Takes a greeting, such as 'howdy', and records it
    pub fn set_greeting(&mut self, greeting: String) {
        // Record a log permanently to the blockchain!
        log!("Saving greeting {}", greeting);
        self.greeting = greeting;
    }
}
```

<br />

# Quickstart

1. Make sure you have installed [rust](https://rust.org/).
2. Install the [`Utility CLI`](https://github.com/utnet-org/utility-cli-rs#setup)

<br />

## 1. Build and Deploy the Contract

You can automatically compile and deploy the contract in the Utility testnet by running:

```bash
# Use unc-cli to import your Utility account or create-account and fund-later
# e.g. fd09e7537ee95fd2e7b78ee0a2b10bb9db4ebe65dc94802ce420c94ebb25bc43
unc account import-account
```

and then use the imported account to sign the transaction: `--accountId <your-account>`.

```bash
./deploy.sh
```

<br />

## 2. Retrieve the Greeting

`get_greeting` is a read-only method (aka `view` method).

`View` methods can be called for **free** by anyone, even people **without a Utility account**!

```bash
# Use unc-cli to get the greeting
unc view <dev-account> get_greeting
```

<br />

## 3. Store a New Greeting

`set_greeting` changes the contract's state, for which it is a `change` method.

`Change` methods can only be invoked using a Utility account, since the account needs to pay GAS for the transaction.

```bash
# Use unc-cli to set a new greeting
unc  call <dev-account> set_greeting '{"greeting":"howdy"}' --accountId <dev-account>
```
