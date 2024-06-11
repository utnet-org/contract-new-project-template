use crate::*;

/// Information about a specific public key. Should be returned in the `get_key_information` view method.
/// Part of the airdrop UIP
#[unc(serializers=[borsh, json])]
pub struct KeyInfo {
    /// attounc$ amount that will be sent to the claiming account (either new or existing)
    /// when the key is successfully used.
    pub balance: U128,
}

#[unc(serializers=[borsh, json])]
/// Information about any limited access keys that are being added to the account as part of `create_account_advanced`.
pub struct LimitedAccessKey {
    /// The public key of the limited access key.
    pub public_key: PublicKey,
    /// The amount of attounc$ that can be spent on Gas by this key.
    pub allowance: UncToken,
    /// Which contract should this key be allowed to call.
    pub receiver_id: AccountId,
    /// Which methods should this key be allowed to call.
    pub method_names: String,
}

#[unc(serializers=[borsh, json])]
/// Options for `create_account_advanced`.
pub struct CreateAccountOptions {
    pub full_access_keys: Option<Vec<PublicKey>>,
    pub limited_access_keys: Option<Vec<LimitedAccessKey>>,
    pub contract_bytes: Option<Vec<u8>>,
}
