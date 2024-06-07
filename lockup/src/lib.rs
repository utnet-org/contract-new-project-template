//! A smart contract that allows tokens to be locked up.

use unc_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use unc_sdk::{env, ext_contract, unc_bindgen, AccountId, UncToken, Gas};

pub use crate::types::*;

pub mod foundation;
pub mod foundation_callbacks;
pub mod gas;
pub mod owner_callbacks;
pub mod types;

pub mod getters;
pub mod internal;
pub mod owner;

/// Indicates there are no deposit for a cross contract call for better readability.
const NO_DEPOSIT: UncToken = UncToken::from_attounc(0);

/// The contract keeps at least 3.5 UNC in the account to avoid being transferred out to cover
/// contract code storage and some internal state.
pub const MIN_BALANCE_FOR_STORAGE: u128 = 3_500_000_000_000_000_000_000_000;

#[ext_contract(ext_staking_pool)]
pub trait ExtStakingPool {
    fn get_account_staked_balance(&self, account_id: AccountId) -> WrappedBalance;

    fn get_account_unstaked_balance(&self, account_id: AccountId) -> WrappedBalance;

    fn get_account_total_balance(&self, account_id: AccountId) -> WrappedBalance;

    fn deposit(&mut self);

    fn deposit_and_stake(&mut self);

    fn withdraw(&mut self, amount: WrappedBalance);

    fn stake(&mut self, amount: WrappedBalance);

    fn unstake(&mut self, amount: WrappedBalance);

    fn unstake_all(&mut self);
}

#[ext_contract(ext_whitelist)]
pub trait ExtStakingPoolWhitelist {
    fn is_whitelisted(&self, staking_pool_account_id: AccountId) -> bool;
}

#[ext_contract(ext_transfer_poll)]
pub trait ExtTransferPoll {
    fn get_result(&self) -> Option<PollResult>;
}

#[ext_contract(ext_self_owner)]
pub trait ExtLockupContractOwner {
    fn on_whitelist_is_whitelisted(
        &mut self,
        #[callback] is_whitelisted: bool,
        staking_pool_account_id: AccountId,
    ) -> bool;

    fn on_staking_pool_deposit(&mut self, amount: WrappedBalance) -> bool;

    fn on_staking_pool_deposit_and_stake(&mut self, amount: WrappedBalance) -> bool;

    fn on_staking_pool_withdraw(&mut self, amount: WrappedBalance) -> bool;

    fn on_staking_pool_stake(&mut self, amount: WrappedBalance) -> bool;

    fn on_staking_pool_unstake(&mut self, amount: WrappedBalance) -> bool;

    fn on_staking_pool_unstake_all(&mut self) -> bool;

