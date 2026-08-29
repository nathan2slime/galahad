//! Actix integration for Galahad.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

use actix_web::cookie::{time::Duration as CookieDuration, Cookie, SameSite};
use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use actix_web::{web, FromRequest, HttpRequest, HttpResponse, ResponseError};
use galahad_core::{
    AuthError, AuthenticatedSession, EmailPasswordSignInService, EmailPasswordSignUpService,
    SessionLogoutService, SessionLookupService, SessionToken, SignInInput, SignUpInput, User,
};
use serde::{Deserialize, Serialize};

const DEFAULT_SESSION_COOKIE_NAME: &str = "galahad_session";

/// Actix authentication configuration and route dependencies.
#[derive(Clone)]
pub struct GalahadActix {
    sign_up_service: Arc<EmailPasswordSignUpService>,
    sign_in_service: Arc<EmailPasswordSignInService>,
    logout_service: Arc<SessionLogoutService>,
    lookup_service: Arc<SessionLookupService>,
    session_cookie_name: String,
}

impl GalahadActix {
    /// Creates Actix route configuration from core authentication services.
    pub fn new(
        sign_up_service: Arc<EmailPasswordSignUpService>,
        sign_in_service: Arc<EmailPasswordSignInService>,
        logout_service: Arc<SessionLogoutService>,
        lookup_service: Arc<SessionLookupService>,
    ) -> Self {
        Self {
            sign_up_service,
            sign_in_service,
            logout_service,
            lookup_service,
            session_cookie_name: String::from(DEFAULT_SESSION_COOKIE_NAME),
        }
    }

    /// Sets the session cookie name.
    pub fn with_session_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.session_cookie_name = name.into();
        self
    }

    /// Configures the MVP authentication routes under `/auth`.
    pub fn routes(&self, config: &mut web::ServiceConfig) {
        config.app_data(web::Data::new(self.clone())).service(
            web::scope("/auth")
                .route("/sign-up", web::post().to(sign_up))
                .route("/sign-in", web::post().to(sign_in))
                .route("/sign-out", web::post().to(sign_out))
                .route("/session", web::get().to(current_session)),
        );
    }

    fn session_cookie(&self, token: &SessionToken) -> Cookie<'static> {
        session_cookie_for(&self.session_cookie_name, token)
    }

    fn expired_session_cookie(&self) -> Cookie<'static> {
        Cookie::build(self.session_cookie_name.clone(), String::new())
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(CookieDuration::seconds(0))
            .finish()
    }
}

fn session_cookie_for(name: &str, token: &SessionToken) -> Cookie<'static> {
    Cookie::build(name.to_owned(), token.as_str().to_owned())
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .finish()
}

/// Authenticated user extractor backed by the configured Galahad session cookie.
pub struct AuthenticatedUser(pub User);

impl FromRequest for AuthenticatedUser {
    type Error = ActixAuthError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth = request.app_data::<web::Data<GalahadActix>>().cloned();
        let token = auth.as_ref().and_then(|auth| {
            request
                .cookie(&auth.session_cookie_name)
                .map(|cookie| SessionToken::from(cookie.value()))
        });

        Box::pin(async move {
            let auth = auth.ok_or(ActixAuthError(AuthError::SessionNotFound))?;
            let token = token.ok_or(ActixAuthError(AuthError::SessionNotFound))?;
            let authenticated_session = auth
                .lookup_service
                .lookup(&token, SystemTime::now())
                .await?
                .ok_or(ActixAuthError(AuthError::SessionNotFound))?;

            Ok(Self(authenticated_session.user))
        })
    }
}

/// Optional authenticated user extractor backed by the configured Galahad session cookie.
pub struct OptionalUser(pub Option<User>);

impl FromRequest for OptionalUser {
    type Error = ActixAuthError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth = request.app_data::<web::Data<GalahadActix>>().cloned();
        let token = auth.as_ref().and_then(|auth| {
            request
                .cookie(&auth.session_cookie_name)
                .map(|cookie| SessionToken::from(cookie.value()))
        });

        Box::pin(async move {
            let Some(auth) = auth else {
                return Ok(Self(None));
            };
            let Some(token) = token else {
                return Ok(Self(None));
            };

            let authenticated_session = auth
                .lookup_service
                .lookup(&token, SystemTime::now())
                .await?;

            Ok(Self(authenticated_session.map(|session| session.user)))
        })
    }
}

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
            AuthError::InvalidEmail | AuthError::InvalidPassword => StatusCode::BAD_REQUEST,
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

#[derive(Deserialize)]
struct AuthRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: &'static str,
}

#[derive(Serialize)]
struct UserResponse {
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
struct AuthenticatedSessionResponse {
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

async fn sign_up(
    auth: web::Data<GalahadActix>,
    request: web::Json<AuthRequest>,
) -> Result<web::Json<UserResponse>, ActixAuthError> {
    let user = auth
        .sign_up_service
        .sign_up(&SignUpInput::new(&request.email, &request.password))
        .await?;

    Ok(web::Json(UserResponse::from(user)))
}

async fn sign_in(
    auth: web::Data<GalahadActix>,
    request: web::Json<AuthRequest>,
) -> Result<HttpResponse, ActixAuthError> {
    let signed_in_session = auth
        .sign_in_service
        .sign_in(&SignInInput::new(&request.email, &request.password))
        .await?;
    let cookie = auth.session_cookie(&signed_in_session.token);

    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .json(AuthenticatedSessionResponse::from(
            signed_in_session.authenticated_session,
        )))
}

async fn sign_out(
    auth: web::Data<GalahadActix>,
    request: HttpRequest,
) -> Result<HttpResponse, ActixAuthError> {
    if let Some(cookie) = request.cookie(&auth.session_cookie_name) {
        auth.logout_service
            .logout(&SessionToken::from(cookie.value()), SystemTime::now())
            .await?;
    }

    Ok(HttpResponse::NoContent()
        .cookie(auth.expired_session_cookie())
        .finish())
}

async fn current_session(
    auth: web::Data<GalahadActix>,
    request: HttpRequest,
) -> Result<HttpResponse, ActixAuthError> {
    let Some(cookie) = request.cookie(&auth.session_cookie_name) else {
        return Err(ActixAuthError(AuthError::SessionNotFound));
    };

    let authenticated_session = auth
        .lookup_service
        .lookup(&SessionToken::from(cookie.value()), SystemTime::now())
        .await?
        .ok_or(ActixAuthError(AuthError::SessionNotFound))?;

    Ok(HttpResponse::Ok().json(AuthenticatedSessionResponse::from(authenticated_session)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_session_cookie_is_http_only() {
        let cookie = super::session_cookie_for(
            "galahad_session",
            &galahad_core::SessionToken::from("token"),
        );

        assert_eq!(cookie.name(), "galahad_session");
        assert_eq!(cookie.value(), "token");
        assert_eq!(cookie.http_only(), Some(true));
    }
}
