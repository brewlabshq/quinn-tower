use {
    // crate::{receiver::init_receiver, sender::init_sender},
    crate::quic::init_sender,
    anyhow::Result,
    clap::{self, Parser, Subcommand},
    futures::future::join_all,
    tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt},
};

mod checker;
mod cloudflare;
mod config;
mod quic;

pub const TOWER_SIZE: usize = 2319;
pub const TOWER_REQUEST_CMD: &str = "tower-request";
pub const MAX_CATCHUP_SLOT: u64 = 30;
#[tokio::main]
async fn main() -> Result<()> {
    let stdout_layer = fmt::layer()
        .with_timer(fmt::time::UtcTime::rfc_3339()) // 2025-06-07T03:37:59Z
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .compact(); // concise one-liner
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(stdout_layer)
        .init();

    let j = tokio::spawn(async move { init_sender().await });

    let join_handles = vec![j];

    join_all(join_handles).await;
    Ok(())
}
