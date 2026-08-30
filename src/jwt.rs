use std::time::Duration;

/// JWT configuration shared by high-level web integrations.
#[derive(Clone, Debug)]
pub struct GalahadJwt {
    pub(crate) secret: Vec<u8>,
    pub(crate) ttl: Duration,
}

impl GalahadJwt {
    /// Creates JWT configuration from an HMAC secret.
    pub fn secret(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
            ttl: Duration::from_secs(60 * 60),
        }
    }

    /// Sets the JWT time-to-live.
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }
}
