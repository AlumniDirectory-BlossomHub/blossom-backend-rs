use aws_config::Region;
use aws_sdk_s3::config::http::HttpResponse;
use aws_sdk_s3::config::{BehaviorVersion, Credentials};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::create_bucket::CreateBucketError;
use aws_sdk_s3::{Client, Config};

/// 创建 s3 连接
///
/// 有关配置将从环境变量中读取
///
/// ```text
/// MINIO_ENDPOINT=http://127.0.0.1:9000
/// MINIO_ACCESS_KEY=your_minio_access_key
/// MINIO_SECRET_KEY=your_minio_secret_key
/// MINIO_REGION=your_minio_region
/// ```
pub async fn create_client() -> Client {
    let config = Config::builder()
        .endpoint_url(std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set"))
        .region(Region::new(
            std::env::var("MINIO_REGION").expect("MINIO_REGION must be set"),
        ))
        .credentials_provider(Credentials::new(
            std::env::var("APP_MINIO_ACCESS_KEY").expect("APP_MINIO_ACCESS_KEY must be set"),
            std::env::var("APP_MINIO_SECRET_KEY").expect("APP_MINIO_SECRET_KEY must be set"),
            None,
            None,
            "Static",
        ))
        .force_path_style(true)
        .behavior_version(BehaviorVersion::latest())
        // .timeout_config(
        //     TimeoutConfig::builder()
        //         .operation_attempt_timeout(Duration::from_secs(120))
        //         .build(),
        // )
        .build();

    Client::from_conf(config)
}

/// 确保指定的 bucket 存在
///
/// * `client` - S3 Client
/// * `bucket_name` - bucket name
///
pub async fn ensure_bucket_exists(
    client: &Client,
    bucket_name: &String,
) -> Result<(), SdkError<CreateBucketError, HttpResponse>> {
    if let Err(e) = client.head_bucket().bucket(bucket_name).send().await {
        // println!("Bucket {} not found", bucket_name);
        // println!("{}", e);
        client
            .create_bucket()
            .bucket(bucket_name.clone())
            .send()
            .await?;
    };
    Ok(())
}
