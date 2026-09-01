pub mod archival;
pub mod audit;
pub mod compliance_api;
pub mod contracts;
pub mod export;
pub mod pagination;
pub mod proofflow;
pub mod search;
pub mod signing_api;
pub mod verification;
pub mod ws;
// analytics module removed — legacy endpoints were never registered (#906)
#[cfg(test)]
mod pagination_boundary_tests;
