//! Core functionality for Galahad.

mod credential;
mod email;
mod error;
mod password;
mod repository;
mod service;
mod session;
mod signin;
mod signup;
mod token;
mod user;

pub use credential::PasswordCredential;
pub use error::AuthError;
pub use repository::{
    BoxRepositoryFuture, CredentialRepository, RepositoryResult, SessionRepository, UserRepository,
};
pub use service::{
    Argon2idPasswordService, AuthService, AuthenticatedSession, BoxServiceFuture, PasswordService,
    ServiceResult, SessionService, SignInInput, SignUpInput,
};
pub use session::{Session, SessionId};
pub use signin::{EmailPasswordSignInService, SignInSessionInput, SignInSessionInputProvider};
pub use signup::{EmailPasswordSignUpService, UserIdGenerator};
pub use token::{OsSessionTokenGenerator, SessionToken, SessionTokenGenerator};
pub use user::{User, UserId};