    fn on_get_result_from_transfer_poll(&mut self, #[callback] poll_result: PollResult) -> bool;

    fn on_get_account_total_balance(&mut self, #[callback] total_balance: WrappedBalance);

    fn on_get_account_unstaked_balance_to_withdraw_by_owner(
        &mut self,
        #[callback] unstaked_balance: WrappedBalance,
    );
}

#[ext_contract(ext_self_foundation)]
pub trait ExtLockupContractFoundation {
    fn on_withdraw_unvested_amount(
        &mut self,
        amount: WrappedBalance,
        receiver_id: AccountId,
    ) -> bool;

    fn on_get_account_staked_balance_to_unstake(
        &mut self,
        #[callback] staked_balance: WrappedBalance,
    );

    fn on_staking_pool_unstake_for_termination(&mut self, amount: WrappedBalance) -> bool;

    fn on_get_account_unstaked_balance_to_withdraw(
        &mut self,
        #[callback] unstaked_balance: WrappedBalance,
    );

    fn on_staking_pool_withdraw_for_termination(&mut self, amount: WrappedBalance) -> bool;
}

#[unc_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LockupContract {
    /// The account ID of the owner.
    pub owner_account_id: AccountId,

    /// Information about lockup schedule and the amount.
    pub lockup_information: LockupInformation,

    /// Information about vesting including schedule or termination status.
    pub vesting_information: VestingInformation,

    /// Account ID of the staking pool whitelist contract.
    pub staking_pool_whitelist_account_id: AccountId,

    /// Information about staking and delegation.
    /// `Some` means the staking information is available and the staking pool contract is selected.
    /// `None` means there is no staking pool selected.
    pub staking_information: Option<StakingInformation>,

    /// The account ID that the UNC Foundation, that has the ability to terminate vesting.
    pub foundation_account_id: Option<AccountId>,
}

impl Default for LockupContract {
    fn default() -> Self {
        env::panic_str("The contract is not initialized.");
    }
}

#[unc_bindgen]
impl LockupContract {
    /// Requires 25 TGas (1 * BASE_GAS)
    ///
    /// Initializes lockup contract.
    /// - `owner_account_id` - the account ID of the owner. Only this account can call owner's
    ///    methods on this contract.
    /// - `lockup_duration` [deprecated] - the duration in nanoseconds of the lockup period from
    ///    the moment the transfers are enabled. During this period tokens are locked and
    ///    the release doesn't start. Instead of this, use `lockup_timestamp` and `release_duration`
    /// - `lockup_timestamp` - the optional absolute lockup timestamp in nanoseconds which locks
    ///    the tokens until this timestamp passes. Until this moment the tokens are locked and the
    ///    release doesn't start.
    /// - `transfers_information` - the information about the transfers. Either transfers are
    ///    already enabled, then it contains the timestamp when they were enabled. Or the transfers
    ///    are currently disabled and it contains the account ID of the transfer poll contract.
    /// - `vesting_schedule` - If provided, then it's either a base64 encoded hash of vesting
    ///    schedule with salt or an explicit vesting schedule.
    ///    Vesting schedule affects the amount of tokens the UNC Foundation will get in case of
    ///    employment termination as well as the amount of tokens available for transfer by
    ///    the employee. If Hash provided, it's expected that vesting started before lockup and
    ///    it only needs to be revealed in case of termination.
    /// - `release_duration` - is the duration when the full lockup amount will be available.
    ///    The tokens are liuncly released from the moment tokens are unlocked.
    ///    The unlocking happens at the timestamp defined by:
    ///    `max(transfers_timestamp + lockup_duration, lockup_timestamp)`.
    ///    If it's used in addition to the vesting schedule, then the amount of tokens available to
    ///    transfer is subject to the minimum between vested tokens and released tokens.
    /// - `staking_pool_whitelist_account_id` - the Account ID of the staking pool whitelist contract.
    /// - `foundation_account_id` - the account ID of the UNC Foundation, that has the ability to
    ///    terminate vesting schedule.
    #[init]
    pub fn new(
        owner_account_id: AccountId,
        lockup_duration: WrappedDuration,
        lockup_timestamp: Option<WrappedTimestamp>,
        transfers_information: TransfersInformation,
        vesting_schedule: Option<VestingScheduleOrHash>,
        release_duration: Option<WrappedDuration>,
        staking_pool_whitelist_account_id: AccountId,
        foundation_account_id: Option<AccountId>,
    ) -> Self {
        assert!(
            env::is_valid_account_id(owner_account_id.as_bytes()),
            "The account ID of the owner is invalid"
        );
        assert!(
            env::is_valid_account_id(staking_pool_whitelist_account_id.as_bytes()),
            "The staking pool whitelist account ID is invalid"
        );
        if let TransfersInformation::TransfersDisabled {
            transfer_poll_account_id,
        } = &transfers_information
        {
            assert!(
                env::is_valid_account_id(transfer_poll_account_id.as_bytes()),
                "The transfer poll account ID is invalid"
            );
        }
        let lockup_information = LockupInformation {
            lockup_amount: env::account_balance().as_attounc(),
            termination_withdrawn_tokens: 0,
            lockup_duration: lockup_duration.0,
            release_duration: release_duration.map(|d| d.0),
            lockup_timestamp: lockup_timestamp.map(|d| d.0),
            transfers_information,
        };
        let vesting_information = match vesting_schedule {
            None => {
                assert!(
                    foundation_account_id.is_none(),
                    "Foundation account can't be added without vesting schedule"
                );
                VestingInformation::None
            }
            Some(VestingScheduleOrHash::VestingHash(hash)) => VestingInformation::VestingHash(hash),
            Some(VestingScheduleOrHash::VestingSchedule(vs)) => {
                VestingInformation::VestingSchedule(vs)
            }
        };
        assert!(
            vesting_information == VestingInformation::None ||
                env::is_valid_account_id(foundation_account_id.as_ref().unwrap().as_bytes()),
            "Foundation account should be added for vesting schedule"
        );

        Self {
            owner_account_id,
            lockup_information,
            vesting_information,
            staking_information: None,
            staking_pool_whitelist_account_id,
            foundation_account_id,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use unc_sdk::{test_vm_config, testing_env, PromiseResult, RuntimeFeesConfig, VMContext};
    use unc_sdk::test_utils::VMContextBuilder;

    use test_utils::*;

    use super::*;
    use unc_primitives_core::config::ViewConfig;

    mod test_utils;

    const SALT: [u8; 3] = [1, 2, 3];

    fn basic_context() -> VMContext {
        VMContextBuilder::new()
            .current_account_id(lockup_account())
            .predecessor_account_id(system_account())
            .account_balance(UncToken::from_attounc(to_atto(LOCKUP_UNC)))
            .account_locked_balance(UncToken::from_attounc(0))
            .block_timestamp(to_ts(GENESIS_TIME_IN_DAYS))
            .is_view(false)
            .build()
    }

    fn new_vesting_schedule(offset_in_days: u64) -> VestingSchedule {
        VestingSchedule {
            start_timestamp: to_ts(GENESIS_TIME_IN_DAYS - YEAR + offset_in_days).into(),
            cliff_timestamp: to_ts(GENESIS_TIME_IN_DAYS + offset_in_days).into(),
            end_timestamp: to_ts(GENESIS_TIME_IN_DAYS + YEAR * 3 + offset_in_days).into(),
        }
    }

    fn no_vesting_schedule() -> VestingSchedule {
        VestingSchedule {
            start_timestamp: to_ts(0).into(),
            cliff_timestamp: to_ts(0).into(),
            end_timestamp: to_ts(0).into(),
        }
    }

    fn new_contract_with_lockup_duration(
        transfers_enabled: bool,
        vesting_schedule: Option<VestingSchedule>,
        release_duration: Option<WrappedDuration>,
        foundation_account: bool,
        lockup_duration: Duration,
    ) -> LockupContract {
        let lockup_start_information = if transfers_enabled {
            TransfersInformation::TransfersEnabled {
                transfers_timestamp: to_ts(GENESIS_TIME_IN_DAYS).into(),
            }
        } else {
            TransfersInformation::TransfersDisabled {
                transfer_poll_account_id: "transfers".parse().unwrap(),
            }
        };
        let foundation_account_id = if foundation_account {
            Some(account_foundation())
        } else {
            None
        };
        let vesting_schedule = vesting_schedule.map(|vesting_schedule| {
            VestingScheduleOrHash::VestingHash(
                VestingScheduleWithSalt {
                    vesting_schedule,
                    salt: SALT.to_vec().into(),
                }
                    .hash()
                    .into(),
            )
        });
        LockupContract::new(
            account_owner(),
            lockup_duration.into(),
            None,
            lockup_start_information,
            vesting_schedule,
            release_duration,
           "whitelist".parse().unwrap(),
            foundation_account_id,
        )
    }

    fn new_contract(
        transfers_enabled: bool,
        vesting_schedule: Option<VestingSchedule>,
        release_duration: Option<WrappedDuration>,
        foundation_account: bool,
    ) -> LockupContract {
        new_contract_with_lockup_duration(
            transfers_enabled,
            vesting_schedule,
            release_duration,
            foundation_account,
            to_nanos(YEAR),
        )
    }

    fn lockup_only_setup() -> (VMContext, LockupContract) {
        let context = basic_context();
        testing_env!(context.clone());
        let contract = new_contract(true, None, None, false);
        (context, contract)
    }

    #[test]
    fn test_lockup_only_basic() {
        let (mut context, contract) = lockup_only_setup();
        // Checking initial values at genesis time
        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());

        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(
            contract.get_locked_vested_amount(no_vesting_schedule()).0,
            to_atto(LOCKUP_UNC)
        );

        // Checking values in 1 day after genesis time
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 1);

        assert_eq!(contract.get_owners_balance().0, 0);

        // Checking values next day after lockup timestamp
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR + 1);
        testing_env!(context.clone());

