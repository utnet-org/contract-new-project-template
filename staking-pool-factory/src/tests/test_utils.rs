use unc_sdk::AccountId;

pub fn account_unc() -> AccountId {
    "unc".parse().unwrap()
}
pub fn account_whitelist() -> AccountId {
    "whitelist".parse().unwrap()
}
pub fn staking_pool_id() -> AccountId {
    "pool".parse().unwrap()
}
pub fn account_pool() -> AccountId {
    "pool.factory".parse().unwrap()
}
pub fn account_factory() -> AccountId {
    "factory".parse().unwrap()
}
pub fn account_tokens_owner() -> AccountId {
    "tokens-owner".parse().unwrap()
}
pub fn account_pool_owner() -> AccountId {
    "pool-owner".parse().unwrap()
}

pub fn ntoy(unc_amount: u128) -> u128 {
    unc_amount * 10u128.pow(24)
}