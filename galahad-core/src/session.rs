use std::fmt;
use std::time::{Duration, SystemTime};

use crate::{AuthError, ServiceResult, UserId};

const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 7);

/// The identifier of a session.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SessionId(String);

impl SessionId {
    /// Creates a session identifier from its string representation.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the string representation of this identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SessionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SessionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A user session with an expiry time and optional revocation time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: SystemTime,
    pub revoked_at: Option<SystemTime>,
}

impl Session {
    /// Creates an active session that expires at `expires_at`.
    pub fn new(
        id: SessionId,
        user_id: UserId,
        token_hash: impl Into<String>,
        expires_at: SystemTime,
    ) -> Self {
        Self {
            id,
            user_id,
            token_hash: token_hash.into(),
            expires_at,
            revoked_at: None,
        }
    }

    /// Returns whether the session has expired at `now`.
    pub fn is_expired_at(&self, now: SystemTime) -> bool {
        now >= self.expires_at
    }

    /// Returns whether the session has been revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    /// Returns whether the session is neither expired nor revoked at `now`.
    pub fn is_active_at(&self, now: SystemTime) -> bool {
        !self.is_expired_at(now) && !self.is_revoked()
    }

    /// Revokes the session at `revoked_at`.
    pub fn revoke(&mut self, revoked_at: SystemTime) {
        self.revoked_at = Some(revoked_at);
    }
}

/// Calculates session expiration times from a configured time-to-live.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionExpirationPolicy {
    ttl: Duration,
}

impl Default for SessionExpirationPolicy {
    fn default() -> Self {
        Self::new(DEFAULT_SESSION_TTL)
    }
}

impl SessionExpirationPolicy {
    /// Creates a session expiration policy with the provided time-to-live.
    pub const fn new(ttl: Duration) -> Self {
        Self { ttl }
    }

    /// Returns the configured session time-to-live.
    pub const fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Calculates when a session started at `now` should expire.
    pub fn expires_at(&self, now: SystemTime) -> ServiceResult<SystemTime> {
        now.checked_add(self.ttl).ok_or(AuthError::SessionExpired)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::{Session, SessionExpirationPolicy, SessionId};
    use crate::UserId;

    fn session(expires_at: SystemTime) -> Session {
        Session::new(
            SessionId::from("session-1"),
            UserId::from("user-1"),
            "token-hash",
            expires_at,
        )
    }

    #[test]
    fn session_id_preserves_and_displays_value() {
        let id = SessionId::from("session-1");

        assert_eq!(id.as_str(), "session-1");
        assert_eq!(id.to_string(), "session-1");
    }

    #[test]
    fn session_is_active_before_expiry() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let session = session(now + Duration::from_secs(60));

        assert!(!session.is_expired_at(now));
        assert!(!session.is_revoked());
        assert!(session.is_active_at(now));
    }

    #[test]
    fn session_expires_at_boundary() {
        let expiry = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let session = session(expiry);

        assert!(session.is_expired_at(expiry));
        assert!(!session.is_active_at(expiry));
    }

    #[test]
    fn revoked_session_is_not_active() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let mut session = session(now + Duration::from_secs(60));

        session.revoke(now);

        assert!(session.is_revoked());
        assert!(!session.is_active_at(now));
    }

    #[test]
    fn default_expiration_policy_uses_one_week_ttl() {
        let policy = SessionExpirationPolicy::default();

        assert_eq!(policy.ttl(), Duration::from_secs(60 * 60 * 24 * 7));
    }

    #[test]
    fn expiration_policy_calculates_expiry_from_now() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let policy = SessionExpirationPolicy::new(Duration::from_secs(30));

        assert_eq!(
            policy.expires_at(now),
            Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(130))
        );
    }

    #[test]
    fn expiration_policy_rejects_time_overflow() {
        let policy = SessionExpirationPolicy::new(Duration::MAX);

        assert_eq!(
            policy.expires_at(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
            Err(crate::AuthError::SessionExpired)
        );
    }
}
