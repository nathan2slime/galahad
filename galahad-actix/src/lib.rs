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

pub use config::GalahadActix;
pub use error::ActixAuthError;
pub use extractor::{AuthenticatedUser, OptionalUser};
pub use jwt::JwtConfig;
#[cfg(feature = "openapi")]
pub use openapi::GalahadActixOpenApi;
