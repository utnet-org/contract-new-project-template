use unc_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use unc_sdk::serde::{Deserialize, Serialize};
use unc_sdk::serde_json::json;
use unc_sdk::{env, unc_bindgen, AccountId, Gas, Promise, PublicKey, UncToken};


const CODE: &[u8] = include_bytes!("../../res/multisig.wasm");

/// This gas spent on the call & account creation, the rest goes to the `new` call.
const CREATE_CALL_GAS: Gas = Gas::from_gas(50_000_000_000_000);

#[derive(Serialize, Deserialize)]
#[serde(crate = "unc_sdk::serde", untagged)]
pub enum MultisigMember {
    AccessKey { public_key: PublicKey },
    Account { account_id: AccountId },
}

#[unc_bindgen]
#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct MultisigFactory {}

#[unc_bindgen]
impl MultisigFactory {
    #[payable]
    pub fn create(
        &mut self,
        name: AccountId,
        members: Vec<MultisigMember>,
        num_confirmations: u64,
    ) -> Promise {
        let account_id = format!("{}.{}", name, env::current_account_id()).parse().unwrap();
        Promise::new(account_id)
            .create_account()
            .deploy_contract(CODE.to_vec())
            .transfer(env::attached_deposit())
            .function_call(
                "new".to_string(),
                json!({ "members": members, "num_confirmations": num_confirmations })
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                UncToken::from_unc(0),
                env::prepaid_gas().saturating_sub(CREATE_CALL_GAS),
            )
    }
}
