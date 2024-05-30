mod account;
mod api;
mod legacy;
mod node;
mod permission;
mod shared_storage;
mod storage_tracker;
mod upgrade;
mod utils;

pub use crate::account::*;
pub use crate::api::*;
use crate::legacy::*;
pub use crate::node::*;
pub use crate::permission::*;
pub use crate::shared_storage::*;
use crate::storage_tracker::*;
use crate::utils::*;

use unc_sdk::borsh::{BorshDeserialize, BorshSerialize};
use unc_sdk::serde::{Deserialize, Serialize};

use unc_sdk::collections::{LookupMap, UnorderedMap};
use unc_sdk::{
    assert_one_atto, env, require, unc_bindgen, AccountId, BorshStorageKey, PanicOnDefault,
    Promise, StorageUsage, UncToken,
};

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "unc_sdk::borsh")]
#[allow(unused)]
enum StorageKey {
    Account,
    Nodes,
    Node { node_id: NodeId },
    Permissions { node_id: NodeId },
    SharedStoragePools,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Copy, Clone)]
#[borsh(crate = "unc_sdk::borsh")]
#[serde(crate = "unc_sdk::serde")]
pub enum ContractStatus {
    Genesis,
    Live,
    ReadOnly,
}

pub type NodeId = u32;

#[unc_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "unc_sdk::borsh")]
pub struct Contract {
    pub accounts: LookupMap<NodeId, VAccount>,
    pub root_node: Node,
    pub nodes: LookupMap<NodeId, VNode>,
    pub node_count: NodeId,
    pub status: ContractStatus,
    pub shared_storage_pools: LookupMap<AccountId, VSharedStoragePool>,
}

#[unc_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            accounts: LookupMap::new(StorageKey::Account),
            root_node: Node::new(0, None),
            nodes: LookupMap::new(StorageKey::Nodes),
            node_count: 1,
            status: ContractStatus::Genesis,
            shared_storage_pools: LookupMap::new(StorageKey::SharedStoragePools),
        }
    }

    #[private]
    pub fn set_status(&mut self, status: ContractStatus) {
        require!(
            !matches!(status, ContractStatus::Genesis),
            "The status can't be set to Genesis"
        );
        self.status = status;
    }

    pub fn get_status(&self) -> ContractStatus {
        self.status
    }
}

impl Contract {
    pub fn create_node_id(&mut self) -> NodeId {
        let node_id = self.node_count;
        self.node_count += 1;
        node_id
    }

    pub fn assert_live(&self) {
        require!(
            matches!(self.status, ContractStatus::Live),
            "The contract status is not Live"
        );
    }
}
