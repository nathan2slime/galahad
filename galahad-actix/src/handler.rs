use std::time::SystemTime;

use actix_web::{web, HttpRequest, HttpResponse};
use galahad_core::{AuthError, SignInInput, SignUpInput};

use crate::extractor::session_token_from_request;
use crate::request::AuthRequest;
use crate::response::{AuthenticatedSessionResponse, UserResponse};
use crate::sign_up::SignUpContext;
use crate::{ActixAuthError, GalahadActix};

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        post,
        path = "/auth/sign-up",
        tag = "Galahad",
        request_body = AuthRequest,
        responses(
            (status = 200, description = "User signed up successfully", body = UserResponse),
            (status = 400, description = "Invalid request or user already exists", body = crate::response::ErrorResponse),
            (status = 500, description = "Authentication service failure", body = crate::response::ErrorResponse)
        )
    )
)]
pub(crate) async fn sign_up(
    auth: web::Data<GalahadActix>,
    request: web::Json<AuthRequest>,
) -> Result<web::Json<UserResponse>, ActixAuthError> {
    let (email, password, fields) = request.into_inner().into_parts();

    let user = auth
        .sign_up_service
        .sign_up(&SignUpInput::new(email, password))
        .await?;

    if let Some(after_sign_up) = &auth.after_sign_up {
        after_sign_up
            .after_sign_up(SignUpContext {
                user: user.clone(),
                fields,
            })
            .await?;
    }

    Ok(web::Json(UserResponse::from(user)))
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        post,
        path = "/auth/sign-in",
        tag = "Galahad",
        request_body = AuthRequest,
        responses(
            (status = 200, description = "User signed in successfully", body = AuthenticatedSessionResponse),
            (status = 400, description = "Invalid request", body = crate::response::ErrorResponse),
            (status = 401, description = "Invalid credentials", body = crate::response::ErrorResponse),
            (status = 500, description = "Authentication service failure", body = crate::response::ErrorResponse)
        )
    )
)]
pub(crate) async fn sign_in(
    auth: web::Data<GalahadActix>,
    request: web::Json<AuthRequest>,
) -> Result<HttpResponse, ActixAuthError> {
    let signed_in_session = auth
        .sign_in_service
        .sign_in(&SignInInput::new(&request.email, &request.password))
        .await?;
    let cookie = auth.session_cookie(&signed_in_session.token);
    let access_token = auth
        .jwt
        .as_ref()
        .map(|jwt| jwt.issue(&signed_in_session.token, SystemTime::now()))
        .transpose()?;
    let mut response = AuthenticatedSessionResponse::from(signed_in_session.authenticated_session);

    if let Some(access_token) = access_token {
        response = response.with_access_token(access_token);
    }

    Ok(HttpResponse::Ok().cookie(cookie).json(response))
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        post,
        path = "/auth/sign-out",
        tag = "Galahad",
        security(("GalahadSession" = []), ("GalahadBearer" = [])),
        responses(
            (status = 204, description = "User signed out successfully"),
            (status = 500, description = "Authentication service failure", body = crate::response::ErrorResponse)
        )
    )
)]
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

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        path = "/auth/session",
        tag = "Galahad",
        security(("GalahadSession" = []), ("GalahadBearer" = [])),
        responses(
            (status = 200, description = "Current authenticated session", body = AuthenticatedSessionResponse),
            (status = 401, description = "Missing, expired, revoked, or unknown session", body = crate::response::ErrorResponse),
            (status = 500, description = "Authentication service failure", body = crate::response::ErrorResponse)
        )
    )
)]
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
