use chrono::{NaiveDate, Utc};
use entity::person::{Gender, PersonProfile};
use image_service::utils::open_image;
use image_service::{ImageServices, S3Client};
use rocket::fs::TempFile;
use rocket::serde::json::Json;
use rocket::{post, routes, FromForm, State};
use sqlx::PgPool;
use utils::guards::{ValidateError, ValidatedForm, ValidatedFormResult};
use utils::validate_opt;
use utils::validators::{is_email, is_image_file, is_phone_number, is_ymd_date};

#[derive(FromForm)]
struct CreatePersonReq<'r> {
    #[field(validate = len(2..31).or_else(msg!("Person name length must be between 2 and 32 characters")))]
    name: String,
    /// 生日 YYYY-MM-DD
    #[field(validate = validate_opt!(is_ymd_date())())]
    birthday: Option<String>,
    gender: Gender,
    #[field(validate = validate_opt!(is_image_file())())]
    photo: Option<TempFile<'r>>,
    #[field(validate = validate_opt!(is_phone_number())())]
    phone: Option<String>,
    #[field(validate = validate_opt!(is_email())())]
    email: Option<String>,
    // #[field(validate=validate_opt!(is_qq()))]
    qq: Option<String>,
    wechat: Option<String>,
}

#[post("/person", data = "<data>")]
async fn create_person(
    s3_client: &State<S3Client>,
    pool: &State<PgPool>,
    image_services: &State<ImageServices>,
    data: ValidatedFormResult<CreatePersonReq<'_>>,
) -> Result<Json<PersonProfile>, ValidateError> {
    let ValidatedForm(data) = data?;
    let mut query = sqlx::QueryBuilder::new("INSERT INTO person (name, gender, ");
    if data.birthday.is_some() {
        query.push("birthday, ");
    }
    if data.phone.is_some() {
        query.push("phone, ");
    }
    if data.email.is_some() {
        query.push("email, ");
    }
    if data.qq.is_some() {
        query.push("qq, ");
    }
    if data.wechat.is_some() {
        query.push("wechat, ");
    }
    if data.photo.is_some() {
        query.push("photo_id, ");
    }
    query.push("created_at, updated_at) VALUES (");
    query.push_bind(&data.name);
    query.push(", ");
    query.push_bind(&data.gender);
    query.push(", ");
    if let Some(birthday) = data.birthday {
        query.push_bind(NaiveDate::parse_from_str(&birthday, "%Y-%m-%d").unwrap());
        query.push(", ");
    }
    if let Some(phone) = data.phone {
        query.push_bind(
            phonenumber::parse(Some(phonenumber::country::CN), phone.trim())
                .unwrap()
                .to_string(),
        );
        query.push(", ");
    }
    if let Some(email) = data.email {
        query.push_bind(email);
        query.push(", ");
    }
    if let Some(qq) = data.qq {
        query.push_bind(qq);
        query.push(", ");
    }
    if let Some(wechat) = data.wechat {
        query.push_bind(wechat);
        query.push(", ");
    }
    if let Some(photo) = data.photo {
        let photo = open_image(photo.path().unwrap());
        let key = image_services
            .person_photo
            .upload_image(&s3_client.internal, photo)
            .await
            .unwrap();
        query.push_bind(key);
        query.push(", ");
    }
    let now = Utc::now();
    query.push_bind(&now);
    query.push(", ");
    query.push_bind(&now);
    query.push(") RETURNING *");
    let mut person = query
        .build_query_as::<PersonProfile>()
        .fetch_one(pool.inner())
        .await
        .unwrap();
    person
        .sign(&image_services.person_photo, &s3_client.external)
        .await
        .unwrap();
    Ok(Json(person))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create_person]
}
