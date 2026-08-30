use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

use actix_web::dev::Payload;
use actix_web::http::header;
use actix_web::{web, FromRequest, HttpRequest};
use galahad_core::{AuthError, SessionToken, User};

use crate::{ActixAuthError, GalahadActix};

/// Authenticated user extractor backed by the configured Galahad session cookie.
pub struct AuthenticatedUser(pub User);

impl FromRequest for AuthenticatedUser {
    type Error = ActixAuthError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth = request.app_data::<web::Data<GalahadActix>>().cloned();
        let token = auth
            .as_ref()
            .and_then(|auth| session_token_from_request(request, auth));

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
        let token = auth
            .as_ref()
            .and_then(|auth| session_token_from_request(request, auth));

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

pub(crate) fn session_token_from_request(
    request: &HttpRequest,
    auth: &GalahadActix,
) -> Option<SessionToken> {
    bearer_session_token_from_request(request, auth).or_else(|| {
        request
            .cookie(&auth.session_cookie_name)
            .map(|cookie| SessionToken::from(cookie.value()))
    })
}

fn bearer_session_token_from_request(
    request: &HttpRequest,
    auth: &GalahadActix,
) -> Option<SessionToken> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;

    auth.jwt.as_ref()?.verify(token)
}
