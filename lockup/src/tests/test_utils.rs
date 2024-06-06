use unc_sdk::AccountId;

pub const LOCKUP_UNC: u128 = 1000;
pub const GENESIS_TIME_IN_DAYS: u64 = 500;
pub const YEAR: u64 = 365;

pub fn lockup_account() -> AccountId {
    "lockup".parse().unwrap()
}

pub fn system_account() -> AccountId {
    "system".parse().unwrap()
}

pub fn account_owner() -> AccountId {
    "account_owner".parse().unwrap()
}

pub fn non_owner() -> AccountId {
    "non_owner".parse().unwrap()
}

pub fn account_foundation() -> AccountId {
    "unc".parse().unwrap()
}

pub fn to_atto(unc_balance: u128) -> u128 {
    unc_balance * 10u128.pow(24)
}

pub fn to_nanos(num_days: u64) -> u64 {
    num_days * 86400_000_000_000
}

pub fn to_ts(num_days: u64) -> u64 {
    // 2018-08-01 UTC in nanoseconds
    1533081600_000_000_000 + to_nanos(num_days)
}

pub fn assert_almost_eq_with_max_delta(left: u128, right: u128, max_delta: u128) {
    assert!(
        std::cmp::max(left, right) - std::cmp::min(left, right) < max_delta,
        "{}",
        format!(
            "Left {} is not even close to Right {} within delta {}",
            left, right, max_delta
        )
    );
}

pub fn assert_almost_eq(left: u128, right: u128) {
    assert_almost_eq_with_max_delta(left, right, to_atto(10));
}