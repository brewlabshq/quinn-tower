use {
    crate::{
        config::make_endpoint,
        quic::{init_receiver, init_sender},
    },
    anyhow::Result,
    futures::future::join_all,
    std::sync::Arc,
    tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt},
};

mod checker;
mod cloudflare;
mod config;
mod quic;

pub const TOWER_SIZE: usize = 2319;
pub const TOWER_REQUEST_CMD: &str = "tower-request";
pub const TOWER_RECEIVE_CONFIRM_CMD:&str = "tower-request-complete";
pub const MAX_CATCHUP_SLOT: u64 = 30;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
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

    let endpoint = Arc::new(make_endpoint().expect("Error: unable to init endpoint"));
    let endpoint_sender_clone = endpoint.clone();
    let endpoint_receiver_clone = endpoint.clone();

    let jh_sender = tokio::spawn(async move { init_sender(endpoint_sender_clone).await });
    let jh_receiver = tokio::spawn(async move { init_receiver(endpoint_receiver_clone).await });

    join_all(vec![jh_sender, jh_receiver]).await;
    Ok(())
}
