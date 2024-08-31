use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::error::Error;
use std::time::{Duration, Instant};

pub struct TroubleshootResults {
    pub connection_status: bool,
    pub version_mismatch: bool,
    pub high_latency: bool,
    pub network_congestion: bool,
}

impl Default for TroubleshootResults {
    fn default() -> Self {
        TroubleshootResults {
            connection_status: false,
            version_mismatch: false,
            high_latency: false,
            network_congestion: false,
        }
    }
}

pub fn run_troubleshoot(url: &str) -> Result<TroubleshootResults, Box<dyn Error>> {
    let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());
    let mut results = TroubleshootResults::default();

    results.connection_status = client.get_health().is_ok();

    if let Ok(version) = client.get_version() {
        // Assuming the expected version is "1.14.0"
        results.version_mismatch = version.solana_core != "1.14.0";
    }

    let start = Instant::now();
    if client.get_slot().is_ok() {
        let latency = start.elapsed();
        results.high_latency = latency > Duration::from_millis(500);
    }

    if let Ok(recent_performance) = client.get_recent_performance_samples(Some(1)) {
        if let Some(latest) = recent_performance.first() {
            let tps = latest.num_transactions as f64 / latest.sample_period_secs as f64;
            // Assuming network congestion if TPS is above 1500
            results.network_congestion = tps > 1500.0;
        }
    }

    Ok(results)
}

pub fn print_troubleshoot_results(results: &TroubleshootResults) {
    println!("Troubleshoot Results:");
    println!("Connection Status: {}", if results.connection_status { "OK" } else { "Failed" });
    println!("Version Mismatch: {}", if results.version_mismatch { "Yes" } else { "No" });
    println!("High Latency: {}", if results.high_latency { "Yes" } else { "No" });
    println!("Network Congestion: {}", if results.network_congestion { "Yes" } else { "No" });
}
