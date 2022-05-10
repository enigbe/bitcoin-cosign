use crate::domain::{UserEmail, UserPassword};

#[derive(Debug)]
pub struct NewUser {
    pub email: UserEmail,
    pub password: UserPassword,
}
