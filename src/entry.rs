use crate::session::GalahadSession;
use crate::web::GalahadActixBuilder;

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
}
