#![warn(rust_2018_idioms)]
#![allow(missing_docs)]

//! Rust binding for ibverbs
#[allow(warnings, clippy::all)]
mod ibverbs;
pub use ibverbs::*;

// include!(concat!(env!("OUT_DIR"), "/lib.rs"));
