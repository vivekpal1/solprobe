use crate::ui::app::{App, NodeHealth, NetworkPerformance, TroubleshootResults};
use crate::ui::ui;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use tokio::time::{Duration, interval};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

pub async fn run_monitor(url: &str) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App {
        selected_tab: 0,
        node_health: NodeHealth::default(),
        network_performance: NetworkPerformance::default(),
        troubleshoot_results: TroubleshootResults::default(),
    };

    let mut update_interval = interval(Duration::from_secs(5));
    let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => app.selected_tab = (app.selected_tab + 3) % 4,
                    KeyCode::Right => app.selected_tab = (app.selected_tab + 1) % 4,
                    _ => {}
                }
            }
        }

        tokio::select! {
            _ = update_interval.tick() => {
                update_data(&mut app, &client);
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn update_data(app: &mut App, client: &RpcClient) {
    match client.get_version() {
        Ok(version) => {
            app.node_health.is_responsive = true;
            app.node_health.version = Some(version.solana_core);
        },
        Err(_) => app.node_health.is_responsive = false,
    }

    if let Ok(slot) = client.get_slot() {
        app.node_health.current_slot = Some(slot);
    }

    if let Ok(epoch_info) = client.get_epoch_info() {
        app.node_health.current_epoch = Some(epoch_info.epoch);
    }

    if let Ok(cluster_nodes) = client.get_cluster_nodes() {
        app.node_health.total_nodes = Some(cluster_nodes.len() as u64);
    }

    if let Ok(recent_performance) = client.get_recent_performance_samples(Some(1)) {
        if let Some(latest) = recent_performance.first() {
            app.network_performance.tps = latest.num_transactions as f64 / latest.sample_period_secs as f64;
            app.network_performance.avg_block_time = Some(latest.sample_period_secs as f64 / latest.num_slots as f64);
        }
    }

    app.network_performance.confirmation_time = Some(0.5);

    if let Ok(vote_accounts) = client.get_vote_accounts() {
        app.troubleshoot_results.delinquent_validators = vote_accounts.delinquent.len() as u64;
    }

    if let Ok(blocks) = client.get_blocks_with_limit(client.get_slot().unwrap_or(0), 100) {
        app.troubleshoot_results.empty_blocks = blocks.len() as u64;
    }

    if let Ok(largest_accounts) = client.get_largest_accounts(None) {
        app.troubleshoot_results.large_accounts = largest_accounts.value.len() as u64;
    }
}


