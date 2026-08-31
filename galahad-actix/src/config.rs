use std::sync::Arc;

use actix_web::cookie::Cookie;
use actix_web::web;
use galahad_core::{
    EmailPasswordSignInService, EmailPasswordSignUpService, SessionLogoutService,
    SessionLookupService, SessionToken,
};

use crate::cookie::{expired_session_cookie_for, session_cookie_for};
use crate::handler::{current_session, sign_in, sign_out, sign_up};
use crate::jwt::JwtConfig;
use crate::sign_up::AfterSignUp;

pub(crate) const DEFAULT_SESSION_COOKIE_NAME: &str = "galahad_session";

/// Actix authentication configuration and route dependencies.
#[derive(Clone)]
pub struct GalahadActix {
    pub(crate) sign_up_service: Arc<EmailPasswordSignUpService>,
    pub(crate) sign_in_service: Arc<EmailPasswordSignInService>,
    pub(crate) logout_service: Arc<SessionLogoutService>,
    pub(crate) lookup_service: Arc<SessionLookupService>,
    pub(crate) session_cookie_name: String,
    pub(crate) after_sign_up: Option<Arc<dyn AfterSignUp>>,
    pub(crate) jwt: Option<JwtConfig>,
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
            after_sign_up: None,
            jwt: None,
        }
    }

    /// Sets the session cookie name.
    pub fn with_session_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.session_cookie_name = name.into();
        self
    }

    /// Sets a hook that runs after a successful sign-up.
    pub fn with_after_sign_up(mut self, hook: Arc<dyn AfterSignUp>) -> Self {
        self.after_sign_up = Some(hook);
        self
    }

    /// Enables Bearer JWT issuance and extraction.
    pub fn with_jwt(mut self, jwt: JwtConfig) -> Self {
        self.jwt = Some(jwt);
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

    pub(crate) fn session_cookie(&self, token: &SessionToken) -> Cookie<'static> {
        session_cookie_for(&self.session_cookie_name, token)
    }

    pub(crate) fn expired_session_cookie(&self) -> Cookie<'static> {
        expired_session_cookie_for(&self.session_cookie_name)
    }
}
