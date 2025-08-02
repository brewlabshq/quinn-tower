use {
    crate::{TOWER_REQUEST_CMD, TOWER_SIZE, config::make_server_config},
    anyhow::Context,
    quinn::{ClientConfig, RecvStream, SendStream},
    std::{
        env,
        fs::{self, File},
        io::{Read, Write},
        net::{IpAddr, Ipv4Addr, SocketAddr},
        str::FromStr,
    },
};

pub async fn init_sender() -> Result<(), anyhow::Error> {
    let _ = match rustls::crypto::aws_lc_rs::default_provider().install_default() {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!(
            "Error:  installing default provider: {:?}",
            e
        )),
    };

    let server_config = make_server_config()?;

    let port: u16 = env::var("PORT")
        .context("Error: unable to get port from environment variable")?
        .parse()
        .context("Error: unable to parse port")?;

    let server = match quinn::Endpoint::server(
        server_config,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
    ) {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Error: Unable to start server: {:?}", e);
            return Err(anyhow::Error::msg("Error: Unable to start server"));
        }
    };

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
