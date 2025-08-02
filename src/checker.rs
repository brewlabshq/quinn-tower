use {
    crate::MAX_CATCHUP_SLOT,
    once_cell::sync::Lazy,
    solana_commitment_config::CommitmentConfig,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    std::{env, time::Duration},
    tokio::{sync::watch, time::sleep},
};

pub static SWITCH_CHANNEL: Lazy<watch::Sender<bool>> = Lazy::new(|| {
    let (tx, _) = watch::channel(false);

    tx
});

pub fn request_switch() {
    let _ = SWITCH_CHANNEL.send(true);
}

pub fn should_switch() -> bool {
    *SWITCH_CHANNEL.borrow()
}

pub async fn check_rpc() -> Result<(), anyhow::Error> {
    let node_url =
        std::env::var("NODE_URL").unwrap_or_else(|_| "http://localhost:8899".to_string());
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:8899".to_string());

    let node_client = RpcClient::new(node_url);
    let rpc_client = RpcClient::new(rpc_url);

    let mut retry_count: u64 = 0;
    let max_retry_count = 5;
    let mut get_slot_while_retrying = async |client: &RpcClient| {
        loop {
            match client
                .get_slot_with_commitment(CommitmentConfig::confirmed())
                .await
            {
                Ok(r) => {
                    retry_count = 0;
                    return Ok(r);
                }
                Err(e) => {
                    if retry_count >= max_retry_count {
                        return Err(e);
                    }
                    retry_count = retry_count.saturating_add(1);

                    sleep(Duration::from_millis(100));
                }
            };
        }
    };

    loop {
        let node_slot = get_slot_while_retrying(&node_client).await?;
        let rpc_slot = get_slot_while_retrying(&rpc_client).await?;
        let slot_distance = rpc_slot.saturating_sub(node_slot);
        if slot_distance > MAX_CATCHUP_SLOT {
            request_switch();
        }

        sleep(Duration::from_millis(100)).await;
    }
}
