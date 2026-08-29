//! Actix integration for Galahad.

mod config;
mod cookie;
mod error;
mod extractor;
mod handler;
mod request;
mod response;

pub use config::GalahadActix;
pub use error::ActixAuthError;
pub use extractor::{AuthenticatedUser, OptionalUser};
