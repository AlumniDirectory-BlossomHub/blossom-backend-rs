use crate::auth::jwt;
use crate::auth::jwt::JWTConfig;
use crate::guards::User;
use crate::validators::validate_password_level;
use chrono::Utc;
use email::{templates, EmailBackend};
use entity::user::{hash_password, AccountStatus, AuthUser, UserProfile, UserVerificationToken};
use image_service::utils::open_image;
use image_service::{ImageServices, S3Client};
use lettre::{AsyncTransport, Message};
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::{get, patch, post, put, routes, FromForm, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use utils::generate_partial_form;
use utils::guards::{ValidateError, ValidatedForm, ValidatedFormResult};
use utils::validators::{is_email, is_image_file};
use uuid::Uuid;

#[derive(Debug, FromForm, Serialize, Deserialize)]
struct RegisterReq {
    #[field(validate = is_email())]
    email: String,
    #[field(validate =
        len(2..32).or_else(msg!("Username length must be between 2 and 32 characters"))
    )]
    username: String,
    #[field(validate =
        len(8..32).or_else(msg!("Password length must be between 8 and 32 characters"))
    )]
    #[field(validate = validate_password_level())]
    password: String,
}

#[post("/account/register", data = "<form>")]
async fn register(
    form: ValidatedFormResult<RegisterReq>,
    image_services: &State<ImageServices>,
    s3_client: &State<S3Client>,
    email_backend: &State<EmailBackend>,
    pool: &State<PgPool>,
) -> Result<Json<UserProfile>, ValidateError> {
    // 表单校验
    let ValidatedForm(data) = form?;
    // 校验邮箱是否存在
    let exists: bool = sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM "user" WHERE email=$1)"#)
        .bind(&data.email)
        .fetch_one(pool.inner())
        .await
        .unwrap();
    if exists {
        return Err(BadRequest(Json(HashMap::from([(
            "email".to_string(),
            Vec::from(["Email existed".to_string()]),
        )]))));
    }
    // 创建用户
    sqlx::query(r#"INSERT INTO "user" (email, username, password) VALUES ($1, $2, $3)"#)
        .bind(&data.email)
        .bind(&data.username)
        .bind(hash_password(&data.password).unwrap())
        .execute(pool.inner())
        .await
        .unwrap();

    // 获取用户信息并返回
    let mut user = sqlx::query_as::<_, UserProfile>(
        r#"SELECT id, email, admin_level, username, avatar_id, status, created_at, updated_at FROM "user" WHERE email=$1"#,
    )
        .bind(&data.email)
        .fetch_one(pool.inner())
        .await
        .unwrap();

    // 创建激活码
    let verification_token = UserVerificationToken::new(user.id);
    println!("{:?}", verification_token.token);
    // 发送邮件
    let mut context = tera::Context::new();
    context.insert(
        "verification_url",
        std::env::var("APP_ACCOUNT_ACTIVATION_URL_FORMAT")
            .unwrap_or("http://localhost:8000/account/verification/<token>".to_string())
            .replace("<token>", &verification_token.token.to_string())
            .as_str(),
    );
    let body = templates().render("verification.html", &context).unwrap();
    // 构建邮件
    let email = Message::builder()
        .from(email_backend.from.clone())
        .to(user.email.parse().unwrap())
        .subject("Welcome to our service!")
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(body)
        .unwrap();
    match email_backend.send(email).await {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    }

    // 保存
    verification_token.save(pool.inner()).await.unwrap();

    user.sign_avatar(&image_services.avatar, &s3_client.external)
        .await
        .unwrap();
    Ok(Json(user))
}

