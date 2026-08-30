use crate::database::{build_actix_seaorm, GalahadSeaOrm};
use crate::jwt::GalahadJwt;
use crate::session::GalahadSession;
use crate::sign_up::GalahadSignUp;

/// Builder for Actix Web integrations.
pub struct GalahadActixBuilder;

impl GalahadActixBuilder {
    /// Selects a SeaORM-backed database for the Actix integration.
    pub fn database(self, database: GalahadSeaOrm) -> GalahadActixSeaOrmBuilder {
        GalahadActixSeaOrmBuilder {
            database,
            session: GalahadSession::default(),
            sign_up: GalahadSignUp::default(),
            jwt: None,
        }
    }
}

/// Builder for the default Actix Web + SeaORM authentication setup.
pub struct GalahadActixSeaOrmBuilder {
    database: GalahadSeaOrm,
    session: GalahadSession,
    sign_up: GalahadSignUp,
    jwt: Option<GalahadJwt>,
}

impl GalahadActixSeaOrmBuilder {
    /// Sets the session configuration for the Actix integration.
    pub fn session(mut self, session: GalahadSession) -> Self {
        self.session = session;
        self
    }

    /// Sets the sign-up configuration for the Actix integration.
    pub fn sign_up(mut self, sign_up: GalahadSignUp) -> Self {
        self.sign_up = sign_up;
        self
    }

    /// Enables JWT support for the Actix integration.
    pub fn jwt(mut self, jwt: GalahadJwt) -> Self {
        self.jwt = Some(jwt);
        self
    }

    /// Builds the Actix authentication integration.
    pub fn build(self) -> galahad_actix::GalahadActix {
        build_actix_seaorm(self.database.db, self.session, self.sign_up, self.jwt)
    }
}
