//! Galahad's public facade crate.

mod actix_setup;
mod database;
mod entry;
#[cfg(feature = "openapi")]
mod openapi;
mod session;

pub use galahad_actix as actix;
#[cfg(feature = "openapi")]
pub use galahad_actix::GalahadActixOpenApi;
pub use galahad_core as core;
pub use galahad_seaorm as seaorm;

pub use actix_setup::{GalahadActixBuilder, GalahadActixSeaOrmBuilder};
pub use database::GalahadSeaOrm;
pub use entry::Galahad;
#[cfg(feature = "openapi")]
pub use openapi::{GalahadActixOpenApiBuilder, GalahadOpenApi};
pub use session::GalahadSession;
