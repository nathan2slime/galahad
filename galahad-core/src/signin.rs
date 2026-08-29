use std::sync::Arc;
use std::time::SystemTime;

use crate::{
    AuthenticatedSession, BoxServiceFuture, CredentialRepository, PasswordService, ServiceResult,
    SessionExpirationPolicy, SessionService, SessionToken, SessionTokenGenerator,
    SessionTokenHasher, SignInInput, UserRepository,
};

/// Data needed to create a session after successful sign-in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignInSessionInput {
    pub now: SystemTime,
}

impl SignInSessionInput {
    /// Creates session input from the current time.
    pub fn new(now: SystemTime) -> Self {
        Self { now }
    }
}

/// The authenticated session and raw token returned after successful sign-in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedInSession {
    pub authenticated_session: AuthenticatedSession,
    pub token: SessionToken,
}

impl SignedInSession {
    /// Creates a signed-in session result.
    pub fn new(authenticated_session: AuthenticatedSession, token: SessionToken) -> Self {
        Self {
            authenticated_session,
            token,
        }
    }
}

/// Provides session creation data for a successful sign-in operation.
pub trait SignInSessionInputProvider: Send + Sync {
    /// Returns the data used to create the next authenticated session.
    fn next_session_input(&self) -> SignInSessionInput;
}

impl<F> SignInSessionInputProvider for F
where
    F: Fn() -> SignInSessionInput + Send + Sync,
{
    fn next_session_input(&self) -> SignInSessionInput {
        self()
    }
}

/// Signs users in with an email address and password.
pub struct EmailPasswordSignInService {
    user_repository: Arc<dyn UserRepository>,
    credential_repository: Arc<dyn CredentialRepository>,
    password_service: Arc<dyn PasswordService>,
    session_service: Arc<dyn SessionService>,
    token_generator: Arc<dyn SessionTokenGenerator>,
    token_hasher: Arc<dyn SessionTokenHasher>,
    expiration_policy: SessionExpirationPolicy,
    session_input_provider: Arc<dyn SignInSessionInputProvider>,
}

/// Dependencies required by `EmailPasswordSignInService`.
pub struct EmailPasswordSignInDependencies {
    pub user_repository: Arc<dyn UserRepository>,
    pub credential_repository: Arc<dyn CredentialRepository>,
    pub password_service: Arc<dyn PasswordService>,
    pub session_service: Arc<dyn SessionService>,
    pub token_generator: Arc<dyn SessionTokenGenerator>,
    pub token_hasher: Arc<dyn SessionTokenHasher>,
    pub expiration_policy: SessionExpirationPolicy,
    pub session_input_provider: Arc<dyn SignInSessionInputProvider>,
}

impl EmailPasswordSignInService {
    /// Creates a sign-in service from repository, password, token, and session dependencies.
    pub fn new(dependencies: EmailPasswordSignInDependencies) -> Self {
        Self {
            user_repository: dependencies.user_repository,
            credential_repository: dependencies.credential_repository,
            password_service: dependencies.password_service,
            session_service: dependencies.session_service,
            token_generator: dependencies.token_generator,
            token_hasher: dependencies.token_hasher,
            expiration_policy: dependencies.expiration_policy,
            session_input_provider: dependencies.session_input_provider,
        }
    }

