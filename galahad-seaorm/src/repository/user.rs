use galahad_core::{
    AuthError, BoxRepositoryFuture, RepositoryResult, User, UserId, UserRepository,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entity::user;

/// A SeaORM-backed repository for users.
pub struct SeaOrmUserRepository {
    db: DatabaseConnection,
}

impl SeaOrmUserRepository {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<user::Model> for User {
    fn from(model: user::Model) -> Self {
        Self::new(UserId::from(model.id), model.email)
    }
}

impl UserRepository for SeaOrmUserRepository {
    fn find_by_id<'a>(
        &'a self,
        id: &'a UserId,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>> {
        Box::pin(async move {
            user::Entity::find_by_id(id.as_str().to_owned())
                .one(&self.db)
                .await
                .map(|model| model.map(User::from))
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }

    fn find_by_email<'a>(
        &'a self,
        email: &'a str,
    ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>> {
        Box::pin(async move {
            user::Entity::find()
                .filter(user::Column::Email.eq(email))
                .one(&self.db)
                .await
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

            model
                .insert(&self.db)
                .await
                .map(|_| ())
                .map_err(|_| AuthError::PersistenceFailure)
        })
    }
}
