mod credential;
mod session;
mod user;

use sea_orm::{DatabaseConnection, DatabaseTransaction};

pub use credential::SeaOrmCredentialRepository;
pub use session::SeaOrmSessionRepository;
pub use user::SeaOrmUserRepository;

pub(crate) enum SeaOrmConnection<'db> {
    Database(DatabaseConnection),
    Transaction(&'db DatabaseTransaction),
}
