use std::time::SystemTime;

use actix_web::{web, HttpRequest, HttpResponse};
use galahad_core::{AuthError, SignInInput, SignUpInput};

use crate::extractor::session_token_from_request;
use crate::request::AuthRequest;
use crate::response::{AuthenticatedSessionResponse, UserResponse};
use crate::{ActixAuthError, GalahadActix};

pub(crate) async fn sign_up(
    auth: web::Data<GalahadActix>,
    request: web::Json<AuthRequest>,
) -> Result<web::Json<UserResponse>, ActixAuthError> {
    let user = auth
        .sign_up_service
        .sign_up(&SignUpInput::new(&request.email, &request.password))
        .await?;

    Ok(web::Json(UserResponse::from(user)))
}

pub(crate) async fn sign_in(
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

pub(crate) async fn sign_out(
    auth: web::Data<GalahadActix>,
    request: HttpRequest,
) -> Result<HttpResponse, ActixAuthError> {
    if let Some(token) = session_token_from_request(&request, &auth) {
        auth.logout_service
            .logout(&token, SystemTime::now())
            .await?;
    }

    Ok(HttpResponse::NoContent()
        .cookie(auth.expired_session_cookie())
        .finish())
}

pub(crate) async fn current_session(
    auth: web::Data<GalahadActix>,
    request: HttpRequest,
) -> Result<HttpResponse, ActixAuthError> {
    let token = session_token_from_request(&request, &auth)
        .ok_or(ActixAuthError(AuthError::SessionNotFound))?;

    let authenticated_session = auth
        .lookup_service
        .lookup(&token, SystemTime::now())
        .await?
        .ok_or(ActixAuthError(AuthError::SessionNotFound))?;

    Ok(HttpResponse::Ok().json(AuthenticatedSessionResponse::from(authenticated_session)))
}
