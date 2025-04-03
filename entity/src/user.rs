use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use aws_sdk_s3::Client;
use chrono::{DateTime, Utc};
use image_service::errors::ImageError;
use image_service::service::ImageService;
use serde::{Deserialize, Serialize, Serializer};
use sqlx::{FromRow, Type};

#[derive(Clone, Debug, PartialEq, Eq, Type, Serialize, Deserialize)]
#[repr(i16)]
pub enum AccountStatus {
    Inactive = 0,
    Active = 1,
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String, // 密码 hash 值
    pub admin_level: i16,
    pub username: String,
    pub avatar_id: Option<String>,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i32,
    pub email: String,
    pub admin_level: i16,
    pub username: String,
    #[sqlx(skip)]
    pub avatar: String,
    #[serde(skip_serializing)]
    avatar_id: Option<String>,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
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

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}
