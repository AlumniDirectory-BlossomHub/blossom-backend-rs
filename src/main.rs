#[macro_use]
extern crate rocket;

use crate::tests::image;
use image_service::storage::create_client;
use sea_orm::Database;

mod tests;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
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

    rocket::build()
        .manage(db)
        .manage(s3_client)
        .mount("/", routes![index])
        .mount("/test", image::routes())
}
