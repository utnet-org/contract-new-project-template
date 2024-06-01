use crate::*;
use unc_sdk::json_types::U128;
use unc_sdk::{assert_one_atto, env, log, Promise, UncToken};
use unc_contract_standards::storage_management::StorageManagement;

#[unc_bindgen]
impl Contract {
    /// Deposit UNC to mint wUNC tokens to the predecessor account in this contract.
    /// Requirements:
    /// * The predecessor account doesn't need to be registered.
    /// * Requires positive attached deposit.
    /// * If account is not registered will fail if attached deposit is below registration limit.
    #[payable]
    pub fn unc_deposit(&mut self) {
        let mut amount = env::attached_deposit();
        assert!(amount > UncToken::from_attounc(0), "Requires positive attached deposit");
        let account_id = env::predecessor_account_id();
        if !self.ft.accounts.contains_key(&account_id) {
            // Not registered, register if enough $UNC has been attached.
            // Subtract registration amount from the account balance.
            assert!(
                amount >= self.ft.storage_balance_bounds().min,
                "ERR_DEPOSIT_TOO_SMALL"
            );
            self.ft.internal_register_account(&account_id);
            amount = amount.checked_sub(self.ft.storage_balance_bounds().min).unwrap_or(UncToken::from_attounc(0));
        }
        self.ft.internal_deposit(&account_id, amount.as_attounc());
        log!("Deposit {} UNC to {}", amount, account_id);
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
        self.ft.internal_withdraw(&account_id, amount);
        log!("Withdraw {} attoUNC from {}", amount, account_id);
        // Transferring UNC and refunding 1 attoUNC.
        Promise::new(account_id).transfer(UncToken::from_attounc(amount + 1))
    }

    // view 
    pub fn ft_balance_of(&self, account_id: unc_sdk::AccountId) -> U128 {
        self.ft.internal_unwrap_balance_of(&account_id).into()
    }

    pub fn ft_transfer(&mut self, receiver_id: unc_sdk::AccountId, amount: U128, memo: Option<String>) {
        assert_one_atto();
        let sender_id = env::predecessor_account_id();
        self.ft.internal_transfer(&sender_id, &receiver_id, amount.into(), memo);
    }
    pub fn ft_total_supply(&self) -> U128 {
        self.ft.total_supply.into()
    }

}
