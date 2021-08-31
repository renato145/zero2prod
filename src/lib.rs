pub mod configuration;
pub mod routes;
pub mod startup;

pub use startup::*;

#[macro_use]
extern crate rocket;
