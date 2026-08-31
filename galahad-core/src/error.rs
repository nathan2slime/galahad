use std::fmt;

/// Errors that can occur while handling authentication domain operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthError {
    InvalidEmail,
    InvalidPassword,
    InvalidCredentials,
    UserNotFound,
    SessionNotFound,
    SessionExpired,
    SessionRevoked,
    UserAlreadyExists,
    PersistenceFailure,
    PasswordHashingFailure,
}

impl AuthError {
    /// Returns a stable machine-readable code suitable for localization keys.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidEmail => "auth.invalid_email",
            Self::InvalidPassword => "auth.invalid_password",
            Self::InvalidCredentials => "auth.invalid_credentials",
            Self::UserNotFound => "auth.user_not_found",
            Self::SessionNotFound => "auth.session_not_found",
            Self::SessionExpired => "auth.session_expired",
            Self::SessionRevoked => "auth.session_revoked",
            Self::UserAlreadyExists => "auth.user_already_exists",
            Self::PersistenceFailure => "auth.persistence_failure",
            Self::PasswordHashingFailure => "auth.password_hashing_failure",
        }
    }

    /// Returns a stable machine-readable code safe to expose to clients.
    pub const fn public_code(&self) -> &'static str {
        match self {
            Self::UserAlreadyExists | Self::UserNotFound => "auth.request_failed",
            _ => self.code(),
        }
    }

    /// Returns a message safe to expose to clients.
    pub const fn public_message(&self) -> &'static str {
        match self {
            Self::UserAlreadyExists | Self::UserNotFound => "request failed",
            Self::InvalidEmail => "invalid email",
            Self::InvalidPassword => "invalid password",
            Self::InvalidCredentials => "invalid credentials",
            Self::SessionNotFound => "session not found",
            Self::SessionExpired => "session has expired",
            Self::SessionRevoked => "session has been revoked",
            Self::PersistenceFailure => "persistence operation failed",
            Self::PasswordHashingFailure => "password hashing failed",
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidEmail => "invalid email",
            Self::InvalidPassword => "invalid password",
            Self::InvalidCredentials => "invalid credentials",
            Self::UserNotFound => "user not found",
            Self::SessionNotFound => "session not found",
            Self::SessionExpired => "session has expired",
            Self::SessionRevoked => "session has been revoked",
            Self::UserAlreadyExists => "user already exists",
            Self::PersistenceFailure => "persistence operation failed",
            Self::PasswordHashingFailure => "password hashing failed",
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
        assert_eq!(AuthError::InvalidEmail.to_string(), "invalid email");
        assert_eq!(AuthError::InvalidPassword.to_string(), "invalid password");
        assert_eq!(
            AuthError::InvalidCredentials.to_string(),
            "invalid credentials"
        );
        assert_eq!(AuthError::SessionExpired.to_string(), "session has expired");
        assert_eq!(
            AuthError::UserAlreadyExists.to_string(),
            "user already exists"
        );
        assert_eq!(
            AuthError::PersistenceFailure.to_string(),
            "persistence operation failed"
        );
        assert_eq!(
            AuthError::PasswordHashingFailure.to_string(),
            "password hashing failed"
        );
    }

    #[test]
    fn implements_error() {
        let error: &dyn Error = &AuthError::SessionRevoked;

        assert_eq!(error.to_string(), "session has been revoked");
    }

    #[test]
    fn exposes_stable_localization_codes() {
        assert_eq!(AuthError::InvalidEmail.code(), "auth.invalid_email");
        assert_eq!(AuthError::InvalidPassword.code(), "auth.invalid_password");
        assert_eq!(
            AuthError::InvalidCredentials.code(),
            "auth.invalid_credentials"
        );
        assert_eq!(AuthError::UserNotFound.code(), "auth.user_not_found");
        assert_eq!(AuthError::SessionNotFound.code(), "auth.session_not_found");
        assert_eq!(AuthError::SessionExpired.code(), "auth.session_expired");
        assert_eq!(AuthError::SessionRevoked.code(), "auth.session_revoked");
        assert_eq!(
            AuthError::UserAlreadyExists.code(),
            "auth.user_already_exists"
        );
        assert_eq!(
            AuthError::PersistenceFailure.code(),
            "auth.persistence_failure"
        );
        assert_eq!(
            AuthError::PasswordHashingFailure.code(),
            "auth.password_hashing_failure"
        );
    }

    #[test]
    fn exposes_user_enumeration_safe_public_codes() {
        assert_eq!(
            AuthError::UserAlreadyExists.public_code(),
            "auth.request_failed"
        );
        assert_eq!(AuthError::UserNotFound.public_code(), "auth.request_failed");
        assert_eq!(
            AuthError::InvalidCredentials.public_code(),
            "auth.invalid_credentials"
        );
        assert_eq!(AuthError::InvalidEmail.public_code(), "auth.invalid_email");
    }

    #[test]
    fn exposes_user_enumeration_safe_public_messages() {
        assert_eq!(
            AuthError::UserAlreadyExists.public_message(),
            "request failed"
        );
        assert_eq!(AuthError::UserNotFound.public_message(), "request failed");
        assert_eq!(
            AuthError::InvalidCredentials.public_message(),
            "invalid credentials"
        );
        assert_eq!(
            AuthError::InvalidPassword.public_message(),
            "invalid password"
        );
    }
}
