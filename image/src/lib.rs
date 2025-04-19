//! 图片 s3 储存服务
//!

use aws_sdk_s3::Client;
use image::ImageFormat::Jpeg;
use service::ImageService;

pub mod errors;
pub mod service;
pub mod storage;
pub mod utils;

/// 所有图片服务集合
#[derive(Clone)]
pub struct ImageServices {
    pub avatar: ImageService,
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
            test: ImageService::new("test", Some(Jpeg), Some((200u32, 128u32)), None)
                .ensure(s3_client)
                .await,
        }
    }
}
