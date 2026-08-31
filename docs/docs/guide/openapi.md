# OpenAPI

Enable OpenAPI documentation when you want Galahad to expose an Utoipa document
for the Actix authentication endpoints.

```toml
[dependencies]
galahad = { git = "https://github.com/nathan2slime/galahad", features = ["openapi"] }
galahad-seaorm = { git = "https://github.com/nathan2slime/galahad", features = ["postgres"] }
utoipa = "5"
utoipa-swagger-ui = { version = "9", features = ["actix-web"] }
```

## Merge Into an App Spec

```rust
use actix_web::{App, HttpServer};
use galahad::seaorm::Migrator;
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
- Bearer authentication when JWT support is documented

Use `GalahadOpenApi::actix(openapi).build()` with the default
`galahad_session` cookie name. Add `.session_cookie_name(...)` when Actix uses a
custom cookie name.
