use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct UserEmail(String);

impl UserEmail {
    /// Returns an instance of UserEmail if the input satisfies all our
    /// validation constraints on a user's email
    /// It panics otherwise
    pub fn parse(s: String) -> Result<UserEmail, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid email.", s))
        } else {
            Ok(Self(s))
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
    use crate::domain::user_email::UserEmail;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_graphene_long_email_is_valid() {
        let email = "a".repeat(256);
        assert_ok!(UserEmail::parse(email));
    }

    #[test]
    fn email_longer_than_256_graphemes_is_rejected() {
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
