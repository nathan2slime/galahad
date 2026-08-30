use std::time::{Duration, SystemTime};

use galahad_core::{AuthError, SessionToken};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Supported HMAC JWT algorithms for secret-based JWT signing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JwtAlgorithm {
    Hs256,
    Hs384,
    Hs512,
}

impl From<JwtAlgorithm> for Algorithm {
    fn from(algorithm: JwtAlgorithm) -> Self {
        match algorithm {
            JwtAlgorithm::Hs256 => Self::HS256,
            JwtAlgorithm::Hs384 => Self::HS384,
            JwtAlgorithm::Hs512 => Self::HS512,
        }
    }
}

/// JWT configuration for Actix authentication routes and extractors.
#[derive(Clone)]
pub struct JwtConfig {
    secret: Vec<u8>,
    ttl: Duration,
    algorithm: JwtAlgorithm,
    issuer: Option<String>,
    audience: Vec<String>,
    leeway: u64,
}

impl JwtConfig {
    /// Creates JWT configuration from an HMAC secret.
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
            ttl: Duration::from_secs(60 * 60),
            algorithm: JwtAlgorithm::Hs256,
            issuer: None,
            audience: Vec::new(),
            leeway: 60,
        }
    }

    /// Sets the JWT time-to-live.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Sets the HMAC JWT signing algorithm.
    pub fn with_algorithm(mut self, algorithm: JwtAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Sets the JWT issuer claim and validation requirement.
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Adds a JWT audience claim and validation requirement.
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience.push(audience.into());
        self
    }

    /// Sets accepted clock skew in seconds when validating JWT claims.
    pub const fn with_leeway(mut self, leeway: u64) -> Self {
        self.leeway = leeway;
        self
    }

    pub(crate) fn issue(&self, token: &SessionToken, now: SystemTime) -> Result<String, AuthError> {
        let issued_at = unix_timestamp(now)?;
        let expires_at =
            unix_timestamp(now.checked_add(self.ttl).ok_or(AuthError::SessionExpired)?)?;
        let claims = JwtClaims {
            session_token: token.as_str().to_owned(),
            iat: issued_at,
            exp: expires_at,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };

        jsonwebtoken::encode(
            &Header::new(self.algorithm.into()),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )
        .map_err(|_| AuthError::PersistenceFailure)
    }

    pub(crate) fn verify(&self, token: &str) -> Option<SessionToken> {
        let mut validation = Validation::new(self.algorithm.into());
        validation.leeway = self.leeway;
        if let Some(issuer) = &self.issuer {
            validation.set_issuer(&[issuer]);
        }
        if !self.audience.is_empty() {
            validation.set_audience(&self.audience);
        }

        let claims = jsonwebtoken::decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &validation,
        )
        .ok()?
        .claims;

        Some(SessionToken::from(claims.session_token))
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct JwtClaims {
    session_token: String,
    iat: usize,
    exp: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    iss: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    aud: Vec<String>,
}

fn unix_timestamp(time: SystemTime) -> Result<usize, AuthError> {
    let seconds = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| AuthError::PersistenceFailure)?
        .as_secs();

    usize::try_from(seconds).map_err(|_| AuthError::PersistenceFailure)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use galahad_core::SessionToken;

    use super::{JwtAlgorithm, JwtConfig};

    #[test]
    fn jwt_round_trips_session_token() {
        let config = JwtConfig::new("secret").with_ttl(Duration::from_secs(60));
        let token = SessionToken::from("session-token");

        let jwt = config.issue(&token, SystemTime::now()).unwrap();

        assert_eq!(config.verify(&jwt), Some(token));
    }

    #[test]
    fn jwt_supports_custom_algorithm_and_claim_validation() {
        let config = JwtConfig::new("secret")
            .with_algorithm(JwtAlgorithm::Hs512)
            .with_issuer("galahad")
            .with_audience("web")
            .with_leeway(5);
        let token = SessionToken::from("session-token");

        let jwt = config.issue(&token, SystemTime::now()).unwrap();

        assert_eq!(config.verify(&jwt), Some(token));
    }

    #[test]
    fn jwt_rejects_invalid_token() {
        let config = JwtConfig::new("secret");

        assert_eq!(config.verify("invalid"), None);
    }
}
