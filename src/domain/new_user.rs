use crate::domain::user_email::UserEmail;

#[derive(Debug)]
pub struct NewUser {
    pub email: UserEmail,
    pub password: String,
}
