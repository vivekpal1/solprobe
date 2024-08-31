use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::error::Error;
use std::time::{Duration, Instant};

pub struct NetworkPerformance {
    pub tps: f64,
    pub avg_block_time: Option<f64>,
    pub confirmation_time: Option<f64>,
}

impl Default for NetworkPerformance {
    fn default() -> Self {
        NetworkPerformance {
            tps: 0.0,
            avg_block_time: None,
            confirmation_time: None,
        }
    }
}

pub fn get_network_performance(url: &str) -> Result<NetworkPerformance, Box<dyn Error>> {
    let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());
    let mut performance = NetworkPerformance::default();

    // Get TPS and average block time
    if let Ok(recent_performance) = client.get_recent_performance_samples(Some(1)) {
        if let Some(latest) = recent_performance.first() {
            performance.tps = latest.num_transactions as f64 / latest.sample_period_secs as f64;
            performance.avg_block_time = Some(latest.sample_period_secs as f64 / latest.num_slots as f64);
        }
    }

    // Estimate confirmation time
    let start = Instant::now();
    let start_slot = client.get_slot()?;
    
    loop {
        if start.elapsed() > Duration::from_secs(30) {
            return Err("Timeout waiting for confirmation".into());
        }
        
        let current_slot = client.get_slot()?;
        if current_slot > start_slot {
            performance.confirmation_time = Some(start.elapsed().as_secs_f64());
            break;
        }
        
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(performance)
}

pub fn run_network_performance(url: &str) -> Result<(), Box<dyn Error>> {
    let performance = get_network_performance(url)?;

    println!("Network Performance:");
    println!("TPS: {:.2}", performance.tps);
    if let Some(avg_block_time) = performance.avg_block_time {
        println!("Average Block Time: {:.3}s", avg_block_time);
    }
    if let Some(confirmation_time) = performance.confirmation_time {
        println!("Estimated Confirmation Time: {:.3}s", confirmation_time);
    }

    Ok(())
}
