//! Compliance domain logic module
//!
//! This module contains all business logic related to compliance checking,
//! separated from the HTTP layer.

pub mod service;
pub mod validator;
pub mod models;

pub use service::ComplianceService;
pub use service::{CheckRequest, ComplianceResult};
pub use validator::ComplianceValidator;
pub use models::*;
