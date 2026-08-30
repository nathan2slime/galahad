//! Galahad's public facade crate.

mod entry;
#[cfg(feature = "openapi")]
mod openapi;
mod postgres;
mod session;
mod web;

pub use galahad_actix as actix;
#[cfg(feature = "openapi")]
pub use galahad_actix::GalahadActixOpenApi;
pub use galahad_core as core;
pub use galahad_seaorm as seaorm;

pub use entry::Galahad;
#[cfg(feature = "openapi")]
pub use openapi::{GalahadActixOpenApiBuilder, GalahadOpenApi};
pub use postgres::GalahadPostgres;
pub use session::GalahadSession;
pub use web::{GalahadActixBuilder, GalahadActixPostgres, GalahadActixPostgresBuilder};
