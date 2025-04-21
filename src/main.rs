//! blossom-backend-rs
//!
//! - run
//! ```shell
//! cargo run
//! ```
//! - run with hot reload
//! ```shell
//! cargo watch -x run
//! ```
#[macro_use]
extern crate rocket;

use crate::tests::image;
use account::auth::jwt::JWTConfig;
use email::EmailBackend;
use image_service::storage::create_client;
use image_service::{ImageServices, S3Client};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

mod tests;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

async fn db_connection() -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_with(
            PgConnectOptions::new()
                .host(
                    std::env::var("DATABASE_HOST")
                        .unwrap_or("localhost".to_string())
                        .as_str(),
                )
                .port(match std::env::var("DATABASE_PORT") {
                    Ok(port_str) => port_str.parse::<u16>().unwrap(),
                    _ => 5432,
                })
                .username(
                    std::env::var("APP_DB_USER")
                        .expect("APP_DB_USER must be set")
                        .as_str(),
                )
                .password(
                    std::env::var("APP_DB_PASSWORD")
                        .expect("APP_DB_PASSWORD must be set")
                        .as_str(),
                )
                .database(
                    std::env::var("APP_DB_NAME")
                        .expect("APP_DB_NAME must be set")
                        .as_str(),
                ),
        )
        .await
}

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    // 创建数据库连接池
    let db = db_connection()
        .await
        .expect("Failed to connect to database");

    // 执行迁移
    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    // 初始化 MinIO
    let internal_endpoint = std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set");
    let external_endpoint =
        std::env::var("MINIO_EXTERNAL_ENDPOINT").unwrap_or(internal_endpoint.clone());
    let region = std::env::var("MINIO_REGION").expect("MINIO_REGION must be set");
    let access_key =
        std::env::var("APP_MINIO_ACCESS_KEY").expect("APP_MINIO_ACCESS_KEY must be set");
    let secret_key =
        std::env::var("APP_MINIO_SECRET_KEY").expect("APP_MINIO_SECRET_KEY must be set");
    let s3_internal_client =
        create_client(&internal_endpoint, &region, &access_key, &secret_key).await;
    let s3_external_client =
        create_client(&external_endpoint, &region, &access_key, &secret_key).await;

    let s3_client = S3Client {
        internal: s3_internal_client,
        external: s3_external_client,
    };

    // 初始化图像服务
    let image_services = ImageServices::init(&s3_client.internal).await;

    // 初始化邮件服务
    let email = EmailBackend::from_env();
    match email.test_connection().await {
        Ok(true) => {}
        Ok(false) => {
            println!("Email connection does not exist");
        }
        Err(e) => {
            panic!("{}", e)
        }
    }

    rocket::build()
        .manage(db)
        .manage(s3_client)
        .manage(image_services)
        .manage(JWTConfig::from_env())
        .manage(email)
        .mount("/", routes![index])
        .mount("/", account::routes())
        .mount("/", person::routes())
        .mount("/test", image::routes())
}
