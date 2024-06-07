use unc_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
#[allow(deprecated)]
use unc_sdk::store::UnorderedSet;
use unc_sdk::json_types::U128;
use unc_sdk::serde::{Deserialize, Serialize};
use unc_sdk::{env, ext_contract, unc_bindgen, AccountId, UncToken, UncSchema, PublicKey, Promise, PromiseOrValue};

mod utils;
use crate::utils::*;

/// The 30 UNC tokens required for the storage of the staking pool.
const MIN_ATTACHED_BALANCE: u128 = 30_000_000_000_000_000_000_000_000;

pub mod gas {
    use unc_sdk::Gas;

    /// The base amount of gas for a regular execution.
    const BASE: u64 = 25_000_000_000_000;

    /// The amount of Gas the contract will attach to the promise to create the staking pool.
    /// The base for the execution and the base for staking action to verify the staking key.
    pub const STAKING_POOL_NEW: Gas = Gas::from_gas(BASE * 2);

    /// The amount of Gas the contract will attach to the callback to itself.
    /// The base for the execution and the base for whitelist call or cash rollback.
    pub const CALLBACK: Gas = Gas::from_gas(BASE * 2);

    /// The amount of Gas the contract will attach to the promise to the whitelist contract.
    /// The base for the execution.
    pub const WHITELIST_STAKING_POOL: Gas = Gas::from_gas(BASE);
}

/// There is no deposit balance attached.
const NO_DEPOSIT: UncToken = UncToken::from_attounc(0);

#[unc_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct StakingPoolFactory {
    /// Account ID of the staking pool whitelist contract.
    staking_pool_whitelist_account_id: AccountId,

    #[allow(deprecated)]
    /// The account ID of the staking pools created.
    staking_pool_account_ids: UnorderedSet<AccountId>,
}

impl Default for StakingPoolFactory {
    fn default() -> Self {
        env::panic_str("The contract should be initialized before usage")
    }
}

/// Rewards fee fraction structure for the staking pool contract.
#[derive(Serialize, Deserialize, UncSchema, Clone)]
#[serde(crate = "unc_sdk::serde")]
pub struct RewardFeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

impl RewardFeeFraction {
    pub fn assert_valid(&self) {
        assert_ne!(self.denominator, 0, "Denominator must be a positive number");
        assert!(
            self.numerator <= self.denominator,
            "The reward fee must be less or equal to 1"
        );
    }
}

#[derive(Serialize)]
#[serde(crate = "unc_sdk::serde")]
pub struct StakingPoolArgs {
    /// Owner account ID of the staking pool.
    owner_id: AccountId,
    /// The initial staking key.
    stake_public_key: PublicKey,
    /// The initial reward fee fraction.
    reward_fee_fraction: RewardFeeFraction,
}

/// External interface for the callbacks to self.
#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_staking_pool_create(
        &mut self,
        staking_pool_account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> Promise;
}

/// External interface for the whitelist contract.
#[ext_contract(ext_whitelist)]
pub trait ExtWhitelist {
    fn add_staking_pool(&mut self, staking_pool_account_id: AccountId) -> bool;
}

