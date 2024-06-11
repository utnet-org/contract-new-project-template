use crate::*;
use unc_contract_standards::storage_management::StorageManagement;

#[unc]
impl Contract {
    pub fn storage_minimum_balance(&self) -> U128 {
        self.ft.storage_balance_bounds().min.as_attounc().into()
    }
}
