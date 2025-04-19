use aws_config::Region;
use aws_sdk_s3::config::http::HttpResponse;
use aws_sdk_s3::config::{BehaviorVersion, Credentials};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::create_bucket::CreateBucketError;
use aws_sdk_s3::{Client, Config};

/// 创建 s3 连接
///
/// ```
/// # use image_service::storage::create_client;
/// #
/// let endpoint = "http://endpoint:9000".to_string();
/// let region = "your-region".to_string();
/// let access_key = "your-access-key".to_string();
/// let secret_key = "your-secret-key".to_string();
///
/// let client = create_client(&endpoint, &region, &access_key, &secret_key)
/// ```
pub async fn create_client(
    endpoint: &String,
    region: &String,
    access_key: &String,
    secret_key: &String,
) -> Client {
    let config = Config::builder()
        .endpoint_url(endpoint)
        .region(Region::new(region.clone()))
        .credentials_provider(Credentials::new(
            access_key, secret_key, None, None, "Static",
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
    if let Err(_) = client.head_bucket().bucket(bucket_name).send().await {
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
