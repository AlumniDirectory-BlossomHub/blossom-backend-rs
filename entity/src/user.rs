use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use aws_sdk_s3::Client;
use chrono::{DateTime, Utc};
use image_service::errors::ImageError;
use image_service::service::ImageService;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::{FromRow, PgPool, Type};

/// 账户状态
#[derive(Clone, Debug, PartialEq, Eq, Type, Serialize, Deserialize)]
#[repr(i16)]
pub enum AccountStatus {
    /// 未激活
    Inactive = 0,
    /// 已激活
    Active = 1,
}

/// 数据库完整表单
#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    /// VARCHAR(255)
    pub email: String,
    /// VARCHAR(255), argon2 加密
    pub password: String,
    /// SMALLINT
    pub admin_level: i16,
    /// VARCHAR(255)
    pub username: String,
    /// 头像 s3_key, VARCHAR(36), uuid 转字符串
    pub avatar_id: Option<String>,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户认证表单
#[derive(Clone, Debug, FromRow)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub status: AccountStatus,
}

/// 用户资料
///
/// 前端展示给用户本人
#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i32,
    pub email: String,
    pub admin_level: i16,
    pub username: String,
    #[sqlx(skip)]
    pub avatar: String,
    #[serde(skip_serializing)]
    pub avatar_id: Option<String>,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 账户激活 token
#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize)]
pub struct UserVerificationToken {
    pub user_id: i32,
    pub token: Uuid,
    pub expire: DateTime<Utc>,
}

impl User {
    /// 加密并修改密码
    ///
    /// attention: 不会保存到数据库
    pub fn set_password(&mut self, password: &str) -> Result<(), &'static str> {
        self.password = hash_password(password).map_err(|_| "Unable to hash password")?;
        Ok(())
    }
}

impl AuthUser {
    pub fn verify_password(&self, password: &str) -> Result<bool, &'static str> {
        let parsed_hash =
            PasswordHash::new(self.password.as_str()).map_err(|_| "Invalid password hash")?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

impl UserProfile {
    pub fn from_user(user: User) -> UserProfile {
        Self {
            id: user.id,
            email: user.email,
            admin_level: user.admin_level,
            username: user.username,
            avatar: String::new(),
            avatar_id: user.avatar_id,
            status: user.status,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }

    /// 给头像签名
    ///
    /// attention: 向前端返回前必须调用本函数
    pub async fn sign_avatar(
        &mut self,
        image_service: &ImageService,
        s3_client: &Client,
    ) -> Result<(), ImageError> {
        if let Some(avatar_id) = &self.avatar_id {
            self.avatar = image_service
                .get_presigned_url(s3_client, avatar_id)
                .await?;
        };
        Ok(())
    }
}

impl UserVerificationToken {
    pub fn new(user_id: i32) -> UserVerificationToken {
        Self {
            user_id,
            token: Uuid::new_v4(),
            expire: Utc::now()
                .checked_add_signed(chrono::Duration::days(3))
                .expect("Invalid timestamp"),
        }
    }

    pub async fn save(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO "user_verification_token" (user_id, token, expire) VALUES ($1, $2, $3)"#,
        )
        .bind(self.user_id)
        .bind(&self.token)
        .bind(&self.expire)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn verify(pool: &PgPool, token: &Uuid) -> Result<Option<Self>, sqlx::Error> {
        let row =
            sqlx::query_as::<_, Self>(r#"SELECT * FROM user_verification_token WHERE token = $1"#)
                .bind(token)
                .fetch_optional(pool)
                .await?;
        match row {
            None => Ok(None),
            Some(token) => {
                if token.expire >= Utc::now() {
                    Ok(Some(token))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn delete(self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM user_verification_token WHERE token = $1"#)
            .bind(self.token)
            .execute(pool)
            .await?;
        Ok(())
    }
}

/// 加密密码原文
///
/// 使用 argon2 加密
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}
