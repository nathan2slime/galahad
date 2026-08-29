use std::sync::Arc;
use std::time::SystemTime;

use crate::{
    AuthError, AuthenticatedSession, BoxServiceFuture, ServiceResult, SessionRepository,
    SessionToken, SessionTokenHasher, UserRepository,
};

/// Looks up authenticated sessions from raw session tokens.
pub struct SessionLookupService {
    user_repository: Arc<dyn UserRepository>,
    session_repository: Arc<dyn SessionRepository>,
    token_hasher: Arc<dyn SessionTokenHasher>,
}

impl SessionLookupService {
    /// Creates a session lookup service from repository and token hashing dependencies.
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        session_repository: Arc<dyn SessionRepository>,
        token_hasher: Arc<dyn SessionTokenHasher>,
    ) -> Self {
        Self {
            user_repository,
            session_repository,
            token_hasher,
        }
    }

    /// Finds the active authenticated session associated with `token` at `now`.
    pub fn lookup<'a>(
        &'a self,
        token: &'a SessionToken,
        now: SystemTime,
    ) -> BoxServiceFuture<'a, ServiceResult<Option<AuthenticatedSession>>> {
        Box::pin(async move {
            let token_hash = self.token_hasher.hash_token(token);
            let Some(session) = self
                .session_repository
                .find_by_token_hash(token_hash.as_str())
                .await?
            else {
                return Ok(None);
            };

            if session.is_expired_at(now) {
                return Err(AuthError::SessionExpired);
            }

            if session.is_revoked() {
                return Err(AuthError::SessionRevoked);
            }

            let user = self
                .user_repository
                .find_by_id(&session.user_id)
                .await?
                .ok_or(AuthError::UserNotFound)?;

            Ok(Some(AuthenticatedSession::new(user, session)))
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

    use super::SessionLookupService;
    use crate::{
        BoxRepositoryFuture, CredentialRepository, PasswordCredential, RepositoryResult, Session,
        SessionId, SessionRepository, SessionToken, SessionTokenHash, SessionTokenHasher, User,
        UserId, UserRepository,
    };

    #[derive(Default)]
    struct InMemoryRepositories {
        users: Mutex<HashMap<UserId, User>>,
        credentials: Mutex<HashMap<UserId, PasswordCredential>>,
        sessions: Mutex<HashMap<SessionId, Session>>,
    }

    impl UserRepository for InMemoryRepositories {
        fn find_by_id<'a>(
            &'a self,
            id: &'a UserId,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>> {
            let user = self.users.lock().unwrap().get(id).cloned();
            Box::pin(std::future::ready(Ok(user)))
        }

        fn find_by_email<'a>(
            &'a self,
            email: &'a str,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>> {
            let user = self
                .users
                .lock()
                .unwrap()
                .values()
                .find(|user| user.email == email)
                .cloned();
            Box::pin(std::future::ready(Ok(user)))
        }

        fn save<'a>(&'a self, user: &'a User) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
            self.users
                .lock()
                .unwrap()
                .insert(user.id.clone(), user.clone());
            Box::pin(std::future::ready(Ok(())))
        }
    }

    impl CredentialRepository for InMemoryRepositories {
        fn find_by_user_id<'a>(
            &'a self,
            user_id: &'a UserId,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<PasswordCredential>>> {
            let credential = self.credentials.lock().unwrap().get(user_id).cloned();
            Box::pin(std::future::ready(Ok(credential)))
        }

        fn save<'a>(
            &'a self,
            credential: &'a PasswordCredential,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
            self.credentials
                .lock()
                .unwrap()
                .insert(credential.user_id.clone(), credential.clone());
            Box::pin(std::future::ready(Ok(())))
        }
    }

    impl SessionRepository for InMemoryRepositories {
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

    fn service(repositories: &Arc<InMemoryRepositories>) -> SessionLookupService {
        SessionLookupService::new(
            repositories.clone(),
            repositories.clone(),
            Arc::new(FakeSessionTokenHasher),
        )
    }

    fn insert_active_session(repositories: &Arc<InMemoryRepositories>) -> User {
        let user = User::new(UserId::from("user-1"), "user@example.com");
        let session = Session::new(
            SessionId::from("session-1"),
            user.id.clone(),
            "hash:token-value",
            SystemTime::UNIX_EPOCH + Duration::from_secs(100),
        );

        repositories
            .users
            .lock()
            .unwrap()
            .insert(user.id.clone(), user.clone());
        repositories
            .sessions
            .lock()
            .unwrap()
            .insert(session.id.clone(), session);

        user
    }

    #[test]
    fn lookup_returns_authenticated_session_for_active_session() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let user = insert_active_session(&repositories);
        let service = service(&repositories);

        let authenticated_session = block_on(service.lookup(
            &SessionToken::from("token-value"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(50),
        ))
        .unwrap()
        .unwrap();

        assert_eq!(authenticated_session.user, user);
        assert_eq!(
            authenticated_session.session.id,
            SessionId::from("session-1")
        );
        assert_eq!(authenticated_session.session.token_hash, "hash:token-value");
    }

    #[test]
    fn lookup_returns_none_for_missing_session() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let service = service(&repositories);

        let result = block_on(service.lookup(
            &SessionToken::from("missing-token"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(50),
        ));

        assert_eq!(result, Ok(None));
    }

    #[test]
    fn lookup_rejects_expired_session() {
        let repositories = Arc::new(InMemoryRepositories::default());
        insert_active_session(&repositories);
        let service = service(&repositories);

        let result = block_on(service.lookup(
            &SessionToken::from("token-value"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(100),
        ));

        assert_eq!(result, Err(crate::AuthError::SessionExpired));
    }

    #[test]
    fn lookup_rejects_revoked_session() {
        let repositories = Arc::new(InMemoryRepositories::default());
        insert_active_session(&repositories);
        repositories
            .sessions
            .lock()
            .unwrap()
            .get_mut(&SessionId::from("session-1"))
            .unwrap()
            .revoke(SystemTime::UNIX_EPOCH + Duration::from_secs(40));
        let service = service(&repositories);

        let result = block_on(service.lookup(
            &SessionToken::from("token-value"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(50),
        ));

        assert_eq!(result, Err(crate::AuthError::SessionRevoked));
    }

    #[test]
    fn lookup_rejects_session_without_user() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let session = Session::new(
            SessionId::from("session-1"),
            UserId::from("missing-user"),
            "hash:token-value",
            SystemTime::UNIX_EPOCH + Duration::from_secs(100),
        );
        repositories
            .sessions
            .lock()
            .unwrap()
            .insert(session.id.clone(), session);
        let service = service(&repositories);

        let result = block_on(service.lookup(
            &SessionToken::from("token-value"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(50),
        ));

        assert_eq!(result, Err(crate::AuthError::UserNotFound));
    }
}