    /// Authenticates a user and creates a session.
    pub fn sign_in<'a>(
        &'a self,
        input: &'a SignInInput,
    ) -> BoxServiceFuture<'a, ServiceResult<SignedInSession>> {
        Box::pin(async move {
            crate::email::validate_email(&input.email)?;

            let Some(user) = self.user_repository.find_by_email(&input.email).await? else {
                return Err(crate::AuthError::InvalidCredentials);
            };

            let Some(credential) = self.credential_repository.find_by_user_id(&user.id).await?
            else {
                return Err(crate::AuthError::InvalidCredentials);
            };

            let password_matches = self
                .password_service
                .verify_password(&input.password, &credential.password_hash)
                .await?;

            if !password_matches {
                return Err(crate::AuthError::InvalidCredentials);
            }

            let session_input = self.session_input_provider.next_session_input();
            let token = self.token_generator.generate();
            let token_hash = self.token_hasher.hash_token(&token);
            let expires_at = self.expiration_policy.expires_at(session_input.now)?;
            let session = self
                .session_service
                .create_session(&user.id, token_hash.as_str(), expires_at)
                .await?;
            let authenticated_session = AuthenticatedSession::new(user, session);

            Ok(SignedInSession::new(authenticated_session, token))
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

    use super::{
        EmailPasswordSignInDependencies, EmailPasswordSignInService, SignInSessionInput,
        SignInSessionInputProvider,
    };
    use crate::{
        BoxRepositoryFuture, BoxServiceFuture, CredentialRepository, PasswordCredential,
        PasswordService, RepositoryResult, ServiceResult, Session, SessionExpirationPolicy,
        SessionId, SessionRepository, SessionService, SessionToken, SessionTokenGenerator,
        SessionTokenHash, SessionTokenHasher, SignInInput, User, UserId, UserRepository,
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

    impl SessionService for InMemoryRepositories {
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

    struct FakePasswordService;

    impl PasswordService for FakePasswordService {
        fn hash_password<'a>(
            &'a self,
            password: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<String>> {
            Box::pin(std::future::ready(Ok(format!("fake-hash:{password}"))))
        }

        fn verify_password<'a>(
            &'a self,
            password: &'a str,
            password_hash: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<bool>> {
            Box::pin(std::future::ready(Ok(
                password_hash == format!("fake-hash:{password}")
            )))
        }
    }

    struct FakeSessionTokenGenerator;

    impl SessionTokenGenerator for FakeSessionTokenGenerator {
        fn generate(&self) -> SessionToken {
            SessionToken::from("token-value")
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

    fn service(repositories: &Arc<InMemoryRepositories>) -> EmailPasswordSignInService {
        let session_input_provider: Arc<dyn SignInSessionInputProvider> =
            Arc::new(|| SignInSessionInput::new(SystemTime::UNIX_EPOCH + Duration::from_secs(10)));

        EmailPasswordSignInService::new(EmailPasswordSignInDependencies {
            user_repository: repositories.clone(),
            credential_repository: repositories.clone(),
            password_service: Arc::new(FakePasswordService),
            session_service: repositories.clone(),
            token_generator: Arc::new(FakeSessionTokenGenerator),
            token_hasher: Arc::new(FakeSessionTokenHasher),
            expiration_policy: SessionExpirationPolicy::new(Duration::from_secs(90)),
            session_input_provider,
        })
    }

    fn insert_user_with_credential(repositories: &Arc<InMemoryRepositories>) -> User {
        let user = User::new(UserId::from("user-1"), "user@example.com");
        let credential = PasswordCredential::new(user.id.clone(), "fake-hash:correct horse");
        repositories
            .users
            .lock()
            .unwrap()
            .insert(user.id.clone(), user.clone());
        repositories
            .credentials
            .lock()
            .unwrap()
            .insert(user.id.clone(), credential);
        user
    }

    #[test]
    fn successful_sign_in_returns_token_and_stores_only_token_hash() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let user = insert_user_with_credential(&repositories);
        let service = service(&repositories);
        let input = SignInInput::new("user@example.com", "correct horse");

        let signed_in_session = block_on(service.sign_in(&input)).unwrap();

        assert_eq!(signed_in_session.token, SessionToken::from("token-value"));
        assert_eq!(signed_in_session.authenticated_session.user, user);
        assert_eq!(
            signed_in_session.authenticated_session.session.token_hash,
            "hash:token-value"
        );
        assert_ne!(
            signed_in_session.authenticated_session.session.token_hash,
            signed_in_session.token.as_str()
        );
        assert_eq!(
            signed_in_session.authenticated_session.session.expires_at,
            SystemTime::UNIX_EPOCH + Duration::from_secs(100)
        );
    }

    #[test]
    fn missing_user_is_rejected_without_creating_session() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let service = service(&repositories);

        let result =
            block_on(service.sign_in(&SignInInput::new("missing@example.com", "correct horse")));

        assert_eq!(result, Err(crate::AuthError::InvalidCredentials));
        assert!(repositories.sessions.lock().unwrap().is_empty());
    }

    #[test]
    fn invalid_password_is_rejected_without_creating_session() {
        let repositories = Arc::new(InMemoryRepositories::default());
        insert_user_with_credential(&repositories);
        let service = service(&repositories);

        let result =
            block_on(service.sign_in(&SignInInput::new("user@example.com", "wrong horse")));

        assert_eq!(result, Err(crate::AuthError::InvalidCredentials));
        assert!(repositories.sessions.lock().unwrap().is_empty());
    }

    #[test]
    fn invalid_email_is_rejected_without_creating_session() {
        let repositories = Arc::new(InMemoryRepositories::default());
        insert_user_with_credential(&repositories);
        let service = service(&repositories);

        let result = block_on(service.sign_in(&SignInInput::new("not-an-email", "correct horse")));

        assert_eq!(result, Err(crate::AuthError::InvalidEmail));
        assert!(repositories.sessions.lock().unwrap().is_empty());
    }
}
