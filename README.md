<div align="center">
  <h1>Galahad</h1>
  <p>
    <strong>Authentication for Rust applications using Actix Web and SeaORM</strong>
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
Actix Web routes and extractors, and SeaORM persistence.

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
- SeaORM repositories and migrations
- Stable public error codes for API responses and localization

## Installation

```toml
[dependencies]
actix-web = "4"
galahad = { git = "https://github.com/nathan2slime/galahad" }
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["postgres"] }
sea-orm = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
sea-orm-migration = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
```

Use the Git dependency until the next crate release includes these facade
APIs. Enable database-driver features on `galahad-seaorm`; PostgreSQL is shown
here as the current driver example.

Enable OpenAPI documentation when you want to expose the generated spec or
Swagger UI from your application:

```toml
[dependencies]
galahad = { git = "https://github.com/nathan2slime/galahad", features = ["openapi"] }
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["postgres"] }
utoipa = "5"
utoipa-swagger-ui = { version = "9", features = ["actix-web"] }
```

## Quick Start

Create the database connection, run Galahad migrations, build the default
Actix + SeaORM integration, and register the routes.

```rust
use std::time::Duration;

use actix_web::{App, HttpServer};
use galahad::seaorm::Migrator;
use galahad::{Galahad, GalahadJwtAlgorithm, GalahadSeaOrm};
use sea_orm::Database;
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

    let session = Galahad::session()
        .cookie_name("my_session_cookie")
        .ttl(Duration::from_secs(60 * 60));
    let sign_up = Galahad::sign_up()
        .required_field("name")
        .required_field("company_id");
    let jwt = Galahad::jwt(std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"))
        .algorithm(GalahadJwtAlgorithm::Hs512)
        .issuer("my-api")
        .audience("my-web-app")
        .ttl(Duration::from_secs(60 * 15))
        .leeway(30);
    let auth = Galahad::actix()
        .database(GalahadSeaOrm::new(db))
        .session(session)
        .sign_up(sign_up)
        .jwt(jwt)
        .build();

    HttpServer::new(move || App::new().configure(|config| auth.routes(config)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

## OpenAPI Documentation

With the `openapi` feature enabled, Galahad exposes an Utoipa document for the
Actix authentication endpoints. Applications can serve the generated document as
JSON, attach it to Swagger UI, or merge it into a larger application OpenAPI
spec.

```rust
use galahad::seaorm::Migrator;
use actix_web::{App, HttpServer};
use galahad::{Galahad, GalahadOpenApi, GalahadSeaOrm};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(info(title = "My API", version = "1.0.0"))]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Database::connect(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("database connection failed");

    Migrator::up(&db, None)
        .await
        .expect("database migration failed");

    let session = Galahad::session().cookie_name("app_session");
    let auth = Galahad::actix()
        .database(GalahadSeaOrm::new(db))
        .session(session)
        .build();
    let openapi = GalahadOpenApi::actix(ApiDoc::openapi())
        .session_cookie_name("app_session")
        .build();

    HttpServer::new(move || {
        App::new()
            .configure(|config| auth.routes(config))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

The generated OpenAPI document includes:

- `POST /auth/sign-up`
- `POST /auth/sign-in`
- `POST /auth/sign-out`
- `GET /auth/session`
- Request and response schemas
- Cookie authentication using the configured session cookie name

Use `GalahadOpenApi::actix(openapi).build()` when using the default
`galahad_session` cookie name. Add `.session_cookie_name(...)` when the Actix
integration is configured with a custom cookie name.

## HTTP API

### Sign Up

```http
POST /auth/sign-up
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "correct horse battery staple",
  "name": "Ada Lovelace",
  "company_id": "company-1"
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
  },
  "access_token": "jwt-token"
}
```

### Current Session

```http
GET /auth/session
Cookie: galahad_session=<token>
```

Or use the JWT returned by sign in:

```http
GET /auth/session
Authorization: Bearer <jwt-token>
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
- `galahad-seaorm`: SeaORM persistence repositories and migrations. Enable its database features, such as `postgres`, in applications that need a specific SeaORM driver.
- `galahad-core`: Authentication domain types, traits, services, and errors.

## Development

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## License

This project is licensed under the MIT license.
