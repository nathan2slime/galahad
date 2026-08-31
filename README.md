<div align="center">
  <h1>Galahad</h1>
  <p>
    <strong>Focused authentication for Rust applications using Actix Web and SeaORM</strong>
  </p>
  <p>

<!-- prettier-ignore-start -->

[![CI](https://github.com/nathan2slime/galahad/actions/workflows/ci.yml/badge.svg)](https://github.com/nathan2slime/galahad/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-stable-ab6000.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

<!-- prettier-ignore-end -->

  </p>
</div>

Galahad provides email/password authentication, secure password hashing, session
management, Actix Web routes and extractors, optional JWTs, optional OpenAPI
docs, and SeaORM persistence.

Galahad handles authentication only. Authorization, roles, organizations,
profiles, billing, and product-specific user workflows belong in your
application.

## Documentation

Full documentation lives at [galahad.nathan3boss.dev](https://galahad.nathan3boss.dev).

## Features

- Email/password sign up and sign in
- Argon2id password hashing
- Secure session tokens stored as hashes
- Session expiration, lookup, logout, and revocation
- HttpOnly `SameSite=Lax` session cookies
- Actix Web routes and authenticated-user extractors
- Optional JWT bearer authentication
- Optional OpenAPI document generation
- SeaORM repositories and migrations for PostgreSQL, SQLite, and MySQL
- Stable public error codes for API responses and localization

## Quick Start

```toml
[dependencies]
actix-web = "4"
galahad = { git = "https://github.com/nathan2slime/galahad" }
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["postgres"] }
sea-orm = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
sea-orm-migration = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
```

```rust
use actix_web::{App, HttpServer};
use galahad::seaorm::Migrator;
use galahad::{Galahad, GalahadSeaOrm};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Database::connect(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("database connection failed");

    Migrator::up(&db, None)
        .await
        .expect("database migration failed");

    let auth = Galahad::actix()
        .database(GalahadSeaOrm::new(db))
        .build();

    HttpServer::new(move || App::new().configure(|config| auth.routes(config)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

## Crates

- `galahad`: Facade crate that re-exports Galahad integrations.
- `galahad-actix`: Actix Web routes, extractors, cookies, and JWT support.
- `galahad-seaorm`: SeaORM repositories and migrations.
- `galahad-core`: Authentication domain types, traits, services, and errors.

## Development

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## License

This project is licensed under the MIT license.
