use crate::actix_setup::GalahadActixBuilder;
use crate::jwt::GalahadJwt;
use crate::session::GalahadSession;
use crate::sign_up::GalahadSignUp;

/// Entry point for Galahad's high-level setup API.
pub struct Galahad;

impl Galahad {
    /// Starts configuring the Actix Web integration.
    pub const fn actix() -> GalahadActixBuilder {
        GalahadActixBuilder
    }

    /// Starts configuring session behavior for high-level integrations.
    pub fn session() -> GalahadSession {
        GalahadSession::default()
    }

    /// Starts configuring sign-up behavior for high-level integrations.
    pub fn sign_up() -> GalahadSignUp {
        GalahadSignUp::default()
    }

    /// Starts configuring JWT support for high-level web integrations.
    pub fn jwt(secret: impl Into<Vec<u8>>) -> GalahadJwt {
        GalahadJwt::secret(secret)
    }
}
