use unc_sdk::AccountId;

pub fn staking() -> AccountId {
    "staking".parse().unwrap()
}

pub fn alice() -> AccountId {
    "alice".parse().unwrap()
}
pub fn bob() -> AccountId {
    "bob".parse().unwrap()
}
pub fn owner() -> AccountId {
    "owner".parse().unwrap()
}

pub fn ntoy(unc_amount: u128) -> u128 {
    unc_amount * 10u128.pow(24)
}

/// Rounds to uncest
pub fn yton(atto_amount: u128) -> u128 {
    (atto_amount + (5 * 10u128.pow(23))) / 10u128.pow(24)
}

#[macro_export]
macro_rules! assert_eq_in_unc {
    ($a:expr, $b:expr) => {
        assert_eq!(yton($a), yton($b))
    };
    ($a:expr, $b:expr, $c:expr) => {
        assert_eq!(yton($a), yton($b), $c)
    };
}