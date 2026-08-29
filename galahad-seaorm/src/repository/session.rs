use galahad_core::{
    AuthError, BoxRepositoryFuture, RepositoryResult, Session, SessionId, SessionRepository, UserId,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entity::session;
use crate::repository::SeaOrmConnection;

/// A SeaORM-backed repository for sessions.
pub struct SeaOrmSessionRepository<'db> {
    db: SeaOrmConnection<'db>,
}

impl SeaOrmSessionRepository<'_> {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db: SeaOrmConnection::Database(db),
        }
    }
}

impl<'db> SeaOrmSessionRepository<'db> {
    /// Creates a repository backed by an existing database transaction.
    pub fn from_transaction(transaction: &'db sea_orm::DatabaseTransaction) -> Self {
        Self {
            db: SeaOrmConnection::Transaction(transaction),
        }
    }
}

impl From<session::Model> for Session {
    fn from(model: session::Model) -> Self {
        Self {
            id: SessionId::from(model.id),
            user_id: UserId::from(model.user_id),
            token_hash: model.token_hash,
            expires_at: model.expires_at.into(),
            revoked_at: model.revoked_at.map(Into::into),
        }
    }
}

impl SessionRepository for SeaOrmSessionRepository<'_> {
    fn find_by_id<'a>(
        &'a self,
        id: &'a SessionId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<Session>>> {
        Box::pin(async move {
            let query = session::Entity::find_by_id(id.as_str().to_owned());
            let model = match &self.db {
                SeaOrmConnection::Database(db) => query.one(db).await,
                SeaOrmConnection::Transaction(transaction) => query.one(*transaction).await,
            };

            model
                .map(|model| model.map(Session::from))
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }

    fn find_by_token_hash<'a>(
        &'a self,
        token_hash: &'a str,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<Session>>> {
        Box::pin(async move {
            let query = session::Entity::find().filter(session::Column::TokenHash.eq(token_hash));
            let model = match &self.db {
                SeaOrmConnection::Database(db) => query.one(db).await,
                SeaOrmConnection::Transaction(transaction) => query.one(*transaction).await,
            };

            model
                .map(|model| model.map(Session::from))
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }

    fn save<'a>(&'a self, session: &'a Session) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
        Box::pin(async move {
            let now = sea_orm::prelude::ChronoUtc::now();
            let model = crate::entity::session::ActiveModel {
                id: Set(session.id.to_string()),
                user_id: Set(session.user_id.to_string()),
                token_hash: Set(session.token_hash.clone()),
                expires_at: Set(session.expires_at.into()),
                revoked_at: Set(session.revoked_at.map(Into::into)),
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

    fn delete<'a>(&'a self, id: &'a SessionId) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
        Box::pin(async move {
            let query = session::Entity::delete_by_id(id.as_str().to_owned());
            let result = match &self.db {
                SeaOrmConnection::Database(db) => query.exec(db).await,
                SeaOrmConnection::Transaction(transaction) => query.exec(*transaction).await,
            };

            result
                .map(|_| ())
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }
}
