use rocket::data::{FromData, Outcome};
use rocket::form::{Form, FromForm};
use rocket::http::Status;
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::{Data, Request};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ValidatedForm<T>(pub T);

pub type ValidatedFormResult<T> = Result<ValidatedForm<T>, ValidateError>;

pub type ValidateError = BadRequest<Json<HashMap<String, Vec<String>>>>;

#[rocket::async_trait]
impl<'r, T> FromData<'r> for ValidatedForm<T>
where
    T: FromForm<'r> + Send,
{
    type Error = ValidateError;
    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        match Form::<T>::from_data(req, data).await {
            Outcome::Success(form) => Outcome::Success(ValidatedForm(form.into_inner())),
            Outcome::Error(f) => {
                let mut errors = HashMap::new();
                for error in f.1 {
                    let field_name = error.name.as_ref().unwrap().to_string();
                    let message = error.to_string();

                    errors
                        .entry(field_name)
                        .or_insert_with(Vec::new)
                        .push(message);
                }
                Outcome::Error((Status::BadRequest, BadRequest(Json(errors))))
            }
            Outcome::Forward(f) => Outcome::Forward(f),
        }
    }
}
