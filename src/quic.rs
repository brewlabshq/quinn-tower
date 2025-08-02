use {
    crate::{TOWER_REQUEST_CMD, TOWER_SIZE, config::make_server_config},
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

pub async fn init_sender(server: Arc<Endpoint>) -> Result<(), anyhow::Error> {
    while let Some(incoming) = server.accept().await {
        match incoming.await {
            Ok(conn) => {
                let _stream = match conn.open_bi().await {
                    Ok((mut send_stream, mut recv_stream)) => {
                        match handle_stream((&mut send_stream, &mut recv_stream)).await {
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

pub async fn handle_stream(
    (send_stream, recv_stream): (&mut SendStream, &mut RecvStream),
) -> Result<(), anyhow::Error> {
    let mut tower_req_cmd = TOWER_REQUEST_CMD.as_bytes().to_vec();
    loop {
        let rec = recv_stream.read_exact(&mut tower_req_cmd).await;
        match rec {
            Ok(_) => {
                let tower_file = fs::read(env::var("TOWER_FILE_PATH").expect("Missing Tower path"))
                    .expect("Error: unable to open the file");

                send_stream.write(&tower_file).await?;
            }
            Err(e) => {
                eprintln!("Error: Unable to read data: {:?}", e);
            }
        }
    }
}

pub async fn init_receiver(endpoint: Arc<Endpoint>) -> Result<(), anyhow::Error> {
    let client_addr = env::var("QUIC_SERVER_URL").expect("Missing QUIC server URL");

    let client_socket_addr =
        SocketAddr::from_str(&client_addr.as_str()).expect("Error: unable to parse client addr");

    match endpoint.connect(client_socket_addr, "server") {
        Ok(client) => match client.await {
            Ok(connection) => {
                tracing::info!("Connected to server");
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
