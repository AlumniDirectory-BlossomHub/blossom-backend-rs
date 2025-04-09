use lettre::Address;
use rocket::form;
use zxcvbn::zxcvbn;

pub fn validate_email<'v>(email: &String) -> form::Result<'v, ()> {
    match email.parse::<Address>() {
        Ok(_) => Ok(()),
        Err(_) => Err(form::Error::validation("Invalid email").into()),
    }
}

pub fn validate_password_level<'v>(password: &String) -> form::Result<'v, ()> {
    let result = zxcvbn(password, &[]);
    if result.score() <= zxcvbn::Score::Two {
        Err(form::Error::validation("Password strength is too low"))?;
    }
    Ok(())
}
