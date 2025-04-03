// src/auth/jwt.rs
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,   // 用户ID
    pub exp: usize, // 过期时间戳
}

pub struct JWTConfig {
    pub secret: String,
    pub expiration: i64, // 秒
}

impl JWTConfig {
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            expiration: 3600 * 24,
        }
    }
}

pub fn create_token(user_id: i32, config: &JWTConfig) -> String {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(config.expiration))
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_ref()),
    )
    .unwrap()
}

pub fn validate_token(
    token: &str,
    config: &JWTConfig,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}