#[unc_bindgen]
impl StakingPoolFactory {
    /// Initializes the staking pool factory with the given account ID of the staking pool whitelist
    /// contract.
    #[init]
    pub fn new(staking_pool_whitelist_account_id: AccountId) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");
        assert!(
            env::is_valid_account_id(staking_pool_whitelist_account_id.as_bytes()),
            "The staking pool whitelist account ID is invalid"
        );
        Self {
            staking_pool_whitelist_account_id,
            #[allow(deprecated)]
            staking_pool_account_ids: UnorderedSet::new(b"s".to_vec()),
        }
    }

    /// Returns the minimum amount of tokens required to attach to the function call to
    /// create a new staking pool.
    pub fn get_min_attached_balance(&self) -> U128 {
        MIN_ATTACHED_BALANCE.into()
    }

    /// Returns the total number of the staking pools created from this factory.
    pub fn get_number_of_staking_pools_created(&self) -> u64 {
        self.staking_pool_account_ids.len() as u64
    }

    /// Creates a new staking pool.
    /// - `staking_pool_id` - the prefix of the account ID that will be used to create a new staking
    ///    pool account. It'll be prepended to the staking pool factory account ID separated by dot.
    /// - `owner_id` - the account ID of the staking pool owner. This account will be able to
    ///    control the staking pool, set reward fee, update staking key and vote on behalf of the
    ///     pool.
    /// - `stake_public_key` - the initial staking key for the staking pool.
    /// - `reward_fee_fraction` - the initial reward fee fraction for the staking pool.
    #[payable]
    pub fn create_staking_pool(
        &mut self,
        staking_pool_id: AccountId,
        owner_id: AccountId,
        stake_public_key: PublicKey,
        reward_fee_fraction: RewardFeeFraction,
    ) -> Promise {
        assert!(
            env::attached_deposit() >= UncToken::from_attounc(MIN_ATTACHED_BALANCE),
            "Not enough attached deposit to complete staking pool creation"
        );

        assert!(
            staking_pool_id.to_string().find('.').is_none(),
            "The staking pool ID can't contain `.`"
        );

        let staking_pool_account_id: AccountId = format!("{}.{}", staking_pool_id, env::current_account_id()).parse().unwrap();

        assert!(
            env::is_valid_account_id(staking_pool_account_id.as_bytes()),
            "The staking pool account ID is invalid"
        );

        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "The owner account ID is invalid"
        );
        reward_fee_fraction.assert_valid();

        assert!(
            self.staking_pool_account_ids
                .insert(staking_pool_account_id.clone()),
            "The staking pool account ID already exists"
        );

        Promise::new(staking_pool_account_id.clone())
            .create_account()
            .transfer(env::attached_deposit())
            .deploy_contract(include_bytes!("../../res/staking_pool.wasm").to_vec())
            .function_call(
                "new".to_string(),
                unc_sdk::serde_json::to_vec(&StakingPoolArgs {
                    owner_id,
                    stake_public_key,
                    reward_fee_fraction,
                })
                .unwrap(),
                NO_DEPOSIT,
                gas::STAKING_POOL_NEW,
            )
            .then(ext_self::ext(env::current_account_id())
                .with_static_gas(gas::CALLBACK)
                .with_attached_deposit(NO_DEPOSIT)
                .on_staking_pool_create(
                    staking_pool_account_id,
                    env::attached_deposit().as_attounc().into(),
                    env::predecessor_account_id())
                )
    }

    /// Callback after a staking pool was created.
    /// Returns the promise to whitelist the staking pool contract if the pool creation succeeded.
    /// Otherwise refunds the attached deposit and returns `false`.
    pub fn on_staking_pool_create(
        &mut self,
        staking_pool_account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> PromiseOrValue<bool> {
        assert_self();

        let staking_pool_created = is_promise_success();

        if staking_pool_created {
            env::log_str(
                format!(
                    "The staking pool @{} was successfully created. Whitelisting...",
                    staking_pool_account_id
                )
                .as_str(),
            );
            ext_whitelist::ext(env::current_account_id())
                .with_attached_deposit(NO_DEPOSIT) 
                .with_static_gas(gas::WHITELIST_STAKING_POOL)
                .add_staking_pool(
                    staking_pool_account_id,
                    //&self.staking_pool_whitelist_account_id,
                )
            .into()
        } else {
            self.staking_pool_account_ids
                .remove(&staking_pool_account_id);
            env::log_str(
                format!(
                    "The staking pool @{} creation has failed. Returning attached deposit of {} to @{}",
                    staking_pool_account_id,
                    attached_deposit.0,
                    predecessor_account_id
                ).as_str()
            );
            Promise::new(predecessor_account_id).transfer(UncToken::from_attounc(attached_deposit.0));
            PromiseOrValue::Value(false)
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use unc_sdk::test_utils::VMContextBuilder;
    use unc_sdk::{testing_env, MockedBlockchain, PromiseResult};

    mod test_utils;
    use test_utils::*;
    use std::convert::TryInto;

    #[test]
    fn test_create_staking_pool_success() {
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_unc())
            .build());

        let mut contract = StakingPoolFactory::new(account_whitelist());

        testing_env!(VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());

        assert_eq!(contract.get_min_attached_balance().0, MIN_ATTACHED_BALANCE);
        assert_eq!(contract.get_number_of_staking_pools_created(), 0);

        testing_env!(VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_tokens_owner())
            .attached_deposit(UncToken::from_attounc(ntoy(31)))
            .is_view(false)
            .build());
        contract.create_staking_pool(
            staking_pool_id(),
            account_pool_owner(),
            "ed25519:KuTCtARNzxZQ3YvXDeLjx83FDqxv2SdQTSbiq876zR7"
                .parse()
                .unwrap(),
            RewardFeeFraction {
                numerator: 10,
                denominator: 100,
            },
        );

        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_factory())
            .attached_deposit(UncToken::from_attounc(ntoy(0)))
            .build();
        testing_env!(
            context,
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_create(account_pool(), ntoy(31).into(), account_tokens_owner());

        testing_env!(VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_factory())
            .attached_deposit(UncToken::from_attounc(ntoy(0)))
            .is_view(true)
            .build());
        assert_eq!(contract.get_number_of_staking_pools_created(), 1);
    }

    #[test]
    #[should_panic(expected = "Not enough attached deposit to complete staking pool creation")]
    fn test_create_staking_pool_not_enough_deposit() {
        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_unc())
            .build();
        testing_env!(context);

        let mut contract = StakingPoolFactory::new(account_whitelist());

        // Checking the pool is still whitelisted
        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build();
        testing_env!(context);

        assert_eq!(contract.get_min_attached_balance().0, MIN_ATTACHED_BALANCE);
        assert_eq!(contract.get_number_of_staking_pools_created(), 0);

        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_tokens_owner())
            .attached_deposit(UncToken::from_attounc(ntoy(20)))
            .is_view(false)
            .build();
        testing_env!(context);

        contract.create_staking_pool(
            staking_pool_id(),
            account_pool_owner(),
            "ed25519:KuTCtARNzxZQ3YvXDeLjx83FDqxv2SdQTSbiq876zR7"
                .parse()
                .unwrap(),
            RewardFeeFraction {
                numerator: 10,
                denominator: 100,
            },
        );
    }

    #[test]
    fn test_create_staking_pool_rollback() {
        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_unc())
            .build();
        testing_env!(context);

        let mut contract = StakingPoolFactory::new(account_whitelist());

        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build();
        testing_env!(context);
        assert_eq!(contract.get_min_attached_balance().0, MIN_ATTACHED_BALANCE);
        assert_eq!(contract.get_number_of_staking_pools_created(), 0);

        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_tokens_owner())
            .is_view(false)
            .attached_deposit(UncToken::from_attounc(ntoy(31)))
            .build();
        testing_env!(context.clone());
        contract.create_staking_pool(
            staking_pool_id(),
            account_pool_owner(),
            "ed25519:KuTCtARNzxZQ3YvXDeLjx83FDqxv2SdQTSbiq876zR7"
                .parse()
                .unwrap(),
            RewardFeeFraction {
                numerator: 10,
                denominator: 100,
            },
        );

        let balance = context.account_balance.saturating_add(UncToken::from_attounc(ntoy(31)));
        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_factory())
            .attached_deposit(UncToken::from_attounc(ntoy(0)))
            .account_balance(balance)
            .build();

        testing_env!(context, PromiseResult::Failed);
        let res = contract.on_staking_pool_create(
            account_pool(),
            ntoy(31).into(),
            account_tokens_owner(),
        );
        match res {
            PromiseOrValue::Promise(_) => panic!("Unexpected result, should return Value(false)"),
            PromiseOrValue::Value(value) => assert!(!value),
        };

        let context = VMContextBuilder::new()
            .current_account_id(account_factory())
            .predecessor_account_id(account_factory())
            .attached_deposit(UncToken::from_attounc(ntoy(0)))
            .account_balance(balance)
            .is_view(true)
            .build();
        testing_env!(context);
        assert_eq!(contract.get_number_of_staking_pools_created(), 0);
    }
}
