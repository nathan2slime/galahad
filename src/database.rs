use std::sync::Arc;
use std::time::SystemTime;

use galahad_core::{
    Argon2idPasswordService, BoxServiceFuture, EmailPasswordSignInDependencies,
    EmailPasswordSignInService, EmailPasswordSignUpService, OsSessionTokenGenerator, ServiceResult,
    Session, SessionExpirationPolicy, SessionId, SessionLogoutService, SessionLookupService,
    SessionRepository, SessionService, Sha256SessionTokenHasher, SignInSessionInput, UserId,
    UserIdGenerator,
};
use galahad_seaorm::{SeaOrmCredentialRepository, SeaOrmSessionRepository, SeaOrmUserRepository};
use sea_orm::DatabaseConnection;

use crate::jwt::{GalahadJwt, GalahadJwtAlgorithm};
use crate::session::GalahadSession;
use crate::sign_up::GalahadSignUp;

/// SeaORM-backed database configuration for Galahad integrations.
pub struct GalahadSeaOrm {
    pub(crate) db: DatabaseConnection,
}

impl GalahadSeaOrm {
    /// Creates a SeaORM backend from a SeaORM database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

pub(crate) fn build_actix_seaorm(
    db: DatabaseConnection,
    session: GalahadSession,
    sign_up: GalahadSignUp,
    jwt: Option<GalahadJwt>,
) -> galahad_actix::GalahadActix {
    let users = Arc::new(SeaOrmUserRepository::new(db.clone()));
    let credentials = Arc::new(SeaOrmCredentialRepository::new(db.clone()));
    let sessions = Arc::new(SeaOrmSessionRepository::new(db.clone()));
    let session_service = Arc::new(SeaOrmSessionService::new(db));
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

    let auth = galahad_actix::GalahadActix::new(
        sign_up_service,
        sign_in_service,
        logout_service,
        lookup_service,
    );

    let auth = auth.with_required_sign_up_fields(sign_up.required_fields);
    let auth = match jwt {
        Some(jwt) => auth.with_jwt(jwt_config(jwt)),
        None => auth,
    };

    match session.cookie_name {
        Some(name) => auth.with_session_cookie_name(name),
        None => auth,
    }
}

fn jwt_config(jwt: GalahadJwt) -> galahad_actix::JwtConfig {
    let config = galahad_actix::JwtConfig::new(jwt.secret)
        .with_ttl(jwt.ttl)
        .with_algorithm(match jwt.algorithm {
            GalahadJwtAlgorithm::Hs256 => galahad_actix::JwtAlgorithm::Hs256,
            GalahadJwtAlgorithm::Hs384 => galahad_actix::JwtAlgorithm::Hs384,
            GalahadJwtAlgorithm::Hs512 => galahad_actix::JwtAlgorithm::Hs512,
        })
        .with_leeway(jwt.leeway);
    let config = match jwt.issuer {
        Some(issuer) => config.with_issuer(issuer),
        None => config,
    };

    jwt.audience
        .into_iter()
        .fold(config, |config, audience| config.with_audience(audience))
}

struct UuidUserIdGenerator;

impl UserIdGenerator for UuidUserIdGenerator {
    fn generate(&self) -> UserId {
        UserId::from(uuid::Uuid::new_v4().to_string())
    }
}

struct SeaOrmSessionService {
    db: DatabaseConnection,
}

impl SeaOrmSessionService {
    fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl SessionService for SeaOrmSessionService {
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
