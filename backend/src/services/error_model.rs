/// ProofFlow error model.
///
/// Maps contract errors and backend errors into a unified error hierarchy
/// with proper HTTP status codes.
use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum ProofFlowError {
    // ── Contract errors (mirrors contract error variants) ──────────────────────

    Unauthorized,
    AdminRequired,
    UserAlreadyRegistered,
    UserNotFound,
    InvalidRole,
    InvalidAddress,

    JobNotFound,
    JobNotInDraft,
    JobNotFunded,
    JobNotActive,
    JobAlreadySettled,
    AlreadyCancelled,

    MilestoneNotFound,
    MilestoneAlreadySubmitted,
    MilestoneNotSubmitted,

    InsufficientFunds,
    InsufficientEscrowBalance,
    NothingToRelease,
    EscrowAlreadyCompleted,

    DisputeNotFound,
    DisputeAlreadyResolved,
    CannotDisputeCompleted,

    AlreadyAdded,
    AlreadyRemoved,
    VerifierAlreadyAdded,
    VerifierNotAdded,

    InvalidJobTitle,
    InvalidMilestoneIndex,
    InvalidAmount,
    InvalidDeadline,
    TitleTooLong,
    DescriptionTooLong,
    NoMilestones,

    // ── Backend errors ────────────────────────────────────────────────────────

    NotFound,
    ValidationFailed(String),
    RpcError(String),
    SerializationError(String),
    InternalError(String),
}

impl ProofFlowError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Unauthorized | Self::AdminRequired => 403,
            Self::NotFound | Self::JobNotFound | Self::MilestoneNotFound | Self::DisputeNotFound | Self::UserNotFound => 404,
            Self::ValidationFailed(_) | Self::InvalidJobTitle | Self::InvalidMilestoneIndex
            | Self::InvalidAmount | Self::InvalidDeadline | Self::TitleTooLong
            | Self::DescriptionTooLong | Self::NoMilestones | Self::InvalidRole
            | Self::InvalidAddress => 400,
            Self::UserAlreadyRegistered | Self::JobNotInDraft | Self::JobAlreadySettled
            | Self::AlreadyCancelled | Self::MilestoneAlreadySubmitted | Self::EscrowAlreadyCompleted
            | Self::DisputeAlreadyResolved | Self::AlreadyAdded | Self::AlreadyRemoved
            | Self::VerifierAlreadyAdded | Self::VerifierNotAdded => 409,
            Self::JobNotFunded | Self::JobNotActive | Self::MilestoneNotSubmitted | Self::CannotDisputeCompleted => 400,
            Self::InsufficientFunds | Self::InsufficientEscrowBalance | Self::NothingToRelease => 422,
            Self::RpcError(_) | Self::SerializationError(_) | Self::InternalError(_) => 500,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::AdminRequired => "ADMIN_REQUIRED",
            Self::UserAlreadyRegistered => "USER_ALREADY_REGISTERED",
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::InvalidRole => "INVALID_ROLE",
            Self::InvalidAddress => "INVALID_ADDRESS",
            Self::JobNotFound => "JOB_NOT_FOUND",
            Self::JobNotInDraft => "JOB_NOT_IN_DRAFT",
            Self::JobNotFunded => "JOB_NOT_FUNDED",
            Self::JobNotActive => "JOB_NOT_ACTIVE",
            Self::JobAlreadySettled => "JOB_ALREADY_SETTLED",
            Self::AlreadyCancelled => "ALREADY_CANCELLED",
            Self::MilestoneNotFound => "MILESTONE_NOT_FOUND",
            Self::MilestoneAlreadySubmitted => "MILESTONE_ALREADY_SUBMITTED",
            Self::MilestoneNotSubmitted => "MILESTONE_NOT_SUBMITTED",
            Self::InsufficientFunds => "INSUFFICIENT_FUNDS",
            Self::InsufficientEscrowBalance => "INSUFFICIENT_ESCROW",
            Self::NothingToRelease => "NOTHING_TO_RELEASE",
            Self::EscrowAlreadyCompleted => "ESCROW_ALREADY_COMPLETED",
            Self::DisputeNotFound => "DISPUTE_NOT_FOUND",
            Self::DisputeAlreadyResolved => "DISPUTE_ALREADY_RESOLVED",
            Self::CannotDisputeCompleted => "CANNOT_DISPUTE_COMPLETED",
            Self::AlreadyAdded => "ALREADY_ADDED",
            Self::AlreadyRemoved => "ALREADY_REMOVED",
            Self::VerifierAlreadyAdded => "VERIFIER_ALREADY_ADDED",
            Self::VerifierNotAdded => "VERIFIER_NOT_ADDED",
            Self::InvalidJobTitle => "INVALID_JOB_TITLE",
            Self::InvalidMilestoneIndex => "INVALID_MILESTONE_INDEX",
            Self::InvalidAmount => "INVALID_AMOUNT",
            Self::InvalidDeadline => "INVALID_DEADLINE",
            Self::TitleTooLong => "TITLE_TOO_LONG",
            Self::DescriptionTooLong => "DESCRIPTION_TOO_LONG",
            Self::NoMilestones => "NO_MILESTONES",
            Self::NotFound => "NOT_FOUND",
            Self::ValidationFailed(_) => "VALIDATION_FAILED",
            Self::RpcError(_) => "RPC_ERROR",
            Self::SerializationError(_) => "SERIALIZATION_ERROR",
            Self::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::ValidationFailed(msg) | Self::RpcError(msg) | Self::SerializationError(msg) | Self::InternalError(msg) => msg.clone(),
            Self::Unauthorized => "Authorization required".into(),
            Self::AdminRequired => "Admin access required".into(),
            Self::UserAlreadyRegistered => "User already registered".into(),
            Self::UserNotFound => "User not found".into(),
            Self::InvalidRole => "Invalid role".into(),
            Self::InvalidAddress => "Invalid address".into(),
            Self::JobNotFound => "Job not found".into(),
            Self::JobNotInDraft => "Job is not in draft status".into(),
            Self::JobNotFunded => "Job is not funded".into(),
            Self::JobNotActive => "Job is not active".into(),
            Self::JobAlreadySettled => "Job is already settled".into(),
            Self::AlreadyCancelled => "Job is already cancelled".into(),
            Self::MilestoneNotFound => "Milestone not found".into(),
            Self::MilestoneAlreadySubmitted => "Milestone already submitted".into(),
            Self::MilestoneNotSubmitted => "Milestone not yet submitted".into(),
            Self::InsufficientFunds => "Insufficient funds".into(),
            Self::InsufficientEscrowBalance => "Insufficient escrow balance".into(),
            Self::NothingToRelease => "Nothing to release".into(),
            Self::EscrowAlreadyCompleted => "Escrow already completed".into(),
            Self::DisputeNotFound => "Dispute not found".into(),
            Self::DisputeAlreadyResolved => "Dispute already resolved".into(),
            Self::CannotDisputeCompleted => "Cannot dispute a completed milestone".into(),
            Self::AlreadyAdded => "Already added".into(),
            Self::AlreadyRemoved => "Already removed".into(),
            Self::VerifierAlreadyAdded => "Verifier already added".into(),
            Self::VerifierNotAdded => "Verifier not added".into(),
            Self::InvalidJobTitle => "Invalid job title".into(),
            Self::InvalidMilestoneIndex => "Invalid milestone index".into(),
            Self::InvalidAmount => "Invalid amount".into(),
            Self::InvalidDeadline => "Invalid deadline".into(),
            Self::TitleTooLong => "Title too long".into(),
            Self::DescriptionTooLong => "Description too long".into(),
            Self::NoMilestones => "No milestones provided".into(),
            Self::NotFound => "Resource not found".into(),
        }
    }

    pub fn to_http_response(&self) -> HttpResponse {
        let status = self.status_code();
        let body = serde_json::json!({
            "success": false,
            "error": {
                "code": self.error_code(),
                "message": self.message(),
            }
        });
        match status {
            400 => HttpResponse::BadRequest().json(body),
            403 => HttpResponse::Forbidden().json(body),
            404 => HttpResponse::NotFound().json(body),
            409 => HttpResponse::Conflict().json(body),
            422 => HttpResponse::UnprocessableEntity().json(body),
            _ => HttpResponse::InternalServerError().json(body),
        }
    }
}

