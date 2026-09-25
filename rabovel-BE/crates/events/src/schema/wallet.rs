use serde::{Deserialize, Serialize};

use crate::{envelope::DomainEvent, topics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletLinked {
    pub user_id: String,
    pub wallet_id: String,
    pub chain: String,
    pub provider: String,
    pub verified_at: u64,
}

impl DomainEvent for WalletLinked {
    const EVENT_TYPE: &'static str = "wallet.linked";
    const TOPIC: &'static str = topics::WALLET_LINKED_V1;
}
