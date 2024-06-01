mod get_workspace_dir;

use crate::get_workspace_dir::get_workspace_dir;
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use utility_workspaces::network::Sandbox;
use utility_workspaces::types::UncToken;
use utility_workspaces::{Account, Contract, Worker};

static CONTRACT_WASM_FILEPATH: &str = "./res/w_unc.wasm";

const CONTRACT_ID: &str = "wrapunc";
const LEGACY_BYTE_COST: Balance = 10_000_000_000_000_000_000;

const STORAGE_BALANCE: Balance = 125 * LEGACY_BYTE_COST;

//Environment Variables: UNC_ENABLE_SANDBOX_LOG = 1
/// Tests the `set` method.
#[tokio::main]
async fn main() -> Result<()> {
    test_ft_transfer().await?;
    test_wrap_without_storage_deposit().await?;
    Ok(())
}

/// Sanity check that we can `set` and `get` a value.
async fn test_ft_transfer() -> Result<()> {
    let (_, contract, user) = init_contract_and_user().await?;

    let user_id = user.id().to_string();
    let args = json!({
        "account_id": user_id
    });

    user.call(contract.id(), "unc_deposit")
        .args_json(args)
        .deposit(UncToken::from_attounc(10u128))
        .transact()
        .await?
        .into_result()?;

    // ft_balance_of(bob)
    // assert_eq!(alice_balance.0, to_yocto("10"));
    let result = user
        .view(contract.id(), "ft_balance_of")
        .args_json(json!({ "keys": [name_key] }))
        .await?
        .json::<HashMap<String, HashMap<String, HashMap<String, String>>>>()?;
    
    // call ft_transfer(bob.valid_account_id(), to_yocto("5").into(), None),

    // ft_balance_of(bob)
    // assert_eq!(bob_balance.0, to_yocto("5"));

    assert_eq!(name, result_name);

    Ok(())
}


async fn test_wrap_without_storage_deposit() -> Result<()> {
    let (_, contract, user) = init_contract_and_user().await?;

    let user_id = user.id().to_string();
    let args = json!({
        "account_id": user_id
    });

    user.call(contract.id(), "unc_deposit")
        .args_json(args)
        .deposit(UncToken::from_attounc(10u128))
        .transact()
        .await?
        .into_result()?;

    // assert_eq!(alice_balance.0, to_yocto("10") - STORAGE_BALANCE);
    // ft_balance_of(alice)

    let prev_balance = user.view_account().await?.balance;
    let deposit = UncToken::from_unc(1);

    let post_balance = user.view_account().await?.balance;

    assert!(post_balance < prev_balance);
    assert!(post_balance > prev_balance.checked_sub(deposit).unwrap());

    Ok(())
}

async fn init_contract_and_user() -> Result<(Worker<Sandbox>, Contract, Account)> {
    let workspace_dir = get_workspace_dir();
    let wasm_filepath = workspace_dir.join(CONTRACT_WASM_FILEPATH);

    // Create a sandboxed environment.
    // NOTE: Each call will create a new sandboxed environment
    let worker = utility_workspaces::sandbox().await?;
    // or for testnet:
    //let worker = utility_workspaces::testnet().await?;
    let wasm = fs::read(wasm_filepath)?;

    //let wasm = utility_workspaces::compile_project(wasm_filepath.).await?;

    let contract = worker.dev_deploy(&wasm).await?;
    contract.call("new").transact().await?.into_result()?;

    let account = worker.dev_create_account().await?;
    let user = account
        .create_subaccount("alice")
        .initial_balance(UncToken::from_attounc(100))
        .transact()
        .await?
        .into_result()?;
    Ok((worker, contract, user))
}
