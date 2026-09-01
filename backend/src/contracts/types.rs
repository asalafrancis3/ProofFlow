use serde::{Deserialize, Serialize};

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum UserRole {
    Client = 0,
    Worker = 1,
    Verifier = 2,
    Arbitrator = 3,
    Admin = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum JobStatus {
    Draft = 0,
    Funded = 1,
    Active = 2,
    InReview = 3,
    Settled = 4,
    Cancelled = 5,
    Disputed = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum MilestoneStatus {
    Pending = 0,
    Submitted = 1,
    Approved = 2,
    Rejected = 3,
    Released = 4,
    Disputed = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum EscrowStatus {
    Created = 0,
    Funded = 1,
    PartialRelease = 2,
    Completed = 3,
    Frozen = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum DisputeStatus {
    Filed = 0,
    UnderReview = 1,
    Resolved = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum Resolution {
    UpholdWorker = 0,
    UpholdClient = 1,
    PartialSplit = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum AttestationOutcome {
    Approved = 0,
    Rejected = 1,
}

// ── Core structs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub address: String,
    pub org_id: String,
    pub has_org: bool,
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
pub struct Submission {
    pub job_id: u64,
    pub milestone_idx: u32,
    pub worker: String,
    pub evidence_uri: String,
    pub notes: String,
    pub submitted_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub job_id: u64,
    pub milestone_idx: u32,
    pub verifier: String,
    pub outcome: AttestationOutcome,
    pub notes: String,
    pub created_at: u64,
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

// ── Contract Events ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractEvent {
    JobCreated { job_id: u64, client: String },
    JobFunded { job_id: u64 },
    JobActivated { job_id: u64 },
    JobCancelled { job_id: u64 },
    JobSettled { job_id: u64 },
    MilestoneCreated { job_id: u64, index: u32 },
    MilestoneSubmitted { job_id: u64, index: u32, worker: String },
    MilestoneApproved { job_id: u64, index: u32 },
    MilestoneRejected { job_id: u64, index: u32 },
    MilestoneReleased { job_id: u64, index: u32 },
    EscrowCreated { job_id: u64 },
    EscrowFunded { job_id: u64, amount: u128 },
    EscrowReleased { job_id: u64, milestone_idx: u32, amount: u128 },
    EscrowFrozen { job_id: u64 },
    EscrowUnfrozen { job_id: u64 },
    DisputeFiled { job_id: u64, dispute_id: u32 },
    DisputeResolved { job_id: u64, dispute_id: u32 },
    ReputationUpdated { address: String },
    UserRegistered { address: String, role: UserRole },
    VerifierAdded { address: String },
    VerifierRemoved { address: String },
}

// ── Helper methods ────────────────────────────────────────────────────────────

impl JobStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, JobStatus::Settled | JobStatus::Cancelled)
    }

    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(JobStatus::Draft),
            1 => Some(JobStatus::Funded),
            2 => Some(JobStatus::Active),
            3 => Some(JobStatus::InReview),
            4 => Some(JobStatus::Settled),
            5 => Some(JobStatus::Cancelled),
            6 => Some(JobStatus::Disputed),
            _ => None,
        }
    }
}

impl MilestoneStatus {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(MilestoneStatus::Pending),
            1 => Some(MilestoneStatus::Submitted),
            2 => Some(MilestoneStatus::Approved),
            3 => Some(MilestoneStatus::Rejected),
            4 => Some(MilestoneStatus::Released),
            5 => Some(MilestoneStatus::Disputed),
            _ => None,
        }
    }
}

impl Escrow {
    pub fn remaining(&self) -> u128 {
        self.total_funded
            .saturating_sub(self.total_released)
            .saturating_sub(self.total_frozen)
    }
}

impl Reputation {
    pub fn compute_score(&self) -> u64 {
        let base = self.completed_jobs * 10;
        let attestation_bonus = self.successful_attestations * 5;
        let dispute_penalty = self.disputes_involved.saturating_sub(self.disputes_won) * 3;
        base.saturating_add(attestation_bonus).saturating_sub(dispute_penalty)
    }
}

