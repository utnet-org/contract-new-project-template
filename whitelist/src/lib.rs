use unc_sdk::store::LookupSet;
use unc_sdk::{env, unc, AccountId};

#[unc(contract_state)]
pub struct WhitelistContract {
    /// The account ID of the UNC Foundation. It allows to whitelist new staking pool accounts.
    /// It also allows to whitelist new Staking Pool Factories, which can whitelist staking pools.
    pub foundation_account_id: AccountId,

    /// The whitelisted account IDs of approved staking pool contracts.
    pub whitelist: LookupSet<AccountId>,

    /// The whitelist of staking pool factories. Any account from this list can whitelist staking
    /// pools.
    pub factory_whitelist: LookupSet<AccountId>,
}

impl Default for WhitelistContract {
    fn default() -> Self {
        env::panic_str("The contract should be initialized before usage")
    }
}

#[unc]
impl WhitelistContract {
    /// Initializes the contract with the given UNC foundation account ID.
    #[init]
    pub fn new(foundation_account_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert!(
            env::is_valid_account_id(foundation_account_id.as_bytes()),
            "The UNC Foundation account ID is invalid"
        );
        Self {
            foundation_account_id,
            whitelist: LookupSet::new(b"w".to_vec()),
            factory_whitelist: LookupSet::new(b"f".to_vec()),
        }
    }

    /***********/
    /* Getters */
    /***********/

    /// Returns `true` if the given staking pool account ID is whitelisted.
    pub fn is_whitelisted(&self, staking_pool_account_id: AccountId) -> bool {
        assert!(
            env::is_valid_account_id(staking_pool_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.whitelist.contains(&staking_pool_account_id)
    }

    /// Returns `true` if the given factory contract account ID is whitelisted.
    pub fn is_factory_whitelisted(&self, factory_account_id: AccountId) -> bool {
        assert!(
            env::is_valid_account_id(factory_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.factory_whitelist.contains(&factory_account_id)
    }

    /************************/
    /* Factory + Foundation */
    /************************/

    /// Adds the given staking pool account ID to the whitelist.
    /// Returns `true` if the staking pool was not in the whitelist before, `false` otherwise.
    /// This method can be called either by the UNC foundation or by a whitelisted factory.
    pub fn add_staking_pool(&mut self, staking_pool_account_id: AccountId) -> bool {
        assert!(
            env::is_valid_account_id(staking_pool_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        // Can only be called by a whitelisted factory or by the foundation.
        if !self
            .factory_whitelist
            .contains(&env::predecessor_account_id())
        {
            self.assert_called_by_foundation();
        }
        self.whitelist.insert(staking_pool_account_id)
    }

    /**************/
    /* Foundation */
    /**************/

    /// Removes the given staking pool account ID from the whitelist.
    /// Returns `true` if the staking pool was present in the whitelist before, `false` otherwise.
    /// This method can only be called by the UNC foundation.
    pub fn remove_staking_pool(&mut self, staking_pool_account_id: AccountId) -> bool {
        self.assert_called_by_foundation();
        assert!(
            env::is_valid_account_id(staking_pool_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.whitelist.remove(&staking_pool_account_id)
    }

    /// Adds the given staking pool factory contract account ID to the factory whitelist.
    /// Returns `true` if the factory was not in the whitelist before, `false` otherwise.
    /// This method can only be called by the UNC foundation.
    pub fn add_factory(&mut self, factory_account_id: AccountId) -> bool {
        assert!(
            env::is_valid_account_id(factory_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.assert_called_by_foundation();
        self.factory_whitelist.insert(factory_account_id)
    }

    /// Removes the given staking pool factory account ID from the factory whitelist.
    /// Returns `true` if the factory was present in the whitelist before, `false` otherwise.
    /// This method can only be called by the UNC foundation.
    pub fn remove_factory(&mut self, factory_account_id: AccountId) -> bool {
        self.assert_called_by_foundation();
        assert!(
            env::is_valid_account_id(factory_account_id.as_bytes()),
            "The given account ID is invalid"
        );
        self.factory_whitelist.remove(&factory_account_id)
    }

    /************/
    /* Internal */
    /************/

    /// Internal method to verify the predecessor was the UNC Foundation account ID.
    fn assert_called_by_foundation(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.foundation_account_id,
            "Can only be called by UNC Foundation"
        );
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use unc_sdk::test_utils::VMContextBuilder;
    use unc_sdk::testing_env;

    mod test_utils;
    use test_utils::*;

    #[test]
    fn test_whitelist() {
        // Initialize the mocked blockchain
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .build());

        let mut contract = WhitelistContract::new(account_unc());

        // Check initial whitelist
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());
        assert!(!contract.is_whitelisted(account_pool()));

        // Adding to whitelist by foundation
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(false)
            .build());
        assert!(contract.add_staking_pool(account_pool()));

        // Checking it's whitelisted now
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());
        assert!(contract.is_whitelisted(account_pool()));

        // Adding again. Should return false
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(false)
            .build());
        assert!(!contract.add_staking_pool(account_pool()));

        // Checking the pool is still whitelisted
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());
        assert!(contract.is_whitelisted(account_pool()));

        // Removing from the whitelist.
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(false)
            .build());
        assert!(contract.remove_staking_pool(account_pool()));

        // Checking the pool is not whitelisted anymore
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());
        assert!(!contract.is_whitelisted(account_pool()));

        // Removing again from the whitelist, should return false.
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(false)
            .build());
        assert!(!contract.remove_staking_pool(account_pool()));

        // Checking the pool is still not whitelisted
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());
        assert!(!contract.is_whitelisted(account_pool()));

        // Adding again after it was removed. Should return true
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(false)
            .build());
        assert!(contract.add_staking_pool(account_pool()));

