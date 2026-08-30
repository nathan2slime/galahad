use std::time::Duration;

/// Supported HMAC JWT algorithms for secret-based JWT signing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GalahadJwtAlgorithm {
    Hs256,
    Hs384,
    Hs512,
}

/// JWT configuration shared by high-level web integrations.
#[derive(Clone, Debug)]
pub struct GalahadJwt {
    pub(crate) secret: Vec<u8>,
    pub(crate) ttl: Duration,
    pub(crate) algorithm: GalahadJwtAlgorithm,
    pub(crate) issuer: Option<String>,
    pub(crate) audience: Vec<String>,
    pub(crate) leeway: u64,
}

impl GalahadJwt {
    /// Creates JWT configuration from an HMAC secret.
    pub fn secret(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
            ttl: Duration::from_secs(60 * 60),
            algorithm: GalahadJwtAlgorithm::Hs256,
            issuer: None,
            audience: Vec::new(),
            leeway: 60,
        }
    }

    /// Sets the JWT time-to-live.
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Sets the HMAC JWT signing algorithm.
    pub fn algorithm(mut self, algorithm: GalahadJwtAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Sets the JWT issuer claim and validation requirement.
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Adds a JWT audience claim and validation requirement.
    pub fn audience(mut self, audience: impl Into<String>) -> Self {
        self.audience.push(audience.into());
        self
    }

    /// Sets accepted clock skew in seconds when validating JWT claims.
    pub const fn leeway(mut self, leeway: u64) -> Self {
        self.leeway = leeway;
        self
    }
}
