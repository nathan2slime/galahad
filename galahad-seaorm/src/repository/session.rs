use galahad_core::{
    AuthError, BoxRepositoryFuture, RepositoryResult, Session, SessionId, SessionRepository, UserId,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entity::session;

/// A SeaORM-backed repository for sessions.
pub struct SeaOrmSessionRepository {
    db: DatabaseConnection,
}

impl SeaOrmSessionRepository {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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

impl SessionRepository for SeaOrmSessionRepository {
    fn find_by_id<'a>(
        &'a self,
        id: &'a SessionId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<Session>>> {
        Box::pin(async move {
            session::Entity::find_by_id(id.as_str().to_owned())
                .one(&self.db)
                .await
                .map(|model| model.map(Session::from))
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }

    fn find_by_token_hash<'a>(
        &'a self,
        token_hash: &'a str,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<Session>>> {
        Box::pin(async move {
            session::Entity::find()
                .filter(session::Column::TokenHash.eq(token_hash))
                .one(&self.db)
                .await
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

            model
                .insert(&self.db)
                .await
                .map(|_| ())
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }

    fn delete<'a>(&'a self, id: &'a SessionId) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
        Box::pin(async move {
            session::Entity::delete_by_id(id.as_str().to_owned())
                .exec(&self.db)
                .await
                .map(|_| ())
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }
}
