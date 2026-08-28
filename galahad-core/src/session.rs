use std::fmt;
use std::time::SystemTime;

use crate::UserId;

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

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::{Session, SessionId};
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
}
