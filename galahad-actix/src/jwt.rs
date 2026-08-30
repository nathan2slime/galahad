use std::time::{Duration, SystemTime};

use galahad_core::{AuthError, SessionToken};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT configuration for Actix authentication routes and extractors.
#[derive(Clone)]
pub struct JwtConfig {
    secret: Vec<u8>,
    ttl: Duration,
}

impl JwtConfig {
    /// Creates JWT configuration from an HMAC secret.
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
            ttl: Duration::from_secs(60 * 60),
        }
    }

    /// Sets the JWT time-to-live.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
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
        };

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )
        .map_err(|_| AuthError::PersistenceFailure)
    }

    pub(crate) fn verify(&self, token: &str) -> Option<SessionToken> {
        let claims = jsonwebtoken::decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &Validation::default(),
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

    use super::JwtConfig;

    #[test]
    fn jwt_round_trips_session_token() {
        let config = JwtConfig::new("secret").with_ttl(Duration::from_secs(60));
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
