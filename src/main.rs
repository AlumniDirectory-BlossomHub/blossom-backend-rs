#[macro_use]
extern crate rocket;

use crate::tests::image;
use image_service::storage::create_client;
use image_service::ImageServices;
use sqlx::postgres::PgPoolOptions;

mod tests;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    // 创建数据库连接池
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set")
                .as_str(),
        )
        .await
        .expect("Failed to connect to database");
    // 初始化 MinIO
    let s3_client = create_client().await;

    // 初始化图像服务
    let image_services = ImageServices::init();

    rocket::build()
        .manage(db)
        .manage(s3_client)
        .manage(image_services)
        .mount("/", routes![index])
        .mount("/test", image::routes())
}
