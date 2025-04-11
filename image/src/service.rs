use crate::errors::ImageError;
use crate::errors::ImageError::{ProcessError, S3Error};
use crate::storage::ensure_bucket_exists;
use aws_sdk_s3::Client;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;
use std::time::Duration;

/// 图片服务
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageService {
    bucket_name: String,
    image_format: Option<ImageFormat>,
    image_size: Option<(u32, u32)>,
    image_filter: Option<image::imageops::FilterType>,
}

impl ImageService {
    /// 创建服务对象
    ///
    /// * `bucket_name` - 储存容器名称（不带前缀）
    /// * `image_format` - 储存的图片格式
    /// * `image_size` - 储存的图片大小
    /// * `image_filter` - 图片缩放过滤器
    ///
    /// Examples:
    /// ```
    /// # use image_service::service::ImageService;
    /// use image::ImageFormat::Jpeg;
    ///
    /// let image_service = ImageService::new(
    ///     "test_bucket",
    ///     Some(Jpeg),
    ///     Some((128, 128)),
    ///     None, // 默认 image::imageops::FilterType::Lanczos3
    /// );
    /// ```
    pub fn new(
        bucket_name: impl Into<String>,
        image_format: Option<ImageFormat>,
        image_size: Option<(u32, u32)>,
        image_filter: Option<image::imageops::FilterType>,
    ) -> Self {
        let bucket_name = std::env::var("MINIO_BUCKET_PREFIX").unwrap_or_else(|_| String::new())
            + &bucket_name.into();
        Self {
            bucket_name,
            image_format,
            image_size,
            image_filter,
        }
    }
    /// 确保本服务的 bucket 存在
    pub async fn ensure(self, s3_client: &Client) -> Self {
        ensure_bucket_exists(s3_client, &self.bucket_name)
            .await
            .unwrap();
        self
    }

    #[doc(hidden)]
    pub fn image_content_type(&self) -> &'static str {
        match self.image_format {
            Some(ImageFormat::Png) => "image/png",
            Some(ImageFormat::Jpeg) | None => "image/jpeg",
            Some(ImageFormat::Gif) => "image/gif",
            Some(ImageFormat::Bmp) => "image/bmp",
            Some(ImageFormat::WebP) => "image/webp",
            _ => "application/octet-stream",
        }
    }

    /// 处理图片
    ///
    /// 将传入的图片按照要求处理
    async fn process_image(&self, image: DynamicImage) -> Result<Vec<u8>, ImageError> {
        let resized = match self.image_size {
            Some((width, height)) => image.resize_to_fill(
                width,
                height,
                self.image_filter
                    .unwrap_or_else(|| image::imageops::FilterType::Lanczos3),
            ),
            None => image,
        };
        let mut bytes = Cursor::new(Vec::new());
        resized
            .write_to(
                &mut bytes,
                self.image_format.unwrap_or_else(|| ImageFormat::Jpeg),
            )
            .map_err(|_| ProcessError("Cannot write image to bytes"))?;
        Ok(bytes.into_inner())
    }

    /// 上传并保存图片
    ///
    /// Examples:
    /// ```
    /// # use image::ImageFormat::Jpeg;
    /// # use image_service::service::ImageService;
    /// # use image_service::storage::create_client;
    /// use image_service::utils::open_image;
    /// # async fn main() {
    ///
    /// # let s3_client = create_client().await;
    /// let service = ImageService::new("test", Some(Jpeg), Some((128, 128)), None);
    /// let image = open_image("/path/to/image".as_ref());
    ///
    /// service.upload_image(&s3_client, image)
    ///     .await
    ///     .map_err(|_| "fail to upload image")?;
    /// # }
    /// ```
    pub async fn upload_image(
        &self,
        s3_client: &Client,
        image: DynamicImage,
    ) -> Result<String, ImageError> {
        let key = uuid::Uuid::new_v4().to_string();

        let image_processed = self.process_image(image).await?;

        s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .content_type(self.image_content_type())
            .body(image_processed.into())
            .send()
            .await
            .map_err(|_| S3Error("Cannot upload image to s3 server"))?;

        println!("Uploaded image successfully");

        Ok(key)
    }

    /// 生成预签名 url
    pub async fn get_presigned_url(
        &self,
        s3_client: &Client,
        s3_key: impl Into<String>,
    ) -> Result<String, ImageError> {
        let presigned_url = s3_client
            .get_object()
            .bucket(&self.bucket_name)
            .key(s3_key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(Duration::from_secs(3600))
                    .map_err(|_| S3Error("Cannot build PresigningConfig"))?,
            )
            .await
            .map_err(|_| S3Error("Cannot get image url from s3"))?;
        Ok(presigned_url.uri().to_string())
    }

    /// 从 s3 储存桶删除图片
    pub async fn delete_image(&self, s3_client: &Client, key: &String) -> Result<(), ImageError> {
        s3_client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|_| S3Error("Cannot delete image"))?;
        Ok(())
    }
}
