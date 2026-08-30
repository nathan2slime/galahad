use crate::database::{build_actix_seaorm, GalahadSeaOrm};
use crate::session::GalahadSession;

/// Builder for Actix Web integrations.
pub struct GalahadActixBuilder;

impl GalahadActixBuilder {
    /// Selects a SeaORM-backed database for the Actix integration.
    pub fn database(self, database: GalahadSeaOrm) -> GalahadActixSeaOrmBuilder {
        GalahadActixSeaOrmBuilder {
            database,
            session: GalahadSession::default(),
        }
    }
}

/// Builder for the default Actix Web + SeaORM authentication setup.
pub struct GalahadActixSeaOrmBuilder {
    database: GalahadSeaOrm,
    session: GalahadSession,
}

impl GalahadActixSeaOrmBuilder {
    /// Sets the session configuration for the Actix integration.
    pub fn session(mut self, session: GalahadSession) -> Self {
        self.session = session;
        self
    }

    /// Builds the Actix authentication integration.
    pub fn build(self) -> galahad_actix::GalahadActix {
        build_actix_seaorm(self.database.db, self.session)
    }
}