        // Checking the pool is now whitelisted again
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());
        assert!(contract.is_whitelisted(account_pool()));
    }

    #[test]
    #[should_panic(expected = "Can only be called by UNC Foundation")]
    fn test_factory_whitelist_fail() {
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .build());

        let mut contract = WhitelistContract::new(account_unc());

        // Trying ot add to the whitelist by NOT whitelisted factory.
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_factory())
            .is_view(false)
            .build());
        assert!(contract.add_staking_pool(account_pool()));
    }

    #[test]
    #[should_panic(expected = "Can only be called by UNC Foundation")]
    fn test_trying_to_whitelist_factory() {
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .build());

        let mut contract = WhitelistContract::new(account_unc());

        // Trying ot whitelist the factory not by the UNC Foundation.
        testing_env!(VMContextBuilder::new()
        .current_account_id(account_whitelist())
        .predecessor_account_id(account_factory())
        .is_view(false)
        .build());
        assert!(contract.add_factory(account_factory()));
    }

    #[test]
    #[should_panic(expected = "Can only be called by UNC Foundation")]
    fn test_trying_to_remove_by_factory() {
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .build());
        let mut contract = WhitelistContract::new(account_unc());

        // Adding factory
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(false)
            .build());
        assert!(contract.add_factory(account_factory()));
        // Trying to remove the pool by the factory.
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_factory())
            .build());
        assert!(contract.remove_staking_pool(account_pool()));
    }

    #[test]
    fn test_whitelist_factory() {
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .build());

        let mut contract = WhitelistContract::new(account_unc());

        // Check the factory is not whitelisted
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());
        assert!(!contract.is_factory_whitelisted(account_factory()));

        // Whitelisting factory
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(false)
            .build());
        assert!(contract.add_factory(account_factory()));

        // Check the factory is whitelisted now
        testing_env!(VMContextBuilder::new()
            .current_account_id(account_whitelist())
            .predecessor_account_id(account_unc())
            .is_view(true)
            .build());
        assert!(contract.is_factory_whitelisted(account_factory()));
        // Check the pool is not whitelisted
        assert!(!contract.is_whitelisted(account_pool()));

        // Adding to whitelist by foundation
        testing_env!(VMContextBuilder::new()
        .current_account_id(account_whitelist())
        .predecessor_account_id(account_factory())
        .is_view(false)
        .build());
        assert!(contract.add_staking_pool(account_pool()));

        // Checking it's whitelisted now
        testing_env!(VMContextBuilder::new()
        .current_account_id(account_whitelist())
        .predecessor_account_id(account_factory())
        .is_view(true)
        .build());
        assert!(contract.is_whitelisted(account_pool()));

        // Removing the pool from the whitelisted by the UNC foundation.
        testing_env!(VMContextBuilder::new()
        .current_account_id(account_whitelist())
        .predecessor_account_id(account_unc())
        .is_view(false)
        .build());
        assert!(contract.remove_staking_pool(account_pool()));

        // Checking the pool is not whitelisted anymore
        testing_env!(VMContextBuilder::new()
        .current_account_id(account_whitelist())
        .predecessor_account_id(account_unc())
        .is_view(true)
        .build());
        assert!(!contract.is_whitelisted(account_pool()));

        // Removing the factory
        testing_env!(VMContextBuilder::new()
        .current_account_id(account_whitelist())
        .predecessor_account_id(account_unc())
        .is_view(false)
        .build());
        assert!(contract.remove_factory(account_factory()));

        // Check the factory is not whitelisted anymore
        testing_env!(VMContextBuilder::new()
        .current_account_id(account_whitelist())
        .predecessor_account_id(account_unc())
        .is_view(true)
        .build());
        assert!(!contract.is_factory_whitelisted(account_factory()));
    }
}