impl std::fmt::Display for ProofFlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.error_code(), self.message())
    }
}

impl std::error::Error for ProofFlowError {}

impl From<ProofFlowError> for HttpResponse {
    fn from(err: ProofFlowError) -> Self {
        err.to_http_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_status_codes() {
        assert_eq!(ProofFlowError::Unauthorized.status_code(), 403);
        assert_eq!(ProofFlowError::JobNotFound.status_code(), 404);
        assert_eq!(ProofFlowError::InvalidAmount.status_code(), 400);
        assert_eq!(ProofFlowError::UserAlreadyRegistered.status_code(), 409);
        assert_eq!(ProofFlowError::InsufficientFunds.status_code(), 422);
        assert_eq!(ProofFlowError::InternalError("x".into()).status_code(), 500);
    }

    #[test]
    fn error_codes_unique() {
        let codes = [
            ProofFlowError::Unauthorized.error_code(),
            ProofFlowError::AdminRequired.error_code(),
            ProofFlowError::UserAlreadyRegistered.error_code(),
            ProofFlowError::UserNotFound.error_code(),
            ProofFlowError::JobNotFound.error_code(),
            ProofFlowError::JobNotInDraft.error_code(),
            ProofFlowError::MilestoneNotFound.error_code(),
            ProofFlowError::InsufficientFunds.error_code(),
            ProofFlowError::DisputeNotFound.error_code(),
        ];
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len());
    }

    #[test]
    fn http_response_format() {
        let err = ProofFlowError::JobNotFound;
        let resp = err.to_http_response();
        assert_eq!(resp.status(), 404);
    }
}
