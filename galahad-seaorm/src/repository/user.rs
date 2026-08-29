use galahad_core::{
    AuthError, BoxRepositoryFuture, RepositoryResult, User, UserId, UserRepository,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entity::user;
use crate::repository::SeaOrmConnection;

/// A SeaORM-backed repository for users.
pub struct SeaOrmUserRepository<'db> {
    db: SeaOrmConnection<'db>,
}

impl SeaOrmUserRepository<'_> {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db: SeaOrmConnection::Database(db),
        }
    }
}

impl<'db> SeaOrmUserRepository<'db> {
    /// Creates a repository backed by an existing database transaction.
    pub fn from_transaction(transaction: &'db sea_orm::DatabaseTransaction) -> Self {
        Self {
            db: SeaOrmConnection::Transaction(transaction),
        }
    }
}

impl From<user::Model> for User {
    fn from(model: user::Model) -> Self {
        Self::new(UserId::from(model.id), model.email)
    }
}

impl UserRepository for SeaOrmUserRepository<'_> {
    fn find_by_id<'a>(
        &'a self,
        id: &'a UserId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>> {
        Box::pin(async move {
            let query = user::Entity::find_by_id(id.as_str().to_owned());
            let model = match &self.db {
                SeaOrmConnection::Database(db) => query.one(db).await,
                SeaOrmConnection::Transaction(transaction) => query.one(*transaction).await,
            };

            model
                .map(|model| model.map(User::from))
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }

    fn find_by_email<'a>(
        &'a self,
        email: &'a str,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>> {
        Box::pin(async move {
            let query = user::Entity::find().filter(user::Column::Email.eq(email));
            let model = match &self.db {
                SeaOrmConnection::Database(db) => query.one(db).await,
                SeaOrmConnection::Transaction(transaction) => query.one(*transaction).await,
            };

            model
                .map(|model| model.map(User::from))
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }

    fn save<'a>(&'a self, user: &'a User) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
        Box::pin(async move {
            let now = sea_orm::prelude::ChronoUtc::now();
            let model = user::ActiveModel {
                id: Set(user.id.to_string()),
                email: Set(user.email.clone()),
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
