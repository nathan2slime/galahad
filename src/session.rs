use std::time::Duration;

use galahad_core::SessionExpirationPolicy;

/// Session configuration shared by high-level integrations.
#[derive(Clone, Debug)]
pub struct GalahadSession {
    pub(crate) cookie_name: Option<String>,
    pub(crate) ttl: Duration,
}

impl Default for GalahadSession {
    fn default() -> Self {
        Self {
            cookie_name: None,
            ttl: SessionExpirationPolicy::default().ttl(),
        }
    }
}

impl GalahadSession {
    /// Sets the session cookie name used by web integrations.
    pub fn cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = Some(name.into());
        self
    }

    /// Sets the session time-to-live used when new sessions are created.
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }
}
