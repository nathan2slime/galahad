//! Core functionality for Galahad.

mod credential;
mod error;
mod repository;
mod service;
mod session;
mod user;

pub use credential::PasswordCredential;
pub use error::AuthError;
pub use repository::{
    BoxRepositoryFuture, CredentialRepository, RepositoryResult, SessionRepository, UserRepository,
};
pub use service::{BoxServiceFuture, PasswordService, ServiceResult};
pub use session::{Session, SessionId};
pub use user::{User, UserId};
