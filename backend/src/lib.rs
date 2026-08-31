// lib.rs — re-exports all modules so integration tests in tests/ can use
// `crate::` (or `scavenger_backend::`) paths.

pub mod api;
pub mod cache;
pub mod compliance;
pub mod config;
pub mod container;
pub mod crypto;
pub mod errors;
pub mod middleware;
pub mod redis;
pub mod rpc;
pub mod search;
pub mod security;
pub mod services;
pub mod validation;
