use anyhow::Result;
use serde_json::json;
use unc_sdk::json_types::U128;
use utility_workspaces::network::Sandbox;
use utility_workspaces::types::UncToken;
use utility_workspaces::{Account, Contract, Worker};

const LEGACY_BYTE_COST: u128 = 10_000_000_000_000_000_000;
const STORAGE_BALANCE: u128 = 125 * LEGACY_BYTE_COST;

//Environment Variables: UNC_ENABLE_SANDBOX_LOG = 1
/// Sanity check that we can `set` and `get` a value.
#[tokio::test]
async fn test_ft_transfer() -> Result<()> {
    let (worker, contract, user) = init_contract_and_user().await?;

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

    let result_alice_balance = user
        .view(contract.id(), "ft_balance_of")
        .args_json(json!({ "account_id": user_id }))
        .await?
        .json::<U128>()?;
    assert_eq!(result_alice_balance.0, to_atto(10));

    let second_user = worker.dev_create_account().await?;
    let bob = second_user.id().to_string();
    //TODO: call ft_transfer(bob.valid_account_id(), to_atto("5").into(), None),
    let result = user
        .call(contract.id(), "ft_transfer")
        .args_json(json!({ "account_id": bob }))
        .transact()
        .await?;

    assert!(result.is_success());

    let result_bob_balance = user
        .view(contract.id(), "ft_balance_of")
        .args_json(json!({ "account_id": bob }))
        .await?
        .json::<U128>()?;

    assert_eq!(result_bob_balance.0, to_atto(5));

    Ok(())
}

#[tokio::test]
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

    let result_alice_balance = user
        .view(contract.id(), "ft_balance_of")
        .args_json(json!({ "account_id": user_id }))
        .await?
        .json::<U128>()?;

    assert_eq!(result_alice_balance.0, to_atto(10) - STORAGE_BALANCE);

    Ok(())
}

async fn init_contract_and_user() -> Result<(Worker<Sandbox>, Contract, Account)> {
    // Create a sandboxed environment.
    // NOTE: Each call will create a new sandboxed environment
    let worker = utility_workspaces::sandbox().await?;
    // or for testnet:
    //let worker = utility_workspaces::testnet().await?;
    let contract_wasm = utility_workspaces::compile_project("./").await?;

    let contract = worker.dev_deploy(&contract_wasm).await?;
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

pub fn to_atto(unc_balance: u128) -> u128 {
    unc_balance * 10u128.pow(24)
}