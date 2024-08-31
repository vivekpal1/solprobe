use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::error::Error;

pub struct NodeHealth {
    pub is_responsive: bool,
    pub current_slot: Option<u64>,
    pub version: Option<String>,
    pub current_epoch: Option<u64>,
    pub total_nodes: Option<u64>,
}

impl Default for NodeHealth {
    fn default() -> Self {
        NodeHealth {
            is_responsive: false,
            current_slot: None,
            version: None,
            current_epoch: None,
            total_nodes: None,
        }
    }
}

pub fn get_node_health(url: &str) -> Result<NodeHealth, Box<dyn Error>> {
    let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());
    let mut health = NodeHealth::default();

    if client.get_health().is_ok() {
        health.is_responsive = true;
    }

    if let Ok(slot) = client.get_slot() {
        health.current_slot = Some(slot);
    }

    if let Ok(version) = client.get_version() {
        health.version = Some(version.solana_core);
    }

    if let Ok(epoch_info) = client.get_epoch_info() {
        health.current_epoch = Some(epoch_info.epoch);
    }

    if let Ok(validators) = client.get_vote_accounts() {
        health.total_nodes = Some((validators.current.len() + validators.delinquent.len()) as u64);
    }

    Ok(health)
}

pub fn run_node_health(url: &str) -> Result<(), Box<dyn Error>> {
    let health = get_node_health(url)?;

    println!("Node Health:");
    println!("Is Responsive: {}", health.is_responsive);
    if let Some(slot) = health.current_slot {
        println!("Current Slot: {}", slot);
    }
    if let Some(version) = health.version {
        println!("Version: {}", version);
    }
    if let Some(epoch) = health.current_epoch {
        println!("Current Epoch: {}", epoch);
    }
    if let Some(total_nodes) = health.total_nodes {
        println!("Total Nodes: {}", total_nodes);
    }

    Ok(())
}
