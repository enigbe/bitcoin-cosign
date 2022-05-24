use validator::validate_email;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserEmail(String);

impl UserEmail {
    /// Returns an instance of UserEmail if the input satisfies all our
    /// validation constraints on a user's email
    /// It panics otherwise
    pub fn parse(s: String) -> Result<UserEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email.", s))
        }
    }
}

impl AsRef<str> for UserEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::UserEmail;
    use claim::{assert_err, assert_ok};

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "johndoe.com".to_string();
        assert_err!(UserEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@email.com".to_string();
        assert_err!(UserEmail::parse(email));
    }

    #[test]
    fn email_longer_than_256_is_rejected() {
        let email = "a".repeat(257);
        assert_err!(UserEmail::parse(email));
    }

    #[test]
    fn whitespace_only_emails_are_rejected() {
        let email = " ".to_string();
        assert_err!(UserEmail::parse(email));
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(UserEmail::parse(email));
    }

    #[test]
    fn emails_containing_an_invalid_character_are_rejected() {
        for email in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let email = email.to_string();
            assert_err!(UserEmail::parse(email));
        }
    }

    #[test]
    fn a_valid_email_is_parsed_successfully() {
        let email = "janedoe@email.com".to_string();
        assert_ok!(UserEmail::parse(email));
    }
}
