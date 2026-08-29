use galahad_core::{
    AuthError, BoxRepositoryFuture, CredentialRepository, PasswordCredential, RepositoryResult,
    UserId,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::entity::credential;
use crate::repository::SeaOrmConnection;

/// A SeaORM-backed repository for password credentials.
pub struct SeaOrmCredentialRepository<'db> {
    db: SeaOrmConnection<'db>,
}

impl SeaOrmCredentialRepository<'_> {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db: SeaOrmConnection::Database(db),
        }
    }
}

impl<'db> SeaOrmCredentialRepository<'db> {
    /// Creates a repository backed by an existing database transaction.
    pub fn from_transaction(transaction: &'db sea_orm::DatabaseTransaction) -> Self {
        Self {
            db: SeaOrmConnection::Transaction(transaction),
        }
    }
}

impl From<credential::Model> for PasswordCredential {
    fn from(model: credential::Model) -> Self {
        Self::new(UserId::from(model.user_id), model.password_hash)
    }
}

impl CredentialRepository for SeaOrmCredentialRepository<'_> {
    fn find_by_user_id<'a>(
        &'a self,
        user_id: &'a UserId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<PasswordCredential>>> {
        Box::pin(async move {
            let query = credential::Entity::find_by_id(user_id.as_str().to_owned());
            let model = match &self.db {
                SeaOrmConnection::Database(db) => query.one(db).await,
                SeaOrmConnection::Transaction(transaction) => query.one(*transaction).await,
            };

            model
                .map(|model| model.map(PasswordCredential::from))
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }

    fn save<'a>(
        &'a self,
        credential: &'a PasswordCredential,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
        Box::pin(async move {
            let now = sea_orm::prelude::ChronoUtc::now();
            let model = crate::entity::credential::ActiveModel {
                user_id: Set(credential.user_id.to_string()),
                password_hash: Set(credential.password_hash.clone()),
                created_at: Set(now),
                updated_at: Set(now),
            };

            let result = match &self.db {
                SeaOrmConnection::Database(db) => model.insert(db).await,
                SeaOrmConnection::Transaction(transaction) => model.insert(*transaction).await,
            };

            result
                .map(|_| ())
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }
}
