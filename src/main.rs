#[macro_use]
extern crate rocket;

use crate::tests::image;
use account::auth::jwt::JWTConfig;
use image_service::storage::create_client;
use image_service::ImageServices;
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
    // 初始化 MinIO
    let s3_client = create_client().await;

    // 初始化图像服务
    let image_services = ImageServices::init(&s3_client).await;

    rocket::build()
        .manage(db)
        .manage(s3_client)
        .manage(image_services)
        .manage(JWTConfig::from_env())
        .mount("/", routes![index])
        .mount("/", account::routes())
        .mount("/test", image::routes())
}
