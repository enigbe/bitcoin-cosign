use crate::domain::{UserEmail, UserPassword};

#[derive(serde::Deserialize)]
pub struct User {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct NewUser {
    pub email: UserEmail,
    pub password: UserPassword,
}

impl TryFrom<User> for NewUser {
    type Error = String;

    fn try_from(value: User) -> Result<Self, Self::Error> {
        let email = UserEmail::parse(value.email)?;
        let password = UserPassword::parse(value.password)?;

        Ok(Self { email, password })
    }
}
