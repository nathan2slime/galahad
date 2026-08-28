use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

use crate::{AuthError, Session, SessionId, UserId};

/// A boxed, sendable future returned by a service operation.
pub type BoxServiceFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The result type shared by service operations.
pub type ServiceResult<T> = Result<T, AuthError>;

/// Password hashing and verification operations.
pub trait PasswordService: Send + Sync {
    /// Hashes a plaintext password.
    fn hash_password<'a>(
        &'a self,
        password: &'a str,
    ) -> BoxServiceFuture<'a, ServiceResult<String>>;

    /// Verifies a plaintext password against a password hash.
    fn verify_password<'a>(
        &'a self,
        password: &'a str,
        password_hash: &'a str,
    ) -> BoxServiceFuture<'a, ServiceResult<bool>>;
}

/// Session lifecycle operations.
pub trait SessionService: Send + Sync {
    /// Creates a session for a user using an already-hashed token.
    fn create_session<'a>(
        &'a self,
        user_id: &'a UserId,
        token_hash: &'a str,
        expires_at: SystemTime,
    ) -> BoxServiceFuture<'a, ServiceResult<Session>>;

    /// Finds a session by its token hash.
    fn find_session_by_token_hash<'a>(
        &'a self,
        token_hash: &'a str,
    ) -> BoxServiceFuture<'a, ServiceResult<Option<Session>>>;

    /// Revokes a session at the given time.
    fn revoke_session<'a>(
        &'a self,
        session_id: &'a SessionId,
        revoked_at: SystemTime,
    ) -> BoxServiceFuture<'a, ServiceResult<()>>;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Mutex;
    use std::task::{Context, Poll, Waker};
    use std::time::{Duration, SystemTime};

    use super::{BoxServiceFuture, PasswordService, ServiceResult, SessionService};
    use crate::{Session, SessionId, UserId};

    struct FakePasswordService;

    impl PasswordService for FakePasswordService {
        fn hash_password<'a>(
            &'a self,
            password: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<String>> {
            Box::pin(std::future::ready(Ok(format!("test-hash:{password}"))))
        }

        fn verify_password<'a>(
            &'a self,
            password: &'a str,
            password_hash: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<bool>> {
            Box::pin(std::future::ready(Ok(
                password_hash == format!("test-hash:{password}")
            )))
        }
    }

    #[derive(Default)]
    struct FakeSessionService {
        sessions: Mutex<HashMap<SessionId, Session>>,
    }

    impl SessionService for FakeSessionService {
        fn create_session<'a>(
            &'a self,
            user_id: &'a UserId,
            token_hash: &'a str,
            expires_at: SystemTime,
        ) -> BoxServiceFuture<'a, ServiceResult<Session>> {
            let session_id = SessionId::from(format!(
                "session-{}",
                self.sessions.lock().unwrap().len() + 1
            ));
            let session = Session::new(session_id.clone(), user_id.clone(), token_hash, expires_at);
            self.sessions
                .lock()
                .unwrap()
                .insert(session_id, session.clone());

            Box::pin(std::future::ready(Ok(session)))
        }

        fn find_session_by_token_hash<'a>(
            &'a self,
            token_hash: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<Option<Session>>> {
            let session = self
                .sessions
                .lock()
                .unwrap()
                .values()
                .find(|session| session.token_hash == token_hash)
                .cloned();

            Box::pin(std::future::ready(Ok(session)))
        }

        fn revoke_session<'a>(
            &'a self,
            session_id: &'a SessionId,
            revoked_at: SystemTime,
        ) -> BoxServiceFuture<'a, ServiceResult<()>> {
            let result = self
                .sessions
                .lock()
                .unwrap()
                .get_mut(session_id)
                .map(|session| session.revoke(revoked_at))
                .ok_or(crate::AuthError::SessionNotFound);

            Box::pin(std::future::ready(result))
        }
    }

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let mut future = std::pin::pin!(future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[test]
    fn password_service_is_object_safe_and_usable_without_runtime() {
        let service: Box<dyn PasswordService> = Box::new(FakePasswordService);

        let password_hash = block_on(service.hash_password("correct horse")).unwrap();

        assert_eq!(password_hash, "test-hash:correct horse");
        assert!(block_on(service.verify_password("correct horse", &password_hash)).unwrap());
        assert!(!block_on(service.verify_password("wrong horse", &password_hash)).unwrap());
    }

    #[test]
    fn session_service_is_object_safe_and_usable_without_runtime() {
        let service: Box<dyn SessionService> = Box::new(FakeSessionService::default());
        let user_id = UserId::from("user-1");
        let expires_at = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let revoked_at = SystemTime::UNIX_EPOCH + Duration::from_secs(50);

        let session = block_on(service.create_session(&user_id, "token-hash", expires_at)).unwrap();

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.token_hash, "token-hash");
        assert_eq!(session.expires_at, expires_at);
        assert!(!session.is_revoked());
        assert_eq!(
            block_on(service.find_session_by_token_hash("token-hash")).unwrap(),
            Some(session.clone())
        );

        block_on(service.revoke_session(&session.id, revoked_at)).unwrap();

        let revoked_session = block_on(service.find_session_by_token_hash("token-hash"))
            .unwrap()
            .unwrap();
        assert_eq!(revoked_session.revoked_at, Some(revoked_at));
    }
}