#[post("/account/verification/<token>")]
async fn verification(pool: &State<PgPool>, token: Uuid) -> (Status, &'static str) {
    match UserVerificationToken::verify(pool.inner(), &token)
        .await
        .unwrap()
    {
        Some(token) => {
            sqlx::query(r#"UPDATE "user" SET status=$1 WHERE id=$2"#)
                .bind(AccountStatus::Active)
                .bind(token.user_id)
                .execute(pool.inner())
                .await
                .unwrap();
            token.delete(pool.inner()).await.unwrap();
            (Status::Ok, "Success")
        }
        None => (Status::Unauthorized, "Invalid token"),
    }
}

#[allow(unused_variables)]
#[get("/account/verification/<token>")]
async fn verification_get(token: Uuid) -> Status {
    Status::MethodNotAllowed
}

#[derive(Debug, FromForm, Serialize, Deserialize)]
struct LoginReq {
    #[field(validate = is_email())]
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResp {
    token: String,
}

#[post("/auth/login", data = "<credentials>")]
async fn login(
    pool: &State<PgPool>,
    jwt_config: &State<JWTConfig>,
    credentials: Form<LoginReq>,
) -> Result<Json<LoginResp>, (Status, &'static str)> {
    let user = sqlx::query_as::<_, AuthUser>(
        r#"SELECT id, email, password, status FROM "user" WHERE email=$1"#,
    )
    .bind(&credentials.email)
    .fetch_optional(pool.inner())
    .await
    .unwrap();
    match user {
        Some(user) => match user.verify_password(credentials.password.as_str()) {
            Ok(true) => match user.status {
                AccountStatus::Inactive => Err((Status::Unauthorized, "your account is inactive")),
                AccountStatus::Active => Ok(Json(LoginResp {
                    token: jwt::create_token(user.id, jwt_config.inner()),
                })),
            },
            _ => Err((Status::Unauthorized, "wrong email or password.")),
        },
        None => Err((Status::Unauthorized, "wrong email or password.")),
    }
}

#[get("/account/profile", rank = 1)]
async fn profile(
    image_services: &State<ImageServices>,
    s3_client: &State<S3Client>,
    user: User,
) -> Json<UserProfile> {
    let mut user_profile = UserProfile::from_user(user.0);
    user_profile
        .sign_avatar(&image_services.avatar, &s3_client.external)
        .await
        .unwrap();
    Json(user_profile)
}

#[get("/account/profile", rank = 2)]
async fn profile_unauthorized() -> (Status, &'static str) {
    (Status::Unauthorized, "invalid token")
}

generate_partial_form! {
    #[derive(Debug, FromForm)]
    struct UpdateProfileReq<'r> {
        #[validate(len(2..32).or_else(msg!("Username length must be between 2 and 32 characters")))]
        username: String,
        #[validate(is_image_file())]
        avatar: TempFile<'r>,
    }
}

#[put("/account/profile", data = "<data>")]
async fn update_profile(
    pool: &State<PgPool>,
    image_services: &State<ImageServices>,
    s3_client: &State<S3Client>,
    user: User,
    data: ValidatedFormResult<UpdateProfileReq<'_>>,
) -> Result<Json<UserProfile>, ValidateError> {
    let ValidatedForm(data) = data?;
    let avatar = open_image(data.avatar.path().unwrap());
    let new_key = image_services
        .avatar
        .upload_image(&s3_client.internal, avatar)
        .await
        .unwrap();
    if let Some(old_key) = &user.0.avatar_id {
        image_services
            .avatar
            .delete_image(&s3_client.internal, old_key)
            .await
            .unwrap();
    }
    sqlx::query(r#"UPDATE "user" SET username=$1, avatar_id=$2 WHERE id=$3"#)
        .bind(&data.username)
        .bind(&new_key)
        .bind(&user.0.id)
        .execute(pool.inner())
        .await
        .unwrap();

    let mut profile = UserProfile::from_user(user.0);
    profile.username = data.username;
    profile.avatar_id = Some(new_key);
    profile.updated_at = Utc::now(); // 这里其实是不准确的
    profile
        .sign_avatar(&image_services.avatar, &s3_client.external)
        .await
        .unwrap();
    Ok(Json(profile))
}

#[patch("/account/profile", data = "<data>")]
async fn partial_update_profile(
    pool: &State<PgPool>,
    image_services: &State<ImageServices>,
    s3_client: &State<S3Client>,
    user: User,
    data: ValidatedFormResult<PartialUpdateProfileReq<'_>>,
) -> Result<Json<UserProfile>, ValidateError> {
    let ValidatedForm(data) = data?;
    let mut query = sqlx::QueryBuilder::new(r#"UPDATE "user" SET "#);
    let User(mut user) = user;
    if let Some(avatar) = data.avatar {
        let avatar = open_image(avatar.path().unwrap());
        let new_key = image_services
            .avatar
            .upload_image(&s3_client.internal, avatar)
            .await
            .unwrap();
        if let Some(old_key) = &user.avatar_id {
            image_services
                .avatar
                .delete_image(&s3_client.internal, old_key)
                .await
                .unwrap();
        }
        user.avatar_id = Some(new_key);
        query.push("avatar_id=");
        query.push_bind(&user.avatar_id);
        query.push(",");
    }
    if let Some(username) = data.username {
        user.username = username;
        query.push("username=");
        query.push_bind(&user.username);
        query.push(",");
    }
    let now = Utc::now();
    query.push("updated_at=");
    query.push_bind(&now);
    query.push(" WHERE id=");
    query.push_bind(&user.id);
    query.build().execute(pool.inner()).await.unwrap();
    let mut profile = UserProfile::from_user(user);
    profile.updated_at = now; // 这里其实是不准确的
    profile
        .sign_avatar(&image_services.avatar, &s3_client.external)
        .await
        .unwrap();
    Ok(Json(profile))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        register,
        login,
        verification,
        verification_get,
        profile,
        profile_unauthorized,
        update_profile,
        partial_update_profile
    ]
}
