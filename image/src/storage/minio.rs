use aws_config::Region;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;

pub async fn create_client() -> Client {
    let config = aws_config::from_env()
        .endpoint_url(std::env::var("MINIO_ENDPOINT").unwrap())
        .region(Region::new(std::env::var("MINIO_REGION").unwrap()))
        .credentials_provider(Credentials::new(
            std::env::var("MINIO_ACCESS_KEY").unwrap(),
            std::env::var("MINIO_SECRET_KEY").unwrap(),
            None,
            None,
            "s3",
        ))
        .load()
        .await;

    Client::new(&config)
}

pub async fn ensure_bucket_exists(client: &Client, bucket_name: &String) -> Result<(), ()> {
    if let Err(_) = client.head_bucket().bucket(bucket_name).send().await {
        client
            .create_bucket()
            .bucket(bucket_name)
            .send()
            .await
            .map_err(|_| ())?;
    };
    Ok(())
}
