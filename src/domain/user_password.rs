use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct UserPassword(String);

impl UserPassword {
    /// Returns an instance of UserEmail if the input satisfies all our
    /// validation constraints on a user's email
    /// It panics otherwise
    pub fn parse(s: String) -> Result<UserPassword, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid password.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for UserPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::UserPassword;
    use claim::{assert_err, assert_ok};

    #[test]
    fn password_longer_than_256_is_rejected() {
        let password = "a".repeat(257);
        assert_err!(UserPassword::parse(password));
    }

    #[test]
    fn whitespace_only_passwords_are_rejected() {
        let password = " ".to_string();
        assert_err!(UserPassword::parse(password));
    }

    #[test]
    fn empty_password_is_rejected() {
        let password = "".to_string();
        assert_err!(UserPassword::parse(password));
    }

    #[test]
    fn passwords_containing_an_invalid_character_are_rejected() {
        for password in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let password = password.to_string();
            assert_err!(UserPassword::parse(password));
        }
    }

    #[test]
    fn a_valid_password_is_parsed_successfully() {
        let password = "secretpassword".to_string();
        assert_ok!(UserPassword::parse(password));
    }
}
