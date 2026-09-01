/// ProofFlow domain models for the backend.
///
/// These are the backend's view of the domain, independent of the on-chain
/// contract types. The indexer converts contract events to these types.
/// The API serializes these to JSON for clients.
use serde::{Deserialize, Serialize};

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Client,
    Worker,
    Verifier,
    Arbitrator,
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Client => "client",
            UserRole::Worker => "worker",
            UserRole::Verifier => "verifier",
            UserRole::Arbitrator => "arbitrator",
            UserRole::Admin => "admin",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "client" => Some(UserRole::Client),
            "worker" => Some(UserRole::Worker),
            "verifier" => Some(UserRole::Verifier),
            "arbitrator" => Some(UserRole::Arbitrator),
            "admin" => Some(UserRole::Admin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Draft,
    Funded,
    Active,
    InReview,
    Settled,
    Cancelled,
    Disputed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    Pending,
    Submitted,
    Approved,
    Rejected,
    Released,
    Disputed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowStatus {
    Created,
    Funded,
    PartialRelease,
    Completed,
    Frozen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisputeStatus {
    Filed,
    UnderReview,
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    UpholdWorker,
    UpholdClient,
    PartialSplit,
}

// ── Core types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub address: String,
    pub role: UserRole,
    pub name: String,
    pub registered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub client: String,
    pub title: String,
    pub description: String,
    pub status: JobStatus,
    pub total_funded: u128,
    pub milestone_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub job_id: u64,
    pub index: u32,
    pub title: String,
    pub description: String,
    pub amount: u128,
    pub status: MilestoneStatus,
    pub worker: String,
    pub evidence_uri: String,
    pub has_evidence: bool,
    pub submitted_at: u64,
    pub resolved_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    pub job_id: u64,
    pub total_funded: u128,
    pub total_released: u128,
    pub total_frozen: u128,
    pub status: EscrowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub job_id: u64,
    pub milestone_idx: u32,
    pub dispute_id: u32,
    pub raised_by: String,
    pub reason: String,
    pub status: DisputeStatus,
    pub resolution: Resolution,
    pub has_resolution: bool,
    pub created_at: u64,
    pub resolved_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reputation {
    pub address: String,
    pub completed_jobs: u64,
    pub successful_attestations: u64,
    pub disputes_involved: u64,
    pub disputes_won: u64,
    pub total_earned: u128,
    pub score: u64,
    pub updated_at: u64,
}

// ── API request/response types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub client: String,
    pub title: String,
    pub description: String,
    pub milestone_titles: Vec<String>,
    pub milestone_amounts: Vec<u128>,
    pub milestone_workers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitEvidenceRequest {
    pub worker: String,
    pub job_id: u64,
    pub milestone_idx: u32,
    pub evidence_uri: String,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct FileDisputeRequest {
    pub raised_by: String,
    pub job_id: u64,
    pub milestone_idx: u32,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ResolveDisputeRequest {
    pub arbitrator: String,
    pub job_id: u64,
    pub dispute_id: u32,
    pub resolution: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: &str) -> Self {
        Self { success: false, data: None, error: Some(msg.to_string()) }
    }
}
