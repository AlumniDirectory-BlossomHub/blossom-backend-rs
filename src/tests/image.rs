use aws_sdk_s3::Client;
use image::{ImageFormat, ImageReader};
use image_service::service::ImageService;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::State;
use sea_orm::DatabaseConnection;

#[derive(FromForm)]
struct Upload<'r> {
    file: TempFile<'r>,
}

#[post("/image/upload", data = "<data>")]
async fn upload_image(
    data: Form<Upload<'_>>,
    db: &State<DatabaseConnection>,
    s3_client: &State<Client>,
) {
    let image_service = ImageService::new(
        db,
        s3_client,
        "test".to_string(),
        Some(ImageFormat::Jpeg),
        Some((500u32, 500u32)),
        None,
    )
    .await;

    println!("{:?}", data.file.path());
    let image = ImageReader::open(data.file.path().unwrap())
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    let result = image_service.upload_image(image).await;
    println!("{:?}", result);
}

#[get("/image/<key>")]
async fn get_image_url(
    db: &State<DatabaseConnection>,
    s3_client: &State<Client>,
    key: String,
) -> String {
    let image_service = ImageService::new(
        db,
        s3_client,
        "test".to_string(),
        Some(ImageFormat::Jpeg),
        Some((500u32, 500u32)),
        None,
    )
    .await;
    image_service.get_image_url_by_key(key).await.unwrap()
}
pub fn routes() -> Vec<rocket::Route> {
    routes![upload_image, get_image_url]
}
