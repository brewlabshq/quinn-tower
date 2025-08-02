use {
    anyhow::Context,
    quinn::{Endpoint, ServerConfig, crypto::rustls::QuicServerConfig},
    rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    std::{
        env,
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
    },
};

pub fn make_server_config() -> Result<quinn::ServerConfig, anyhow::Error> {
    let cert =
        CertificateDer::from_pem_file("cert.pem").context("Error reading certificate file")?;
    let key = PrivateKeyDer::from_pem_file("key.pem").context("Error reading private key file")?;

    let server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    let server_config =
        ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));

    Ok(server_config)
}

pub fn make_endpoint() -> Result<Endpoint, anyhow::Error> {
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

    Ok(server)
}
