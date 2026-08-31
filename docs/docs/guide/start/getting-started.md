# Getting Started

## Install

Use the Git dependency until the next crate release includes the current facade
APIs.

```toml
[dependencies]
actix-web = "4"
galahad = { git = "https://github.com/nathan2slime/galahad" }
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["postgres"] }
sea-orm = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
sea-orm-migration = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
```

Enable the matching SeaORM driver feature for your database:

```toml
# PostgreSQL
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["postgres"] }
sea-orm = { version = "2.0.2", features = ["sqlx-postgres", "runtime-tokio-rustls"] }

# SQLite
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["sqlite"] }
sea-orm = { version = "2.0.2", features = ["sqlx-sqlite", "runtime-tokio-rustls"] }

# MySQL
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["mysql"] }
sea-orm = { version = "2.0.2", features = ["sqlx-mysql", "runtime-tokio-rustls"] }
```

## Register Routes

Create the database connection, run Galahad migrations, build the Actix + SeaORM
integration, and register the routes.

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
