//! 图片 s3 储存服务
//!

use aws_sdk_s3::Client;
use image::ImageFormat::Jpeg;
use service::ImageService;

pub mod errors;
pub mod service;
pub mod storage;
pub mod utils;

/// s3 服务连接
///
/// 由于预签名不需要与 s3 服务器通信，所以可以设置一个不连接的 Client 用于外部 domain 预签名
///
/// Examples:
/// ```
/// # use image_service::S3Client;
/// # use image_service::storage::create_client;
/// # async fn main () {
/// let internal_endpoint = std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set");
/// let external_endpoint =
///     std::env::var("MINIO_EXTERNAL_ENDPOINT").unwrap_or(internal_endpoint.clone());
/// let region = std::env::var("MINIO_REGION").expect("MINIO_REGION must be set");
/// let access_key =
///     std::env::var("APP_MINIO_ACCESS_KEY").expect("APP_MINIO_ACCESS_KEY must be set");
/// let secret_key =
///     std::env::var("APP_MINIO_SECRET_KEY").expect("APP_MINIO_SECRET_KEY must be set");
/// // 创建两个 client，使用相同的 region, access_key, secret_key
/// // s3_internal_client 用于内部实际请求（储存、读取、删除）
/// // s3_external_client 用于预签名
/// let s3_internal_client =
///     create_client(&internal_endpoint, &region, &access_key, &secret_key).await;
/// let s3_external_client =
///     create_client(&external_endpoint, &region, &access_key, &secret_key).await;
///
/// let s3_client = S3Client {
///     internal: s3_internal_client,
///     external: s3_external_client,
/// };
/// # }
/// ```
pub struct S3Client {
    /// 服务器内部连接
    pub internal: Client,
    /// 对外预签名连接
    pub external: Client,
}

/// 所有图片服务集合
#[derive(Clone)]
pub struct ImageServices {
    pub avatar: ImageService,
    pub person_photo: ImageService,
    pub test: ImageService,
}

impl ImageServices {
    /// 初始化图片服务
    ///
    ///
    pub async fn init(s3_client: &Client) -> Self {
        Self {
            avatar: ImageService::new("avatar", Some(Jpeg), Some((128u32, 128u32)), None)
                .ensure(s3_client)
                .await,
            person_photo: ImageService::new(
                "person-photo",
                Some(Jpeg),
                Some((768u32, 1024u32)),
                None,
            )
            .ensure(s3_client)
            .await,
            test: ImageService::new("test", Some(Jpeg), Some((200u32, 128u32)), None)
                .ensure(s3_client)
                .await,
        }
    }
}
