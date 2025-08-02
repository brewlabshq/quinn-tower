use {
    anyhow::Context,
    quinn::{ServerConfig, crypto::rustls::QuicServerConfig},
    rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    std::sync::Arc,
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
