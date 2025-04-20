use aws_sdk_s3::Client;
use chrono::{DateTime, NaiveDate, Utc};
use image_service::errors::ImageError;
use image_service::service::ImageService;
use rocket::form::{FromFormField, ValueField};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "gender")]
#[sqlx(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
}

#[rocket::async_trait]
impl<'v> FromFormField<'v> for Gender {
    fn from_value(field: ValueField<'v>) -> rocket::form::Result<'v, Self> {
        match field.value.to_lowercase().as_str() {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            other => {
                Err(rocket::form::Error::validation(format!("invalid gender: {}", other)).into())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct Person {
    pub id: i32,

    /* 基本信息 */
    pub name: String,
    pub birthday: Option<NaiveDate>,
    pub gender: Gender,

    pub photo_id: Option<String>,

    /* 联系方式 */
    pub phone: Option<String>,
    pub email: Option<String>,
    pub qq: Option<String>,
    pub wechat: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct PersonProfile {
    pub id: i32,

    /* 基本信息 */
    pub name: String,
    pub birthday: Option<NaiveDate>,
    pub gender: Gender,

    #[sqlx(skip)]
    pub photo: String,
    #[serde(skip_serializing)]
    pub photo_id: Option<String>,

    /* 联系方式 */
    pub phone: Option<String>,
    pub email: Option<String>,
    pub qq: Option<String>,
    pub wechat: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PersonProfile {
    pub fn from_person(person: Person) -> PersonProfile {
        Self {
            id: person.id,
            name: person.name,
            birthday: person.birthday,
            gender: person.gender,
            photo: String::new(),
            photo_id: person.photo_id,
            phone: person.phone,
            email: person.email,
            qq: person.qq,
            wechat: person.wechat,
            created_at: person.created_at,
            updated_at: person.updated_at,
        }
    }

    pub async fn sign(
        &mut self,
        image_service: &ImageService,
        s3_client: &Client,
    ) -> Result<(), ImageError> {
        if let Some(photo_id) = &self.photo_id {
            self.photo = image_service.get_presigned_url(s3_client, photo_id).await?;
        };
        Ok(())
    }
}
