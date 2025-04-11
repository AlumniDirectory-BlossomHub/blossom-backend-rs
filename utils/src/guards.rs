use rocket::data::{FromData, Outcome};
use rocket::form::{Form, FromForm};
use rocket::http::Status;
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::{Data, Request};
use std::collections::HashMap;

#[doc(hidden)]
#[derive(Debug)]
pub struct ValidatedForm<T>(pub T);

/// Validation guard
///
/// 捕获并向客户端返回 BadRequest
///
/// Examples:
/// ```
/// # use rocket::{post, FromForm};
/// # use utils::guards::{ValidateError, ValidatedForm, ValidatedFormResult};
/// # type YourResp = ();
/// # #[derive(FromForm)]
/// # struct YourDataReq {}
/// #
/// #[post("/your_route", data="<data>")]
/// async fn your_route(data: ValidatedFormResult<YourDataReq>) -> Result<YourResp, ValidateError> {
///     let ValidatedForm(data) = data?;
///
///     // your code
///
///     Ok(YourResp {})
/// }
/// ```
///
/// 如果需要进入路由函数再判断
/// ```
/// # use rocket::{post, FromForm};
/// use rocket::response::status::BadRequest;
/// use rocket::serde::json::Json;
/// use std::collections::HashMap;
/// # use utils::guards::{ValidateError, ValidatedForm, ValidatedFormResult};
/// # type YourResp = ();
/// # #[derive(FromForm)]
/// # struct YourDataReq {}
/// #
///
/// #[post("/your_route", data="<data>")]
/// async fn your_route(data: ValidatedFormResult<YourDataReq>) -> Result<YourResp, ValidateError> {
///     let ValidatedForm(data) = data?;
///
///     // return ValidateError
///     Err(BadRequest(Json(HashMap::from([(
///             "email".to_string(),
///             Vec::from(["Email existed".to_string()]),
///         )]))))
/// }
/// ```
///
pub type ValidatedFormResult<T> = Result<ValidatedForm<T>, ValidateError>;

/// 表单验证错误
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
