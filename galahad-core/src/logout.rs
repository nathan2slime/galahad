use std::sync::Arc;
use std::time::SystemTime;

use crate::{BoxServiceFuture, ServiceResult, SessionRepository, SessionToken, SessionTokenHasher};

/// Signs sessions out using raw session tokens.
pub struct SessionLogoutService {
    session_repository: Arc<dyn SessionRepository>,
    token_hasher: Arc<dyn SessionTokenHasher>,
}

impl SessionLogoutService {
    /// Creates a logout service from repository and token hashing dependencies.
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        token_hasher: Arc<dyn SessionTokenHasher>,
    ) -> Self {
        Self {
            session_repository,
            token_hasher,
        }
    }

    /// Revokes the session associated with `token`, if one exists.
    pub fn logout<'a>(
        &'a self,
        token: &'a SessionToken,
        revoked_at: SystemTime,
    ) -> BoxServiceFuture<'a, ServiceResult<()>> {
        Box::pin(async move {
            let token_hash = self.token_hasher.hash_token(token);
            let Some(session) = self
                .session_repository
                .find_by_token_hash(token_hash.as_str())
                .await?
            else {
                return Ok(());
            };

            self.session_repository
                .revoke(&session.id, revoked_at)
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, Waker};
    use std::time::{Duration, SystemTime};

    use super::SessionLogoutService;
    use crate::{
        BoxRepositoryFuture, RepositoryResult, Session, SessionId, SessionRepository, SessionToken,
        SessionTokenHash, SessionTokenHasher, UserId,
    };

    #[derive(Default)]
    struct InMemorySessionRepository {
        sessions: Mutex<HashMap<SessionId, Session>>,
    }

    impl SessionRepository for InMemorySessionRepository {
        fn find_by_id<'a>(
            &'a self,
            id: &'a SessionId,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<Session>>> {
            let session = self.sessions.lock().unwrap().get(id).cloned();
            Box::pin(std::future::ready(Ok(session)))
        }

        fn find_by_token_hash<'a>(
            &'a self,
            token_hash: &'a str,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<Session>>> {
            let session = self
                .sessions
                .lock()
                .unwrap()
                .values()
                .find(|session| session.token_hash == token_hash)
                .cloned();
            Box::pin(std::future::ready(Ok(session)))
        }

        fn save<'a>(
            &'a self,
            session: &'a Session,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
            self.sessions
                .lock()
                .unwrap()
                .insert(session.id.clone(), session.clone());
            Box::pin(std::future::ready(Ok(())))
        }

        fn revoke<'a>(
            &'a self,
            id: &'a SessionId,
            revoked_at: SystemTime,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
            let result = self
                .sessions
                .lock()
                .unwrap()
                .get_mut(id)
                .map(|session| session.revoke(revoked_at))
                .ok_or(crate::AuthError::SessionNotFound);
            Box::pin(std::future::ready(result))
        }

        fn delete<'a>(
            &'a self,
            id: &'a SessionId,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
            self.sessions.lock().unwrap().remove(id);
            Box::pin(std::future::ready(Ok(())))
        }
    }

    struct FakeSessionTokenHasher;

    impl SessionTokenHasher for FakeSessionTokenHasher {
        fn hash_token(&self, token: &SessionToken) -> SessionTokenHash {
            SessionTokenHash::from(format!("hash:{}", token.as_str()))
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
    fn logout_revokes_existing_session_by_token_hash() {
        let repository = Arc::new(InMemorySessionRepository::default());
        let session = Session::new(
            SessionId::from("session-1"),
            UserId::from("user-1"),
            "hash:token-value",
            SystemTime::UNIX_EPOCH + Duration::from_secs(100),
        );
        block_on(repository.save(&session)).unwrap();
        let service =
            SessionLogoutService::new(repository.clone(), Arc::new(FakeSessionTokenHasher));
        let revoked_at = SystemTime::UNIX_EPOCH + Duration::from_secs(50);

        block_on(service.logout(&SessionToken::from("token-value"), revoked_at)).unwrap();

        assert_eq!(
            repository
                .sessions
                .lock()
                .unwrap()
                .get(&SessionId::from("session-1"))
                .unwrap()
                .revoked_at,
            Some(revoked_at)
        );
    }

    #[test]
    fn logout_is_idempotent_for_missing_session() {
        let repository = Arc::new(InMemorySessionRepository::default());
        let service = SessionLogoutService::new(repository, Arc::new(FakeSessionTokenHasher));

        let result = block_on(service.logout(
            &SessionToken::from("missing-token"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(50),
        ));

        assert_eq!(result, Ok(()));
    }
}
