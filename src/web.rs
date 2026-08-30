use std::time::Duration;

use sea_orm::DatabaseConnection;

use crate::entry::Galahad;
use crate::postgres::{build_actix_postgres, GalahadPostgres};
use crate::session::GalahadSession;

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
    pub fn build(self) -> galahad_actix::GalahadActix {
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
    pub fn build(self) -> galahad_actix::GalahadActix {
        self.builder.build()
    }
}
