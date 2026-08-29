use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

use crate::{AuthError, PasswordCredential, Session, SessionId, User, UserId};

/// A boxed, sendable future returned by a repository operation.
pub type BoxRepositoryFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The result type shared by repository operations.
pub type RepositoryResult<T> = Result<T, AuthError>;

/// Persistence operations for users.
pub trait UserRepository: Send + Sync {
    /// Finds a user by identifier.
    fn find_by_id<'a>(
        &'a self,
        id: &'a UserId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>>;

    /// Finds a user by email address.
    fn find_by_email<'a>(
        &'a self,
        email: &'a str,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>>;

    /// Persists a user.
    fn save<'a>(&'a self, user: &'a User) -> BoxRepositoryFuture<'a, RepositoryResult<()>>;
}

/// Persistence operations for password credentials.
pub trait CredentialRepository: Send + Sync {
    /// Finds the password credential belonging to a user.
    fn find_by_user_id<'a>(
        &'a self,
        user_id: &'a UserId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<PasswordCredential>>>;

    /// Persists a password credential.
    fn save<'a>(
        &'a self,
        credential: &'a PasswordCredential,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<()>>;
}

/// Persistence operations for sessions.
pub trait SessionRepository: Send + Sync {
    /// Finds a session by identifier.
    fn find_by_id<'a>(
        &'a self,
        id: &'a SessionId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<Session>>>;

    /// Finds a session by token hash.
    fn find_by_token_hash<'a>(
        &'a self,
        token_hash: &'a str,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<Session>>>;

    /// Persists a session.
    fn save<'a>(&'a self, session: &'a Session) -> BoxRepositoryFuture<'a, RepositoryResult<()>>;

    /// Revokes a session by identifier.
    fn revoke<'a>(
        &'a self,
        id: &'a SessionId,
        revoked_at: SystemTime,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<()>>;

    /// Deletes a session by identifier.
    fn delete<'a>(&'a self, id: &'a SessionId) -> BoxRepositoryFuture<'a, RepositoryResult<()>>;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Mutex;
    use std::task::{Context, Poll, Waker};
    use std::time::{Duration, SystemTime};

    use super::{
        BoxRepositoryFuture, CredentialRepository, RepositoryResult, SessionRepository,
        UserRepository,
    };
    use crate::{PasswordCredential, Session, SessionId, User, UserId};

    #[derive(Default)]
    struct FakeRepositories {
        users: Mutex<HashMap<UserId, User>>,
        credentials: Mutex<HashMap<UserId, PasswordCredential>>,
        sessions: Mutex<HashMap<SessionId, Session>>,
    }

    impl UserRepository for FakeRepositories {
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

    impl CredentialRepository for FakeRepositories {
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

    impl SessionRepository for FakeRepositories {
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
    fn user_repository_is_object_safe_and_usable_without_runtime() {
        let repository: Box<dyn UserRepository> = Box::new(FakeRepositories::default());
        let user = User::new(UserId::from("user-1"), "user@example.com");

        assert_eq!(block_on(repository.find_by_id(&user.id)).unwrap(), None);
        block_on(repository.save(&user)).unwrap();
        assert_eq!(
            block_on(repository.find_by_id(&user.id)).unwrap(),
            Some(user.clone())
        );
        assert_eq!(
            block_on(repository.find_by_email("user@example.com")).unwrap(),
            Some(user)
        );
    }

    #[test]
    fn credential_repository_is_usable_without_runtime() {
        let repository: Box<dyn CredentialRepository> = Box::new(FakeRepositories::default());
        let credential = PasswordCredential::new(UserId::from("user-1"), "password-hash");

        assert_eq!(
            block_on(repository.find_by_user_id(&credential.user_id)).unwrap(),
            None
        );
        block_on(repository.save(&credential)).unwrap();
        assert_eq!(
            block_on(repository.find_by_user_id(&credential.user_id)).unwrap(),
            Some(credential)
        );
    }

    #[test]
    fn session_repository_is_usable_without_runtime() {
        let repository: Box<dyn SessionRepository> = Box::new(FakeRepositories::default());
        let session = Session::new(
            SessionId::from("session-1"),
            UserId::from("user-1"),
            "token-hash",
            SystemTime::UNIX_EPOCH + Duration::from_secs(60),
        );

        assert_eq!(block_on(repository.find_by_id(&session.id)).unwrap(), None);
        block_on(repository.save(&session)).unwrap();
        assert_eq!(
            block_on(repository.find_by_id(&session.id)).unwrap(),
            Some(session.clone())
        );
        assert_eq!(
            block_on(repository.find_by_token_hash("token-hash")).unwrap(),
            Some(session.clone())
        );
        let revoked_at = SystemTime::UNIX_EPOCH + Duration::from_secs(30);
        block_on(repository.revoke(&SessionId::from("session-1"), revoked_at)).unwrap();
        let revoked_session = block_on(repository.find_by_id(&SessionId::from("session-1")))
            .unwrap()
            .unwrap();
        assert_eq!(revoked_session.revoked_at, Some(revoked_at));
        assert_eq!(revoked_session.user_id, session.user_id);
        assert_eq!(revoked_session.token_hash, session.token_hash);
        assert_eq!(revoked_session.expires_at, session.expires_at);
        assert_eq!(
            block_on(repository.revoke(&SessionId::from("missing-session"), revoked_at)),
            Err(crate::AuthError::SessionNotFound)
        );
        block_on(repository.delete(&SessionId::from("session-1"))).unwrap();
        assert_eq!(
            block_on(repository.find_by_id(&SessionId::from("session-1"))).unwrap(),
            None
        );
    }
}
