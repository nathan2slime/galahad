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

/// Entry point for Galahad's high-level setup API.
pub struct Galahad;

impl Galahad {
    /// Starts configuring the Actix Web integration.
    pub const fn actix() -> GalahadActixBuilder {
        GalahadActixBuilder
    }

    /// Starts configuring session behavior for high-level integrations.
    pub fn session() -> GalahadSession {
        GalahadSession::default()
    }
}

/// PostgreSQL backend configuration for Galahad integrations.
pub struct GalahadPostgres {
    db: DatabaseConnection,
}

impl GalahadPostgres {
    /// Creates a PostgreSQL backend from a SeaORM database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

/// Session configuration shared by high-level integrations.
#[derive(Clone, Debug)]
pub struct GalahadSession {
    cookie_name: Option<String>,
    ttl: Duration,
}

impl Default for GalahadSession {
    fn default() -> Self {
        Self {
            cookie_name: None,
            ttl: SessionExpirationPolicy::default().ttl(),
        }
    }
}

impl GalahadSession {
    /// Sets the session cookie name used by web integrations.
    pub fn cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = Some(name.into());
        self
    }

    /// Sets the session time-to-live used when new sessions are created.
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }
}

/// Builder for Actix Web integrations.
pub struct GalahadActixBuilder;

impl GalahadActixBuilder {
    /// Selects PostgreSQL persistence for the Actix integration.
    pub fn database(self, database: GalahadPostgres) -> GalahadActixPostgresBuilder {
        GalahadActixPostgresBuilder {
            database,
            session: GalahadSession::default(),
        }
    }
}

/// Builder for the default Actix Web + PostgreSQL authentication setup.
pub struct GalahadActixPostgresBuilder {
    database: GalahadPostgres,
    session: GalahadSession,
}

impl GalahadActixPostgresBuilder {
    /// Sets the session configuration for the Actix integration.
    pub fn session(mut self, session: GalahadSession) -> Self {
        self.session = session;
        self
    }

    /// Builds the Actix authentication integration.
    pub fn build(self) -> actix::GalahadActix {
        build_actix_postgres(self.database.db, self.session)
    }
}

/// Backwards-compatible builder for the default Actix Web + PostgreSQL setup.
pub struct GalahadActixPostgres {
    builder: GalahadActixPostgresBuilder,
}

impl GalahadActixPostgres {
    /// Creates a builder backed by a SeaORM database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            builder: Galahad::actix().database(GalahadPostgres::new(db)),
        }
    }

    /// Sets the session cookie name used by the Actix integration.
    pub fn with_session_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.builder.session.cookie_name = Some(name.into());
        self
    }

    /// Sets the session time-to-live used when new sessions are created.
    pub fn with_session_ttl(mut self, ttl: Duration) -> Self {
        self.builder.session.ttl = ttl;
        self
    }

    /// Builds the Actix authentication integration.
    pub fn build(self) -> actix::GalahadActix {
        self.builder.build()
    }
}

#[cfg(feature = "openapi")]
/// Entry point for Galahad's OpenAPI integration API.
pub struct GalahadOpenApi;

#[cfg(feature = "openapi")]
impl GalahadOpenApi {
    /// Starts injecting Galahad's Actix endpoint documentation into an OpenAPI document.
    pub fn actix(openapi: utoipa::openapi::OpenApi) -> GalahadActixOpenApiBuilder {
        GalahadActixOpenApiBuilder {
            openapi,
            session_cookie_name: None,
        }
    }
}

#[cfg(feature = "openapi")]
/// Builder for injecting Galahad Actix documentation into an OpenAPI document.
pub struct GalahadActixOpenApiBuilder {
    openapi: utoipa::openapi::OpenApi,
    session_cookie_name: Option<String>,
}

#[cfg(feature = "openapi")]
impl GalahadActixOpenApiBuilder {
    /// Sets the session cookie name documented by the OpenAPI security scheme.
    pub fn session_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.session_cookie_name = Some(name.into());
        self
    }

    /// Injects Galahad's Actix endpoint documentation and returns the OpenAPI document.
    pub fn build(mut self) -> utoipa::openapi::OpenApi {
        let galahad_openapi = match self.session_cookie_name {
            Some(name) => GalahadActixOpenApi::openapi_with_session_cookie_name(name),
            None => GalahadActixOpenApi::openapi(),
        };

        self.openapi.merge(galahad_openapi);
        self.openapi
    }
}

fn build_actix_postgres(db: DatabaseConnection, session: GalahadSession) -> actix::GalahadActix {
    let users = Arc::new(SeaOrmUserRepository::new(db.clone()));
    let credentials = Arc::new(SeaOrmCredentialRepository::new(db.clone()));
    let sessions = Arc::new(SeaOrmSessionRepository::new(db.clone()));
    let session_service = Arc::new(PostgresSessionService::new(db));
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
            expiration_policy: SessionExpirationPolicy::new(session.ttl),
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

    match session.cookie_name {
        Some(name) => auth.with_session_cookie_name(name),
        None => auth,
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

#[cfg(all(test, feature = "openapi"))]
mod tests {
    use super::GalahadOpenApi;
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(info(title = "Application API", version = "1.0.0"))]
    struct ApplicationOpenApi;

    #[test]
    fn openapi_builder_injects_galahad_actix_paths() {
        let openapi = GalahadOpenApi::actix(ApplicationOpenApi::openapi()).build();
        let json = openapi.to_json().unwrap();

        assert!(json.contains("Application API"));
        assert!(json.contains("/auth/sign-up"));
        assert!(json.contains("/auth/sign-in"));
        assert!(json.contains("/auth/sign-out"));
        assert!(json.contains("/auth/session"));
    }

    #[test]
    fn openapi_builder_documents_custom_session_cookie_name() {
        let openapi = GalahadOpenApi::actix(ApplicationOpenApi::openapi())
            .session_cookie_name("app_session")
            .build();
        let json = openapi.to_json().unwrap();

        assert!(json.contains("app_session"));
    }
}
