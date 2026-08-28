use std::fmt;

/// Errors that can occur while handling authentication domain operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    SessionNotFound,
    SessionExpired,
    SessionRevoked,
}

impl fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidCredentials => "invalid credentials",
            Self::UserNotFound => "user not found",
            Self::SessionNotFound => "session not found",
            Self::SessionExpired => "session has expired",
            Self::SessionRevoked => "session has been revoked",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for AuthError {}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::AuthError;

    #[test]
    fn displays_authentication_error_in_english() {
        assert_eq!(
            AuthError::InvalidCredentials.to_string(),
            "invalid credentials"
        );
        assert_eq!(AuthError::SessionExpired.to_string(), "session has expired");
    }

    #[test]
    fn implements_error() {
        let error: &dyn Error = &AuthError::SessionRevoked;

        assert_eq!(error.to_string(), "session has been revoked");
    }
}
