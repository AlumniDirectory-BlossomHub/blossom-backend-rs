use rocket::form::{Error, Result};
use rocket::fs::TempFile;

/// 用于校验 Option 字段的宏
/// Usage:
///   validate_opt!( expr )()
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

pub fn is_image_file<'v>(file: &TempFile<'_>) -> Result<'v, ()> {
    if let Some(ct) = file.content_type() {
        if ct.top() == "image" {
            return Ok(());
        }
    }
    Err(Error::validation("content_type must be image/*").into())
}
