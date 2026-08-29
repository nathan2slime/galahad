<div align="center">
  <h1>Galahad</h1>
  <p>
    <strong>Authentication for Rust applications using Actix Web and PostgreSQL</strong>
  </p>
  <p>

<!-- prettier-ignore-start -->

[![CI](https://github.com/nathan2slime/galahad/actions/workflows/ci.yml/badge.svg)](https://github.com/nathan2slime/galahad/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-stable-ab6000.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

<!-- prettier-ignore-end -->

  </p>
</div>

Galahad is a focused authentication library for Rust applications. It provides
email/password authentication, secure password hashing, session management,
Actix Web routes and extractors, and SeaORM persistence for PostgreSQL.

Galahad handles authentication concerns only. Authorization, roles, permissions,
organizations, billing, profiles, and product-specific user workflows belong in
your application.

## Features

- Email/password sign up and sign in
- Argon2id password hashing
- Secure random session tokens
- Hashed session-token storage
- Session expiration, lookup, logout, and revocation
- HttpOnly `SameSite=Lax` session cookies
- Actix Web routes and authenticated-user extractors
- SeaORM repositories and migrations for PostgreSQL
- Stable public error codes for API responses and localization

## Installation

```toml
[dependencies]
actix-web = "4"
galahad = { git = "https://github.com/nathan2slime/galahad" }
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["postgres"] }
sea-orm = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
sea-orm-migration = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
uuid = { version = "1", features = ["v4"] }
```

## Quick Start

Create the database connection, run Galahad migrations, wire the authentication
services, and register the Actix routes.

```rust
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use actix_web::{App, HttpServer};
use galahad::actix::GalahadActix;
use galahad::core::{
    Argon2idPasswordService, BoxServiceFuture, EmailPasswordSignInDependencies,
    EmailPasswordSignInService, EmailPasswordSignUpService, OsSessionTokenGenerator,
    ServiceResult, Session, SessionExpirationPolicy, SessionId, SessionLogoutService,
    SessionLookupService, SessionRepository, SessionService, Sha256SessionTokenHasher,
    SignInSessionInput, UserId,
};
use galahad::seaorm::{
    Migrator, SeaOrmCredentialRepository, SeaOrmSessionRepository, SeaOrmUserRepository,
};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let db = Database::connect(database_url)
        .await
        .expect("database connection failed");

    Migrator::up(&db, None)
        .await
        .expect("database migration failed");

    let auth = build_auth(db);

    HttpServer::new(move || App::new().configure(|config| auth.routes(config)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

fn build_auth(db: DatabaseConnection) -> GalahadActix {
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
        Arc::new(|| UserId::from(uuid::Uuid::new_v4().to_string())),
    ));

    let sign_in_service = Arc::new(EmailPasswordSignInService::new(
        EmailPasswordSignInDependencies {
            user_repository: users.clone(),
            credential_repository: credentials,
            password_service,
            session_service,
            token_generator: Arc::new(OsSessionTokenGenerator::new()),
            token_hasher: token_hasher.clone(),
            expiration_policy: SessionExpirationPolicy::new(Duration::from_secs(60 * 60 * 24 * 7)),
            session_input_provider: Arc::new(|| SignInSessionInput::new(SystemTime::now())),
        },
    ));

    let logout_service = Arc::new(SessionLogoutService::new(
        sessions.clone(),
        token_hasher.clone(),
    ));
    let lookup_service = Arc::new(SessionLookupService::new(
        users,
        sessions,
        token_hasher,
    ));

    GalahadActix::new(
        sign_up_service,
        sign_in_service,
        logout_service,
        lookup_service,
    )
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
```

## HTTP API

### Sign Up

```http
POST /auth/sign-up
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "correct horse battery staple"
}
```

Response:

```json
{
  "id": "generated-user-id",
  "email": "user@example.com"
}
```

### Sign In

```http
POST /auth/sign-in
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "correct horse battery staple"
}
```

Response sets the session cookie and returns the authenticated session:

```http
Set-Cookie: galahad_session=<token>; HttpOnly; SameSite=Lax; Path=/
```

```json
{
  "user": {
    "id": "generated-user-id",
    "email": "user@example.com"
  },
  "session": {
    "id": "session-id",
    "expires_at_unix_seconds": 1728000000
  }
}
```

### Current Session

```http
GET /auth/session
Cookie: galahad_session=<token>
```

Returns the same authenticated-session shape as sign in.

### Sign Out

```http
POST /auth/sign-out
Cookie: galahad_session=<token>
```

Response:

```http
204 No Content
Set-Cookie: galahad_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0
```

## Route Protection

Use `AuthenticatedUser` for routes that require a valid session, and
`OptionalUser` for routes that can behave differently when a user is signed in.

```rust
use actix_web::HttpResponse;
use galahad::actix::{AuthenticatedUser, OptionalUser};

async fn account(user: AuthenticatedUser) -> HttpResponse {
    HttpResponse::Ok().body(user.0.email)
}

async fn home(user: OptionalUser) -> HttpResponse {
    match user.0 {
        Some(user) => HttpResponse::Ok().body(format!("Signed in as {}", user.email)),
        None => HttpResponse::Ok().body("Signed out"),
    }
}
```

## Crates

- `galahad`: Facade crate that re-exports all Galahad integrations.
- `galahad-actix`: Actix Web routes, extractors, and session-cookie support.
- `galahad-seaorm`: SeaORM persistence repositories and migrations.
- `galahad-core`: Authentication domain types, traits, services, and errors.

## Development

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## License

This project is licensed under the MIT license.
