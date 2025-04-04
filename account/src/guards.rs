use crate::auth::jwt::{validate_token, Claims, JWTConfig};
use entity::user::User as UserModel;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use sqlx::PgPool;

/// 仅校验 jwt claims 正确性，不进行数据库校验
pub struct UserClaims(pub Claims);
/// 仅判定用户是否存在
pub struct UserId(pub i32);
/// 判定用户是否存在并返回用户表全部内容
pub struct User(pub UserModel);

async fn get_claims_from_req(req: &Request<'_>) -> Result<Claims, &'static str> {
    let jwt_config = req.rocket().state::<JWTConfig>().unwrap();
    let token = req
        .headers()
        .get_one("Authorization")
        .and_then(|h| h.strip_prefix("Bearer "));
    match token {
        Some(token) => match validate_token(token, jwt_config) {
            Ok(claims) => Ok(claims),
            Err(_) => Err("Invalid token"),
        },
        None => Err("Not logged in"),
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserClaims {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match get_claims_from_req(req).await {
            Ok(claims) => request::Outcome::Success(UserClaims(claims)),
            Err(err) => request::Outcome::Error((Status::Unauthorized, err)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserId {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match get_claims_from_req(req).await {
            Ok(claims) => {
                let pool = req.rocket().state::<PgPool>().unwrap();
                let exists: bool =
                    sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM user WHERE id = $1)"#)
                        .bind(claims.sub)
                        .fetch_one(pool)
                        .await
                        .unwrap();

                if exists {
                    request::Outcome::Success(UserId(claims.sub))
                } else {
                    request::Outcome::Error((Status::Unauthorized, "Invalid token"))
                }
            }
            Err(err) => request::Outcome::Error((Status::Unauthorized, err)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match get_claims_from_req(req).await {
            Ok(claims) => {
                let pool = req.rocket().state::<PgPool>().unwrap();
                let user = sqlx::query_as::<_, UserModel>(r#"SELECT * FROM "user" WHERE id = $1"#)
                    .bind(claims.sub)
                    .fetch_optional(pool)
                    .await
                    .unwrap();
                match user {
                    Some(user) => request::Outcome::Success(User(user)),
                    None => request::Outcome::Error((Status::Unauthorized, "Invalid token")),
                }
            }
            Err(err) => request::Outcome::Error((Status::Unauthorized, err)),
        }
    }
}

//TODO: UserWithRole<RoleType>
