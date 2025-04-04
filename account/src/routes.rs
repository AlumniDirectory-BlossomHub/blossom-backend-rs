use crate::auth::jwt;
use crate::auth::jwt::JWTConfig;
use crate::guards::User;
use crate::validators::{validate_email, validate_password_level};
use aws_sdk_s3::Client;
use entity::user::{
    hash_password, AccountStatus, AuthUser, User as UserModel, UserProfile, UserVerificationToken,
};
use image_service::ImageServices;
use rocket::figment::Profile;
use rocket::form::{Contextual, Form};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, routes, FromForm, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, FromForm, Serialize, Deserialize)]
struct RegisterReq {
    #[field(validate = validate_email())]
    email: String,
    #[field(validate = len(2..32).or_else(msg!("Username length must be between 2 and 32 characters")))]
    username: String,
    #[field(validate = len(8..32).or_else(msg!("Password length must be between 8 and 32 characters")))]
    #[field(validate = validate_password_level())]
    password: String,
}

#[post("/account/register", data = "<form>")]
async fn register(
    form: Form<Contextual<'_, RegisterReq>>,
    image_services: &State<ImageServices>,
    s3_client: &State<Client>,
    pool: &State<PgPool>,
) -> Result<Json<UserProfile>, (Status, Json<HashMap<String, Vec<String>>>)> {
    // 表单校验
    let inner = form.into_inner();
    if let None = inner.value {
        let mut errors = HashMap::new();
        for error in inner.context.errors() {
            let field_name = error.name.as_ref().unwrap().to_string();
            let message = error.to_string();

            errors
                .entry(field_name)
                .or_insert_with(Vec::new)
                .push(message);
        }
        return Err((Status::BadRequest, Json(errors)));
    }
    let data = inner.value.unwrap();
    // 校验邮箱是否存在
    let exists: bool = sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM "user" WHERE email=$1)"#)
        .bind(&data.email)
        .fetch_one(pool.inner())
        .await
        .unwrap();
    if exists {
        return Err((
            Status::BadRequest,
            Json(HashMap::from([(
                "email".to_string(),
                Vec::from(["Email existed".to_string()]),
            )])),
        ));
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
    // TODO: 发送邮件
    verification_token.save(pool.inner()).await.unwrap();

    user.sign_avatar(&image_services.avatar, s3_client.inner())
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

#[derive(Debug, FromForm, Serialize, Deserialize)]
struct LoginReq {
    #[field(validate = validate_email())]
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
    s3_client: &State<Client>,
    user: User,
) -> Json<UserProfile> {
    let mut user_profile = UserProfile::from_user(user.0);
    user_profile
        .sign_avatar(&image_services.avatar, s3_client.inner())
        .await
        .unwrap();
    Json(user_profile)
}

#[get("/account/profile", rank = 2)]
async fn profile_unauthorized() -> (Status, &'static str) {
    (Status::Unauthorized, "invalid token")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![register, login, verification, profile, profile_unauthorized]
}
