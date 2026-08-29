use std::time::SystemTime;

use galahad_core::{AuthenticatedSession, User};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    pub(crate) code: &'static str,
    pub(crate) message: &'static str,
}

#[derive(Serialize)]
pub(crate) struct UserResponse {
    id: String,
    email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
        }
    }
}

#[derive(Serialize)]
struct SessionResponse {
    id: String,
    expires_at_unix_seconds: u64,
}

impl From<galahad_core::Session> for SessionResponse {
    fn from(session: galahad_core::Session) -> Self {
        let expires_at_unix_seconds = session
            .expires_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();

        Self {
            id: session.id.to_string(),
            expires_at_unix_seconds,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct AuthenticatedSessionResponse {
    user: UserResponse,
    session: SessionResponse,
}

impl From<AuthenticatedSession> for AuthenticatedSessionResponse {
    fn from(authenticated_session: AuthenticatedSession) -> Self {
        Self {
            user: UserResponse::from(authenticated_session.user),
            session: SessionResponse::from(authenticated_session.session),
        }
    }
}
