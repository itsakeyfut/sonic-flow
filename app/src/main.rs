#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod app;
mod ui;

use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    init_logging();

    info!("Starting Sonic Flow v{}", env!("CARGO_PKG_VERSION"));

    // Build tokio runtime (Slint event loop runs on main thread)
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _guard = runtime.enter();

    // Command/Event channels
    let (tx_cmd, rx_cmd) = tokio::sync::mpsc::channel(64);
    let (tx_event, rx_event) = tokio::sync::mpsc::channel(64);

    // Spawn application controller
    let controller = app::Controller::new(tx_event)?;
    tokio::spawn(controller.run(rx_cmd));

    // Create and run UI (blocks on Slint event loop)
    let ui = ui::Ui::new(tx_cmd, rx_event)?;
    ui.run()?;

    info!("Sonic Flow shut down");
    Ok(())
}

fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            EnvFilter::new("sonic_flow=debug,warn")
        } else {
            EnvFilter::new("sonic_flow=info,warn")
        }
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_ansi(true),
        )
        .init();
}
