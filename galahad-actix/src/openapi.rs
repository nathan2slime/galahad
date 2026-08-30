use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;

use crate::config::DEFAULT_SESSION_COOKIE_NAME;
#[allow(unused_imports)]
use crate::handler::{__path_current_session, __path_sign_in, __path_sign_out, __path_sign_up};
use crate::request::AuthRequest;
use crate::response::{AuthenticatedSessionResponse, ErrorResponse, SessionResponse, UserResponse};

const SESSION_SECURITY_SCHEME: &str = "GalahadSession";
const BEARER_SECURITY_SCHEME: &str = "GalahadBearer";

/// OpenAPI document for Galahad's Actix authentication routes.
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        crate::handler::sign_up,
        crate::handler::sign_in,
        crate::handler::sign_out,
        crate::handler::current_session,
    ),
    components(schemas(
        AuthRequest,
        UserResponse,
        SessionResponse,
        AuthenticatedSessionResponse,
        ErrorResponse,
    )),
    modifiers(&SessionCookieSecurity),
    tags((name = "Galahad", description = "Galahad authentication endpoints"))
)]
pub struct GalahadActixOpenApi;

impl GalahadActixOpenApi {
    /// Returns the OpenAPI document using the default Galahad session cookie name.
    pub fn openapi() -> utoipa::openapi::OpenApi {
        <Self as utoipa::OpenApi>::openapi()
    }

    /// Returns the OpenAPI document using a custom session cookie name when provided.
    pub fn openapi_for_session_cookie_name(name: Option<String>) -> utoipa::openapi::OpenApi {
        match name {
            Some(name) => Self::openapi_with_session_cookie_name(name),
            None => Self::openapi(),
        }
    }

    /// Returns the OpenAPI document using a custom session cookie name.
    pub fn openapi_with_session_cookie_name(name: impl Into<String>) -> utoipa::openapi::OpenApi {
        let mut openapi = Self::openapi();
        add_session_cookie_security(&mut openapi, name.into());
        openapi
    }
}

struct SessionCookieSecurity;

impl Modify for SessionCookieSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        add_session_cookie_security(openapi, DEFAULT_SESSION_COOKIE_NAME);
    }
}

fn add_session_cookie_security(
    openapi: &mut utoipa::openapi::OpenApi,
    cookie_name: impl Into<String>,
) {
    if let Some(components) = openapi.components.as_mut() {
        components.add_security_scheme(
            SESSION_SECURITY_SCHEME,
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(cookie_name))),
        );
        components.add_security_scheme(
            BEARER_SECURITY_SCHEME,
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::GalahadActixOpenApi;

    #[test]
    fn openapi_includes_auth_routes() {
        let json = GalahadActixOpenApi::openapi().to_json().unwrap();

        assert!(json.contains("/auth/sign-up"));
        assert!(json.contains("/auth/sign-in"));
        assert!(json.contains("/auth/sign-out"));
        assert!(json.contains("/auth/session"));
    }

    #[test]
    fn openapi_supports_custom_session_cookie_name() {
        let json = GalahadActixOpenApi::openapi_with_session_cookie_name("app_session")
            .to_json()
            .unwrap();

        assert!(json.contains("app_session"));
    }
}