        assert_almost_eq(contract.get_owners_balance().0, to_atto(LOCKUP_UNC));
    }

    #[test]
    fn test_add_full_access_key() {
        let (mut context, mut contract) = lockup_only_setup();
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(1).try_into().unwrap();
        testing_env!(context.clone());

        contract.add_full_access_key(public_key(4));
    }

    #[test]
    #[should_panic(expected = "Tokens are still locked/unvested")]
    fn test_add_full_access_key_when_vesting_is_not_finished() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(YEAR);
        let mut contract = new_contract(true, Some(vesting_schedule), None, true);

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR - 10);
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(1).try_into().unwrap();
        testing_env!(context.clone());

        contract.add_full_access_key(public_key(4));
    }

    #[test]
    #[should_panic(expected = "Tokens are still locked/unvested")]
    fn test_add_full_access_key_when_lockup_is_not_finished() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let mut contract = new_contract(true, None, Some(to_nanos(YEAR).into()), false);

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR - 10);
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(1).try_into().unwrap();
        testing_env!(context.clone());

        contract.add_full_access_key(public_key(4));
    }

    #[test]
    #[should_panic(expected = "Can only be called by the owner")]
    fn test_call_by_non_owner() {
        let (mut context, mut contract) = lockup_only_setup();
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        context.predecessor_account_id = non_owner();
        context.signer_account_id = non_owner();
        testing_env!(context.clone());

        contract.select_staking_pool("staking_pool".parse().unwrap());
    }

    #[test]
    #[should_panic(expected = "Presented vesting schedule and salt don't match the hash")]
    fn test_vesting_doesnt_match() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(5);
        let mut contract = new_contract(true, Some(vesting_schedule), None, true);
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        context.predecessor_account_id = account_foundation();
        context.signer_account_id = non_owner();
        testing_env!(context.clone());

        let not_real_vesting = new_vesting_schedule(100);
        contract.terminate_vesting(Some(VestingScheduleWithSalt {
            vesting_schedule: not_real_vesting,
            salt: SALT.to_vec().into(),
        }));
    }

    #[test]
    #[should_panic(expected = "Expected vesting schedule and salt, but it was not provided")]
    fn test_vesting_schedule_and_salt_not_provided() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(5);
        let mut contract = new_contract(true, Some(vesting_schedule), None, true);
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        context.predecessor_account_id = account_foundation();
        context.signer_account_id = non_owner();
        testing_env!(context.clone());

        contract.terminate_vesting(None);
    }

    #[test]
    #[should_panic(expected = "Explicit vesting schedule exists")]
    fn test_explicit_vesting() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(5);
        let mut contract = LockupContract::new(
            account_owner(),
            to_nanos(YEAR).into(),
            None,
            TransfersInformation::TransfersEnabled {
                transfers_timestamp: to_ts(GENESIS_TIME_IN_DAYS).into(),
            },
            Some(VestingScheduleOrHash::VestingSchedule(
                vesting_schedule.clone(),
            )),
            None,
           "whitelist".parse().unwrap(),
            Some(account_foundation()),
        );
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        context.predecessor_account_id = account_foundation();
        context.signer_account_id = non_owner();
        testing_env!(context.clone());

        contract.terminate_vesting(Some(VestingScheduleWithSalt {
            vesting_schedule,
            salt: SALT.to_vec().into(),
        }));
    }

    #[test]
    #[should_panic(expected = "Foundation account can't be added without vesting schedule")]
    fn test_init_foundation_key_no_vesting() {
        let context = basic_context();
        testing_env!(context.clone());
        new_contract(true, None, None, true);
    }

    #[test]
    #[should_panic(expected = "Foundation account can't be added without vesting schedule")]
    fn test_init_foundation_key_no_vesting_with_release() {
        let context = basic_context();
        testing_env!(context.clone());
        new_contract(true, None, Some(to_nanos(YEAR).into()), true);
    }

    #[test]
    #[should_panic(expected = "Can only be called by UNC Foundation")]
    fn test_call_by_non_foundation() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(0);
        let mut contract = new_contract(true, Some(vesting_schedule.clone()), None, true);
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        context.predecessor_account_id = non_owner();
        context.signer_account_id = non_owner();
        testing_env!(context.clone());

        contract.terminate_vesting(None);
    }

    #[test]
    #[should_panic(expected = "Transfers are disabled")]
    fn test_transfers_not_enabled() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let mut contract = new_contract(false, None, None, false);
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR + 1);
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(1).try_into().unwrap();
        context.view_config = None;
        testing_env!(context.clone());

        contract.transfer(to_atto(100).into(), non_owner());
    }

    #[test]
    fn test_enable_transfers() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let mut contract = new_contract(false, None, None, false);
        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert!(!contract.are_transfers_enabled());

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR + 1);
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(1).try_into().unwrap();
        context.view_config = None;
        testing_env!(context.clone());

        contract.check_transfers_vote();

        let poll_result = Some(to_ts(GENESIS_TIME_IN_DAYS + 10).into());
        context.predecessor_account_id = lockup_account();
        // NOTE: Unit tests don't need to read the content of the promise result. So here we don't
        // have to pass serialized result from the transfer poll.
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        assert!(contract.on_get_result_from_transfer_poll(poll_result));

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        // Not unlocked yet
        assert_eq!(contract.get_owners_balance().0, 0);
        assert!(contract.are_transfers_enabled());
        assert_eq!(contract.get_vesting_information(), VestingInformation::None);

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR + 10);
        testing_env!(context.clone());
        // Unlocked yet
        assert_eq!(
            contract.get_owners_balance().0,
            to_atto(LOCKUP_UNC).into()
        );

        context.view_config = None;
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.transfer(to_atto(100).into(), non_owner());
    }

    #[test]
    fn test_check_transfers_vote_false() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let mut contract = new_contract(false, None, None, false);
        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert!(!contract.are_transfers_enabled());

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR + 1);
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(1).try_into().unwrap();
        context.view_config = None;
        testing_env!(context.clone());

        contract.check_transfers_vote();

        let poll_result = None;
        // NOTE: Unit tests don't need to read the content of the promise result. So here we don't
        // have to pass serialized result from the transfer poll.
        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        assert!(!contract.on_get_result_from_transfer_poll(poll_result));

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        // Not enabled
        assert!(!contract.are_transfers_enabled());
    }

    #[test]
    fn test_lockup_only_transfer_call_by_owner() {
        let (mut context, mut contract) = lockup_only_setup();
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR + 1);
        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_almost_eq(contract.get_owners_balance().0, to_atto(LOCKUP_UNC));

        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(1).try_into().unwrap();
        context.view_config = None;
        testing_env!(context.clone());

        assert_eq!(env::account_balance(), UncToken::from_attounc(to_atto(LOCKUP_UNC)));
        contract.transfer(to_atto(100).into(), non_owner());
        assert_almost_eq(env::account_balance().as_attounc(), to_atto(LOCKUP_UNC - 100));
    }

    #[test]
    #[should_panic(expected = "Staking pool is not selected")]
    fn test_staking_pool_is_not_selected() {
        let (mut context, mut contract) = lockup_only_setup();
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(2).try_into().unwrap();

        let amount = to_atto(LOCKUP_UNC - 100);
        testing_env!(context.clone());
        contract.deposit_to_staking_pool(amount.into());
    }

    #[test]
    fn test_staking_pool_success() {
        let (mut context, mut contract) = lockup_only_setup();
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(2).try_into().unwrap();

        // Selecting staking pool
        let staking_pool: AccountId = "staking_pool".parse().unwrap();
        testing_env!(context.clone());
        contract.select_staking_pool(staking_pool.clone());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(b"true".to_vec())],
        );
        contract.on_whitelist_is_whitelisted(true, staking_pool.clone());

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_staking_pool_account_id(), Some(staking_pool));
        assert_eq!(contract.get_known_deposited_balance().0, 0);
        context.view_config = None;

        // Deposit to the staking_pool
        let amount = to_atto(LOCKUP_UNC - 100);
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.deposit_to_staking_pool(amount.into());
        context.account_balance = env::account_balance();
        assert_eq!(context.account_balance, UncToken::from_attounc(to_atto(LOCKUP_UNC) - amount));

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_deposit(amount.into());
        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_known_deposited_balance().0, amount);
        context.view_config = None;

        // Staking on the staking pool
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.stake(amount.into());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_stake(amount.into());

        // Assuming there are 20 UNC tokens in rewards. Unstaking.
        let unstake_amount = amount + to_atto(20);
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.unstake(unstake_amount.into());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_unstake(unstake_amount.into());

        // Withdrawing
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.withdraw_from_staking_pool(unstake_amount.into());
        context.account_balance = context.account_balance.saturating_add(UncToken::from_attounc(unstake_amount));

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_withdraw(unstake_amount.into());
        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_known_deposited_balance().0, 0);
        context.view_config = None;

        // Unselecting staking pool
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.unselect_staking_pool();
        assert_eq!(contract.get_staking_pool_account_id(), None);
    }

    #[test]
    fn test_staking_pool_refresh_balance() {
        let (mut context, mut contract) = lockup_only_setup();
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(2).try_into().unwrap();

        // Selecting staking pool
        let staking_pool: AccountId = "staking_pool".parse().unwrap();
        testing_env!(context.clone());
        contract.select_staking_pool(staking_pool.clone());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(b"true".to_vec())],
        );
        contract.on_whitelist_is_whitelisted(true, staking_pool.clone());

        // Deposit to the staking_pool
        let amount = to_atto(LOCKUP_UNC - 100);
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.deposit_to_staking_pool(amount.into());
        context.account_balance = env::account_balance();
        assert_eq!(context.account_balance, UncToken::from_attounc(to_atto(LOCKUP_UNC) - amount));

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_deposit(amount.into());

        // Staking on the staking pool
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.stake(amount.into());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_stake(amount.into());

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(contract.get_known_deposited_balance().0, amount);
        context.view_config = None;

        // Assuming there are 20 UNC tokens in rewards. Refreshing balance.
        let total_balance = amount + to_atto(20);
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.refresh_staking_pool_balance();

        // In unit tests, the following call ignores the promise value, because it's passed directly.
        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_get_account_total_balance(total_balance.into());

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_known_deposited_balance().0, total_balance);
        assert_eq!(contract.get_owners_balance().0, to_atto(20));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(20));
        context.view_config = None;

        // Withdrawing these tokens
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        let transfer_amount = to_atto(15);
        contract.transfer(transfer_amount.into(), non_owner());
        context.account_balance = env::account_balance();

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_known_deposited_balance().0, total_balance);
        assert_eq!(contract.get_owners_balance().0, to_atto(5));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(5));
        context.view_config = None;
    }

    #[test]
    #[should_panic(expected = "Staking pool is already selected")]
    fn test_staking_pool_selected_again() {
        let (mut context, mut contract) = lockup_only_setup();
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(2).try_into().unwrap();

        // Selecting staking pool
        let staking_pool: AccountId = "staking_pool".parse().unwrap();
        testing_env!(context.clone());
        contract.select_staking_pool(staking_pool.clone());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(b"true".to_vec())],
        );
        contract.on_whitelist_is_whitelisted(true, staking_pool.clone());

        // Selecting another staking pool
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.select_staking_pool("staking_pool_2".parse().unwrap());
    }

    #[test]
    #[should_panic(expected = "The given staking pool account ID is not whitelisted")]
    fn test_staking_pool_not_whitelisted() {
        let (mut context, mut contract) = lockup_only_setup();
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(2).try_into().unwrap();

        // Selecting staking pool
        let staking_pool: AccountId = "staking_pool".parse().unwrap();
        testing_env!(context.clone());
        contract.select_staking_pool(staking_pool.clone());

        context.predecessor_account_id = lockup_account();
        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(b"false".to_vec())],
        );
        contract.on_whitelist_is_whitelisted(false, staking_pool.clone());
    }

    #[test]
    #[should_panic(expected = "Staking pool is not selected")]
    fn test_staking_pool_unselecting_non_selected() {
        let (mut context, mut contract) = lockup_only_setup();
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(2).try_into().unwrap();

        // Unselecting staking pool
        testing_env!(context.clone());
        contract.unselect_staking_pool();
    }

    #[test]
    #[should_panic(expected = "There is still a deposit on the staking pool")]
    fn test_staking_pool_unselecting_with_deposit() {
        let (mut context, mut contract) = lockup_only_setup();
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(2).try_into().unwrap();

        // Selecting staking pool
        let staking_pool: AccountId = "staking_pool".parse().unwrap();
        testing_env!(context.clone());
        contract.select_staking_pool(staking_pool.clone());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(b"true".to_vec())],
        );
        contract.on_whitelist_is_whitelisted(true, staking_pool.clone());

        // Deposit to the staking_pool
        let amount = to_atto(LOCKUP_UNC - 100);
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.deposit_to_staking_pool(amount.into());
        context.account_balance = env::account_balance();

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_deposit(amount.into());

        // Unselecting staking pool
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.unselect_staking_pool();
    }

    #[test]
    fn test_staking_pool_owner_balance() {
        let (mut context, mut contract) = lockup_only_setup();
        context.predecessor_account_id = account_owner();
        context.signer_account_id = account_owner();
        context.signer_account_pk = public_key(2).try_into().unwrap();
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR + 1);

        let lockup_amount = to_atto(LOCKUP_UNC);
        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, lockup_amount);
        context.view_config = None;

        // Selecting staking pool
        let staking_pool: AccountId = "staking_pool".parse().unwrap();
        testing_env!(context.clone());
        contract.select_staking_pool(staking_pool.clone());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(b"true".to_vec())],
        );
        contract.on_whitelist_is_whitelisted(true, staking_pool.clone());

        // Deposit to the staking_pool
        let mut total_amount = 0;
        let amount = to_atto(100);
        for _ in 1..=5 {
            total_amount += amount;
            context.predecessor_account_id = account_owner();
            testing_env!(context.clone());
            contract.deposit_to_staking_pool(amount.into());
            context.account_balance = env::account_balance();
            assert_eq!(context.account_balance, UncToken::from_attounc(lockup_amount - total_amount));

            context.predecessor_account_id = lockup_account();
            testing_env!(
                context.clone(),
                test_vm_config(),
                RuntimeFeesConfig::test(),
                Default::default(),
                vec![PromiseResult::Successful(vec![])],
            );
            contract.on_staking_pool_deposit(amount.into());
            context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
            testing_env!(context.clone());
            assert_eq!(contract.get_known_deposited_balance().0, total_amount);
            assert_eq!(contract.get_owners_balance().0, lockup_amount);
            assert_eq!(
                contract.get_liquid_owners_balance().0,
                lockup_amount - total_amount - MIN_BALANCE_FOR_STORAGE
            );
            context.view_config = None;
        }

        // Withdrawing from the staking_pool. Plus one extra time as a reward
        let mut total_withdrawn_amount = 0;
        for _ in 1..=6 {
            total_withdrawn_amount += amount;
            context.predecessor_account_id = account_owner();
            testing_env!(context.clone());
            contract.withdraw_from_staking_pool(amount.into());
            context.account_balance = context.account_balance.saturating_add(UncToken::from_attounc(amount));
            assert_eq!(
                context.account_balance,
                UncToken::from_attounc(lockup_amount - total_amount + total_withdrawn_amount)
            );

            context.predecessor_account_id = lockup_account();
            testing_env!(
                context.clone(),
                test_vm_config(),
                RuntimeFeesConfig::test(),
                Default::default(),
                vec![PromiseResult::Successful(vec![])],
            );
            contract.on_staking_pool_withdraw(amount.into());
            context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
            testing_env!(context.clone());
            assert_eq!(
                contract.get_known_deposited_balance().0,
                total_amount.saturating_sub(total_withdrawn_amount)
            );
            assert_eq!(
                contract.get_owners_balance().0,
                lockup_amount + total_withdrawn_amount.saturating_sub(total_amount)
            );
            assert_eq!(
                contract.get_liquid_owners_balance().0,
                lockup_amount - total_amount + total_withdrawn_amount - MIN_BALANCE_FOR_STORAGE
            );
            context.view_config = None;
        }
    }

    #[test]
    fn test_lock_timestmap() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let contract = LockupContract::new(
            account_owner(),
            0.into(),
            Some(to_ts(GENESIS_TIME_IN_DAYS + YEAR).into()),
            TransfersInformation::TransfersDisabled {
                transfer_poll_account_id: "transfers".parse().unwrap(),
            },
            None,
            None,
           "whitelist".parse().unwrap(),
            None,
        );

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(
            contract.get_locked_vested_amount(no_vesting_schedule()).0,
            to_atto(1000)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));
        assert_eq!(
            contract.get_unvested_amount(no_vesting_schedule()).0,
            to_atto(0)
        );
        assert!(!contract.are_transfers_enabled());

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(
            contract.get_locked_vested_amount(no_vesting_schedule()).0,
            to_atto(1000)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));
        assert_eq!(
            contract.get_unvested_amount(no_vesting_schedule()).0,
            to_atto(0)
        );
    }

    #[test]
    fn test_lock_timestmap_transfer_enabled() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let contract = LockupContract::new(
            account_owner(),
            0.into(),
            Some(to_ts(GENESIS_TIME_IN_DAYS + YEAR).into()),
            TransfersInformation::TransfersEnabled {
                transfers_timestamp: to_ts(GENESIS_TIME_IN_DAYS + YEAR / 2).into(),
            },
            None,
            None,
           "whitelist".parse().unwrap(),
            None,
        );

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(1000));
        assert_eq!(
            contract.get_liquid_owners_balance().0,
            to_atto(1000) - MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract.get_locked_vested_amount(no_vesting_schedule()).0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(0));
        assert_eq!(
            contract.get_unvested_amount(no_vesting_schedule()).0,
            to_atto(0)
        );
    }

    #[test]
    fn test_termination_no_staking() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(0);
        let mut contract = LockupContract::new(
            account_owner(),
            to_nanos(YEAR).into(),
            None,
            TransfersInformation::TransfersEnabled {
                transfers_timestamp: to_ts(GENESIS_TIME_IN_DAYS).into(),
            },
            Some(VestingScheduleOrHash::VestingSchedule(
                vesting_schedule.clone(),
            )),
            None,
           "whitelist".parse().unwrap(),
            Some(account_foundation()),
        );

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(
            contract.get_vesting_information(),
            VestingInformation::VestingSchedule(vesting_schedule.clone())
        );
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(750)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(500));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(500));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(500));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(500)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 2 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(750));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(750));
        assert_eq!(contract.get_locked_amount().0, to_atto(250));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(250)
        );

        // Terminating
        context.view_config = None;
        context.predecessor_account_id = account_foundation();
        context.signer_account_pk = public_key(3).into();
        testing_env!(context.clone());
        contract.terminate_vesting(None);

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(
            contract.get_vesting_information(),
            VestingInformation::Terminating(TerminationInformation {
                unvested_amount: to_atto(250).into(),
                status: TerminationStatus::ReadyToWithdraw,
            })
        );
        assert_eq!(contract.get_owners_balance().0, to_atto(750));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(750));
        assert_eq!(contract.get_locked_amount().0, to_atto(250));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(250)
        );
        assert_eq!(contract.get_terminated_unvested_balance().0, to_atto(250));
        assert_eq!(
            contract.get_terminated_unvested_balance_deficit().0,
            to_atto(0)
        );
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::ReadyToWithdraw)
        );

        // Withdrawing
        context.view_config = None;
        testing_env!(context.clone());
        let receiver_id: AccountId = "unc".parse().unwrap();
        contract.termination_withdraw(receiver_id.clone());
        context.account_balance = env::account_balance();

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_withdraw_unvested_amount(to_atto(250).into(), receiver_id);

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(750));
        assert_eq!(
            contract.get_liquid_owners_balance().0,
            to_atto(750) - MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(0)
        );
        assert_eq!(contract.get_terminated_unvested_balance().0, to_atto(0));
        assert_eq!(contract.get_termination_status(), None);
        assert_eq!(contract.get_vesting_information(), VestingInformation::None);
    }

    #[test]
    fn test_release_duration() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let contract = new_contract(true, None, Some(to_nanos(4 * YEAR).into()), false);

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(
            contract.get_locked_vested_amount(no_vesting_schedule()).0,
            to_atto(1000)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));
        assert_eq!(
            contract.get_unvested_amount(no_vesting_schedule()).0,
            to_atto(0)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(0));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(0));
        assert_eq!(
            contract.get_locked_vested_amount(no_vesting_schedule()).0,
            to_atto(1000)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 2 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(250));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(250));
        assert_eq!(
            contract.get_locked_vested_amount(no_vesting_schedule()).0,
            to_atto(750)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(750));

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 3 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(500));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(500));
        assert_eq!(
            contract.get_locked_vested_amount(no_vesting_schedule()).0,
            to_atto(500)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(500));

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 4 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(750));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(750));
        assert_eq!(contract.get_locked_amount().0, to_atto(250));
    }

    #[test]
    fn test_vesting_and_release_duration() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(0);
        let contract = new_contract_with_lockup_duration(
            true,
            Some(vesting_schedule.clone()),
            Some(to_nanos(4 * YEAR).into()),
            true,
            0,
        );

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(750)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(250));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(250));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(750));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(500)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 2 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(500));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(500));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(500));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(250)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 3 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(750));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(750));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(250));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(0)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 4 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(1000));
        assert_eq!(
            contract.get_liquid_owners_balance().0,
            to_atto(1000) - MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(0));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(0)
        );
    }

    // Vesting post transfers is not supported by Hash vesting.
    #[test]
    fn test_vesting_post_transfers_and_release_duration() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(YEAR * 2);
        let contract = LockupContract::new(
            account_owner(),
            to_nanos(YEAR).into(),
            None,
            TransfersInformation::TransfersEnabled {
                transfers_timestamp: to_ts(GENESIS_TIME_IN_DAYS).into(),
            },
            Some(VestingScheduleOrHash::VestingSchedule(
                vesting_schedule.clone(),
            )),
            Some(to_nanos(4 * YEAR).into()),
            "whitelist".parse().unwrap(),
            Some(account_foundation()),
        );

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(1000)
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(0));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(0));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(1000)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 2 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(250));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(250));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(750));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(750)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 3 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(500));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(500));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(500));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(500)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 4 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(750));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(750));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(250));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(250)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 5 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(1000));
        assert_eq!(
            contract.get_liquid_owners_balance().0,
            to_atto(1000) - MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(0));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(0)
        );
    }

    #[test]
    fn test_termination_no_staking_with_release_duration() {
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(0);
        let mut contract = new_contract_with_lockup_duration(
            true,
            Some(vesting_schedule.clone()),
            Some(to_nanos(4 * YEAR).into()),
            true,
            0,
        );

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(1000));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(750)
        );

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 2 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(500));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(500));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(500));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(250)
        );

        // Terminating
        context.view_config = None;
        context.predecessor_account_id = account_foundation();
        context.signer_account_pk = public_key(3).into();
        testing_env!(context.clone());
        contract.terminate_vesting(Some(VestingScheduleWithSalt {
            vesting_schedule: vesting_schedule.clone(),
            salt: SALT.to_vec().into(),
        }));

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(500));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(500));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_locked_amount().0, to_atto(500));
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(250)
        );
        assert_eq!(contract.get_terminated_unvested_balance().0, to_atto(250));
        assert_eq!(
            contract.get_terminated_unvested_balance_deficit().0,
            to_atto(0)
        );
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::ReadyToWithdraw)
        );

        // Withdrawing
        context.view_config = None;
        testing_env!(context.clone());
        let receiver_id: AccountId = "unc".parse().unwrap();
        contract.termination_withdraw(receiver_id.clone());
        context.account_balance = env::account_balance();

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_withdraw_unvested_amount(to_atto(250).into(), receiver_id);

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(500));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(500));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(0)
        );
        assert_eq!(contract.get_terminated_unvested_balance().0, to_atto(0));
        assert_eq!(contract.get_termination_status(), None);

        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 3 * YEAR);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(750));
        assert_eq!(
            contract.get_liquid_owners_balance().0,
            to_atto(750) - MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(0)
        );
    }

    #[test]
    fn test_termination_before_cliff() {
        let lockup_amount = to_atto(1000);
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(YEAR);
        let mut contract = new_contract(true, Some(vesting_schedule.clone()), None, true);

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(
            contract.get_vesting_information(),
            VestingInformation::VestingHash(
                VestingScheduleWithSalt {
                    vesting_schedule: vesting_schedule.clone(),
                    salt: SALT.to_vec().into(),
                }
                    .hash()
                    .into()
            )
        );
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(contract.get_locked_amount().0, lockup_amount);
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            lockup_amount
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            0
        );

        // Terminating
        context.view_config = None;
        context.predecessor_account_id = account_foundation();
        context.signer_account_pk = public_key(3).into();
        testing_env!(context.clone());
        contract.terminate_vesting(Some(VestingScheduleWithSalt {
            vesting_schedule: vesting_schedule.clone(),
            salt: SALT.to_vec().into(),
        }));

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(
            contract.get_vesting_information(),
            VestingInformation::Terminating(TerminationInformation {
                unvested_amount: lockup_amount.into(),
                status: TerminationStatus::ReadyToWithdraw,
            })
        );
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(contract.get_locked_amount().0, lockup_amount);
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            lockup_amount
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            0
        );
        assert_eq!(contract.get_terminated_unvested_balance().0, lockup_amount);
        assert_eq!(
            contract.get_terminated_unvested_balance_deficit().0,
            MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::ReadyToWithdraw)
        );

        // Withdrawing
        context.view_config = None;
        testing_env!(context.clone());
        let receiver_id: AccountId = account_foundation();
        contract.termination_withdraw(receiver_id.clone());
        context.account_balance = env::account_balance();
        assert_eq!(context.account_balance, UncToken::from_attounc(MIN_BALANCE_FOR_STORAGE));

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_withdraw_unvested_amount(
            (lockup_amount - MIN_BALANCE_FOR_STORAGE).into(),
            receiver_id,
        );

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(contract.get_owners_balance().0, 0);
        assert_eq!(contract.get_liquid_owners_balance().0, 0);
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            0
        );
        assert_eq!(
            contract.get_terminated_unvested_balance().0,
            MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract.get_terminated_unvested_balance_deficit().0,
            MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::ReadyToWithdraw)
        );
    }

    #[test]
    fn test_termination_with_staking() {
        let lockup_amount = to_atto(1000);
        let mut context = basic_context();
        testing_env!(context.clone());
        let vesting_schedule = new_vesting_schedule(0);
        let mut contract = new_contract(true, Some(vesting_schedule.clone()), None, true);

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(0));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(0));
        assert_eq!(contract.get_locked_amount().0, lockup_amount);
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(750)
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        context.view_config = None;

        context.predecessor_account_id = account_owner();
        context.signer_account_pk = public_key(2).into();
        testing_env!(context.clone());

        // Selecting staking pool
        let staking_pool: AccountId = "staking_pool".parse().unwrap();
        testing_env!(context.clone());
        contract.select_staking_pool(staking_pool.clone());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(b"true".to_vec())],
        );
        contract.on_whitelist_is_whitelisted(true, staking_pool.clone());

        // Deposit to the staking_pool
        let stake_amount = to_atto(LOCKUP_UNC - 100);
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.deposit_to_staking_pool(stake_amount.into());
        context.account_balance = env::account_balance();

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_deposit(stake_amount.into());

        // Staking on the staking pool
        context.predecessor_account_id = account_owner();
        testing_env!(context.clone());
        contract.stake(stake_amount.into());

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_stake(stake_amount.into());

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(0));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(0));
        assert_eq!(contract.get_known_deposited_balance().0, stake_amount);
        assert_eq!(contract.get_locked_amount().0, lockup_amount);
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(750)
        );

        // Foundation terminating
        context.view_config = None;
        context.predecessor_account_id = account_foundation();
        context.signer_account_pk = public_key(3).into();
        testing_env!(context.clone());
        contract.terminate_vesting(Some(VestingScheduleWithSalt {
            vesting_schedule: vesting_schedule.clone(),
            salt: SALT.to_vec().into(),
        }));

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(0));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(0));
        assert_eq!(contract.get_locked_amount().0, lockup_amount);
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(750)
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_terminated_unvested_balance().0, to_atto(750));
        assert_eq!(
            contract.get_terminated_unvested_balance_deficit().0,
            to_atto(650) + MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::VestingTerminatedWithDeficit)
        );

        // Proceeding with unstaking from the pool due to termination.
        context.view_config = None;
        testing_env!(context.clone());
        contract.termination_prepare_to_withdraw();
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::UnstakingInProgress)
        );

        let stake_amount_with_rewards = stake_amount + to_atto(50);
        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(format!("{}", stake_amount_with_rewards).into_bytes())],
        );
        contract.on_get_account_staked_balance_to_unstake(stake_amount_with_rewards.into());

        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_staking_pool_unstake_for_termination(stake_amount_with_rewards.into());

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::EverythingUnstaked)
        );

        // Proceeding with withdrawing from the pool due to termination.
        context.view_config = None;
        context.predecessor_account_id = account_foundation();
        testing_env!(context.clone());
        contract.termination_prepare_to_withdraw();
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::WithdrawingFromStakingPoolInProgress)
        );

        let withdraw_amount_with_extra_rewards = stake_amount_with_rewards + to_atto(1);
        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(format!("{}", withdraw_amount_with_extra_rewards).into_bytes())],
        );
        contract
            .on_get_account_unstaked_balance_to_withdraw(withdraw_amount_with_extra_rewards.into());
        context.account_balance = context.account_balance.saturating_add(UncToken::from_attounc(withdraw_amount_with_extra_rewards));

        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract
            .on_staking_pool_withdraw_for_termination(withdraw_amount_with_extra_rewards.into());

            context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(51));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(51));
        assert_eq!(contract.get_locked_amount().0, lockup_amount);
        assert_eq!(
            contract.get_unvested_amount(vesting_schedule.clone()).0,
            to_atto(750)
        );
        assert_eq!(contract.get_terminated_unvested_balance().0, to_atto(750));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_terminated_unvested_balance_deficit().0, 0);
        assert_eq!(contract.get_known_deposited_balance().0, 0);
        assert_eq!(
            contract.get_termination_status(),
            Some(TerminationStatus::ReadyToWithdraw)
        );

        // Withdrawing
        context.view_config = None;
        context.predecessor_account_id = account_foundation();
        testing_env!(context.clone());
        let receiver_id = account_foundation();
        contract.termination_withdraw(receiver_id.clone());
        context.account_balance = env::account_balance();
        assert_eq!(context.account_balance, UncToken::from_attounc(to_atto(250 + 51)));

        context.predecessor_account_id = lockup_account();
        testing_env!(
            context.clone(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_withdraw_unvested_amount(to_atto(750).into(), receiver_id);

        context.view_config = Some(ViewConfig { max_gas_burnt: 200000000000000 });
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(51));
        assert_eq!(contract.get_liquid_owners_balance().0, to_atto(51));
        assert_eq!(contract.get_locked_amount().0, to_atto(250));
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            to_atto(250)
        );
        assert_eq!(contract.get_unvested_amount(vesting_schedule.clone()).0, 0);
        assert_eq!(contract.get_terminated_unvested_balance().0, 0);
        assert_eq!(contract.get_terminated_unvested_balance_deficit().0, 0);
        assert_eq!(contract.get_termination_status(), None);

        // Checking the balance becomes unlocked later
        context.block_timestamp = to_ts(GENESIS_TIME_IN_DAYS + YEAR + 1);
        testing_env!(context.clone());
        assert_eq!(contract.get_owners_balance().0, to_atto(301));
        assert_eq!(
            contract.get_liquid_owners_balance().0,
            to_atto(301) - MIN_BALANCE_FOR_STORAGE
        );
        assert_eq!(
            contract
                .get_locked_vested_amount(vesting_schedule.clone())
                .0,
            0
        );
        assert_eq!(contract.get_locked_amount().0, 0);
    }
}
