//! Actix integration for Galahad.

mod config;
mod cookie;
mod error;
mod extractor;
mod handler;
mod jwt;
#[cfg(feature = "openapi")]
mod openapi;
mod request;
mod response;
mod sign_up;

pub use config::GalahadActix;
pub use error::ActixAuthError;
pub use extractor::{AuthenticatedUser, OptionalUser};
pub use jwt::{JwtAlgorithm, JwtConfig};
#[cfg(feature = "openapi")]
pub use openapi::GalahadActixOpenApi;
pub use sign_up::{AfterSignUp, AfterSignUpFuture, SignUpContext};
