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

pub async fn ensure_bucket_exists(client: &Client) {
    let bucket = std::env::var("MINIO_BUCKET").unwrap();
    match client.head_bucket().bucket(&bucket).send().await {
        Ok(_) => println!("Bucket exists"),
        Err(_) => {
            client
                .create_bucket()
                .bucket(&bucket)
                .send()
                .await
                .expect("Failed to create bucket");
        }
    }
}
