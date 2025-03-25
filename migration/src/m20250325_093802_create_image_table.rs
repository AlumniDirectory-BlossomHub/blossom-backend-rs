use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建表
        manager
            .create_table(
                Table::create()
                    .table(Image::Table)
                    .if_not_exists()
                    .col(pk_auto(Image::Id))
                    .col(string(Image::S3Bucket).not_null())
                    .col(string(Image::S3Key).not_null())
                    .col(
                        timestamp_with_time_zone(Image::CreatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 为 s3_bucket, s3_key 创建联合唯一索引
        manager
            .create_index(
                Index::create()
                    .name("s3_bucket_key_unique")
                    .table(Image::Table)
                    .col(Image::S3Bucket)
                    .col(Image::S3Key)
                    .unique()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除索引
        manager
            .drop_index(
                Index::drop()
                    .name("s3_bucket_key_unique")
                    .table(Image::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Image::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Image {
    Table,
    Id,
    S3Bucket,
    S3Key,
    CreatedAt,
}
