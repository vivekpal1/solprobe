use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NodeHealth {
    pub is_responsive: bool,
    pub version: Option<String>,
    pub current_slot: Option<u64>,
    pub current_epoch: Option<u64>,
    pub total_nodes: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NetworkPerformance {
    pub tps: f64,
    pub avg_block_time: Option<f64>,
    pub confirmation_time: Option<f64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TroubleshootResults {
    pub delinquent_validators: u64,
    pub empty_blocks: u64,
    pub large_accounts: u64,
}