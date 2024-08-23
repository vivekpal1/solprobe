use clap::Parser;
use log::info;
use std::error::Error;

mod commands;
mod types;
mod utils;
mod ui;

use commands::{Commands, Cli};
use utils::config::Config;
use utils::logger;
use ui::app::run_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    logger::init()?;

    let config = Config::load()?;

    info!("SolProbe started");

    match cli.command {
        Commands::NodeHealth => {
            let url = utils::input::get_url_input()?;
            run_app(ui::app::AppMode::NodeHealth, &url, &config).await?;
        }
        Commands::NetworkPerformance => {
            let url = utils::input::get_url_input()?;
            run_app(ui::app::AppMode::NetworkPerformance, &url, &config).await?;
        }
        Commands::Troubleshoot => {
            let url = utils::input::get_url_input()?;
            run_app(ui::app::AppMode::Troubleshoot, &url, &config).await?;
        }
        Commands::Monitor => {
            let url = utils::input::get_url_input()?;
            let interval = utils::input::get_interval_input()?;
            run_app(ui::app::AppMode::Monitor, &url, &config).await?;
        }
    }

    Ok(())
}