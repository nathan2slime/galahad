#![cfg(feature = "postgres")]

use std::time::{Duration, SystemTime};

use galahad_core::{
    CredentialRepository, PasswordCredential, Session, SessionId, SessionRepository, User, UserId,
    UserRepository,
};
use galahad_seaorm::{
    Migrator, SeaOrmCredentialRepository, SeaOrmSessionRepository, SeaOrmUserRepository,
};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

#[tokio::test(flavor = "multi_thread")]
async fn postgres_migrations_and_repositories_work() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(_) => {
            eprintln!("Skipping PostgreSQL integration test: DATABASE_URL is not set");
            return;
        }
    };

    let database = Database::connect(database_url)
        .await
        .expect("PostgreSQL integration test database should be reachable");
    Migrator::up(&database, None)
        .await
        .expect("PostgreSQL migrations should run successfully");

    let run_id = format!(
        "task-016-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system time should be after the Unix epoch")
            .as_nanos()
    );
    let user = User::new(
        UserId::from(format!("{run_id}-user")),
        format!("{run_id}@example.com"),
    );
    let credential = PasswordCredential::new(user.id.clone(), format!("{run_id}-password-hash"));
    let session = Session::new(
        SessionId::from(format!("{run_id}-session")),
        user.id.clone(),
        format!("{run_id}-token-hash"),
        SystemTime::UNIX_EPOCH + Duration::from_secs(4_102_444_800),
    );

    let user_repository = SeaOrmUserRepository::new(database.clone());
    user_repository
        .save(&user)
        .await
        .expect("user should be saved");
    assert_eq!(
        user_repository
            .find_by_id(&user.id)
            .await
            .expect("user lookup by id should succeed"),
        Some(user.clone())
    );
    assert_eq!(
        user_repository
            .find_by_email(&user.email)
            .await
            .expect("user lookup by email should succeed"),
        Some(user)
    );

    let credential_repository = SeaOrmCredentialRepository::new(database.clone());
    credential_repository
        .save(&credential)
        .await
        .expect("password credential should be saved");
    assert_eq!(
        credential_repository
            .find_by_user_id(&credential.user_id)
            .await
            .expect("password credential lookup should succeed"),
        Some(credential)
    );

    let session_repository = SeaOrmSessionRepository::new(database);
    session_repository
        .save(&session)
        .await
        .expect("session should be saved");
    assert_eq!(
        session_repository
            .find_by_id(&session.id)
            .await
            .expect("session lookup by id should succeed"),
        Some(session.clone())
    );
    assert_eq!(
        session_repository
            .find_by_token_hash(&session.token_hash)
            .await
            .expect("session lookup by token hash should succeed"),
        Some(session.clone())
    );

    let revoked_at = SystemTime::UNIX_EPOCH + Duration::from_secs(30);
    session_repository
        .revoke(&session.id, revoked_at)
        .await
        .expect("session should be revoked");
    let revoked_session = session_repository
        .find_by_id(&session.id)
        .await
        .expect("revoked session lookup should succeed")
        .expect("revoked session should still exist");
    assert_eq!(revoked_session.revoked_at, Some(revoked_at));
    assert_eq!(revoked_session.user_id, session.user_id);
    assert_eq!(revoked_session.token_hash, session.token_hash);
    assert_eq!(revoked_session.expires_at, session.expires_at);
    assert_eq!(
        session_repository
            .revoke(
                &SessionId::from("missing-session"),
                SystemTime::UNIX_EPOCH + Duration::from_secs(40),
            )
            .await,
        Err(galahad_core::AuthError::SessionNotFound)
    );

    session_repository
        .delete(&session.id)
        .await
        .expect("session should be deleted");
    assert_eq!(
        session_repository
            .find_by_id(&session.id)
            .await
            .expect("deleted session lookup should succeed"),
        None
    );
}
