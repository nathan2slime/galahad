use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

use crate::{AuthError, Session, SessionId, User, UserId};

/// A boxed, sendable future returned by a service operation.
pub type BoxServiceFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The result type shared by service operations.
pub type ServiceResult<T> = Result<T, AuthError>;

/// Input for a sign-up operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignUpInput {
    pub email: String,
    pub password: String,
}

impl SignUpInput {
    /// Creates sign-up input from an email address and plaintext password.
    pub fn new(email: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}

/// Input for a sign-in operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignInInput {
    pub email: String,
    pub password: String,
}

impl SignInInput {
    /// Creates sign-in input from an email address and plaintext password.
    pub fn new(email: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}

/// The user and session returned after successful authentication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedSession {
    pub user: User,
    pub session: Session,
}

impl AuthenticatedSession {
    /// Creates an authenticated session result.
    pub fn new(user: User, session: Session) -> Self {
        Self { user, session }
    }
}

/// Core authentication operations.
pub trait AuthService: Send + Sync {
    /// Registers a user.
    fn sign_up<'a>(&'a self, input: &'a SignUpInput) -> BoxServiceFuture<'a, ServiceResult<User>>;

    /// Authenticates a user and creates a session.
    fn sign_in<'a>(
        &'a self,
        input: &'a SignInInput,
    ) -> BoxServiceFuture<'a, ServiceResult<AuthenticatedSession>>;

    /// Signs out the session identified by `session_id`.
    fn sign_out<'a>(&'a self, session_id: &'a SessionId)
        -> BoxServiceFuture<'a, ServiceResult<()>>;

    /// Returns the authenticated session associated with a token hash.
    fn current_session<'a>(
        &'a self,
        token_hash: &'a str,
    ) -> BoxServiceFuture<'a, ServiceResult<Option<AuthenticatedSession>>>;
}

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

    use super::{
        AuthService, AuthenticatedSession, BoxServiceFuture, PasswordService, ServiceResult,
        SessionService, SignInInput, SignUpInput,
    };
    use crate::{Session, SessionId, User, UserId};

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

    #[derive(Default)]
    struct FakeAuthService {
        users: Mutex<HashMap<String, User>>,
        sessions: Mutex<HashMap<SessionId, AuthenticatedSession>>,
    }

    impl AuthService for FakeAuthService {
        fn sign_up<'a>(
            &'a self,
            input: &'a SignUpInput,
        ) -> BoxServiceFuture<'a, ServiceResult<User>> {
            let mut users = self.users.lock().unwrap();
            let user = User::new(
                UserId::from(format!("user-{}", users.len() + 1)),
                input.email.clone(),
            );
            users.insert(input.email.clone(), user.clone());

            Box::pin(std::future::ready(Ok(user)))
        }

        fn sign_in<'a>(
            &'a self,
            input: &'a SignInInput,
        ) -> BoxServiceFuture<'a, ServiceResult<AuthenticatedSession>> {
            let user = self
                .users
                .lock()
                .unwrap()
                .get(&input.email)
                .cloned()
                .ok_or(crate::AuthError::UserNotFound);

            let authenticated_session = user.map(|user| {
                let session_id = SessionId::from(format!(
                    "session-{}",
                    self.sessions.lock().unwrap().len() + 1
                ));
                let session = Session::new(
                    session_id.clone(),
                    user.id.clone(),
                    "token-hash",
                    SystemTime::UNIX_EPOCH + Duration::from_secs(100),
                );
                let authenticated_session = AuthenticatedSession::new(user, session);
                self.sessions
                    .lock()
                    .unwrap()
                    .insert(session_id, authenticated_session.clone());
                authenticated_session
            });

            Box::pin(std::future::ready(authenticated_session))
        }

        fn sign_out<'a>(
            &'a self,
            session_id: &'a SessionId,
        ) -> BoxServiceFuture<'a, ServiceResult<()>> {
            let result = self
                .sessions
                .lock()
                .unwrap()
                .remove(session_id)
                .map(|_| ())
                .ok_or(crate::AuthError::SessionNotFound);

            Box::pin(std::future::ready(result))
        }

        fn current_session<'a>(
            &'a self,
            token_hash: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<Option<AuthenticatedSession>>> {
            let authenticated_session = self
                .sessions
                .lock()
                .unwrap()
                .values()
                .find(|authenticated_session| {
                    authenticated_session.session.token_hash == token_hash
                })
                .cloned();

            Box::pin(std::future::ready(Ok(authenticated_session)))
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

    #[test]
    fn auth_service_is_object_safe_and_usable_without_runtime() {
        let service: Box<dyn AuthService> = Box::new(FakeAuthService::default());
        let sign_up_input = SignUpInput::new("user@example.com", "correct horse");

        let user = block_on(service.sign_up(&sign_up_input)).unwrap();

        assert_eq!(user.email, "user@example.com");

        let sign_in_input = SignInInput::new("user@example.com", "correct horse");
        let authenticated_session = block_on(service.sign_in(&sign_in_input)).unwrap();

        assert_eq!(authenticated_session.user, user);
        assert_eq!(authenticated_session.session.user_id, user.id);
        assert_eq!(
            block_on(service.current_session("token-hash")).unwrap(),
            Some(authenticated_session.clone())
        );

        block_on(service.sign_out(&authenticated_session.session.id)).unwrap();

        assert_eq!(
            block_on(service.current_session("token-hash")).unwrap(),
            None
        );
    }
}
