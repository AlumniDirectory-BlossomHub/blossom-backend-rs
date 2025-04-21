use rocket::form;
use zxcvbn::zxcvbn;

/// 密码等级校验
pub fn validate_password_level<'v>(password: &String) -> form::Result<'v, ()> {
    let result = zxcvbn(password, &[]);
    if result.score() <= zxcvbn::Score::Two {
        Err(form::Error::validation("Password strength is too low"))?;
    }
    Ok(())
}
