use {
    crate::{checker::{switch_complete, SWITCH_CHANNEL}, config::make_server_config, TOWER_REQUEST_CMD, TOWER_SIZE},
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
        let mut buf = [0u8;32];
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
                // start switch
                // send tower
                
            },
            TOWER_RECEIVE_CONFIRM_CMD => {
                switch_complete();
                // hotload identity keys
            }
            _=>{}
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
               match connection.accept_bi().await {
                Ok(r) => {

                },
                Err(e) => {
                    tracing::error!("Error: unable to accept bi channel: {:?}",e)
                },
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
