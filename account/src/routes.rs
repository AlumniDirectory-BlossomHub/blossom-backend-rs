use crate::auth::jwt;
use crate::auth::jwt::JWTConfig;
use crate::validators::{validate_email, validate_password_level};
use aws_sdk_s3::Client;
use entity::user::{hash_password, AuthUser, User as UserModel, UserProfile};
use image_service::ImageServices;
use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{post, routes, FromForm, Responder, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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

#[post("/account/register", data = "<data>")]
async fn register(
    data: Form<RegisterReq>,
    image_services: &State<ImageServices>,
    s3_client: &State<Client>,
    pool: &State<PgPool>,
) -> Json<UserProfile> {
    let exists: bool = sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM "user" WHERE email=$1)"#)
        .bind(&data.email)
        .fetch_one(pool.inner())
        .await
        .unwrap();
    if exists {
        todo!()
    }
    sqlx::query(r#"INSERT INTO "user" (email, username, password) VALUES ($1, $2, $3)"#)
        .bind(&data.email)
        .bind(&data.username)
        .bind(hash_password(&data.password).unwrap())
        .execute(pool.inner())
        .await
        .unwrap();

    let mut user = sqlx::query_as::<_, UserProfile>(
        r#"SELECT id, email, admin_level, username, avatar_id, status, created_at, updated_at FROM "user" WHERE email=$1"#,
    )
    .bind(&data.email)
    .fetch_one(pool.inner())
    .await
    .unwrap();
    user.sign_avatar(&image_services.avatar, s3_client.inner())
        .await
        .unwrap();
    Json(user)
}

#[post("/auth/login", data = "<credentials>")]
async fn login(
    pool: &State<PgPool>,
    jwt_config: &State<JWTConfig>,
    credentials: Form<LoginReq>,
) -> Result<Json<LoginResp>, (Status, &'static str)> {
    let user =
        sqlx::query_as::<_, AuthUser>(r#"SELECT id, email, password FROM "user" WHERE email=$1"#)
            .bind(&credentials.email)
            .fetch_optional(pool.inner())
            .await
            .unwrap();
    match user {
        Some(user) => match user.verify_password(credentials.password.as_str()) {
            Ok(true) => Ok(Json(LoginResp {
                token: jwt::create_token(user.id, jwt_config.inner()),
            })),
            _ => Err((Status::Unauthorized, "wrong email or password.")),
        },
        None => Err((Status::Unauthorized, "wrong email or password.")),
    }
}
//
// #[post("/auth/logout")]
// async fn logout() {
//     todo!()
// }

pub fn routes() -> Vec<rocket::Route> {
    routes![register, login]
}
