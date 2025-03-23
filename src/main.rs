#[macro_use]
extern crate rocket;

mod storage;

use crate::storage::minio::{create_client, ensure_bucket_exists};
use aws_sdk_s3::Client;
use sea_orm::{Database, DatabaseConnection};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

pub struct AppState {
    db: DatabaseConnection,
    s3: Client,
}

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    // 创建数据库连接池
    let db = Database::connect(std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");
    // 初始化 MinIO
    let s3_client = create_client().await;
    ensure_bucket_exists(&s3_client).await;

    let state = AppState { db, s3: s3_client };

    rocket::build().manage(state).mount("/", routes![index])
}
