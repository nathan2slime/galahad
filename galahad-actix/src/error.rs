use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use galahad_core::AuthError;

use crate::response::ErrorResponse;

/// HTTP wrapper for Galahad authentication errors.
#[derive(Debug)]
pub struct ActixAuthError(pub AuthError);

impl From<AuthError> for ActixAuthError {
    fn from(error: AuthError) -> Self {
        Self(error)
    }
}

impl std::fmt::Display for ActixAuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0.public_message())
    }
}

impl ResponseError for ActixAuthError {
    fn status_code(&self) -> StatusCode {
        match self.0 {
            AuthError::InvalidEmail
            | AuthError::InvalidPassword
            | AuthError::InvalidSignUpField => StatusCode::BAD_REQUEST,
            AuthError::InvalidCredentials
            | AuthError::SessionExpired
            | AuthError::SessionRevoked
            | AuthError::SessionNotFound => StatusCode::UNAUTHORIZED,
            AuthError::UserAlreadyExists | AuthError::UserNotFound => StatusCode::BAD_REQUEST,
            AuthError::PersistenceFailure | AuthError::PasswordHashingFailure => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            code: self.0.public_code(),
            message: self.0.public_message(),
        })
    }
}
