use {
    crate::{
        TOWER_RECEIVE_CONFIRM_CMD, TOWER_REQUEST_CMD, TOWER_SIZE,
        checker::{SWITCH_CHANNEL, check_keys, should_switch, switch_complete},
        config::make_server_config,
    },
    quinn::{ClientConfig, Endpoint, RecvStream, SendStream},
    std::{
        env,
        fs::{self, File},
        io::{Read, Write},
        net::{IpAddr, Ipv4Addr, SocketAddr},
        str::FromStr,
        sync::Arc,
    },
};

pub async fn run_server(server: Arc<Endpoint>) -> Result<(), anyhow::Error> {
    while let Some(incoming) = server.accept().await {
        match incoming.await {
            Ok(conn) => {
                let _stream = match conn.open_bi().await {
                    Ok((mut send_stream, mut recv_stream)) => {
                        match handle_stream_server((&mut send_stream, &mut recv_stream)).await {
                            Ok(_) => Ok(()),
                            Err(e) => Err(anyhow::Error::msg(format!(
                                "Error: Unable to handle stream: {:?}",
                                e
                            ))),
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: Unable to open bi-directional stream: {:?}", e);
                        continue;
                    }
                };
            }
            Err(e) => {
                eprintln!("Error: Unable to accept connection: {:?}", e);
            }
        }
    }

    Ok(())
}

pub async fn handle_stream_server(
    (send_stream, recv_stream): (&mut SendStream, &mut RecvStream),
) -> Result<(), anyhow::Error> {
    loop {
        let mut buf = [0u8; 32];
        let n = match recv_stream.read(&mut buf).await {
            Ok(Some(n)) => n,
            Ok(None) => {
                println!("Stream closed.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error reading stream: {:?}", e);
                return Err(e.into());
            }
        };
        let cmd = String::from_utf8_lossy(&buf[..n]).trim().to_string();

        match cmd.as_str() {
            TOWER_REQUEST_CMD => {
                // current is primary or not
                let is_primary = match check_keys() {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("Error: unable to check keys: {:?}", e);
                        send_stream.write(&vec![]);
                        continue;
                    }
                };

                if is_primary {
                    let tower_path =
                        env::var("TOWER_FILE_PATH").expect("Error: unable to read tower");

                    let tower_data =
                        fs::read(tower_path).expect("Error: unable to write tower data");

                    if let Err(r) = send_stream.write_all(&tower_data).await {
                        // log error
                    }
                } else {
                    // error
                }
            }
            TOWER_RECEIVE_CONFIRM_CMD => {
                switch_complete();
            }
            _ => {}
        }
    }
}

pub async fn run_client(endpoint: Arc<Endpoint>) -> Result<(), anyhow::Error> {
    let client_addr = env::var("QUIC_SERVER_URL").expect("Missing QUIC server URL");

    let client_socket_addr =
        SocketAddr::from_str(&client_addr.as_str()).expect("Error: unable to parse client addr");

    // if no request then no connection
    // todo - find better way to do this
    loop {
        if should_switch() {
            break;
        }
    }

    match endpoint.connect(client_socket_addr, "server") {
        Ok(client) => match client.await {
            Ok(connection) => {
                tracing::info!("Connected to server");
                match connection.accept_bi().await {
                    Ok(r) => {}
                    Err(e) => {
                        tracing::error!("Error: unable to accept bi channel: {:?}", e)
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error: Unable to connect to server: {:?}", e);
            }
        },
        Err(e) => {
            tracing::error!("Error: Unable to connect to server: {:?}", e);
        }
    }
    Ok(())
}

pub async fn handle_stream_client(
    (send_stream, recv_stream): (&mut SendStream, &mut RecvStream),
) -> Result<(), anyhow::Error> {
    let _ = match send_stream.write(TOWER_REQUEST_CMD.as_bytes()).await {
        Ok(_r) => {
            let tower_data = match recv_stream.read_to_end(TOWER_SIZE).await {
                Ok(r) => r,
                e => {
                    tracing::error!("Error: Unable to read tower {:?}", e);

                    return Err(anyhow::Error::msg("Error: unable to read tower"));
                }
            };

            let tower_path = env::var("TOWER_FILE_PATH").expect("Error: unable to read tower");

            let _ = fs::write(tower_path, tower_data).expect("Error: unable to write tower data");

            if let Err(e) = send_stream
                .write(TOWER_RECEIVE_CONFIRM_CMD.as_bytes())
                .await
            {
                tracing::error!("{:?}", e)
            }
        }
        Err(e) => {
            // cannot read switch to cloudflare
        }
    };

    Ok(())
}
