use crate::{AuthError, ServiceResult};

const MINIMUM_PASSWORD_LENGTH: usize = 8;

/// Validates a password using the domain's intentionally small rule set.
pub(crate) fn validate_password(password: &str) -> ServiceResult<()> {
    if password.chars().count() < MINIMUM_PASSWORD_LENGTH {
        return Err(AuthError::InvalidPassword);
    }

    if !password.chars().any(|character| !character.is_whitespace()) {
        return Err(AuthError::InvalidPassword);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_password;

    #[test]
    fn accepts_password_at_minimum_length() {
        assert_eq!(validate_password("12345678"), Ok(()));
    }

    #[test]
    fn accepts_longer_password_with_spaces() {
        assert_eq!(validate_password("correct horse"), Ok(()));
    }

    #[test]
    fn rejects_short_password() {
        assert_eq!(
            validate_password("1234567"),
            Err(crate::AuthError::InvalidPassword)
        );
    }

    #[test]
    fn rejects_whitespace_only_password() {
        assert_eq!(
            validate_password("        "),
            Err(crate::AuthError::InvalidPassword)
        );
    }
}
