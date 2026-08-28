use galahad_core::{
    AuthError, BoxRepositoryFuture, CredentialRepository, PasswordCredential, RepositoryResult,
    UserId,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::entity::credential;

/// A SeaORM-backed repository for password credentials.
pub struct SeaOrmCredentialRepository {
    db: DatabaseConnection,
}

impl SeaOrmCredentialRepository {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<credential::Model> for PasswordCredential {
    fn from(model: credential::Model) -> Self {
        Self::new(UserId::from(model.user_id), model.password_hash)
    }
}

impl CredentialRepository for SeaOrmCredentialRepository {
    fn find_by_user_id<'a>(
        &'a self,
        user_id: &'a UserId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<PasswordCredential>>> {
        Box::pin(async move {
            credential::Entity::find_by_id(user_id.as_str().to_owned())
                .one(&self.db)
                .await
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

            model
                .insert(&self.db)
                .await
                .map(|_| ())
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }
}
