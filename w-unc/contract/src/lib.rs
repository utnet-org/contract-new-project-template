/*!
* w-xxx UIP-141 Token contract
*
* The aim of the contract is to enable the wrapping of the native UNC token into a UIP-141 compatible token.
* It supports methods `unc_deposit` and `unc_withdraw` that wraps and unwraps UNC tokens.
* They are effectively mint and burn underlying wUNC tokens.
*
* lib.rs is the main entry point.
* w_unc.rs contains interfaces for depositing and withdrawing
*/
use unc_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use unc_contract_standards::fungible_token::FungibleToken;
use unc_sdk::json_types::U128;
use unc_sdk::{unc, PanicOnDefault};

mod legacy_storage;
mod w_unc;

#[unc(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub ft: FungibleToken,
}

#[unc]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            ft: FungibleToken::new(b"a".to_vec()),
        }
    }
}

#[unc]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: String::from("Wrapped UNC fungible token"),
            symbol: String::from("wUNC"),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 24,
        }
    }
}
