use soroban_sdk::{contracttype, Address, String};

// ── Enums ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserRole {
    Client = 0,
    Worker = 1,
    Verifier = 2,
    Arbitrator = 3,
    Admin = 4,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobStatus {
    Draft = 0,
    Funded = 1,
    Active = 2,
    InReview = 3,
    Settled = 4,
    Cancelled = 5,
    Disputed = 6,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MilestoneStatus {
    Pending = 0,
    Submitted = 1,
    Approved = 2,
    Rejected = 3,
    Released = 4,
    Disputed = 5,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Created = 0,
    Funded = 1,
    PartialRelease = 2,
    Completed = 3,
    Frozen = 4,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Filed = 0,
    UnderReview = 1,
    Resolved = 2,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Resolution {
    UpholdWorker = 0,
    UpholdClient = 1,
    PartialSplit = 2,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttestationOutcome {
    Approved = 0,
    Rejected = 1,
}

// ── Core structs ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub address: Address,
    pub org_id: Option<Address>,
    pub role: UserRole,
    pub name: String,
    pub registered_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Job {
    pub id: u64,
    pub client: Address,
    pub title: String,
    pub description: String,
    pub status: JobStatus,
    pub total_funded: u128,
    pub milestone_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub job_id: u64,
    pub index: u32,
    pub title: String,
    pub description: String,
    pub amount: u128,
    pub status: MilestoneStatus,
    pub worker: Address,
    pub evidence_uri: Option<String>,
    pub submitted_at: Option<u64>,
    pub resolved_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub job_id: u64,
    pub total_funded: u128,
    pub total_released: u128,
    pub total_frozen: u128,
    pub status: EscrowStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Submission {
    pub job_id: u64,
    pub milestone_idx: u32,
    pub worker: Address,
    pub evidence_uri: String,
    pub notes: String,
    pub submitted_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attestation {
    pub job_id: u64,
    pub milestone_idx: u32,
    pub verifier: Address,
    pub outcome: AttestationOutcome,
    pub notes: String,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub job_id: u64,
    pub milestone_idx: u32,
    pub dispute_id: u32,
    pub raised_by: Address,
    pub reason: String,
    pub status: DisputeStatus,
    pub resolution: Option<Resolution>,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reputation {
    pub address: Address,
    pub completed_jobs: u64,
    pub successful_attestations: u64,
    pub disputes_involved: u64,
    pub disputes_won: u64,
    pub total_earned: u128,
    pub score: u64,
    pub updated_at: u64,
}

// ── Helper methods ────────────────────────────────────────────────────────────

impl JobStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Settled | JobStatus::Cancelled
        )
    }
}

impl MilestoneStatus {
    pub fn can_submit(&self) -> bool {
        matches!(self, MilestoneStatus::Pending | MilestoneStatus::Rejected)
    }

    pub fn can_settle(&self) -> bool {
        matches!(self, MilestoneStatus::Approved)
    }
}

impl Escrow {
    pub fn remaining(&self) -> u128 {
        self.total_funded
            .saturating_sub(self.total_released)
            .saturating_sub(self.total_frozen)
    }

    pub fn can_release(&self, amount: u128) -> bool {
        self.status != EscrowStatus::Frozen && self.remaining() >= amount
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
