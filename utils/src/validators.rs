use chrono::NaiveDate;
use lettre::Address;
use phonenumber::{country, parse};
use rocket::form;
use rocket::form::{Error, Result};
use rocket::fs::TempFile;

/// 用于校验 Option 字段的宏
///
/// Examples:
/// ```
/// # use utils::validate_opt;
/// # use rocket::FromForm;
/// #[derive(FromForm)]
/// struct PartialUpdateReq {
///     #[field(validate=validate_opt!(len(2..32))())]
///     username: String,
///     // other fields
/// }
/// ```
/// will expand to
/// ```
/// # use utils::validate_opt;
/// # use rocket::FromForm;
/// #[derive(FromForm)]
/// struct PartialUpdateReq {
///     #[field(
///         validate=|x: &Option<_>| match x {
///             Some(v) => len(&v, 2..32),
///             None => Ok(()),
///         }
///     )]
///     username: String,
///     // other fields
/// }
/// ```
#[macro_export]
macro_rules! validate_opt {
    ($func:ident($($args:tt)*) $(.$method:ident($($margs:tt)*))*) => {
        (|x: &Option<_>| match x {
            Some(v) => $func(&v, $($args)*) $(.$method($($margs)*))*,
            None => Ok(()),
        })
    };
    // 去除 ()
    (($($other:tt)*)) => {
        validate_opt!($($other)*)
    }
}

/// 校验文件类型是否是 image/*
pub fn is_image_file<'v>(file: &TempFile<'_>) -> Result<'v, ()> {
    if let Some(ct) = file.content_type() {
        if ct.top() == "image" {
            return Ok(());
        }
    }
    Err(Error::validation("content_type must be image/*").into())
}

/// 校验邮箱合法性
pub fn is_email<'v>(email: &String) -> form::Result<'v, ()> {
    match email.parse::<Address>() {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::validation("Invalid email").into()),
    }
}

/// 校验日期合法性
///
/// format %Y-%m-%d
pub fn is_ymd_date<'v>(date: &String) -> form::Result<'v, ()> {
    match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::validation("Invalid date").into()),
    }
}

/// 校验手机号合法性
pub fn is_phone_number<'v>(phone: &String) -> form::Result<'v, ()> {
    let input = phone.trim();

    // 默认国家设置为中国
    match parse(Some(country::CN), input) {
        Ok(parsed_number) => {
            if parsed_number.is_valid() {
                Ok(())
            } else {
                println!("111");
                Err(Error::validation("Invalid phone number").into())
            }
        }
        Err(_) => Err(Error::validation("Unable to parse phone number").into()),
    }
}
