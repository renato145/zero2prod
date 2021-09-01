pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

pub use startup::*;

#[macro_use]
extern crate rocket;
