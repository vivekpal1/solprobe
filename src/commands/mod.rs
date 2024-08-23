use clap::{Parser, Subcommand};

pub mod node_health;
pub mod network_performance;
pub mod troubleshoot;
pub mod monitor;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    NodeHealth,
    NetworkPerformance,
    Troubleshoot,
    Monitor,
}