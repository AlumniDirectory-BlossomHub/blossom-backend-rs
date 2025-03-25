use aws_sdk_s3::Client;
use entity::image::{ActiveModel, Column, Entity, Model};
use image::{DynamicImage, ImageFormat};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Set,
};
use std::io::Cursor;
use std::time::Duration;

#[derive(Clone)]
pub struct ImageService {
    bucket_name: String,
    image_format: Option<ImageFormat>,
    image_size: Option<(u32, u32)>,
    image_filter: Option<image::imageops::FilterType>,
}

impl ImageService {
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
    async fn process_image(&self, image: DynamicImage) -> Result<Vec<u8>, &str> {
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
            .map_err(|_| "Cannot write image to bytes")?;
        Ok(bytes.into_inner())
    }

    /// 上传并保存图片
    pub async fn upload_image(
        &self,
        db: &DatabaseConnection,
        s3_client: &Client,
        image: DynamicImage,
    ) -> Result<Model, &str> {
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
            .map_err(|_| "Cannot upload image")?;

        println!("Uploaded image successfully");

        // 保存到数据库
        match (ActiveModel {
            s3_bucket: Set(self.bucket_name.clone()),
            s3_key: Set(key.clone()),
            ..Default::default()
        }
        .insert(db)
        .await)
        {
            Ok(model) => {
                println!("Insert image metadata successfully");
                Ok(model)
            }
            Err(err) => {
                println!("{:?}", err);
                s3_client
                    .delete_object()
                    .bucket(&self.bucket_name)
                    .key(&key)
                    .send()
                    .await
                    .map_err(
                        |_| "Unable to save image metadata to database, Cannot delete image",
                    )?;
                Err("Cannot to save image metadata to database")
            }
        }
    }

    /// 通过 id 删除图片
    pub async fn delete_image_by_id(
        &self,
        db: &DatabaseConnection,
        s3_client: &Client,
        image_id: i32,
    ) -> Result<(), &str> {
        let image = self.get_image_by_id(db, image_id).await?;
        self.delete_image(db, s3_client, image).await
    }

    /// 通过 s3_key 删除图片
    pub async fn delete_image_by_key(
        &self,
        db: &DatabaseConnection,
        s3_client: &Client,
        image_key: String,
    ) -> Result<(), &str> {
        let image = self.get_image_by_key(db, image_key).await?;
        self.delete_image(db, s3_client, image).await
    }

    /// 通过 id 获取图片预签名 url
    pub async fn get_image_url_by_id(
        &self,
        db: &DatabaseConnection,
        s3_client: &Client,
        image_id: i32,
    ) -> Result<String, &str> {
        let image = self.get_image_by_id(db, image_id).await?;
        self.get_s3_presigned_url(s3_client, image).await
    }

    /// 通过 s3_key 获取图片预签名 url
    pub async fn get_image_url_by_key(
        &self,
        db: &DatabaseConnection,
        s3_client: &Client,
        image_key: String,
    ) -> Result<String, &str> {
        let image = self.get_image_by_key(db, image_key).await?;
        self.get_s3_presigned_url(s3_client, image).await
    }

    /// 通过 id 查找 image 元数据模型
    async fn get_image_by_id(&self, db: &DatabaseConnection, image_id: i32) -> Result<Model, &str> {
        match Entity::find_by_id(image_id).one(db).await {
            Ok(Some(model)) => {
                if model.s3_bucket != self.bucket_name {
                    Err("Image bucket does not match")
                } else {
                    Ok(model)
                }
            }
            _ => Err("Cannot find image"),
        }
    }

    /// 通过 s3_key 查找 image 元数据模型
    async fn get_image_by_key(
        &self,
        db: &DatabaseConnection,
        image_key: String,
    ) -> Result<Model, &str> {
        match Entity::find()
            .filter(
                Condition::all()
                    .add(Column::S3Key.eq(&image_key))
                    .add(Column::S3Bucket.eq(&self.bucket_name)),
            )
            .one(db)
            .await
        {
            Ok(Some(model)) => Ok(model),
            _ => Err("Cannot find image"),
        }
    }

    /// 执行图片删除操作
    async fn delete_image(
        &self,
        db: &DatabaseConnection,
        s3_client: &Client,
        image: Model,
    ) -> Result<(), &str> {
        let key = image.s3_key.to_string();
        image
            .delete(db)
            .await
            .map_err(|_| "Cannot delete image metadata")?;
        match self.delete_s3_image(s3_client, &key).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot delete image"),
        }
    }

    /// 生成预签名 url
    async fn get_s3_presigned_url(&self, s3_client: &Client, image: Model) -> Result<String, &str> {
        let presigned_url = s3_client
            .get_object()
            .bucket(&self.bucket_name)
            .key(&image.s3_key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(Duration::from_secs(3600))
                    .map_err(|_| "Cannot build PresigningConfig")?,
            )
            .await
            .map_err(|_| "Cannot get image url from s3")?;
        Ok(presigned_url.uri().to_string())
    }

    /// 从 s3 储存桶删除图片
    async fn delete_s3_image(&self, s3_client: &Client, key: &String) -> Result<(), &str> {
        s3_client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|_| "Cannot delete image")?;
        Ok(())
    }
}
