use {
    anyhow::Result,
    aws_config::{Region, meta::region::RegionProviderChain},
    aws_sdk_s3::{Client, config::Builder as S3ConfigBuilder, primitives::ByteStream},
    std::{path::Path, time::Duration},
    tokio::{fs, io::AsyncWriteExt, time::Instant},
};

async fn r2_client(account_id: &str, access_key: &str, secret_key: &str) -> Result<Client> {
    let base = aws_config::from_env()
        .region(Region::new("auto"))
        .load()
        .await;

    let endpoint = format!("https://{account_id}.eu.r2.cloudflarestorage.com");

    let creds = aws_sdk_s3::config::Credentials::new(
        access_key, secret_key, None,     // session token (optional)
        None,     // expiration (optional)
        "static", // provider name
    );

    let s3_config = S3ConfigBuilder::from(&base)
        .endpoint_url(endpoint)
        .credentials_provider(creds)
        .force_path_style(true)
        .build();

    Ok(Client::from_conf(s3_config))
}

async fn upload_file(client: &Client, bucket: &str, object_key: &str, path: &Path) -> Result<()> {
    let bytes = fs::read(path).await?;
    client
        .put_object()
        .bucket(bucket)
        .key(object_key)
        .body(ByteStream::from(bytes))
        .content_type("application/octet-stream")
        .send()
        .await?;
    Ok(())
}

async fn download_file(
    client: &Client,
    bucket: &str,
    object_key: &str,
    dest_path: &Path,
) -> Result<()> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(object_key)
        .send()
        .await?;
    let data = resp.body.collect().await?.into_bytes();

    let mut f = fs::File::create(dest_path).await?;
    f.write_all(&data).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        crate::r2::{download_file, r2_client, upload_file},
        dotenv::dotenv,
        std::{env, fs, path::Path, time::Instant},
        tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt},
    };

    #[tokio::test]
    async fn test_kv_roundtrip() -> Result<(), anyhow::Error> {
        // Load .env once
        dotenv().ok();

        let account_id = std::env::var("R2_ACCOUNT_ID")?;
        let access_key = std::env::var("R2_ACCESS_KEY_ID")?;
        let secret_key = std::env::var("R2_SECRET_ACCESS_KEY")?;
        let bucket = std::env::var("R2_BUCKET")?;

        let client = r2_client(&account_id, &access_key, &secret_key).await?;
        let time = Instant::now();
        upload_file(
            &client,
            &bucket,
            "temp-tower.bin",
            Path::new("temp-tower.bin"),
        )
        .await?;
        println!("Write time: {:?}", time.elapsed());
        let download_time = Instant::now();
        download_file(
            &client,
            &bucket,
            "temp-tower.bin",
            Path::new("temp-tower.bin"),
        )
        .await?;
        let download_time = download_time.elapsed();
        println!("Download time: {:?}", download_time);

        Ok(())
    }
}
