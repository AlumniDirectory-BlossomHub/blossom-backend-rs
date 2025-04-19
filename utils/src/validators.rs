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