// ── Request types for contract calls ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CreateJobRequest {
    pub client: String,
    pub title: String,
    pub description: String,
    pub milestone_titles: Vec<String>,
    pub milestone_amounts: Vec<u128>,
    pub milestone_workers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SubmitEvidenceRequest {
    pub worker: String,
    pub job_id: u64,
    pub milestone_idx: u32,
    pub evidence_uri: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct FileDisputeRequest {
    pub raised_by: String,
    pub job_id: u64,
    pub milestone_idx: u32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ResolveDisputeRequest {
    pub arbitrator: String,
    pub job_id: u64,
    pub dispute_id: u32,
    pub resolution: Resolution,
    pub note: String,
}

// ── Contract Error ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractErrorCode {
    NotAuthorized = 1,
    NotFound = 2,
    InvalidInput = 3,
    AlreadyInitialized = 4,
    ContractPaused = 5,
    JobNotFound = 10,
    JobNotDraft = 11,
    JobNotFunded = 12,
    JobNotActive = 13,
    JobAlreadySettled = 14,
    JobAlreadyCancelled = 15,
    JobCannotBeCancelled = 16,
    JobNotEnoughMilestones = 17,
    EscrowNotFound = 20,
    InsufficientFunds = 21,
    EscrowAlreadyFunded = 22,
    EscrowFrozen = 23,
    EscrowOverRelease = 24,
    EscrowNotFullyFunded = 25,
    MilestoneNotFound = 30,
    MilestoneAlreadySubmitted = 31,
    MilestoneNotSubmitted = 32,
    MilestoneNotApproved = 33,
    MilestoneAlreadyReleased = 34,
    MilestoneAlreadyDisputed = 35,
    MilestoneCannotSubmit = 36,
    InvalidMilestoneIndex = 37,
    SubmissionAlreadyExists = 40,
    EvidenceRequired = 41,
    NotWhitelistedVerifier = 50,
    AttestationAlreadyExists = 51,
    CannotAttestOwnWork = 52,
    DisputeNotFound = 60,
    DisputeAlreadyResolved = 61,
    MilestoneNotDisputed = 62,
    CannotDisputeReleasedMilestone = 63,
    ReputationNotFound = 70,
    VerifierAlreadyWhitelisted = 80,
    VerifierNotWhitelisted = 81,
    InvalidConfigKey = 82,
    NonceAlreadyUsed = 90,
    Unknown = 0,
}

impl ContractErrorCode {
    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::NotAuthorized,
            2 => Self::NotFound,
            3 => Self::InvalidInput,
            4 => Self::AlreadyInitialized,
            5 => Self::ContractPaused,
            10 => Self::JobNotFound,
            11 => Self::JobNotDraft,
            12 => Self::JobNotFunded,
            13 => Self::JobNotActive,
            14 => Self::JobAlreadySettled,
            15 => Self::JobAlreadyCancelled,
            16 => Self::JobCannotBeCancelled,
            17 => Self::JobNotEnoughMilestones,
            20 => Self::EscrowNotFound,
            21 => Self::InsufficientFunds,
            22 => Self::EscrowAlreadyFunded,
            23 => Self::EscrowFrozen,
            24 => Self::EscrowOverRelease,
            25 => Self::EscrowNotFullyFunded,
            30 => Self::MilestoneNotFound,
            31 => Self::MilestoneAlreadySubmitted,
            32 => Self::MilestoneNotSubmitted,
            33 => Self::MilestoneNotApproved,
            34 => Self::MilestoneAlreadyReleased,
            35 => Self::MilestoneAlreadyDisputed,
            36 => Self::MilestoneCannotSubmit,
            37 => Self::InvalidMilestoneIndex,
            40 => Self::SubmissionAlreadyExists,
            41 => Self::EvidenceRequired,
            50 => Self::NotWhitelistedVerifier,
            51 => Self::AttestationAlreadyExists,
            52 => Self::CannotAttestOwnWork,
            60 => Self::DisputeNotFound,
            61 => Self::DisputeAlreadyResolved,
            62 => Self::MilestoneNotDisputed,
            63 => Self::CannotDisputeReleasedMilestone,
            70 => Self::ReputationNotFound,
            80 => Self::VerifierAlreadyWhitelisted,
            81 => Self::VerifierNotWhitelisted,
            82 => Self::InvalidConfigKey,
            90 => Self::NonceAlreadyUsed,
            _ => Self::Unknown,
        }
    }
}
