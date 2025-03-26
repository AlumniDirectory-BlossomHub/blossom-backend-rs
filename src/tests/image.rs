use aws_sdk_s3::Client;
use image::ImageReader;
use image_service::ImageServices;
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
    image_services: &State<ImageServices>,
) -> String {
    println!("{:?}", data.file.path());
    let image = ImageReader::open(data.file.path().unwrap())
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    let result = image_services.test.upload_image(db, s3_client, image).await;
    println!("{:?}", result);
    match result {
        Ok(model) => model.s3_key,
        Err(err) => format!("{:?}", err),
    }
}

#[get("/image/<key>")]
async fn get_image_url(
    db: &State<DatabaseConnection>,
    s3_client: &State<Client>,
    image_services: &State<ImageServices>,
    key: String,
) -> String {
    image_services
        .test
        .get_image_url_by_key(db, s3_client, key)
        .await
        .unwrap()
}
pub fn routes() -> Vec<rocket::Route> {
    routes![upload_image, get_image_url]
}
