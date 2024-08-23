use std::error::Error;
use std::io;
use std::time::{Duration, Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

use crate::types::{NodeHealth, NetworkPerformance, TroubleshootResults};
use crate::utils::config::Config;
use super::components::{create_gauge, create_paragraph, create_status_text};

pub enum AppMode {
    NodeHealth,
    NetworkPerformance,
    Troubleshoot,
    Monitor,
}

struct App {
    mode: AppMode,
    node_health: NodeHealth,
    network_performance: NetworkPerformance,
    troubleshoot_results: TroubleshootResults,
    selected_tab: usize,
}

pub async fn run_app(mode: AppMode, url: &str, config: &Config) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App {
        mode,
        node_health: NodeHealth::default(),
        network_performance: NetworkPerformance::default(),
        troubleshoot_results: TroubleshootResults::default(),
        selected_tab: 0,
    };

    let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());

    let res = run_ui(&mut terminal, &mut app, &client, config);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_ui<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    client: &RpcClient,
    config: &Config,
) -> io::Result<()> {
    let mut last_update = Instant::now();
    loop {
        terminal.draw(|f| ui(f, app))?;

        if last_update.elapsed() >= Duration::from_secs(config.update_interval) {
            update_data(app, client);
            last_update = Instant::now();
        }

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('r') => {
                    update_data(app, client);
                }
                KeyCode::Left => {
                    app.selected_tab = app.selected_tab.saturating_sub(1);
                }
                KeyCode::Right => {
                    if app.selected_tab < 3 {
                        app.selected_tab += 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.size());

    let titles = vec!["Node Health", "Network Performance", "Troubleshoot", "Monitor"];
    let tabs = Tabs::new(titles.into_iter().map(Spans::from).collect())
        .select(app.selected_tab)
        .block(Block::default().borders(Borders::ALL).title("SolProbe"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .divider(Span::raw("|"));

    f.render_widget(tabs, chunks[0]);

    match app.selected_tab {
        0 => render_node_health(f, app, chunks[1]),
        1 => render_network_performance(f, app, chunks[1]),
        2 => render_troubleshoot(f, app, chunks[1]),
        3 => render_monitor(f, app, chunks[1]),
        _ => unreachable!(),
    }
}

fn render_node_health<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App, area: tui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let responsive = create_status_text("Node Status", app.node_health.is_responsive);
    f.render_widget(responsive, chunks[0]);

    if let Some(slot) = app.node_health.current_slot {
        let slot = create_paragraph("Current Slot", format!("{}", slot));
        f.render_widget(slot, chunks[1]);
    }

    if let Some(version) = &app.node_health.version {
        let version = create_paragraph("Version", version.to_string());
        f.render_widget(version, chunks[2]);
    }

    if let Some(epoch) = app.node_health.current_epoch {
        let epoch = create_paragraph("Current Epoch", format!("{}", epoch));
        f.render_widget(epoch, chunks[3]);
    }

    if let Some(total_nodes) = app.node_health.total_nodes {
        let total_nodes = create_paragraph("Total Nodes", format!("{}", total_nodes));
        f.render_widget(total_nodes, chunks[4]);
    }
}

fn render_network_performance<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App, area: tui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);

    let tps = create_gauge("TPS", app.network_performance.tps as u16, Color::Blue);
    f.render_widget(tps, chunks[0]);

    if let Some(block_time) = app.network_performance.avg_block_time {
        let block_time = create_paragraph("Avg Block Time", format!("{:.3}s", block_time));
        f.render_widget(block_time, chunks[1]);
    }

    if let Some(conf_time) = app.network_performance.confirmation_time {
        let conf_time = create_paragraph("Confirmation Time", format!("{:.3}s", conf_time));
        f.render_widget(conf_time, chunks[2]);
    }
}

fn render_troubleshoot<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App, area: tui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let delinquent = create_gauge("Delinquent Validators", app.troubleshoot_results.delinquent_validators as u16, Color::Red);
    f.render_widget(delinquent, chunks[0]);

    let empty_blocks = create_paragraph("Empty Blocks", format!("{}", app.troubleshoot_results.empty_blocks));
    f.render_widget(empty_blocks, chunks[1]);

    let large_accounts = create_paragraph("Large Accounts", format!("{}", app.troubleshoot_results.large_accounts));
    f.render_widget(large_accounts, chunks[2]);

    let recommendations = List::new(vec![
        ListItem::new("Investigate delinquent validators if count is high"),
        ListItem::new("Check for network congestion if many empty blocks"),
        ListItem::new("Optimize large accounts to improve performance"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Recommendations"));
    f.render_widget(recommendations, chunks[3]);
}

fn render_monitor<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &App, area: tui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let responsive = create_status_text("Node Status", app.node_health.is_responsive);
    f.render_widget(responsive, chunks[0]);

    let tps = create_gauge("TPS", app.network_performance.tps as u16, Color::Blue);
    f.render_widget(tps, chunks[1]);

    if let Some(slot) = app.node_health.current_slot {
        let slot = create_paragraph("Current Slot", format!("{}", slot));
        f.render_widget(slot, chunks[2]);
    }

    if let Some(block_time) = app.network_performance.avg_block_time {
        let block_time = create_paragraph("Avg Block Time", format!("{:.3}s", block_time));
        f.render_widget(block_time, chunks[3]);
    }

    let delinquent = create_paragraph("Delinquent Validators", format!("{}", app.troubleshoot_results.delinquent_validators));
    f.render_widget(delinquent, chunks[4]);
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