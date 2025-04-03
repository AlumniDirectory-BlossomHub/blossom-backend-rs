use aws_config::Region;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client;

pub async fn create_client() -> Client {
    let config = aws_config::from_env()
        .endpoint_url(std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set"))
        .region(Region::new(
            std::env::var("MINIO_REGION").expect("MINIO_REGION must be set"),
        ))
        .credentials_provider(Credentials::new(
            std::env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set"),
            std::env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set"),
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
