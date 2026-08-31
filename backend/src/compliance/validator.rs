//! Compliance validation logic

use super::models::*;
use super::service::CheckRequest;

pub struct ComplianceValidator;

impl ComplianceValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_check(&self, request: &CheckRequest) -> Result<(), ComplianceError> {
        // Additional validation logic moved from compliance_api.rs
        if request.amount <= 0 {
            return Err(ComplianceError::InvalidAmount(
                "Amount must be positive".to_string()
            ));
        }

        if request.user_id.len() < 3 {
            return Err(ComplianceError::ValidationError(
                "User ID must be at least 3 characters".to_string()
            ));
        }

        Ok(())
    }

    pub fn validate_status(&self, status: &ComplianceStatus) -> bool {
        matches!(status, ComplianceStatus::Pending | ComplianceStatus::Approved)
    }
}
