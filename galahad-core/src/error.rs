use std::fmt;

/// Errors that can occur while handling authentication domain operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    SessionNotFound,
    SessionExpired,
    SessionRevoked,
    PersistenceFailure,
}

impl AuthError {
    /// Returns a stable machine-readable code suitable for localization keys.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "auth.invalid_credentials",
            Self::UserNotFound => "auth.user_not_found",
            Self::SessionNotFound => "auth.session_not_found",
            Self::SessionExpired => "auth.session_expired",
            Self::SessionRevoked => "auth.session_revoked",
            Self::PersistenceFailure => "auth.persistence_failure",
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidCredentials => "invalid credentials",
            Self::UserNotFound => "user not found",
            Self::SessionNotFound => "session not found",
            Self::SessionExpired => "session has expired",
            Self::SessionRevoked => "session has been revoked",
            Self::PersistenceFailure => "persistence operation failed",
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
        assert_eq!(
            AuthError::PersistenceFailure.to_string(),
            "persistence operation failed"
        );
    }

    #[test]
    fn implements_error() {
        let error: &dyn Error = &AuthError::SessionRevoked;

        assert_eq!(error.to_string(), "session has been revoked");
    }

    #[test]
    fn exposes_stable_localization_codes() {
        assert_eq!(
            AuthError::InvalidCredentials.code(),
            "auth.invalid_credentials"
        );
        assert_eq!(AuthError::UserNotFound.code(), "auth.user_not_found");
        assert_eq!(AuthError::SessionNotFound.code(), "auth.session_not_found");
        assert_eq!(AuthError::SessionExpired.code(), "auth.session_expired");
        assert_eq!(AuthError::SessionRevoked.code(), "auth.session_revoked");
        assert_eq!(
            AuthError::PersistenceFailure.code(),
            "auth.persistence_failure"
        );
    }
}
