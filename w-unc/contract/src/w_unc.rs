use crate::*;
use unc_sdk::json_types::U128;
use unc_sdk::{assert_one_atto, env, log, Promise, UncToken};
use unc_contract_standards::storage_management::StorageManagement;

#[unc]
impl Contract {
    /// Deposit UNC to mint wUNC tokens to the predecessor account in this contract.
    /// Requirements:
    /// * The predecessor account doesn't need to be registered.
    /// * Requires positive attached deposit.
    /// * If account is not registered will fail if attached deposit is below registration limit.
    #[payable]
    pub fn unc_deposit(&mut self) {
        let mut supply = env::attached_deposit();
        assert!(supply > UncToken::from_attounc(0), "Requires positive attached deposit");
        let owner_id = env::predecessor_account_id();
        if !self.token.accounts.contains_key(&owner_id) {
            // Not registered, register if enough $UNC has been attached.
            // Subtract registration amount from the account balance.
            assert!(
                supply >= self.token.storage_balance_bounds().min,
                "ERR_DEPOSIT_TOO_SMALL"
            );
            self.token.internal_register_account(&owner_id);
            supply = supply.checked_sub(self.token.storage_balance_bounds().min).unwrap_or(UncToken::from_attounc(0));
        }
        self.token.internal_deposit(&owner_id, supply.as_attounc());

        unc_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: supply.as_attounc().into(),
            memo: Some("new tokens are minted"),
        }
        .emit();

        log!("Deposit {} UNC to {}", supply, owner_id);
    }

    /// Withdraws wUNC and send UNC back to the predecessor account.
    /// Requirements:
    /// * The predecessor account should be registered.
    /// * `amount` must be a positive integer.
    /// * The predecessor account should have at least the `amount` of wUnc tokens.
    /// * Requires attached deposit of exactly 1 attoUNC.
    #[payable]
    pub fn unc_withdraw(&mut self, amount: U128) -> Promise {
        assert_one_atto();
        let account_id = env::predecessor_account_id();
        let amount = amount.into();
        self.token.internal_withdraw(&account_id, amount);
        log!("Withdraw {} attoUNC from {}", amount, account_id);
        // Transferring UNC and refunding 1 attoUNC.
        Promise::new(account_id).transfer(UncToken::from_attounc(amount + 1))
    }

}
