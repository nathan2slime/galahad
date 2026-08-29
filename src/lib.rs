//! Galahad's public facade crate.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

pub use galahad_actix as actix;
#[cfg(feature = "openapi")]
pub use galahad_actix::GalahadActixOpenApi;
pub use galahad_core as core;
pub use galahad_seaorm as seaorm;

use galahad_core::{
    Argon2idPasswordService, BoxServiceFuture, EmailPasswordSignInDependencies,
    EmailPasswordSignInService, EmailPasswordSignUpService, OsSessionTokenGenerator, ServiceResult,
    Session, SessionExpirationPolicy, SessionId, SessionLogoutService, SessionLookupService,
    SessionRepository, SessionService, Sha256SessionTokenHasher, SignInSessionInput, UserId,
    UserIdGenerator,
};
use sea_orm::DatabaseConnection;
use seaorm::{SeaOrmCredentialRepository, SeaOrmSessionRepository, SeaOrmUserRepository};

/// Builder for the default Actix Web + PostgreSQL authentication setup.
pub struct GalahadActixPostgres {
    db: DatabaseConnection,
    session_cookie_name: Option<String>,
    session_ttl: Duration,
}

impl GalahadActixPostgres {
    /// Creates a builder backed by a SeaORM database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            session_cookie_name: None,
            session_ttl: SessionExpirationPolicy::default().ttl(),
        }
    }

    /// Sets the session cookie name used by the Actix integration.
    pub fn with_session_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.session_cookie_name = Some(name.into());
        self
    }

    /// Sets the session time-to-live used when new sessions are created.
    pub fn with_session_ttl(mut self, ttl: Duration) -> Self {
        self.session_ttl = ttl;
        self
    }

    /// Builds the Actix authentication integration.
    pub fn build(self) -> actix::GalahadActix {
        let users = Arc::new(SeaOrmUserRepository::new(self.db.clone()));
        let credentials = Arc::new(SeaOrmCredentialRepository::new(self.db.clone()));
        let sessions = Arc::new(SeaOrmSessionRepository::new(self.db.clone()));
        let session_service = Arc::new(PostgresSessionService::new(self.db));
        let password_service = Arc::new(Argon2idPasswordService::new());
        let token_hasher = Arc::new(Sha256SessionTokenHasher::new());

        let sign_up_service = Arc::new(EmailPasswordSignUpService::new(
            users.clone(),
            credentials.clone(),
            password_service.clone(),
            Arc::new(UuidUserIdGenerator),
        ));
        let sign_in_service = Arc::new(EmailPasswordSignInService::new(
            EmailPasswordSignInDependencies {
                user_repository: users.clone(),
                credential_repository: credentials,
                password_service,
                session_service,
                token_generator: Arc::new(OsSessionTokenGenerator::new()),
                token_hasher: token_hasher.clone(),
                expiration_policy: SessionExpirationPolicy::new(self.session_ttl),
                session_input_provider: Arc::new(|| SignInSessionInput::new(SystemTime::now())),
            },
        ));
        let logout_service = Arc::new(SessionLogoutService::new(
            sessions.clone(),
            token_hasher.clone(),
        ));
        let lookup_service = Arc::new(SessionLookupService::new(users, sessions, token_hasher));

        let auth = actix::GalahadActix::new(
            sign_up_service,
            sign_in_service,
            logout_service,
            lookup_service,
        );

        match self.session_cookie_name {
            Some(name) => auth.with_session_cookie_name(name),
            None => auth,
        }
    }
}

struct UuidUserIdGenerator;

impl UserIdGenerator for UuidUserIdGenerator {
    fn generate(&self) -> UserId {
        UserId::from(uuid::Uuid::new_v4().to_string())
    }
}

struct PostgresSessionService {
    db: DatabaseConnection,
}

impl PostgresSessionService {
    fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl SessionService for PostgresSessionService {
    fn create_session<'a>(
        &'a self,
        user_id: &'a UserId,
        token_hash: &'a str,
        expires_at: SystemTime,
    ) -> BoxServiceFuture<'a, ServiceResult<Session>> {
        Box::pin(async move {
            let session = Session::new(
                SessionId::from(uuid::Uuid::new_v4().to_string()),
                user_id.clone(),
                token_hash,
                expires_at,
            );

            SeaOrmSessionRepository::new(self.db.clone())
                .save(&session)
                .await?;

            Ok(session)
        })
    }

    fn find_session_by_token_hash<'a>(
        &'a self,
        token_hash: &'a str,
    ) -> BoxServiceFuture<'a, ServiceResult<Option<Session>>> {
        Box::pin(async move {
            SeaOrmSessionRepository::new(self.db.clone())
                .find_by_token_hash(token_hash)
                .await
        })
    }

    fn revoke_session<'a>(
        &'a self,
        session_id: &'a SessionId,
        revoked_at: SystemTime,
    ) -> BoxServiceFuture<'a, ServiceResult<()>> {
        Box::pin(async move {
            SeaOrmSessionRepository::new(self.db.clone())
                .revoke(session_id, revoked_at)
                .await
        })
    }
}
